use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, EventMask},
    },
    rust_connection::RustConnection,
};

use std::time::{Duration, Instant};

use crate::{
    config::Config,
    wm::{autostart::run_autostart, bar::Bar, layout::apply_layout, state::WMState},
    x11::ewmh::register_wm,
};

pub fn start_x11_wm(config: Config) {
    let (conn, screen_num) = RustConnection::connect(None).expect("Failed to connect to X11");

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    register_wm(&conn, root, &config);

    let res = conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::default().event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::KEY_PRESS
                | EventMask::FOCUS_CHANGE,
        ),
    );

    if res.is_err() {
        eprintln!("❌ Another WM is running");
        std::process::exit(1);
    }

    conn.flush().unwrap();
    println!("🚀 OrbitWM running");

    run_autostart(&config.autostart);

    let mut wm = WMState::new();

    let width = screen.width_in_pixels as u32;
    let height = screen.height_in_pixels as u32;

    let mut bar = Bar::create(&conn, root, screen, &config).expect("Failed to create bar");

    bar.draw(&conn);
    conn.flush().unwrap();

    let mut last_tick = Instant::now();
    let timeout = Duration::from_millis(500);

    loop {
        // ======================
        // EVENT HANDLING
        // ======================
        while let Ok(Some(event)) = conn.poll_for_event() {
            match event {
                Event::MapRequest(e) => {
                    let window = e.window;

                    let win_type = crate::x11::ewmh::get_window_type(&conn, window);

                    match Option::as_deref(&win_type) {
                        Some("dock") | Some("desktop") => {
                            // bar/shell (ignore)
                            conn.map_window(window).unwrap();
                        }

                        _ => {
                            // normal app
                            wm.add(window);
                            conn.map_window(window).unwrap();

                            apply_layout(&conn, &wm, width, height, &config);
                            bar.draw(&conn);
                        }
                        None => todo!(),
                    }
                }

                Event::DestroyNotify(e) => {
                    wm.remove(e.window);

                    apply_layout(&conn, &wm, width, height, &config);

                    bar.draw(&conn);
                    conn.flush().unwrap();
                }

                Event::UnmapNotify(e) => {
                    let is_managed = wm.workspaces.iter().any(|ws| ws.contains(&e.window));

                    if !is_managed {
                        wm.remove(e.window);

                        apply_layout(&conn, &wm, width, height, &config);

                        bar.draw(&conn);
                        conn.flush().unwrap();
                    }
                }

                Event::ConfigureRequest(e) => {
                    let aux = ConfigureWindowAux::default()
                        .x(e.x as i32)
                        .y(e.y as i32)
                        .width(e.width as u32)
                        .height(e.height as u32)
                        .border_width(e.border_width as u32);

                    conn.configure_window(e.window, &aux).unwrap();
                    conn.flush().unwrap();
                }

                Event::KeyPress(e) => {
                    crate::wm::keys::handle_key(&conn, &mut wm, &e, width, height, &config);

                    apply_layout(&conn, &wm, width, height, &config);

                    bar.draw(&conn);
                    conn.flush().unwrap();
                }

                Event::FocusIn(_) | Event::FocusOut(_) => {
                    crate::x11::ewmh::grab_keys(&conn, root, &config);
                    conn.flush().unwrap();
                }

                // 🔥 IMPORTANT: force redraw on expose
                Event::Expose(_) => {
                    bar.draw(&conn);
                    conn.flush().unwrap();
                }

                _ => {}
            }
        }

        // ======================
        // PERIODIC REDRAW (CLOCK ETC.)
        // ======================
        if last_tick.elapsed() >= timeout {
            bar.draw(&conn);
            conn.flush().unwrap();
            last_tick = Instant::now();
        }

        // ❌ DO NOT SLEEP — breaks X11 responsiveness
    }
}
