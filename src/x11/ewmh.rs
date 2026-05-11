use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::x11::atoms::Atoms;

pub fn set_number_of_desktops<C: Connection>(conn: &C, root: Window, atoms: &Atoms, count: u32) {
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

pub fn set_current_desktop<C: Connection>(conn: &C, root: Window, atoms: &Atoms, desktop: u32) {
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
