use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, *};

use crate::config::Config;
use crate::wm::keys::MOD_KEY;

fn grab<C: Connection>(conn: &C, root: Window, modmask: ModMask, keycode: u8) {
    let _ = conn.grab_key(
        false,
        root,
        modmask,
        keycode,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    );
}

pub fn grab_keys<C: Connection>(conn: &C, root: Window, config: &Config) {
    let _ = conn.ungrab_key(0, root, ModMask::ANY);

    let workspace_keys: [u8; 9] = [10, 11, 12, 13, 14, 15, 16, 17, 18];

    for key in workspace_keys {
        grab(conn, root, MOD_KEY.into(), key);
        grab(conn, root, (MOD_KEY | ModMask::SHIFT).into(), key);
    }

    let bindings: &[(ModMask, u8)] = &[
        (MOD_KEY.into(), 24),
        ((MOD_KEY | ModMask::SHIFT).into(), 24),
        (MOD_KEY.into(), 25),
        (MOD_KEY.into(), 44),
        (MOD_KEY.into(), 46),
        (MOD_KEY.into(), 36),
    ];

    for &(modmask, keycode) in bindings {
        grab(conn, root, modmask, keycode);
    }

    for ck in &config.custom_keys {
        let (modmask, keycode) = crate::wm::keys::parse_binding(&ck.binding);
        if keycode != 0 {
            grab(conn, root, modmask.into(), keycode);
        }
    }

    let _ = conn.flush();
}
