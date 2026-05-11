use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::config::Config;
use crate::wm::keys::parse_binding;

pub fn grab_keys<C: Connection>(conn: &C, root: Window, config: &Config) {
    let mut bindings: Vec<&str> = vec![
        &config.keybinds.close_window,
        &config.keybinds.quit,
        &config.keybinds.restart,
        &config.keybinds.swap_master,
        &config.keybinds.focus_next,
        &config.keybinds.focus_prev,
        &config.keybinds.terminal,
        &config.keybinds.workspace_switch,
        &config.keybinds.workspace_move,
    ];

    for ck in &config.custom_keys {
        bindings.push(&ck.binding);
    }

    let workspace_keycodes: Vec<u8> = (0x0a..=0x12).collect();

    let switch_mod = crate::wm::keys::parse_binding(&config.keybinds.workspace_switch).0;
    let move_mod = crate::wm::keys::parse_binding(&config.keybinds.workspace_move).0;

    for keycode in &workspace_keycodes {
        conn.grab_key(
            false,
            root,
            ModMask::from(switch_mod),
            *keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )
        .unwrap();

        conn.grab_key(
            false,
            root,
            ModMask::from(move_mod),
            *keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )
        .unwrap();
    }

    for binding in bindings {
        let (modmask, keycode) = parse_binding(binding);
        if keycode == 0 {
            continue;
        }
        conn.grab_key(
            false,
            root,
            ModMask::from(modmask),
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )
        .unwrap();
    }

    conn.flush().unwrap();
}
