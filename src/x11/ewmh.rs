use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

pub struct EwmhAtoms {
    pub net_number_of_desktops: Atom,
    pub net_current_desktop: Atom,
}

pub fn init_atoms<C: Connection>(conn: &C) -> EwmhAtoms {
    macro_rules! atom {
        ($name:expr) => {
            conn.intern_atom(false, $name.as_bytes())
                .unwrap()
                .reply()
                .unwrap()
                .atom
        };
    }

    EwmhAtoms {
        net_number_of_desktops: atom!("_NET_NUMBER_OF_DESKTOPS"),
        net_current_desktop: atom!("_NET_CURRENT_DESKTOP"),
    }
}

pub fn set_number_of_desktops<C: Connection>(
    conn: &C,
    root: Window,
    atoms: &EwmhAtoms,
    count: u32,
) {
    x11rb::wrapper::ConnectionExt::change_property32(
        &conn,
        PropMode::REPLACE,
        root,
        atoms.net_number_of_desktops,
        AtomEnum::CARDINAL,
        &[count],
    )
    .unwrap();
}

pub fn set_current_desktop<C: Connection>(conn: &C, root: Window, atoms: &EwmhAtoms, desktop: u32) {
    x11rb::wrapper::ConnectionExt::change_property32(
        &conn,
        PropMode::REPLACE,
        root,
        atoms.net_current_desktop,
        AtomEnum::CARDINAL,
        &[desktop],
    )
    .unwrap();
}
