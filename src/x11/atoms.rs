use std::collections::HashMap;
use x11rb::{connection::Connection, protocol::xproto::*, rust_connection::RustConnection};

pub struct Atoms {
    map: HashMap<String, Atom>,
}

impl Atoms {
    pub fn new(conn: &RustConnection) -> Self {
        let mut atoms = Atoms {
            map: HashMap::new(),
        };

        let names = [
            "_NET_WM_WINDOW_TYPE",
            "_NET_WM_WINDOW_TYPE_DOCK",
            "_NET_WM_WINDOW_TYPE_DESKTOP",
            "_NET_WM_STATE",
            "_NET_WM_STATE_ABOVE",
            "_NET_WM_STATE_STICKY",
        ];

        for name in names {
            let atom = conn
                .intern_atom(false, name.as_bytes())
                .unwrap()
                .reply()
                .unwrap()
                .atom;

            atoms.map.insert(name.to_string(), atom);
        }

        atoms
    }

    pub fn get(&self, name: &str) -> Option<Atom> {
        self.map.get(name).copied()
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
