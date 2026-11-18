use crate::history::History;
use crate::operation::{Operation, Transaction};
use ropey::{Rope, RopeSlice};
use velox_core::{BufferId, Position, Selection};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

pub struct TextBuffer {
    id: BufferId,
    rope: Rope,
    path: Option<PathBuf>,
    is_dirty: bool,
    history: History,
    line_ending: LineEnding,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            id: BufferId::new(),
            rope: Rope::new(),
            path: None,
            is_dirty: false,
            history: History::default(),
            line_ending: LineEnding::Lf,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &str) -> Self {
        let sample_len = text.len().min(4096);
        let sample = if text.is_char_boundary(sample_len) {
            &text[..sample_len]
        } else {
            let mut boundary = sample_len;
            while boundary > 0 && !text.is_char_boundary(boundary) {
                boundary -= 1;
            }
            &text[..boundary]
        };
        let line_ending = if sample.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
        Self {
            id: BufferId::new(),
            rope: Rope::from_str(text),
            path: None,
            is_dirty: false,
            history: History::default(),
            line_ending,
        }
    }

    pub fn from_rope(rope: Rope) -> Self {
        let sample_len = rope.len_chars().min(2048);
        let sample = rope.slice(0..sample_len);
        let mut line_ending = LineEnding::Lf;
        let mut prev_cr = false;
        for ch in sample.chars() {
            if prev_cr && ch == '\n' {
                line_ending = LineEnding::Crlf;
                break;
            }
            prev_cr = ch == '\r';
        }

        Self {
            id: BufferId::new(),
            rope,
            path: None,
            is_dirty: false,
            history: History::default(),
            line_ending,
        }
    }

    pub fn from_reader<R: std::io::Read>(reader: R) -> Result<Self, std::io::Error> {
        let rope = Rope::from_reader(reader)?;
        Ok(Self::from_rope(rope))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut buffer = Self::from_reader(reader)?;
        buffer.path = Some(path);
        Ok(buffer)
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        self.path = path;
    }

    pub fn file_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn total_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn total_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    pub fn line_slice(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        if line_idx < self.rope.len_lines() {
            Some(self.rope.line(line_idx))
        } else {
            None
        }
    }

    pub fn line_to_string(&self, line_idx: usize) -> Option<String> {
        self.line_slice(line_idx).map(|s| s.to_string())
    }

    /// Returns an iterator over `RopeSlice` for lines in the specified line range `[start_line, end_line)`.
    ///
    /// Note: Yielded `RopeSlice` elements retain trailing newline delimiters (`\n` or `\r\n`) per Ropey line slice conventions.
    pub fn lines_range(&self, start_line: usize, end_line: usize) -> impl Iterator<Item = RopeSlice<'_>> {
        let total_lines = self.line_count();
        let start = start_line.min(total_lines);
        let end = end_line.min(total_lines).max(start);
        let count = end - start;

        let iter = if start >= end || total_lines == 0 {
            let slice = self.rope.slice(self.total_chars()..self.total_chars());
            slice.lines()
        } else {
            let start_char = self.rope.line_to_char(start);
            let end_char = if end < self.rope.len_lines() {
                self.rope.line_to_char(end)
            } else {
                self.total_chars()
            };
            let slice = self.rope.slice(start_char..end_char);
            slice.lines()
        };
        iter.take(count)
    }

    /// Slices up to `count` lines for a viewport starting at `start_line`.
    ///
    /// Note: Yielded `RopeSlice` elements retain trailing newline delimiters (`\n` or `\r\n`) per Ropey line slice conventions.
    pub fn viewport_lines(&self, start_line: usize, count: usize) -> Vec<RopeSlice<'_>> {
        let end_line = start_line.saturating_add(count);
        self.lines_range(start_line, end_line).collect()
    }

    pub fn char_to_line(&self, char_offset: usize) -> usize {
        let clamped = char_offset.min(self.total_chars());
        self.rope.char_to_line(clamped)
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        let clamped = line_idx.min(self.line_count().saturating_sub(1));
        self.rope.line_to_char(clamped)
    }

    pub fn char_to_position(&self, char_offset: usize) -> Position {
        let clamped = char_offset.min(self.total_chars());
        let line = self.rope.char_to_line(clamped);
        let line_start_char = self.rope.line_to_char(line);
        let column = clamped.saturating_sub(line_start_char);
        Position::new(line, column)
    }

    pub fn position_to_char(&self, pos: Position) -> usize {
        if pos.line >= self.line_count() {
            return self.total_chars();
        }
        let line_start = self.rope.line_to_char(pos.line);
        let line_slice = self.rope.line(pos.line);
        let line_chars = line_slice.len_chars();
        
        // Don't include newline in column clamp (zero-allocation check)
        let newline_chars = if line_chars > 0 && line_slice.char(line_chars - 1) == '\n' {
            if line_chars > 1 && line_slice.char(line_chars - 2) == '\r' {
                2
            } else {
                1
            }
        } else {
            0
        };
        let max_col = line_chars.saturating_sub(newline_chars);

        line_start + pos.column.min(max_col)
    }

    pub fn slice_to_string(&self, start_char: usize, end_char: usize) -> String {
        let start = start_char.min(self.total_chars());
        let end = end_char.min(self.total_chars()).max(start);
        self.rope.slice(start..end).to_string()
    }

    // Direct mutators with undo recording
    pub fn insert(&mut self, char_offset: usize, text: &str, cursors_before: Vec<Selection>) -> usize {
        let clamped = char_offset.min(self.total_chars());
        let mut tx = Transaction::new(cursors_before);
        self.rope.insert(clamped, text);
        let inserted_chars = text.chars().count();
        tx.operations.push(Operation::Insert {
            char_offset: clamped,
            text: text.to_string(),
        });
        let new_offset = clamped + inserted_chars;
        tx.cursors_after = vec![Selection::point(new_offset)];
        self.history.push(tx);
        self.is_dirty = true;
        new_offset
    }

    pub fn delete_range(&mut self, start_char: usize, end_char: usize, cursors_before: Vec<Selection>) -> usize {
        let start = start_char.min(self.total_chars());
        let end = end_char.min(self.total_chars()).max(start);
        if start == end {
            return start;
        }

        let deleted_text = self.rope.slice(start..end).to_string();
        let mut tx = Transaction::new(cursors_before);
        self.rope.remove(start..end);
        tx.operations.push(Operation::Delete {
            char_offset: start,
            text: deleted_text,
        });
        tx.cursors_after = vec![Selection::point(start)];
        self.history.push(tx);
        self.is_dirty = true;
        start
    }

    pub fn apply_transaction(&mut self, tx: &Transaction) {
        for op in &tx.operations {
            match op {
                Operation::Insert { char_offset, text } => {
                    let clamped = (*char_offset).min(self.total_chars());
                    self.rope.insert(clamped, text);
                }
                Operation::Delete { char_offset, text } => {
                    let start = (*char_offset).min(self.total_chars());
                    let end = (start + text.chars().count()).min(self.total_chars());
                    if start < end {
                        self.rope.remove(start..end);
                    }
                }
            }
        }
        self.is_dirty = true;
    }

    /// Executes an entire transaction atomically, applying its operations to the buffer
    /// and recording it into the undo history stack.
    pub fn execute_transaction(&mut self, tx: Transaction) {
        if tx.is_empty() {
            return;
        }
        self.apply_transaction(&tx);
        self.history.push(tx);
    }

    pub fn undo(&mut self) -> Option<Vec<Selection>> {
        let inverted = self.history.pop_undo()?;
        self.apply_transaction(&inverted);
        Some(inverted.cursors_after)
    }

    pub fn redo(&mut self) -> Option<Vec<Selection>> {
        let tx = self.history.pop_redo()?;
        self.apply_transaction(&tx);
        Some(tx.cursors_after)
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.path {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            for chunk in self.rope.chunks() {
                writer.write_all(chunk.as_bytes())?;
            }
            writer.flush()?;
            self.is_dirty = false;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path specified for saving buffer",
            ))
        }
    }

    pub fn save_as(&mut self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref().to_path_buf();
        self.path = Some(path);
        self.save()
    }
}
