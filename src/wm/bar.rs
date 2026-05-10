use x11rb::{connection::Connection, protocol::xproto::*, rust_connection::RustConnection};

use crate::config::Config;
use crate::wm::widgets::{clock::ClockWidget, workspaces::WorkspacesWidget};

#[derive(Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Right,
}
pub trait Widget {
    fn width(&self) -> u32;
    fn update(&mut self);
    fn align(&self) -> Align {
        Align::Left
    }

    fn draw(&mut self, conn: &RustConnection, window: Window, gc: Gcontext, x: i16, height: u32);
}

const PADDING_X: i16 = 10;

pub struct Bar {
    pub window: Window,
    pub bg_gc: Gcontext,
    pub fg_gc: Gcontext,
    pub width: u32,
    pub height: u32,
    pub widgets_left: Vec<Box<dyn Widget>>,
    pub widgets_right: Vec<Box<dyn Widget>>,
    pub config: Config,
    pub font: Font,
}

impl Bar {
    pub fn create(
        conn: &RustConnection,
        root: Window,
        screen: &Screen,
        config: &Config,
    ) -> Option<Self> {
        let width = screen.width_in_pixels as u32;
        let height = config.bar.height;

        let window = conn.generate_id().ok()?;
        let bg_gc = conn.generate_id().ok()?;
        let fg_gc = conn.generate_id().ok()?;
        let font = conn.generate_id().ok()?;

        let mut widgets_left: Vec<Box<dyn Widget>> = Vec::new();
        let mut widgets_right: Vec<Box<dyn Widget>> = Vec::new();

        widgets_left.push(Box::new(WorkspacesWidget::new(5)));
        widgets_right.push(Box::new(ClockWidget::new()));

        conn.open_font(
            font,
            b"-misc-fixed-medium-r-semicondensed--0-0-75-75-c-0-iso8859-1",
        )
        .ok()?;

        conn.create_window(
            screen.root_depth,
            window,
            root,
            0,
            0,
            width as u16,
            height as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::default()
                .background_pixel(config.bar.bg_color)
                .event_mask(EventMask::EXPOSURE),
        )
        .ok()?;

        conn.create_gc(
            bg_gc,
            window,
            &CreateGCAux::default()
                .foreground(config.bar.bg_color)
                .background(config.bar.bg_color),
        )
        .ok()?;

        conn.create_gc(
            fg_gc,
            window,
            &CreateGCAux::default()
                .foreground(config.bar.fg_color)
                .background(config.bar.bg_color)
                .font(font),
        )
        .ok()?;

        conn.map_window(window).ok()?;

        Some(Self {
            window,
            bg_gc,
            fg_gc,
            font,
            width,
            height,
            widgets_left,
            widgets_right,
            config: config.clone(),
        })
    }

    pub fn draw(&mut self, conn: &RustConnection) {
        conn.poly_fill_rectangle(
            self.window,
            self.bg_gc,
            &[Rectangle {
                x: 0,
                y: 0,
                width: self.width as u16,
                height: self.height as u16,
            }],
        )
        .unwrap();

        let mut x_left = PADDING_X;
        let mut x_right = self.width as i16 - PADDING_X;

        // LEFT
        for w in self.widgets_left.iter_mut() {
            w.update();
            w.draw(conn, self.window, self.fg_gc, x_left, self.height);
            x_left += w.width() as i16 + PADDING_X;
        }

        // RIGHT
        for w in self.widgets_right.iter_mut().rev() {
            w.update();

            let w_width = w.width() as i16;
            x_right -= w_width;

            w.draw(conn, self.window, self.fg_gc, x_right, self.height);

            x_right -= PADDING_X;
        }

        conn.flush().unwrap();
    }
}
