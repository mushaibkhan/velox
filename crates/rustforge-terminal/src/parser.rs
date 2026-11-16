use crate::grid::{Color, TerminalGrid};
use vte::Perform;

pub struct GridPerformer<'a> {
    pub grid: &'a mut TerminalGrid,
}

impl<'a> Perform for GridPerformer<'a> {
    fn print(&mut self, c: char) {
        self.grid.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.grid.newline(),
            b'\r' => self.grid.carriage_return(),
            8 => self.grid.backspace(),
            9 => {
                // Tab
                let spaces = 4 - (self.grid.cursor_x % 4);
                for _ in 0..spaces {
                    self.grid.write_char(' ');
                }
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'm' => {
                // SGR Color / Style
                if params.is_empty() {
                    self.grid.current_fg = Color::WHITE;
                    self.grid.current_bg = Color::BLACK;
                    self.grid.current_bold = false;
                    self.grid.current_underline = false;
                    return;
                }

                for param in params.iter() {
                    let code = param[0];
                    match code {
                        0 => {
                            self.grid.current_fg = Color::WHITE;
                            self.grid.current_bg = Color::BLACK;
                            self.grid.current_bold = false;
                            self.grid.current_underline = false;
                        }
                        1 => self.grid.current_bold = true,
                        4 => self.grid.current_underline = true,
                        22 => self.grid.current_bold = false,
                        24 => self.grid.current_underline = false,
                        30 => self.grid.current_fg = Color::BLACK,
                        31 => self.grid.current_fg = Color::RED,
                        32 => self.grid.current_fg = Color::GREEN,
                        33 => self.grid.current_fg = Color::YELLOW,
                        34 => self.grid.current_fg = Color::BLUE,
                        35 => self.grid.current_fg = Color::MAGENTA,
                        36 => self.grid.current_fg = Color::CYAN,
                        37 => self.grid.current_fg = Color::WHITE,
                        39 => self.grid.current_fg = Color::WHITE,
                        40 => self.grid.current_bg = Color::BLACK,
                        41 => self.grid.current_bg = Color::RED,
                        42 => self.grid.current_bg = Color::GREEN,
                        43 => self.grid.current_bg = Color::YELLOW,
                        44 => self.grid.current_bg = Color::BLUE,
                        45 => self.grid.current_bg = Color::MAGENTA,
                        46 => self.grid.current_bg = Color::CYAN,
                        47 => self.grid.current_bg = Color::WHITE,
                        49 => self.grid.current_bg = Color::BLACK,
                        _ => {}
                    }
                }
            }
            'H' | 'f' => {
                // Cursor position
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first()).copied().unwrap_or(1).saturating_sub(1);
                let col = iter.next().and_then(|p| p.first()).copied().unwrap_or(1).saturating_sub(1);
                self.grid.cursor_y = (row as usize).min(self.grid.rows.saturating_sub(1));
                self.grid.cursor_x = (col as usize).min(self.grid.cols.saturating_sub(1));
            }
            'J' => {
                // Erase in display
                let code = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(0);
                if code == 2 || code == 3 {
                    self.grid.clear_screen();
                }
            }
            'K' => {
                // Erase in line
                self.grid.clear_line();
            }
            _ => {}
        }
    }
}
