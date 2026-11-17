use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // File
    NewFile,
    OpenFile,
    SaveFile,
    SaveFileAs,
    CloseTab,
    NextTab,
    PrevTab,

    // Edit
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    Replace,

    // Navigation & Views
    CommandPalette,
    QuickOpen,
    ToggleSidebar,
    ToggleTerminal,
    ToggleProblems,
    FormatDocument,
    ShowHover,
    GoToDefinition,
    TriggerCompletion,
    ZoomIn,
    ZoomOut,
    ResetZoom,

    // App
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub action: Action,
    pub shortcut: Option<String>,
}

impl CommandItem {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        category: impl Into<String>,
        action: Action,
        shortcut: Option<&str>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category: category.into(),
            action,
            shortcut: shortcut.map(String::from),
        }
    }
}
