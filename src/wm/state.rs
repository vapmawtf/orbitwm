use x11rb::{CURRENT_TIME, connection::Connection, protocol::xproto::*};

const WORKSPACE_COUNT: usize = 9;

pub struct WMState {
    pub workspaces: [Vec<Window>; WORKSPACE_COUNT],
    pub current: usize,
    pub focused: Option<Window>,
    pub docks: Vec<Window>,
}

impl Default for WMState {
    fn default() -> Self {
        Self {
            workspaces: std::array::from_fn(|_| Vec::new()),
            current: 0,
            focused: None,
            docks: Vec::new(),
        }
    }
}

impl WMState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn windows(&self) -> &Vec<Window> {
        &self.workspaces[self.current]
    }

    pub fn windows_mut(&mut self) -> &mut Vec<Window> {
        &mut self.workspaces[self.current]
    }

    pub fn add(&mut self, win: Window) {
        if !self.windows().contains(&win) {
            self.windows_mut().push(win);
        }
    }

    pub fn remove(&mut self, win: Window) {
        for ws in self.workspaces.iter_mut() {
            ws.retain(|&w| w != win);
        }
        if self.focused == Some(win) {
            self.focused = self.workspaces[self.current].first().copied();
        }
    }

    pub fn switch_workspace<C: Connection>(
        &mut self,
        conn: &C,
        root: Window,
        atoms: &crate::x11::atoms::Atoms,
        idx: usize,
    ) {
        if idx >= WORKSPACE_COUNT || idx == self.current {
            return;
        }

        for &win in &self.workspaces[self.current] {
            if !self.docks.contains(&win) {
                conn.unmap_window(win).unwrap();
            }
        }

        self.current = idx;

        for &win in &self.workspaces[self.current] {
            if !self.docks.contains(&win) {
                conn.map_window(win).unwrap();
            }
        }

        self.focused = self.workspaces[self.current].first().copied();

        x11rb::wrapper::ConnectionExt::change_property32(
            &conn,
            PropMode::REPLACE,
            root,
            atoms.net_current_desktop,
            AtomEnum::CARDINAL,
            &[idx as u32],
        )
        .unwrap();

        conn.flush().unwrap();
    }

    pub fn move_to_workspace<C: Connection>(&mut self, conn: &C, idx: usize) {
        if idx >= WORKSPACE_COUNT || idx == self.current {
            return;
        }

        let Some(&win) = self.focused_or_first() else {
            return;
        };

        self.windows_mut().retain(|&w| w != win);
        self.workspaces[idx].push(win);
        conn.unmap_window(win).unwrap();

        self.focused = self.windows().first().copied();
        conn.flush().unwrap();
    }

    pub fn focused_or_first(&self) -> Option<&Window> {
        self.focused
            .and_then(|f| self.windows().iter().find(|&&w| w == f))
            .or_else(|| self.windows().first())
    }

    pub fn focus_next<C: Connection>(&mut self, conn: &C) {
        self.shift_focus(conn, 1);
    }

    pub fn focus_prev<C: Connection>(&mut self, conn: &C) {
        self.shift_focus(conn, -1);
    }

    fn shift_focus<C: Connection>(&mut self, conn: &C, dir: i32) {
        let len = self.windows().len();
        if len == 0 {
            return;
        }

        let current = self
            .focused
            .and_then(|f| self.windows().iter().position(|&w| w == f))
            .unwrap_or(0);

        let next = (current as i32 + dir).rem_euclid(len as i32) as usize;
        let win = self.windows()[next];
        self.focused = Some(win);

        conn.set_input_focus(InputFocus::POINTER_ROOT, win, CURRENT_TIME)
            .unwrap();
        conn.configure_window(
            win,
            &ConfigureWindowAux::default().stack_mode(StackMode::ABOVE),
        )
        .unwrap();
    }

    pub fn add_dock(&mut self, win: Window) {
        if !self.docks.contains(&win) {
            self.docks.push(win);
        }
    }
}
