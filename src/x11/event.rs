use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use crate::config::Config;
use crate::x11::ewmh;
use crate::x11::ewmh::EwmhAtoms;
use crate::x11::grab_keys::grab_keys;

pub fn start_x11_wm(config: Config) {
    let (conn, screen_num) = RustConnection::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atoms = ewmh::init_atoms(&conn);

    setup(&conn, root, &atoms, &config);

    loop {
        let event = conn.wait_for_event().unwrap();
        handle_event(&conn, event, root, &atoms, &config);
    }
}

fn setup(conn: &RustConnection, root: Window, atoms: &EwmhAtoms, _config: &Config) {
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::default().event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE,
        ),
    )
    .unwrap();

    grab_keys(conn, root);

    // EWMH dla polybara
    ewmh::set_number_of_desktops(conn, root, atoms, 5);
    ewmh::set_current_desktop(conn, root, atoms, 0);

    conn.flush().unwrap();
}

fn handle_event(
    conn: &RustConnection,
    event: Event,
    root: Window,
    atoms: &EwmhAtoms,
    _config: &Config,
) {
    match event {
        Event::MapRequest(e) => {
            conn.map_window(e.window).unwrap();
        }

        Event::KeyPress(e) => {
            println!("Key: {}", e.detail);
        }

        Event::ClientMessage(e) => {
            if e.type_ == atoms.net_current_desktop {
                let desktop = e.data.as_data32()[0];
                ewmh::set_current_desktop(conn, root, atoms, desktop);
                conn.flush().unwrap();
            }
        }

        _ => {}
    }
}
