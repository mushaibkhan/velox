use crate::cursor::{normalize_selections, Direction};
use crate::find::FindState;
use rustforge_core::{Position, Selection};
use rustforge_syntax::{Language, SyntaxHighlighter, TokenSpan};
use rustforge_text_buffer::TextBuffer;

pub struct EditorState {
    pub buffer: TextBuffer,
    pub cursors: Vec<Selection>,
    pub primary_cursor_idx: usize,
    pub highlighter: SyntaxHighlighter,
    pub language: Language,
    pub find_state: FindState,
    pub show_find: bool,
    pub scroll_line: usize,
    pub visible_lines: usize,
    pub tab_size: usize,
}

impl EditorState {
    pub fn new(buffer: TextBuffer) -> Self {
        let language = if let Some(path) = buffer.path() {
            Language::from_path(path)
        } else {
            Language::PlainText
        };

        Self {
            buffer,
            cursors: vec![Selection::point(0)],
            primary_cursor_idx: 0,
            highlighter: SyntaxHighlighter::new(language),
            language,
            find_state: FindState::new(),
            show_find: false,
            scroll_line: 0,
            visible_lines: 40,
            tab_size: 4,
        }
    }

    pub fn primary_cursor(&self) -> Selection {
        self.cursors.get(self.primary_cursor_idx).copied().unwrap_or(Selection::point(0))
    }

    pub fn primary_position(&self) -> Position {
        self.buffer.char_to_position(self.primary_cursor().head)
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        self.highlighter.set_language(language);
    }

    // Viewport lines helper
    pub fn visible_line_range(&self) -> std::ops::Range<usize> {
        let start = self.scroll_line;
        let end = (start + self.visible_lines).min(self.buffer.line_count());
        start..end
    }

    pub fn highlight_line(&self, line_idx: usize) -> Vec<TokenSpan> {
        if let Some(line_str) = self.buffer.line_to_string(line_idx) {
            let line_start_char = self.buffer.line_to_char(line_idx);
            self.highlighter.highlight_line(&line_str, line_start_char)
        } else {
            Vec::new()
        }
    }

    // Text Insertion
    pub fn insert_text(&mut self, text: &str) {
        let mut new_cursors = Vec::with_capacity(self.cursors.len());
        let mut sorted_indices: Vec<usize> = (0..self.cursors.len()).collect();
        sorted_indices.sort_by_key(|&i| std::cmp::Reverse(self.cursors[i].start()));

        for &idx in &sorted_indices {
            let cursor = self.cursors[idx];
            let start = cursor.start();
            let end = cursor.end();

            if start != end {
                self.buffer.delete_range(start, end, self.cursors.clone());
            }

            let inserted_offset = self.buffer.insert(start, text, self.cursors.clone());
            new_cursors.push(Selection::point(inserted_offset));
        }

        new_cursors.reverse();
        normalize_selections(&mut new_cursors);
        self.cursors = new_cursors;
        self.primary_cursor_idx = 0;
    }

    pub fn insert_newline(&mut self) {
        let line_ending = self.buffer.line_ending().as_str();
        self.insert_text(line_ending);
    }

    pub fn backspace(&mut self) {
        let mut new_cursors = Vec::with_capacity(self.cursors.len());
        let mut sorted_indices: Vec<usize> = (0..self.cursors.len()).collect();
        sorted_indices.sort_by_key(|&i| std::cmp::Reverse(self.cursors[i].start()));

        for &idx in &sorted_indices {
            let cursor = self.cursors[idx];
            if !cursor.is_empty() {
                let deleted = self.buffer.delete_range(cursor.start(), cursor.end(), self.cursors.clone());
                new_cursors.push(Selection::point(deleted));
            } else if cursor.head > 0 {
                let prev_offset = cursor.head - 1;
                let deleted = self.buffer.delete_range(prev_offset, cursor.head, self.cursors.clone());
                new_cursors.push(Selection::point(deleted));
            } else {
                new_cursors.push(cursor);
            }
        }

        new_cursors.reverse();
        normalize_selections(&mut new_cursors);
        self.cursors = new_cursors;
        self.primary_cursor_idx = 0;
    }

