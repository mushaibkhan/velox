use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 220, g: 220, b: 220 };
    pub const BLACK: Self = Self { r: 24, g: 24, b: 24 };
    pub const RED: Self = Self { r: 235, g: 87, b: 87 };
    pub const GREEN: Self = Self { r: 39, g: 174, b: 96 };
    pub const YELLOW: Self = Self { r: 242, g: 201, b: 76 };
    pub const BLUE: Self = Self { r: 47, g: 128, b: 237 };
    pub const MAGENTA: Self = Self { r: 187, g: 107, b: 217 };
    pub const CYAN: Self = Self { r: 86, g: 204, b: 242 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::WHITE,
            bg: Color::BLACK,
            bold: false,
            underline: false,
        }
    }
}

pub struct TerminalGrid {
    pub cols: usize,
    pub rows: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub current_fg: Color,
    pub current_bg: Color,
    pub current_bold: bool,
    pub current_underline: bool,
    pub cells: Vec<Vec<Cell>>,
    pub scrollback: VecDeque<Vec<Cell>>,
    pub max_scrollback: usize,
}

impl TerminalGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cols = cols.max(10);
        let rows = rows.max(5);
        let cells = vec![vec![Cell::default(); cols]; rows];
        Self {
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            current_fg: Color::WHITE,
            current_bg: Color::BLACK,
            current_bold: false,
            current_underline: false,
            cells,
            scrollback: VecDeque::new(),
            max_scrollback: 2000,
        }
    }

    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        let new_cols = new_cols.max(10);
        let new_rows = new_rows.max(5);
        if self.cols == new_cols && self.rows == new_rows {
            return;
        }

        let mut new_cells = vec![vec![Cell::default(); new_cols]; new_rows];
        for y in 0..new_rows.min(self.rows) {
            for x in 0..new_cols.min(self.cols) {
                new_cells[y][x] = self.cells[y][x];
            }
        }

        self.cols = new_cols;
        self.rows = new_rows;
        self.cells = new_cells;
        self.cursor_x = self.cursor_x.min(new_cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(new_rows.saturating_sub(1));
    }

    pub fn write_char(&mut self, ch: char) {
        if self.cursor_y >= self.rows {
            self.scroll_up();
        }

        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.cursor_y += 1;
            if self.cursor_y >= self.rows {
                self.scroll_up();
            }
        }

        self.cells[self.cursor_y][self.cursor_x] = Cell {
            ch,
            fg: self.current_fg,
            bg: self.current_bg,
            bold: self.current_bold,
            underline: self.current_underline,
        };
        self.cursor_x += 1;
    }

    pub fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.scroll_up();
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor_x = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if !self.cells.is_empty() {
            let removed = self.cells.remove(0);
            self.scrollback.push_back(removed);
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.pop_front();
            }
            self.cells.push(vec![Cell::default(); self.cols]);
        }
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn clear_line(&mut self) {
        if self.cursor_y < self.rows {
            for cell in &mut self.cells[self.cursor_y] {
                *cell = Cell::default();
            }
            self.cursor_x = 0;
        }
    }
}
