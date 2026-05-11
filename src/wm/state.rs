use std::collections::HashSet;
use x11rb::{CURRENT_TIME, connection::Connection, protocol::xproto::*};

const WORKSPACE_COUNT: usize = 9;

pub struct WMState {
    pub workspaces: [Vec<Window>; WORKSPACE_COUNT],
    pub current: usize,
    pub focused: Option<Window>,
    pub docks: Vec<Window>,
    pub floating: HashSet<Window>,
}

impl Default for WMState {
    fn default() -> Self {
        Self {
            workspaces: std::array::from_fn(|_| Vec::new()),
            current: 0,
            focused: None,
            docks: Vec::new(),
            floating: HashSet::new(),
        }
    }
}

impl WMState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn windows(&self) -> &[Window] {
        &self.workspaces[self.current]
    }

    pub fn windows_mut(&mut self) -> &mut Vec<Window> {
        &mut self.workspaces[self.current]
    }

    pub fn add(&mut self, win: Window) {
        if self.contains(win) || self.is_dock(win) {
            return;
        }
        self.windows_mut().push(win);
    }

    pub fn add_floating(&mut self, win: Window) {
        self.floating.insert(win);
    }

    pub fn is_floating(&self, win: Window) -> bool {
        self.floating.contains(&win)
    }

    pub fn remove(&mut self, win: Window) {
        for ws in self.workspaces.iter_mut() {
            ws.retain(|&w| w != win);
        }
        self.floating.remove(&win);
        if self.focused == Some(win) {
            self.focused = self.windows().first().copied();
        }
    }

    pub fn contains(&self, win: Window) -> bool {
        self.workspaces.iter().any(|ws| ws.contains(&win))
    }

    pub fn is_dock(&self, win: Window) -> bool {
        self.docks.contains(&win)
    }

    pub fn switch_workspace<C: Connection>(
        &mut self,
        conn: &C,
        root: Window,
        atoms: &crate::x11::atoms::Atoms,
        idx: usize,
    ) -> Result<(), x11rb::errors::ReplyError> {
        if idx >= WORKSPACE_COUNT || idx == self.current {
            return Ok(());
        }

        // Unmap all non-dock windows in the current workspace
        for &win in &self.workspaces[self.current] {
            if !self.is_dock(win) {
                conn.unmap_window(win)?;
            }
        }

        self.current = idx;

        // Map all non-dock windows in the new workspace
        for &win in &self.workspaces[self.current] {
            if !self.is_dock(win) {
                conn.map_window(win)?;
            }
        }

        self.focused = self.windows().first().copied();

        x11rb::wrapper::ConnectionExt::change_property32(
            conn,
            PropMode::REPLACE,
            root,
            atoms.net_current_desktop,
            AtomEnum::CARDINAL,
            &[idx as u32],
        )?;

        conn.flush()?;
        Ok(())
    }

    pub fn move_to_workspace<C: Connection>(
        &mut self,
        conn: &C,
        idx: usize,
    ) -> Result<(), x11rb::errors::ReplyError> {
        if idx >= WORKSPACE_COUNT || idx == self.current {
            return Ok(());
        }

        let Some(&win) = self.focused_or_first() else {
            return Ok(());
        };

        // Remove from current workspace
        self.windows_mut().retain(|&w| w != win);
        // Add to target workspace
        self.workspaces[idx].push(win);

        // If moving to the current workspace, remap immediately
        if idx == self.current {
            conn.map_window(win)?;
        } else {
            // Otherwise, unmap (will be remapped when switching to that workspace)
            conn.unmap_window(win)?;
        }

        // Update focused window
        self.focused = self.windows().first().copied();
        conn.flush()?;
        Ok(())
    }

    pub fn focused_or_first(&self) -> Option<&Window> {
        self.focused
            .and_then(|f| self.windows().iter().find(|&&w| w == f))
            .or_else(|| self.windows().first())
    }

    pub fn focus_next<C: Connection>(&mut self, conn: &C) -> Result<(), x11rb::errors::ReplyError> {
        if self.windows().is_empty() {
            return Ok(());
        }
        self.shift_focus(conn, 1)
    }

    pub fn focus_prev<C: Connection>(&mut self, conn: &C) -> Result<(), x11rb::errors::ReplyError> {
        if self.windows().is_empty() {
            return Ok(());
        }
        self.shift_focus(conn, -1)
    }

    fn shift_focus<C: Connection>(
        &mut self,
        conn: &C,
        dir: i32,
    ) -> Result<(), x11rb::errors::ReplyError> {
        let windows = self.windows();
        let len = windows.len();
        if len == 0 {
            return Ok(());
        }

        let current = self
            .focused
            .and_then(|f| windows.iter().position(|&w| w == f))
            .unwrap_or(0);

        let next = (current as i32 + dir).rem_euclid(len as i32) as usize;
        let win = windows[next];
        self.focused = Some(win);

        conn.set_input_focus(InputFocus::POINTER_ROOT, win, CURRENT_TIME)?;
        conn.configure_window(
            win,
            &ConfigureWindowAux::default().stack_mode(StackMode::ABOVE),
        )?;
        Ok(())
    }

    pub fn add_dock(&mut self, win: Window) {
        if !self.docks.contains(&win) {
            self.docks.push(win);
        }
    }
}
