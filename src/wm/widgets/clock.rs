use std::time::{SystemTime, UNIX_EPOCH};

use x11rb::{protocol::xproto::*, rust_connection::RustConnection};

use crate::wm::bar::{Align, Widget};

pub struct ClockWidget {
    text: String,
}

impl ClockWidget {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn align(&self) -> Align {
        Align::Right
    }

    pub fn update_time(&mut self) {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.text = format!(
            "{:02}:{:02}:{:02}",
            (secs / 3600) % 24,
            (secs / 60) % 60,
            secs % 60
        );
    }
}

impl Widget for ClockWidget {
    fn width(&self) -> u32 {
        100
    }

    fn update(&mut self) {
        self.update_time();
    }

    fn draw(&mut self, conn: &RustConnection, window: Window, gc: Gcontext, x: i16, height: u32) {
        conn.image_text8(window, gc, x, (height as i16 / 2) + 4, self.text.as_bytes())
            .unwrap();
    }
}
