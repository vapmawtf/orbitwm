use crate::config::Config;
use crate::wm;
use crate::wm::state::WMState;
use crate::x11::atoms::Atoms;
use crate::x11::ewmh;
use crate::x11::grab_keys::grab_keys;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;

pub fn start_x11_wm(config: &Config, state: &mut WMState) {
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = conn.setup().roots[screen_num].clone();
    let root = screen.root;
    let width = screen.width_in_pixels as u32;
    let height = screen.height_in_pixels as u32;
    let atoms = Atoms::init(&conn);

    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::default().event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE
                | EventMask::KEY_PRESS
                | EventMask::ENTER_WINDOW
                | EventMask::STRUCTURE_NOTIFY,
        ),
    )
    .unwrap();

    grab_keys(&conn, root, config);
    ewmh::set_number_of_desktops(&conn, root, &atoms, 9);
    ewmh::set_current_desktop(&conn, root, &atoms, 0);
    conn.flush().unwrap();

    loop {
        let event = match conn.wait_for_event() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("X connection error: {}", e);
                break;
            }
        };

        match event {
            Event::MapRequest(e) => {
                if state.docks.contains(&e.window) {
                    conn.map_window(e.window).ok();
                    conn.flush().ok();
                    continue;
                }

                let class_str = get_window_class(&conn, e.window);
                let type_vals = get_window_type(&conn, e.window, &atoms);
                let is_dock = type_vals.contains(&atoms.net_wm_window_type_dock);
                let is_polybar = class_str.to_lowercase().contains("polybar");

                eprintln!(
                    "MapRequest: win={} class='{}' type={:?}",
                    e.window, class_str, type_vals
                );

                if is_dock || is_polybar {
                    state.add_dock(e.window);
                    conn.map_window(e.window).ok();
                    conn.flush().ok();
                    continue;
                }

                conn.change_window_attributes(
                    e.window,
                    &ChangeWindowAttributesAux::default()
                        .event_mask(EventMask::ENTER_WINDOW | EventMask::STRUCTURE_NOTIFY),
                )
                .ok();

                conn.map_window(e.window).ok();
                state.add(e.window);
                wm::layout::apply_layout(&conn, state, width, height, config);
                conn.flush().ok();
            }

            Event::ConfigureRequest(e) => {
                let is_dock = state.docks.contains(&e.window);
                let in_tiling = state.workspaces[state.current].contains(&e.window);

                if is_dock {
                    conn.configure_window(
                        e.window,
                        &ConfigureWindowAux::default()
                            .x(e.x as i32)
                            .y(e.y as i32)
                            .width(e.width as u32)
                            .height(e.height as u32)
                            .border_width(e.border_width as u32),
                    )
                    .ok();
                } else if in_tiling {
                    conn.configure_window(
                        e.window,
                        &ConfigureWindowAux::default()
                            .x(e.x as i32)
                            .y(e.y as i32)
                            .width(e.width as u32)
                            .height(e.height as u32),
                    )
                    .ok();
                    wm::layout::apply_layout(&conn, state, width, height, config);
                } else {
                    conn.configure_window(
                        e.window,
                        &ConfigureWindowAux::default()
                            .x(e.x as i32)
                            .y(e.y as i32)
                            .width(e.width as u32)
                            .height(e.height as u32)
                            .border_width(e.border_width as u32)
                            .stack_mode(e.stack_mode),
                    )
                    .ok();
                }
                conn.flush().ok();
            }

            Event::EnterNotify(e) => {
                if state.docks.contains(&e.event) {
                    continue;
                }
                let tracked = state.workspaces.iter().any(|ws| ws.contains(&e.event));
                if tracked {
                    state.focused = Some(e.event);
                    conn.set_input_focus(InputFocus::POINTER_ROOT, e.event, x11rb::CURRENT_TIME)
                        .ok();
                    conn.flush().ok();
                }
            }

            Event::MapNotify(e) => {
                if e.window == 0 || state.docks.contains(&e.window) {
                    continue;
                }
                if state.workspaces[state.current].contains(&e.window) {
                    wm::layout::apply_layout(&conn, state, width, height, config);
                    conn.flush().ok();
                }
            }

            Event::UnmapNotify(e) => {
                if state.docks.contains(&e.window) {
                    continue;
                }
                let in_current = state.workspaces[state.current].contains(&e.window);
                eprintln!("UnmapNotify: win={} in_current={}", e.window, in_current);
                if in_current {
                    state.remove(e.window);
                    wm::layout::apply_layout(&conn, state, width, height, config);
                    conn.flush().ok();
                }
            }

            Event::DestroyNotify(e) => {
                if state.docks.contains(&e.window) {
                    state.docks.retain(|&w| w != e.window);
                    continue;
                }
                let was_tracked = state.workspaces.iter().any(|ws| ws.contains(&e.window));
                let was_in_current = state.workspaces[state.current].contains(&e.window);
                eprintln!(
                    "DestroyNotify: win={} in_current={}",
                    e.window, was_in_current
                );
                if was_tracked {
                    state.remove(e.window);
                    if was_in_current {
                        wm::layout::apply_layout(&conn, state, width, height, config);
                        conn.flush().ok();
                    }
                }
            }

            Event::KeyPress(e) => {
                wm::keys::handle_key(&conn, state, &e, root, width, height, config, &atoms);
            }

            Event::ClientMessage(e) => {
                if e.type_ == atoms.net_current_desktop {
                    let idx = e.data.as_data32()[0] as usize;
                    state.switch_workspace(&conn, root, &atoms, idx);
                }
            }

            _ => {}
        }
    }
}

fn get_window_class<C: Connection>(conn: &C, window: Window) -> String {
    let atom = match conn
        .intern_atom(false, b"WM_CLASS")
        .ok()
        .and_then(|c| c.reply().ok())
    {
        Some(r) => r.atom,
        None => return String::new(),
    };
    match conn
        .get_property(false, window, atom, AtomEnum::STRING, 0, 64)
        .ok()
        .and_then(|c| c.reply().ok())
    {
        Some(r) => String::from_utf8_lossy(&r.value).to_string(),
        None => String::new(),
    }
}

fn get_window_type<C: Connection>(conn: &C, window: Window, atoms: &Atoms) -> Vec<u32> {
    conn.get_property(
        false,
        window,
        atoms.net_wm_window_type,
        AtomEnum::ATOM,
        0,
        32,
    )
    .ok()
    .and_then(|c| c.reply().ok())
    .and_then(|r| r.value32().map(|v| v.collect()))
    .unwrap_or_default()
}
