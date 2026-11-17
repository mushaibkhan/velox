use rustforge_core::TabId;
use rustforge_editor::EditorState;
use rustforge_text_buffer::TextBuffer;
use std::path::Path;
use std::time::Instant;

pub struct Tab {
    #[allow(dead_code)]
    pub id: TabId,
    pub editor: EditorState,
    pub title: String,
    pub has_external_conflict: bool,
    pub last_save_time: Option<Instant>,
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
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.editor.buffer.path()
    }

    pub fn is_dirty(&self) -> bool {
        self.editor.buffer.is_dirty()
    }

    pub fn update_title(&mut self) {
        self.title = self.editor.buffer.file_name();
    }

    pub fn reload_from_disk(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = self.path() {
            let new_buffer = TextBuffer::from_file(path)?;
            let prev_scroll = self.editor.scroll_line;
            let prev_cursor = self.editor.primary_cursor();
            let mut new_editor = EditorState::new(new_buffer);
            new_editor.scroll_line =
                prev_scroll.min(new_editor.buffer.line_count().saturating_sub(1));
            let max_char = new_editor.buffer.total_chars();
            let safe_cursor = rustforge_core::Selection::point(prev_cursor.head.min(max_char));
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
    fn test_tab_reload_from_disk() {
        let temp_dir = std::env::temp_dir().join("rustforge_tab_test");
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
        let temp_dir = std::env::temp_dir().join("rustforge_tab_conflict_test");
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


