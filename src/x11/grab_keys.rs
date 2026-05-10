use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::*;

pub fn grab_keys<C: Connection>(conn: &C, root: Window) {
    let mod_mask = ModMask::M4;

    let keys = [
        10, // 1
        11, // 2
        12, // 3
    ];

    for key in keys {
        let _ = conn.grab_key(false, root, mod_mask, key, GrabMode::ASYNC, GrabMode::ASYNC);
    }
}
