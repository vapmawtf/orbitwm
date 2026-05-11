use x11rb::{connection::Connection, protocol::xproto::*, rust_connection::RustConnection};

pub struct Atoms {
    pub net_number_of_desktops: Atom,
    pub net_current_desktop: Atom,
    pub net_wm_window_type: Atom,
    pub net_wm_window_type_dock: Atom,
    pub net_wm_window_type_desktop: Atom,
    pub net_wm_state: Atom,
    pub net_wm_state_above: Atom,
    pub net_wm_state_sticky: Atom,
}

impl Atoms {
    pub fn init<C: Connection>(conn: &C) -> Self {
        macro_rules! atom {
            ($name:expr) => {
                conn.intern_atom(false, $name.as_bytes())
                    .unwrap()
                    .reply()
                    .unwrap()
                    .atom
            };
        }

        Atoms {
            net_number_of_desktops: atom!("_NET_NUMBER_OF_DESKTOPS"),
            net_current_desktop: atom!("_NET_CURRENT_DESKTOP"),
            net_wm_window_type: atom!("_NET_WM_WINDOW_TYPE"),
            net_wm_window_type_dock: atom!("_NET_WM_WINDOW_TYPE_DOCK"),
            net_wm_window_type_desktop: atom!("_NET_WM_WINDOW_TYPE_DESKTOP"),
            net_wm_state: atom!("_NET_WM_STATE"),
            net_wm_state_above: atom!("_NET_WM_STATE_ABOVE"),
            net_wm_state_sticky: atom!("_NET_WM_STATE_STICKY"),
        }
    }
}

pub fn get_atom(conn: &RustConnection, name: &str) -> Atom {
    conn.intern_atom(false, name.as_bytes())
        .unwrap()
        .reply()
        .unwrap()
        .atom
}

pub fn atom_name(conn: &RustConnection, atom: Atom) -> Option<String> {
    let reply = conn.get_atom_name(atom).unwrap().reply().unwrap();
    String::from_utf8(reply.name).ok()
}