    pub fn delete_forward(&mut self) {
        let mut new_cursors = Vec::with_capacity(self.cursors.len());
        let mut sorted_indices: Vec<usize> = (0..self.cursors.len()).collect();
        sorted_indices.sort_by_key(|&i| std::cmp::Reverse(self.cursors[i].start()));

        for &idx in &sorted_indices {
            let cursor = self.cursors[idx];
            if !cursor.is_empty() {
                let deleted = self.buffer.delete_range(cursor.start(), cursor.end(), self.cursors.clone());
                new_cursors.push(Selection::point(deleted));
            } else if cursor.head < self.buffer.total_chars() {
                let next_offset = cursor.head + 1;
                let deleted = self.buffer.delete_range(cursor.head, next_offset, self.cursors.clone());
                new_cursors.push(Selection::point(deleted));
            } else {
                new_cursors.push(cursor);
            }
        }

        new_cursors.reverse();
        normalize_selections(&mut new_cursors);
        self.cursors = new_cursors;
        self.primary_cursor_idx = 0;
    }

    pub fn move_cursor(&mut self, direction: Direction, select: bool) {
        let mut new_cursors = Vec::with_capacity(self.cursors.len());

        for &cursor in &self.cursors {
            let pos = self.buffer.char_to_position(cursor.head);
            let new_pos = match direction {
                Direction::Left => {
                    if cursor.head > 0 {
                        self.buffer.char_to_position(cursor.head - 1)
                    } else {
                        pos
                    }
                }
                Direction::Right => {
                    if cursor.head < self.buffer.total_chars() {
                        self.buffer.char_to_position(cursor.head + 1)
                    } else {
                        pos
                    }
                }
                Direction::Up => {
                    if pos.line > 0 {
                        Position::new(pos.line - 1, pos.column)
                    } else {
                        Position::new(0, 0)
                    }
                }
                Direction::Down => {
                    if pos.line + 1 < self.buffer.line_count() {
                        Position::new(pos.line + 1, pos.column)
                    } else {
                        Position::new(pos.line, usize::MAX)
                    }
                }
            };

            let new_offset = self.buffer.position_to_char(new_pos);
            let anchor = if select { cursor.anchor } else { new_offset };
            new_cursors.push(Selection::new(anchor, new_offset));
        }

        normalize_selections(&mut new_cursors);
        self.cursors = new_cursors;
        self.primary_cursor_idx = 0;
        self.scroll_cursor_into_view();
    }

    pub fn move_to_line_start(&mut self, select: bool) {
        let mut new_cursors = Vec::with_capacity(self.cursors.len());
        for &cursor in &self.cursors {
            let pos = self.buffer.char_to_position(cursor.head);
            let line_start_char = self.buffer.line_to_char(pos.line);
            let anchor = if select { cursor.anchor } else { line_start_char };
            new_cursors.push(Selection::new(anchor, line_start_char));
        }
        normalize_selections(&mut new_cursors);
        self.cursors = new_cursors;
    }

    pub fn move_to_line_end(&mut self, select: bool) {
        let mut new_cursors = Vec::with_capacity(self.cursors.len());
        for &cursor in &self.cursors {
            let pos = self.buffer.char_to_position(cursor.head);
            let line_end_char = if pos.line + 1 < self.buffer.line_count() {
                self.buffer.line_to_char(pos.line + 1).saturating_sub(1)
            } else {
                self.buffer.total_chars()
            };
            let anchor = if select { cursor.anchor } else { line_end_char };
            new_cursors.push(Selection::new(anchor, line_end_char));
        }
        normalize_selections(&mut new_cursors);
        self.cursors = new_cursors;
    }

    pub fn select_all(&mut self) {
        self.cursors = vec![Selection::new(0, self.buffer.total_chars())];
        self.primary_cursor_idx = 0;
    }

    pub fn undo(&mut self) {
        if let Some(cursors) = self.buffer.undo() {
            self.cursors = cursors;
            self.primary_cursor_idx = 0;
        }
    }

    pub fn redo(&mut self) {
        if let Some(cursors) = self.buffer.redo() {
            self.cursors = cursors;
            self.primary_cursor_idx = 0;
        }
    }

    pub fn scroll_cursor_into_view(&mut self) {
        let pos = self.primary_position();
        if pos.line < self.scroll_line {
            self.scroll_line = pos.line;
        } else if pos.line >= self.scroll_line + self.visible_lines {
            self.scroll_line = pos.line.saturating_sub(self.visible_lines).saturating_add(1);
        }
    }
}
