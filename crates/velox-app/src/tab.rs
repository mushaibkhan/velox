use velox_core::TabId;
use velox_editor::EditorState;
use velox_text_buffer::TextBuffer;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct DiffTabState {
    pub diff: velox_git::FileDiff,
    pub old_label: String,
    pub new_label: String,
    pub path: PathBuf,
}

pub struct Tab {
    #[allow(dead_code)]
    pub id: TabId,
    pub editor: EditorState,
    pub title: String,
    pub has_external_conflict: bool,
    pub last_save_time: Option<Instant>,
    pub diff_view: Option<DiffTabState>,
}

impl Tab {
    pub fn new(editor: EditorState) -> Self {
        let title = editor.buffer.file_name();
        Self {
            id: TabId::new(),
            editor,
            title,
            has_external_conflict: false,
            last_save_time: None,
            diff_view: None,
        }
    }

    pub fn new_diff(
        path: impl Into<PathBuf>,
        diff: velox_git::FileDiff,
        old_label: impl Into<String>,
        new_label: impl Into<String>,
    ) -> Self {
        let path = path.into();
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("diff");
        let title = format!("Diff: {}", file_name);
        Self {
            id: TabId::new(),
            editor: EditorState::new(TextBuffer::new()),
            title,
            has_external_conflict: false,
            last_save_time: None,
            diff_view: Some(DiffTabState {
                diff,
                old_label: old_label.into(),
                new_label: new_label.into(),
                path,
            }),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        if let Some(diff_state) = &self.diff_view {
            Some(&diff_state.path)
        } else {
            self.editor.buffer.path()
        }
    }

    pub fn is_dirty(&self) -> bool {
        if self.diff_view.is_some() {
            false
        } else {
            self.editor.buffer.is_dirty()
        }
    }

    pub fn update_title(&mut self) {
        if let Some(diff_state) = &self.diff_view {
            let file_name = diff_state
                .path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("diff");
            self.title = format!("Diff: {}", file_name);
        } else {
            self.title = self.editor.buffer.file_name();
        }
    }

    pub fn reload_from_disk(&mut self) -> Result<(), std::io::Error> {
        if self.diff_view.is_some() {
            return Ok(());
        }
        if let Some(path) = self.path() {
            let new_buffer = TextBuffer::from_file(path)?;
            let prev_scroll = self.editor.scroll_line;
            let prev_cursor = self.editor.primary_cursor();
            let mut new_editor = EditorState::new(new_buffer);
            new_editor.scroll_line =
                prev_scroll.min(new_editor.buffer.line_count().saturating_sub(1));
            let max_char = new_editor.buffer.total_chars();
            let safe_cursor = velox_core::Selection::point(prev_cursor.head.min(max_char));
            new_editor.cursors = vec![safe_cursor];
            new_editor.primary_cursor_idx = 0;
            new_editor.tab_size = self.editor.tab_size;
            new_editor.show_find = self.editor.show_find;
            new_editor.find_state = self.editor.find_state.clone();
            new_editor.word_wrap = self.editor.word_wrap;
            self.editor = new_editor;
            self.has_external_conflict = false;
            self.update_title();
        }
        Ok(())
    }

    pub fn keep_buffer_changes(&mut self) {
        self.has_external_conflict = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_tab_new_diff() {
        let diff = velox_git::compute_diff("old\n", "new\n");
        let path = PathBuf::from("/path/to/file.rs");
        let tab = Tab::new_diff(path.clone(), diff, "HEAD", "Working Tree");
        assert_eq!(tab.title, "Diff: file.rs");
        assert_eq!(tab.path(), Some(path.as_path()));
        assert!(!tab.is_dirty());
        assert!(tab.diff_view.is_some());
    }

    #[test]
    fn test_tab_reload_from_disk() {
        let temp_dir = std::env::temp_dir().join("velox_tab_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("reload.txt");
        fs::write(&file_path, "original content").unwrap();

        let buffer = TextBuffer::from_file(&file_path).unwrap();
        let mut editor = EditorState::new(buffer);
        editor.tab_size = 8;
        editor.show_find = true;
        editor.find_state.query = "original".to_string();
        editor.word_wrap = true;
        let mut tab = Tab::new(editor);
        assert_eq!(tab.editor.buffer.slice_to_string(0, 8), "original");
        assert!(!tab.is_dirty());
        assert!(!tab.has_external_conflict);

        // Update file on disk
        fs::write(&file_path, "updated content on disk").unwrap();

        // Reload from disk
        tab.reload_from_disk().unwrap();
        assert_eq!(tab.editor.buffer.slice_to_string(0, 7), "updated");
        assert!(!tab.is_dirty());
        assert!(!tab.has_external_conflict);
        assert_eq!(tab.editor.tab_size, 8);
        assert!(tab.editor.show_find);
        assert_eq!(tab.editor.find_state.query, "original");
        assert!(tab.editor.word_wrap);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tab_conflict_and_keep_changes() {
        let temp_dir = std::env::temp_dir().join("velox_tab_conflict_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("conflict.txt");
        fs::write(&file_path, "base").unwrap();

        let buffer = TextBuffer::from_file(&file_path).unwrap();
        let editor = EditorState::new(buffer);
        let mut tab = Tab::new(editor);

        // Make buffer dirty
        tab.editor.insert_text("dirty ");
        assert!(tab.is_dirty());

        // Flag conflict
        tab.has_external_conflict = true;
        assert!(tab.has_external_conflict);

        // Choose to keep buffer changes
        tab.keep_buffer_changes();
        assert!(!tab.has_external_conflict);
        assert!(tab.is_dirty());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
