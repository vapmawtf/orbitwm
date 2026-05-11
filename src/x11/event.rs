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
                | EventMask::ENTER_WINDOW,
        ),
    )
    .unwrap();

    grab_keys(&conn, root, config);

    ewmh::set_number_of_desktops(&conn, root, &atoms, 9);
    ewmh::set_current_desktop(&conn, root, &atoms, 0);

    conn.flush().unwrap();

    loop {
        let event = conn.wait_for_event().unwrap();

        match event {
            Event::MapRequest(e) => {
                let window_type = conn
                    .get_property(
                        false,
                        e.window,
                        atoms.net_wm_window_type,
                        AtomEnum::ATOM,
                        0,
                        32,
                    )
                    .unwrap()
                    .reply()
                    .unwrap();

                let is_dock = window_type
                    .value32()
                    .map(|mut v| v.any(|a| a == atoms.net_wm_window_type_dock))
                    .unwrap_or(false);

                if is_dock {
                    state.add_dock(e.window);
                    conn.map_window(e.window).unwrap();
                    conn.flush().unwrap();
                    continue;
                }

                conn.change_window_attributes(
                    e.window,
                    &ChangeWindowAttributesAux::default().event_mask(EventMask::ENTER_WINDOW),
                )
                .unwrap();

                conn.map_window(e.window).unwrap();
                state.add(e.window);
                wm::layout::apply_layout(&conn, state, width, height, config);
                conn.flush().unwrap();
            }

            Event::EnterNotify(e) => {
                if e.event != root && !state.docks.contains(&e.event) {
                    state.focused = Some(e.event);
                    conn.set_input_focus(InputFocus::POINTER_ROOT, e.event, x11rb::CURRENT_TIME)
                        .unwrap();
                    conn.flush().unwrap();
                }
            }

            Event::UnmapNotify(e) => {
                state.remove(e.window);
                wm::layout::apply_layout(&conn, state, width, height, config);
                conn.flush().unwrap();
            }

            Event::DestroyNotify(e) => {
                state.remove(e.window);
                wm::layout::apply_layout(&conn, state, width, height, config);
                conn.flush().unwrap();
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
