use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::config::Config;
use crate::wm::state::WMState;

pub fn apply_layout<C: Connection>(
    conn: &C,
    wm: &WMState,
    screen_w: u32,
    screen_h: u32,
    config: &Config,
) {
    let wins: Vec<Window> = wm.windows().iter().copied().collect();
    let count = wins.len();
    if count == 0 {
        return;
    }

    let gap = config.gap;
    let bar_h = config.bar.height;
    let usable_h = screen_h - bar_h;
    let ratio = config.master_ratio;

    if count == 1 {
        place(
            conn,
            wins[0],
            gap,
            bar_h + gap,
            screen_w - gap * 2,
            usable_h - gap * 2,
            config,
        );
    } else {
        let master_w = (screen_w as f32 * ratio) as u32 - gap * 2;
        place(
            conn,
            wins[0],
            gap,
            bar_h + gap,
            master_w,
            usable_h - gap * 2,
            config,
        );

        let stack_count = (count - 1) as u32;
        let stack_x = (screen_w as f32 * ratio) as u32 + gap;
        let stack_w = (screen_w as f32 * (1.0 - ratio)) as u32 - gap * 2;
        let stack_h = (usable_h - gap * (stack_count + 1)) / stack_count;

        for (i, &win) in wins[1..].iter().enumerate() {
            let stack_y = bar_h + gap + i as u32 * (stack_h + gap);
            place(conn, win, stack_x, stack_y, stack_w, stack_h, config);
        }
    }

    conn.flush().unwrap();
}

fn place<C: Connection>(conn: &C, win: Window, x: u32, y: u32, w: u32, h: u32, config: &Config) {
    let bw = config.border_width;

    conn.configure_window(
        win,
        &ConfigureWindowAux::default()
            .x(x as i32)
            .y(y as i32)
            .width(w.saturating_sub(bw * 2))
            .height(h.saturating_sub(bw * 2))
            .border_width(bw)
            .stack_mode(StackMode::ABOVE),
    )
    .ok();
}
