use crate::history::History;
use crate::operation::{Operation, Transaction};
use ropey::{Rope, RopeSlice};
use rustforge_core::{BufferId, Position, Selection};
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
        let line_ending = if text.contains("\r\n") {
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

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let rope = Rope::from_reader(reader)?;
        
        let sample = rope.slice(0..rope.len_chars().min(2048)).to_string();
        let line_ending = if sample.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };

        Ok(Self {
            id: BufferId::new(),
            rope,
            path: Some(path),
            is_dirty: false,
            history: History::default(),
            line_ending,
        })
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
        
        // Don't include newline in column clamp
        let max_col = if line_slice.to_string().ends_with("\r\n") {
            line_chars.saturating_sub(2)
        } else if line_slice.to_string().ends_with('\n') {
            line_chars.saturating_sub(1)
        } else {
            line_chars
        };

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
