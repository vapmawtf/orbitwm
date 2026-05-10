use x11rb::{connection::Connection, protocol::xproto::*, rust_connection::RustConnection};

use crate::wm::bar::Widget;

pub struct WorkspacesWidget {
    pub count: u32,
    pub active: u32,
}

impl WorkspacesWidget {
    pub fn new(count: u32) -> Self {
        Self { count, active: 1 }
    }
}

impl Widget for WorkspacesWidget {
    fn width(&self) -> u32 {
        self.count * 28
    }

    fn update(&mut self) {
        // later: sync with WM state
    }

    fn draw(&mut self, conn: &RustConnection, window: Window, gc: Gcontext, x: i16, height: u32) {
        let mut offset = x;

        for i in 1..=self.count {
            let is_active = i == self.active;

            // button background
            let bg = if is_active { 0x8888ff } else { 0x2a2a3e };

            let btn_gc = conn.generate_id().unwrap();
            conn.create_gc(btn_gc, window, &CreateGCAux::default().foreground(bg))
                .unwrap();

            conn.poly_fill_rectangle(
                window,
                btn_gc,
                &[Rectangle {
                    x: offset,
                    y: 3,
                    width: 24,
                    height: (height - 6) as u16,
                }],
            )
            .unwrap();

            // number (simple fallback without font system)
            conn.image_text8(
                window,
                gc,
                offset + 8,
                (height as i16 / 2) + 4,
                format!("{}", i).as_bytes(),
            )
            .unwrap();

            offset += 28;
        }
    }
}
