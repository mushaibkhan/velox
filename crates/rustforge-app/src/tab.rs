use rustforge_core::TabId;
use rustforge_editor::EditorState;
use std::path::Path;

pub struct Tab {
    #[allow(dead_code)]
    pub id: TabId,
    pub editor: EditorState,
    pub title: String,
}

impl Tab {
    pub fn new(editor: EditorState) -> Self {
        let title = editor.buffer.file_name();
        Self {
            id: TabId::new(),
            editor,
            title,
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
}
