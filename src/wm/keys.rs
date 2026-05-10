use x11rb::{connection::Connection, protocol::xproto::*};

use crate::config::Config;
use crate::wm::layout::apply_layout;
use crate::wm::state::WMState;

pub const MOD_KEY: u16 = 0x0040;

pub fn handle_key<C: Connection>(
    conn: &C,
    wm: &mut WMState,
    event: &KeyPressEvent,
    width: u32,
    height: u32,
    config: &Config,
) {
    let modmask = u16::from(event.state) & 0x00FF;
    let keycode = event.detail;
    let kb = &config.keybinds;

    let switch_mod = parse_modmask(&kb.workspace_switch);
    let move_mod = parse_modmask(&kb.workspace_move);

    if let Some(idx) = keycode_to_workspace(keycode) {
        if modmask == move_mod {
            wm.move_to_workspace(conn, idx);
            apply_layout(conn, wm, width, height, config);
            conn.flush().unwrap();
            return;
        } else if modmask == switch_mod {
            wm.switch_workspace(conn, idx);
            apply_layout(conn, wm, width, height, config);
            conn.flush().unwrap();
            return;
        }
    }

    if matches_key(modmask, keycode, &kb.close_window) {
        if let Some(&win) = wm.focused_or_first() {
            close_window(conn, win);
        }
    } else if matches_key(modmask, keycode, &kb.quit) {
        println!("OrbitWM: quit");
        std::process::exit(0);
    } else if matches_key(modmask, keycode, &kb.restart) {
        restart_wm();
    } else if matches_key(modmask, keycode, &kb.swap_master) {
        if wm.windows().len() >= 2 {
            wm.workspaces[wm.current].swap(0, 1);
            apply_layout(conn, wm, width, height, config);
            conn.flush().unwrap();
        }
    } else if matches_key(modmask, keycode, &kb.focus_next) {
        wm.focus_next(conn);
        conn.flush().unwrap();
    } else if matches_key(modmask, keycode, &kb.focus_prev) {
        wm.focus_prev(conn);
        conn.flush().unwrap();
    } else if matches_key(modmask, keycode, &kb.terminal) {
        spawn_cmd(
            &config.terminal,
            &std::env::var("DISPLAY").unwrap_or(":0".to_string()),
        );
    } else {
        // custom keybindy
        handle_custom(conn, wm, modmask, keycode, width, height, config);
    }
}

fn handle_custom<C: Connection>(
    conn: &C,
    wm: &mut WMState,
    modmask: u16,
    keycode: u8,
    width: u32,
    height: u32,
    config: &Config,
) {
    let display = std::env::var("DISPLAY").unwrap_or(":0".to_string());

    for ck in &config.custom_keys {
        if !matches_key(modmask, keycode, &ck.binding) {
            continue;
        }

        let arg = ck.arg.clone().unwrap_or_default();

        match ck.action.as_str() {
            "exec" => {
                spawn_cmd(&arg, &display);
            }

            "master_ratio_inc" => {
                println!("master_ratio_inc not yet mutable at runtime");
            }

            "workspace" => {
                if let Ok(idx) = arg.parse::<usize>() {
                    if idx >= 1 {
                        wm.switch_workspace(conn, idx - 1);
                        apply_layout(conn, wm, width, height, config);
                        conn.flush().unwrap();
                    }
                }
            }

            "move_to_workspace" => {
                if let Ok(idx) = arg.parse::<usize>() {
                    if idx >= 1 {
                        wm.move_to_workspace(conn, idx - 1);
                        apply_layout(conn, wm, width, height, config);
                        conn.flush().unwrap();
                    }
                }
            }

            "close" => {
                if let Some(&win) = wm.focused_or_first() {
                    close_window(conn, win);
                }
            }

            "swap_master" => {
                if wm.windows().len() >= 2 {
                    wm.workspaces[wm.current].swap(0, 1);
                    apply_layout(conn, wm, width, height, config);
                    conn.flush().unwrap();
                }
            }

            "quit" => std::process::exit(0),

            "restart" => restart_wm(),

            unknown => eprintln!("OrbitWM: unknown action '{unknown}'"),
        }

        break;
    }
}

