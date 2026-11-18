use crate::session::TerminalSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitMode {
    #[default]
    None,
    Horizontal,
    Vertical,
}

#[derive(Default)]
pub struct TerminalManager {
    sessions: Vec<TerminalSession>,
    active_idx: usize,
    split_idx: Option<usize>,
    split_mode: SplitMode,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            active_idx: 0,
            split_idx: None,
            split_mode: SplitMode::None,
        }
    }

    pub fn add_session(&mut self, session: TerminalSession) -> usize {
        let idx = self.sessions.len();
        self.sessions.push(session);
        self.active_idx = idx;

        if self.split_mode != SplitMode::None && self.split_idx.is_none() && self.sessions.len() >= 2 {
            self.split_idx = Some(if self.active_idx == 0 { 1 } else { 0 });
        }

        idx
    }

    pub fn close_session(&mut self, idx: usize) -> Option<TerminalSession> {
        if idx >= self.sessions.len() {
            return None;
        }

        let removed = self.sessions.remove(idx);

        if self.sessions.is_empty() {
            self.active_idx = 0;
            self.split_idx = None;
            self.split_mode = SplitMode::None;
        } else {
            // Adjust active_idx
            if self.active_idx > idx {
                self.active_idx -= 1;
            } else if self.active_idx >= self.sessions.len() {
                self.active_idx = self.sessions.len() - 1;
            }

            // Adjust split_idx
            if let Some(s_idx) = self.split_idx {
                if s_idx == idx {
                    if self.sessions.len() >= 2 {
                        self.split_idx = Some(if self.active_idx == 0 { 1 } else { 0 });
                    } else {
                        self.split_idx = None;
                        self.split_mode = SplitMode::None;
                    }
                } else if s_idx > idx {
                    self.split_idx = Some(s_idx - 1);
                }
            }

            // Ensure split_idx is valid and not equal to active_idx
            if let Some(s_idx) = self.split_idx {
                if s_idx == self.active_idx || s_idx >= self.sessions.len() {
                    if self.sessions.len() >= 2 {
                        self.split_idx = Some(if self.active_idx == 0 { 1 } else { 0 });
                    } else {
                        self.split_idx = None;
                        self.split_mode = SplitMode::None;
                    }
                }
            }
        }

        Some(removed)
    }

    pub fn active_session(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active_idx)
    }

    pub fn active_session_mut(&mut self) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(self.active_idx)
    }

    pub fn split_session(&self) -> Option<&TerminalSession> {
        if self.split_mode == SplitMode::None {
            return None;
        }
        let fallback_idx = if self.sessions.len() >= 2 {
            Some(if self.active_idx == 0 { 1 } else { 0 })
        } else {
            None
        };
        let split_i = self.split_idx.or(fallback_idx)?;

        if split_i < self.sessions.len() && split_i != self.active_idx {
            self.sessions.get(split_i)
        } else {
            None
        }
    }

    pub fn split_session_mut(&mut self) -> Option<&mut TerminalSession> {
        if self.split_mode == SplitMode::None {
            return None;
        }
        let active_i = self.active_idx;
        let fallback_idx = if self.sessions.len() >= 2 {
            Some(if active_i == 0 { 1 } else { 0 })
        } else {
            None
        };
        let split_i = self.split_idx.or(fallback_idx)?;

        if split_i < self.sessions.len() && split_i != active_i {
            self.sessions.get_mut(split_i)
        } else {
            None
        }
    }

    pub fn set_active_idx(&mut self, idx: usize) {
        if idx < self.sessions.len() {
            if self.split_mode != SplitMode::None && self.split_idx == Some(idx) {
                self.split_idx = Some(self.active_idx);
            }
            self.active_idx = idx;
        }
    }

    pub fn active_idx(&self) -> usize {
        self.active_idx
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn sessions(&self) -> &[TerminalSession] {
        &self.sessions
    }

    pub fn sessions_mut(&mut self) -> &mut [TerminalSession] {
        &mut self.sessions
    }

    pub fn get(&self, idx: usize) -> Option<&TerminalSession> {
        self.sessions.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(idx)
    }

    pub fn tab_title(&self, idx: usize) -> Option<String> {
        self.sessions
            .get(idx)
            .map(|s| format!("{}: {}", idx + 1, s.display_title()))
    }

    pub fn set_split(&mut self, split_idx: Option<usize>, mode: SplitMode) {
        self.split_mode = mode;
        if mode == SplitMode::None {
            self.split_idx = None;
        } else {
            let valid_split =
                split_idx.filter(|&idx| idx < self.sessions.len() && idx != self.active_idx);
            let fallback_idx = if self.sessions.len() >= 2 {
                Some(if self.active_idx == 0 { 1 } else { 0 })
            } else {
                None
            };
            self.split_idx = valid_split.or(fallback_idx);
        }
    }

    pub fn toggle_split(&mut self) {
        match self.split_mode {
            SplitMode::None => {
                self.split_mode = SplitMode::Vertical;
                if self.split_idx.is_none() && self.sessions.len() >= 2 {
                    self.split_idx = Some(if self.active_idx == 0 { 1 } else { 0 });
                }
            }
            SplitMode::Vertical => {
                self.split_mode = SplitMode::Horizontal;
            }
            SplitMode::Horizontal => {
                self.split_mode = SplitMode::None;
                self.split_idx = None;
            }
        }
    }

    pub fn split_mode(&self) -> SplitMode {
        self.split_mode
    }

    pub fn split_idx(&self) -> Option<usize> {
        self.split_idx
    }

    pub fn active_and_split_mut(
        &mut self,
    ) -> (Option<&mut TerminalSession>, Option<&mut TerminalSession>) {
        if self.sessions.is_empty() {
            return (None, None);
        }
        let active_i = self.active_idx;
        if self.split_mode == SplitMode::None {
            return (self.sessions.get_mut(active_i), None);
        }
        let split_i = self.split_idx.unwrap_or(if active_i == 0 { 1 } else { 0 });
        if split_i >= self.sessions.len() || split_i == active_i {
            return (self.sessions.get_mut(active_i), None);
        }
        if active_i < split_i {
            let (left, right) = self.sessions.split_at_mut(split_i);
            (left.get_mut(active_i), right.get_mut(0))
        } else {
            let (left, right) = self.sessions.split_at_mut(active_i);
            (right.get_mut(0), left.get_mut(split_i))
        }
    }

    pub fn remove_dead_sessions(&mut self) {
        let active_id = self.sessions.get(self.active_idx).map(|s| s.id);
        let split_id = self.split_idx.and_then(|i| self.sessions.get(i)).map(|s| s.id);

        self.sessions.retain(|s| s.is_alive());

        if self.sessions.is_empty() {
            self.active_idx = 0;
            self.split_idx = None;
            self.split_mode = SplitMode::None;
            return;
        }

        // Restore active_idx
        if let Some(id) = active_id {
            if let Some(pos) = self.sessions.iter().position(|s| s.id == id) {
                self.active_idx = pos;
            } else {
                self.active_idx = self.active_idx.min(self.sessions.len() - 1);
            }
        } else {
            self.active_idx = 0;
        }

        // Restore split_idx
        if self.split_mode != SplitMode::None {
            if self.sessions.len() < 2 {
                self.split_idx = None;
                self.split_mode = SplitMode::None;
            } else if let Some(id) = split_id {
                if let Some(pos) = self.sessions.iter().position(|s| s.id == id) {
                    if pos != self.active_idx {
                        self.split_idx = Some(pos);
                    } else {
                        self.split_idx = Some(if self.active_idx == 0 { 1 } else { 0 });
                    }
                } else {
                    self.split_idx = Some(if self.active_idx == 0 { 1 } else { 0 });
                }
            } else {
                self.split_idx = Some(if self.active_idx == 0 { 1 } else { 0 });
            }
        } else {
            self.split_idx = None;
        }
    }
}
