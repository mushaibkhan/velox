use crate::cursor::{normalize_selections, Direction};
use crate::find::FindState;
use crate::minimap::{MinimapLine, MinimapState, MinimapThumb};
use crate::wrap::{wrap_line, WrappedLineLayout};
use velox_core::{Position, Selection};
use velox_syntax::{Language, SyntaxHighlighter, TokenSpan};
use velox_text_buffer::TextBuffer;

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
    pub minimap: MinimapState,
    pub word_wrap: bool,
    pub cached_wrapped_layouts: Vec<WrappedLineLayout>,
    pub cached_wrap_cols: usize,
    pub wrap_dirty: bool,
    pub git_diff: Option<velox_git::FileDiff>,
    pub diagnostics: Vec<velox_language_server::DiagnosticItem>,
    pub completion_items: Vec<lsp_types::CompletionItem>,
    pub show_completion: bool,
    pub completion_selected_idx: usize,
    pub hover_text: Option<String>,
    pub show_hover: bool,
}

impl EditorState {
    pub fn new(buffer: TextBuffer) -> Self {
        let language = if let Some(path) = buffer.path() {
            Language::from_path(path)
        } else {
            Language::PlainText
        };

        let mut minimap = MinimapState::new();
        minimap.update_from_buffer(&buffer);

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
            minimap,
            word_wrap: false,
            cached_wrapped_layouts: Vec::new(),
            cached_wrap_cols: 0,
            wrap_dirty: true,
            git_diff: None,
            diagnostics: Vec::new(),
            completion_items: Vec::new(),
            show_completion: false,
            completion_selected_idx: 0,
            hover_text: None,
            show_hover: false,
        }
    }

    pub fn set_git_diff(&mut self, diff: Option<velox_git::FileDiff>) {
        self.git_diff = diff;
    }

    pub fn line_git_change(&self, line_1_based: usize) -> Option<velox_git::LineChange> {
        let diff = self.git_diff.as_ref()?;
        if line_1_based == 0 || line_1_based > self.buffer.line_count() {
            return None;
        }
        diff.line_change_at(line_1_based)
    }

    // LSP Diagnostic methods
    pub fn set_diagnostics(&mut self, items: Vec<velox_language_server::DiagnosticItem>) {
        self.diagnostics = items;
    }

    pub fn diagnostics_for_line(
        &self,
        line_0_based: usize,
    ) -> Vec<&velox_language_server::DiagnosticItem> {
        self.diagnostics
            .iter()
            .filter(|item| {
                if line_0_based < item.range.start.line || line_0_based > item.range.end.line {
                    return false;
                }
                // Exclude multi-line ranges that terminate at column 0 of target line
                if line_0_based == item.range.end.line
                    && item.range.end.column == 0
                    && item.range.end.line > item.range.start.line
                {
                    return false;
                }
                true
            })
            .collect()
    }

    pub fn highest_severity_for_line(
        &self,
        line_0_based: usize,
    ) -> Option<velox_language_server::DiagnosticSeverity> {
        self.diagnostics_for_line(line_0_based)
            .into_iter()
            .map(|item| item.severity)
            .min_by_key(|&sev| match sev {
                velox_language_server::DiagnosticSeverity::Error => 1,
                velox_language_server::DiagnosticSeverity::Warning => 2,
                velox_language_server::DiagnosticSeverity::Information => 3,
                velox_language_server::DiagnosticSeverity::Hint => 4,
            })
    }

    // LSP Completion methods
    pub fn set_completions(&mut self, items: Vec<lsp_types::CompletionItem>) {
        self.show_completion = !items.is_empty();
        self.completion_items = items;
        self.completion_selected_idx = 0;
    }

    pub fn close_completion(&mut self) {
        self.show_completion = false;
        self.completion_items.clear();
        self.completion_selected_idx = 0;
    }

    pub fn select_next_completion(&mut self) {
        if self.completion_items.is_empty() {
            return;
        }
        self.completion_selected_idx =
            (self.completion_selected_idx + 1) % self.completion_items.len();
    }

    pub fn select_prev_completion(&mut self) {
        if self.completion_items.is_empty() {
            return;
        }
        if self.completion_selected_idx == 0 {
            self.completion_selected_idx = self.completion_items.len() - 1;
        } else {
            self.completion_selected_idx -= 1;
        }
    }

    pub fn selected_completion(&self) -> Option<&lsp_types::CompletionItem> {
        self.completion_items.get(self.completion_selected_idx)
    }

    pub fn apply_selected_completion(&mut self) -> bool {
        let text_to_insert = if let Some(item) = self.selected_completion() {
            if let Some(ref it) = item.insert_text {
                it.clone()
            } else if let Some(lsp_types::CompletionTextEdit::Edit(ref edit)) = item.text_edit {
                edit.new_text.clone()
            } else {
                item.label.clone()
            }
        } else {
            return false;
        };

        // If the preceding characters on this line match the start of text_to_insert, replace the prefix
        let cursor_offset = self.primary_cursor().head;
        let mut prefix_len = 0;
        let line_idx = self.primary_position().line;
        if let Some(line_str) = self.buffer.line_to_string(line_idx) {
            let col = self.primary_position().column;
            let line_chars: Vec<char> = line_str.chars().collect();
            let mut p = col;
            while p > 0 && (line_chars[p - 1].is_alphanumeric() || line_chars[p - 1] == '_') {
                p -= 1;
            }
            let prefix: String = line_chars[p..col].iter().collect();
            if !prefix.is_empty() && text_to_insert.starts_with(&prefix) {
                prefix_len = prefix.chars().count();
            }
        }

        if prefix_len > 0 {
            let start = cursor_offset - prefix_len;
            self.buffer.delete_range(start, cursor_offset, self.cursors.clone());
            self.cursors = vec![Selection::point(start)];
        }

        self.insert_text(&text_to_insert);
        self.close_completion();
        true
    }

    // LSP Hover methods
    pub fn set_hover(&mut self, text: Option<String>) {
        self.show_hover = text.is_some();
        self.hover_text = text;
    }

    pub fn close_hover(&mut self) {
        self.show_hover = false;
        self.hover_text = None;
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
        self.wrap_dirty = true;
        self.update_minimap();
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
        self.wrap_dirty = true;
        self.update_minimap();
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
        self.wrap_dirty = true;
        self.update_minimap();
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
            self.wrap_dirty = true;
            self.update_minimap();
        }
    }

    pub fn redo(&mut self) {
        if let Some(cursors) = self.buffer.redo() {
            self.cursors = cursors;
            self.primary_cursor_idx = 0;
            self.wrap_dirty = true;
            self.update_minimap();
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

    // Minimap helpers
    pub fn update_minimap(&mut self) {
        self.minimap.update_from_buffer(&self.buffer);
    }

    pub fn minimap_lines(&self) -> Vec<MinimapLine> {
        self.minimap.compute_minimap_lines(self.buffer.line_count())
    }

    pub fn minimap_thumb(&self, height: f32) -> MinimapThumb {
        self.minimap.compute_thumb(
            self.scroll_line,
            self.visible_lines,
            self.buffer.line_count(),
            height,
        )
    }

    pub fn minimap_y_to_line(&self, minimap_y: f32, height: f32) -> usize {
        self.minimap
            .minimap_y_to_editor_line(minimap_y, self.buffer.line_count(), height)
    }

    pub fn scroll_to_minimap_y(&mut self, minimap_y: f32, height: f32) {
        let target_line = self.minimap_y_to_line(minimap_y, height);
        let max_scroll = self.buffer.line_count().saturating_sub(self.visible_lines);
        self.scroll_line = target_line.saturating_sub(self.visible_lines / 2).min(max_scroll);
    }

    // Word wrap helpers
    pub fn invalidate_wrap_cache(&mut self) {
        self.wrap_dirty = true;
    }

    pub fn update_wrapped_layouts(&mut self, max_chars_per_row: usize) -> &[WrappedLineLayout] {
        if self.wrap_dirty || self.cached_wrap_cols != max_chars_per_row {
            self.cached_wrapped_layouts = self.compute_wrapped_layouts(max_chars_per_row);
            self.cached_wrap_cols = max_chars_per_row;
            self.wrap_dirty = false;
        }
        &self.cached_wrapped_layouts
    }

    pub fn wrapped_layouts(&self) -> &[WrappedLineLayout] {
        &self.cached_wrapped_layouts
    }

    #[allow(clippy::single_range_in_vec_init)]
    pub fn compute_wrapped_layouts(&self, max_chars_per_row: usize) -> Vec<WrappedLineLayout> {
        let count = self.buffer.line_count();
        let mut layouts = Vec::with_capacity(count);
        for line_idx in 0..count {
            if let Some(line_str) = self.buffer.line_to_string(line_idx) {
                let rows = wrap_line(&line_str, max_chars_per_row);
                layouts.push(WrappedLineLayout::new(line_idx, rows));
            } else {
                layouts.push(WrappedLineLayout::new(line_idx, vec![0..0]));
            }
        }
        layouts
    }
}