fn spawn_cmd(cmd: &str, display: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let Some((&bin, args)) = parts.split_first() else {
        return;
    };
    std::thread::spawn({
        let bin = bin.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let display = display.to_string();
        move || {
            std::process::Command::new(&bin)
                .args(&args)
                .env("DISPLAY", &display)
                .spawn()
                .ok();
        }
    });
}

fn matches_key(modmask: u16, keycode: u8, binding: &str) -> bool {
    let (bmask, bkey) = parse_binding(binding);
    modmask == bmask && keycode == bkey
}

pub(crate) fn parse_binding(binding: &str) -> (u16, u8) {
    let mut modmask: u16 = 0;
    let mut key = "";

    for part in binding.split('+') {
        match part.trim().to_lowercase().as_str() {
            "super" | "mod4" => modmask |= MOD_KEY,
            "shift" => modmask |= u16::from(ModMask::SHIFT),
            "ctrl" | "control" => modmask |= u16::from(ModMask::CONTROL),
            "alt" | "mod1" => modmask |= u16::from(ModMask::M1),
            k => key = Box::leak(k.to_string().into_boxed_str()),
        }
    }

    (modmask, key_name_to_keycode(key))
}

fn parse_modmask(s: &str) -> u16 {
    let mut mask: u16 = 0;
    for part in s.split('+') {
        match part.trim().to_lowercase().as_str() {
            "super" | "mod4" => mask |= MOD_KEY,
            "shift" => mask |= u16::from(ModMask::SHIFT),
            "ctrl" | "control" => mask |= u16::from(ModMask::CONTROL),
            "alt" | "mod1" => mask |= u16::from(ModMask::M1),
            _ => {}
        }
    }
    mask
}

fn keycode_to_workspace(keycode: u8) -> Option<usize> {
    match keycode {
        0x0a => Some(0),
        0x0b => Some(1),
        0x0c => Some(2),
        0x0d => Some(3),
        0x0e => Some(4),
        0x0f => Some(5),
        0x10 => Some(6),
        0x11 => Some(7),
        0x12 => Some(8),
        _ => None,
    }
}

fn key_name_to_keycode(key: &str) -> u8 {
    match key {
        "q" => 0x18,
        "w" => 0x1a,
        "e" => 0x1c,
        "r" => 0x1b,
        "j" => 0x2c,
        "k" => 0x2e,
        "l" => 0x2f,
        "return" | "enter" => 0x24,
        "space" => 0x41,
        _ => 0x00,
    }
}

fn restart_wm() {
    println!("OrbitWM: restarting...");
    let exe = std::env::current_exe().expect("Failed to get exe path");
    let args: Vec<String> = std::env::args().collect();
    std::process::Command::new(exe)
        .args(&args[1..])
        .spawn()
        .unwrap();
    std::process::exit(0);
}

fn close_window<C: Connection>(conn: &C, win: Window) {
    let wm_protocols = conn
        .intern_atom(false, b"WM_PROTOCOLS")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let wm_delete = conn
        .intern_atom(false, b"WM_DELETE_WINDOW")
        .unwrap()
        .reply()
        .unwrap()
        .atom;

    let protocols = conn
        .get_property(false, win, wm_protocols, AtomEnum::ATOM, 0, 32)
        .unwrap()
        .reply()
        .unwrap();

    let supports_delete = protocols
        .value32()
        .map(|i| i.collect::<Vec<_>>())
        .unwrap_or_default()
        .contains(&wm_delete);

    if supports_delete {
        let event = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: win,
            type_: wm_protocols,
            data: ClientMessageData::from([wm_delete, 0, 0, 0, 0]),
        };
        conn.send_event(false, win, EventMask::NO_EVENT, event)
            .unwrap();
    } else {
        conn.kill_client(win).unwrap();
    }
}
