use crate::tab::Tab;
use crate::ui::{
    activity_bar::render_activity_bar, editor_view::render_editor_area, menu_bar::render_menu_bar,
    palette::render_command_palette, sidebar::render_sidebar, status_bar::render_status_bar,
    terminal_view::render_terminal_panel,
};
use egui::{CentralPanel, Key, SidePanel, TopBottomPanel};
use rustforge_core::{Action, CommandItem};
use rustforge_editor::EditorState;
use rustforge_search::{SearchEngine, SearchMatch};
use rustforge_settings::Settings;
use rustforge_terminal::{TerminalManager, TerminalSession};
use rustforge_text_buffer::TextBuffer;
use rustforge_themes::{Theme, ThemeMode};
use rustforge_workspace::Workspace;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Explorer,
    Search,
    Git,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Editor,
    Terminal,
    #[allow(dead_code)]
    Sidebar,
}

pub struct RustForgeApp {
    pub theme: Theme,
    pub settings: Settings,
    pub workspace: Workspace,
    pub tabs: Vec<Tab>,
    pub active_tab_idx: Option<usize>,
    pub terminal_manager: TerminalManager,
    pub focused_terminal_idx: Option<usize>,
    pub active_sidebar: SidebarTab,
    pub sidebar_visible: bool,
    pub terminal_visible: bool,
    pub focused_panel: FocusedPanel,
    pub sidebar_width: f32,
    pub terminal_height: f32,

    // Search panel
    pub search_query: String,
    pub search_results: Vec<SearchMatch>,

    // Command Palette & Quick Open
    pub command_palette_open: bool,
    pub quick_open_open: bool,
    pub palette_query: String,
    pub palette_selected_idx: usize,
    pub commands: Vec<CommandItem>,

    // File Watcher
    pub file_watcher: Option<rustforge_workspace::FileWatcher>,
    pub file_event_rx: Option<std::sync::mpsc::Receiver<rustforge_workspace::FileEvent>>,
    pub egui_ctx: Option<egui::Context>,

    // Git
    pub git_repo: Option<rustforge_git::GitRepository>,
    pub git_status: Option<rustforge_git::RepoStatus>,
    pub git_branch: String,
    pub git_ahead_behind: (usize, usize),
    pub git_commit_message: String,

    // LSP
    pub lsp_manager: Option<rustforge_language_server::LspManager>,
    pub tokio_rt: Option<std::sync::Arc<tokio::runtime::Runtime>>,
}

impl RustForgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let theme = match settings.theme_mode {
            ThemeMode::DarkModern => Theme::dark_modern(),
            ThemeMode::TokyoNight => Theme::tokyo_night(),
            ThemeMode::MonokaiPro => Theme::monokai_pro(),
            _ => Theme::dark_modern(),
        };

        let mut workspace = Workspace::new();
        // Automatically open current working directory if available
        if let Ok(cwd) = std::env::current_dir() {
            workspace.open(&cwd);
        }

        let mut commands = Vec::new();
        Self::register_commands(&mut commands);

        let egui_ctx = Some(cc.egui_ctx.clone());

        let mut app = Self {
            theme,
            settings,
            workspace,
            tabs: Vec::new(),
            active_tab_idx: None,
            terminal_manager: TerminalManager::new(),
            focused_terminal_idx: None,
            active_sidebar: SidebarTab::Explorer,
            sidebar_visible: true,
            terminal_visible: true,
            focused_panel: FocusedPanel::Editor,
            sidebar_width: 250.0,
            terminal_height: 220.0,
            search_query: String::new(),
            search_results: Vec::new(),
            command_palette_open: false,
            quick_open_open: false,
            palette_query: String::new(),
            palette_selected_idx: 0,
            commands,
            file_watcher: None,
            file_event_rx: None,
            egui_ctx,
            git_repo: None,
            git_status: None,
            git_branch: String::new(),
            git_ahead_behind: (0, 0),
            git_commit_message: String::new(),
            lsp_manager: Some(rustforge_language_server::LspManager::new()),
            tokio_rt: tokio::runtime::Runtime::new().ok().map(std::sync::Arc::new),
        };

        // Try to start default terminal session with egui repaint callback
        app.new_terminal_session(Some(cc.egui_ctx.clone()));

        // Start file watcher for initial workspace root if available
        app.start_file_watcher();

        // Initialize Git repo & status from workspace root
        app.refresh_git();

        // Create an initial welcome/untitled file
        app.new_file();

        app
    }

    pub fn new_test_app() -> Self {
        let settings = Settings::default();
        let theme = Theme::dark_modern();
        let workspace = Workspace::new();
        let mut commands = Vec::new();
        Self::register_commands(&mut commands);

        let mut app = Self {
            theme,
            settings,
            workspace,
            tabs: Vec::new(),
            active_tab_idx: None,
            terminal_manager: TerminalManager::new(),
            focused_terminal_idx: None,
            active_sidebar: SidebarTab::Explorer,
            sidebar_visible: true,
            terminal_visible: false,
            focused_panel: FocusedPanel::Editor,
            sidebar_width: 250.0,
            terminal_height: 220.0,
            search_query: String::new(),
            search_results: Vec::new(),
            command_palette_open: false,
            quick_open_open: false,
            palette_query: String::new(),
            palette_selected_idx: 0,
            commands,
            file_watcher: None,
            file_event_rx: None,
            egui_ctx: None,
            git_repo: None,
            git_status: None,
            git_branch: String::new(),
            git_ahead_behind: (0, 0),
            git_commit_message: String::new(),
            lsp_manager: Some(rustforge_language_server::LspManager::new()),
            tokio_rt: tokio::runtime::Runtime::new().ok().map(std::sync::Arc::new),
        };
        app.refresh_git();
        app
    }

    fn register_commands(commands: &mut Vec<CommandItem>) {
        commands.push(CommandItem::new(
            "file.new",
            "New File",
            "File",
            Action::NewFile,
            Some("Ctrl+N"),
        ));
        commands.push(CommandItem::new(
            "file.open",
            "Open File...",
            "File",
            Action::OpenFile,
            Some("Ctrl+O"),
        ));
        commands.push(CommandItem::new(
            "file.save",
            "Save File",
            "File",
            Action::SaveFile,
            Some("Ctrl+S"),
        ));
        commands.push(CommandItem::new(
            "file.save_as",
            "Save File As...",
            "File",
            Action::SaveFileAs,
            Some("Ctrl+Shift+S"),
        ));
        commands.push(CommandItem::new(
            "file.close",
            "Close Active Tab",
            "File",
            Action::CloseTab,
            Some("Ctrl+W"),
        ));
        commands.push(CommandItem::new(
            "view.quick_open",
            "Go to File...",
            "Navigation",
            Action::QuickOpen,
            Some("Ctrl+P"),
        ));
        commands.push(CommandItem::new(
            "view.command_palette",
            "Show All Commands",
            "View",
            Action::CommandPalette,
            Some("Ctrl+Shift+P"),
        ));
        commands.push(CommandItem::new(
            "view.toggle_sidebar",
            "Toggle Primary Side Bar",
            "View",
            Action::ToggleSidebar,
            Some("Ctrl+B"),
        ));
        commands.push(CommandItem::new(
            "view.toggle_terminal",
            "Toggle Terminal",
            "View",
            Action::ToggleTerminal,
            Some("Ctrl+`"),
        ));
        commands.push(CommandItem::new(
            "edit.undo",
            "Undo",
            "Edit",
            Action::Undo,
            Some("Ctrl+Z"),
        ));
        commands.push(CommandItem::new(
            "edit.redo",
            "Redo",
            "Edit",
            Action::Redo,
            Some("Ctrl+Y"),
        ));
        commands.push(CommandItem::new(
            "edit.find",
            "Find in File",
            "Edit",
            Action::Find,
            Some("Ctrl+F"),
        ));
        commands.push(CommandItem::new(
            "edit.select_all",
            "Select All",
            "Edit",
            Action::SelectAll,
            Some("Ctrl+A"),
        ));
        commands.push(CommandItem::new(
            "edit.format",
            "Format Document",
            "Edit",
            Action::FormatDocument,
            Some("Shift+Alt+F"),
        ));
        commands.push(CommandItem::new(
            "lsp.hover",
            "Show Hover Documentation",
            "LSP",
            Action::ShowHover,
            Some("Ctrl+I"),
        ));
        commands.push(CommandItem::new(
            "lsp.definition",
            "Go to Definition",
            "LSP",
            Action::GoToDefinition,
            Some("F12"),
        ));
        commands.push(CommandItem::new(
            "lsp.completion",
            "Trigger Autocomplete",
            "LSP",
            Action::TriggerCompletion,
            Some("Ctrl+Space"),
        ));
        commands.push(CommandItem::new(
            "app.quit",
            "Quit Application",
            "Application",
            Action::Quit,
            Some("Ctrl+Q"),
        ));
    }

    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.settings.theme_mode = mode;
        self.theme = match mode {
            ThemeMode::DarkModern => Theme::dark_modern(),
            ThemeMode::TokyoNight => Theme::tokyo_night(),
            ThemeMode::MonokaiPro => Theme::monokai_pro(),
            _ => Theme::dark_modern(),
        };
    }

    pub fn new_file(&mut self) {
        let buffer = TextBuffer::new();
        let editor = EditorState::new(buffer);
        let tab = Tab::new(editor);
        self.tabs.push(tab);
        self.active_tab_idx = Some(self.tabs.len() - 1);
    }

    pub fn open_file(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        // If already open (in a normal editor tab), switch to it
        for (idx, tab) in self.tabs.iter().enumerate() {
            if tab.diff_view.is_none() {
                if let Some(p) = tab.path() {
                    if paths_match(p, path) {
                        self.active_tab_idx = Some(idx);
                        return;
                    }
                }
            }
        }

        if let Ok(buffer) = TextBuffer::from_file(path) {
            let mut editor = EditorState::new(buffer);
            if let Some(repo) = &self.git_repo {
                if let Ok(diff) = repo.diff_file(path) {
                    editor.set_git_diff(Some(diff));
                }
            }
            let tab = Tab::new(editor);
            self.tabs.push(tab);
            self.active_tab_idx = Some(self.tabs.len() - 1);
        }
    }

    pub fn open_file_at_line(&mut self, path: impl AsRef<Path>, line_number: usize) {
        self.open_file(path);
        if let Some(idx) = self.active_tab_idx {
            if let Some(tab) = self.tabs.get_mut(idx) {
                let target_line = line_number.saturating_sub(1);
                let char_offset = tab.editor.buffer.line_to_char(target_line);
                tab.editor.cursors = vec![rustforge_core::Selection::point(char_offset)];
                tab.editor.primary_cursor_idx = 0;
                tab.editor.scroll_cursor_into_view();
            }
        }
    }

    pub fn open_folder_dialog(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.open_workspace(&folder);
        }
    }

    pub fn open_workspace(&mut self, path: impl AsRef<Path>) {
        self.workspace.open(path);
        self.start_file_watcher();
        self.refresh_git();
    }

    pub fn open_file_dialog(&mut self) {
        if let Some(file) = rfd::FileDialog::new().pick_file() {
            self.open_file(&file);
        }
    }

    pub fn save_current_file(&mut self) {
        if let Some(idx) = self.active_tab_idx {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if tab.diff_view.is_some() {
                    return;
                }
                if tab.path().is_some() {
                    let _ = tab.editor.buffer.save();
                    tab.last_save_time = Some(std::time::Instant::now());
                    tab.update_title();
                } else {
                    self.save_current_file_as();
                    return;
                }
            }
            self.refresh_git();
        }
    }

    pub fn save_current_file_as(&mut self) {
        if let Some(idx) = self.active_tab_idx {
            if let Some(dest) = rfd::FileDialog::new().save_file() {
                if let Some(tab) = self.tabs.get_mut(idx) {
                    if tab.diff_view.is_some() {
                        return;
                    }
                    let _ = tab.editor.buffer.save_as(&dest);
                    tab.last_save_time = Some(std::time::Instant::now());
                    tab.update_title();
                }
                self.refresh_git();
            }
        }
    }

    pub fn refresh_git(&mut self) {
        if let Some(root) = &self.workspace.root_path {
            let need_discover = match &self.git_repo {
                Some(repo) => {
                    let repo_workdir = repo.workdir();
                    if let (Ok(c_root), Ok(c_workdir)) = (root.canonicalize(), repo_workdir.canonicalize()) {
                        !c_root.starts_with(&c_workdir)
                    } else {
                        !root.starts_with(repo_workdir)
                    }
                }
                None => true,
            };

            if need_discover {
                self.git_repo = rustforge_git::GitRepository::discover(root);
            }
        } else {
            self.git_repo = None;
        }

        if let Some(repo) = &self.git_repo {
            self.git_status = repo.status().ok();
            self.git_branch = repo.current_branch().unwrap_or_else(|_| "HEAD".to_string());
            if let Ok(status) = repo.branch_status() {
                self.git_ahead_behind = (status.ahead, status.behind);
            } else {
                self.git_ahead_behind = (0, 0);
            }

            for tab in &mut self.tabs {
                if tab.diff_view.is_none() {
                    if let Some(path) = tab.path() {
                        if let Ok(diff) = repo.diff_file(path) {
                            tab.editor.set_git_diff(Some(diff));
                        } else {
                            tab.editor.set_git_diff(None);
                        }
                    }
                } else if let Some(ref mut diff_state) = tab.diff_view {
                    if let Ok(diff) = repo.diff_file(&diff_state.path) {
                        diff_state.diff = diff;
                    }
                }
            }
        } else {
            self.git_status = None;
            self.git_branch.clear();
            self.git_ahead_behind = (0, 0);
            for tab in &mut self.tabs {
                if tab.diff_view.is_none() {
                    tab.editor.set_git_diff(None);
                }
            }
        }
    }

    pub fn stage_file(&mut self, path: &Path) {
        if let Some(repo) = &self.git_repo {
            if let Err(e) = repo.stage_file(path) {
                tracing::warn!("Failed to stage file {:?}: {}", path, e);
            }
        }
        self.refresh_git();
    }

    pub fn unstage_file(&mut self, path: &Path) {
        if let Some(repo) = &self.git_repo {
            if let Err(e) = repo.unstage_file(path) {
                tracing::warn!("Failed to unstage file {:?}: {}", path, e);
            }
        }
        self.refresh_git();
    }

    pub fn discard_file(&mut self, path: &Path) {
        if let Some(repo) = &self.git_repo {
            if let Err(e) = repo.discard_file(path) {
                tracing::warn!("Failed to discard file {:?}: {}", path, e);
            }
        }
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(repo) = &self.git_repo {
            repo.workdir().join(path)
        } else {
            path.to_path_buf()
        };
        for tab in &mut self.tabs {
            if tab.diff_view.is_none() {
                if let Some(tab_path) = tab.path() {
                    if paths_match(tab_path, &full_path) {
                        let _ = tab.reload_from_disk();
                    }
                }
            }
        }
        self.refresh_git();
    }

    pub fn stage_all(&mut self) {
        if let Some(repo) = &self.git_repo {
            if let Err(e) = repo.stage_all() {
                tracing::warn!("Failed to stage all files: {}", e);
            }
        }
        self.refresh_git();
    }

    pub fn commit(&mut self) -> Result<git2::Oid, String> {
        let trimmed_msg = self.git_commit_message.trim();
        if trimmed_msg.is_empty() {
            return Err("Commit message cannot be empty".to_string());
        }
        let Some(repo) = &self.git_repo else {
            return Err("Not a git repository".to_string());
        };

        let (name, email) = if let Ok(sig) = repo.raw().signature() {
            (
                sig.name().unwrap_or("RustForge User").to_string(),
                sig.email().unwrap_or("user@rustforge.dev").to_string(),
            )
        } else {
            ("RustForge User".to_string(), "user@rustforge.dev".to_string())
        };

        match repo.commit(trimmed_msg, &name, &email) {
            Ok(oid) => {
                self.git_commit_message.clear();
                self.refresh_git();
                Ok(oid)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn open_diff_tab(&mut self, path: &Path) {
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(repo) = &self.git_repo {
            repo.workdir().join(path)
        } else if let Some(root) = &self.workspace.root_path {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        for (idx, tab) in self.tabs.iter_mut().enumerate() {
            if let Some(ref mut diff_state) = tab.diff_view {
                if paths_match(&diff_state.path, &full_path)
                    || diff_state.path == path
                    || diff_state.path == full_path
                {
                    if let Some(repo) = &self.git_repo {
                        if let Ok(diff) = repo.diff_file(&diff_state.path) {
                            diff_state.diff = diff;
                        }
                    }
                    self.active_tab_idx = Some(idx);
                    self.focused_panel = FocusedPanel::Editor;
                    return;
                }
            }
        }

        let diff = if let Some(repo) = &self.git_repo {
            repo.diff_file(&full_path)
                .unwrap_or_else(|_| rustforge_git::compute_diff("", ""))
        } else {
            rustforge_git::compute_diff("", "")
        };

        let file_name = full_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("file");
        let old_label = format!("HEAD:{}", file_name);
        let new_label = format!("Working Tree:{}", file_name);
        let tab = Tab::new_diff(full_path, diff, old_label, new_label);
        self.tabs.push(tab);
        self.active_tab_idx = Some(self.tabs.len() - 1);
        self.focused_panel = FocusedPanel::Editor;
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.tabs.is_empty() {
                self.active_tab_idx = None;
            } else if let Some(cur) = self.active_tab_idx {
                if cur >= self.tabs.len() {
                    self.active_tab_idx = Some(self.tabs.len() - 1);
                }
            }
        }
    }

    pub fn new_terminal_session(&mut self, ctx: Option<egui::Context>) {
        let cwd = self.workspace.root_path.as_deref();
        let repaint_cb = ctx.map(|c| {
            let cb: std::sync::Arc<dyn Fn() + Send + Sync> = std::sync::Arc::new(move || {
                c.request_repaint();
            });
            cb
        });
        match TerminalSession::spawn(80, 24, cwd, repaint_cb) {
            Ok(session) => {
                let idx = self.terminal_manager.add_session(session);
                self.focused_terminal_idx = Some(idx);
            }
            Err(e) => tracing::warn!("Failed to spawn terminal session: {}", e),
        }
    }

    pub fn close_terminal_session(&mut self, idx: usize) {
        let _ = self.terminal_manager.close_session(idx);
        if self.terminal_manager.is_empty() {
            self.focused_terminal_idx = None;
        } else {
            self.focused_terminal_idx = Some(self.terminal_manager.active_idx());
        }
    }

    pub fn terminal(&self) -> Option<&TerminalSession> {
        self.terminal_manager.active_session()
    }

    pub fn terminal_mut(&mut self) -> Option<&mut TerminalSession> {
        self.terminal_manager.active_session_mut()
    }

    pub fn execute_workspace_search(&mut self) {
        if let Some(root) = &self.workspace.root_path {
            self.search_results = SearchEngine::search_in_dir(root, &self.search_query, false, 200);
        }
    }

    pub fn dispatch_action(&mut self, action: Action) {
        match action {
            Action::NewFile => self.new_file(),
            Action::OpenFile => self.open_file_dialog(),
            Action::SaveFile => self.save_current_file(),
            Action::SaveFileAs => self.save_current_file_as(),
            Action::CloseTab => {
                if let Some(idx) = self.active_tab_idx {
                    self.close_tab(idx);
                }
            }
            Action::NextTab => {
                if !self.tabs.is_empty() {
                    let next = self.active_tab_idx.map(|i| (i + 1) % self.tabs.len()).unwrap_or(0);
                    self.active_tab_idx = Some(next);
                }
            }
            Action::PrevTab => {
                if !self.tabs.is_empty() {
                    let prev = self.active_tab_idx.map(|i| {
                        if i == 0 { self.tabs.len() - 1 } else { i - 1 }
                    }).unwrap_or(0);
                    self.active_tab_idx = Some(prev);
                }
            }
            Action::Undo => {
                if let Some(idx) = self.active_tab_idx {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        tab.editor.undo();
                    }
                }
            }
            Action::Redo => {
                if let Some(idx) = self.active_tab_idx {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        tab.editor.redo();
                    }
                }
            }
            Action::Find => {
                if let Some(idx) = self.active_tab_idx {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        tab.editor.show_find = true;
                    }
                }
            }
            Action::Replace => {
                if let Some(idx) = self.active_tab_idx {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        tab.editor.show_find = true;
                    }
                }
            }
            Action::SelectAll => {
                if let Some(idx) = self.active_tab_idx {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        tab.editor.select_all();
                    }
                }
            }
            Action::CommandPalette => {
                self.command_palette_open = true;
                self.quick_open_open = false;
                self.palette_query.clear();
                self.palette_selected_idx = 0;
            }
            Action::QuickOpen => {
                self.quick_open_open = true;
                self.command_palette_open = false;
                self.palette_query.clear();
                self.palette_selected_idx = 0;
            }
            Action::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            Action::ToggleTerminal => self.terminal_visible = !self.terminal_visible,
            Action::FormatDocument => self.trigger_formatting(),
            Action::ShowHover => self.trigger_hover(),
            Action::GoToDefinition => self.trigger_goto_definition(),
            Action::TriggerCompletion => self.trigger_completion(),
            Action::Quit => std::process::exit(0),
            _ => {}
        }
    }

    pub fn start_file_watcher(&mut self) {
        if let Some(root) = self.workspace.root_path.clone() {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = rustforge_workspace::FileWatcher::new();
            let ctx = self.egui_ctx.clone();
            let res = watcher.start_watching(&root, move |event| {
                let _ = tx.send(event);
                if let Some(c) = &ctx {
                    c.request_repaint();
                }
            });
            match res {
                Ok(()) => {
                    self.file_watcher = Some(watcher);
                    self.file_event_rx = Some(rx);
                }
                Err(e) => {
                    tracing::warn!("Failed to start file watcher on {}: {}", root.display(), e);
                    self.file_watcher = None;
                    self.file_event_rx = None;
                }
            }
        } else {
            self.file_watcher = None;
            self.file_event_rx = None;
        }
    }

    pub fn drain_file_events(&mut self) {
        let mut events = Vec::new();
        if let Some(rx) = &self.file_event_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }

        if events.is_empty() {
            return;
        }

        let mut refresh_workspace = false;

        for event in events {
            match event {
                rustforge_workspace::FileEvent::Created(_)
                | rustforge_workspace::FileEvent::Deleted(_) => {
                    refresh_workspace = true;
                }
                rustforge_workspace::FileEvent::Modified(_) => {}
            }

            let event_path = event.path();
            for tab in &mut self.tabs {
                let tab_path = tab.path().map(|p| p.to_path_buf());
                if let Some(tab_path) = tab_path {
                    if paths_match(&tab_path, event_path) {
                        let is_recent_save = tab
                            .last_save_time
                            .is_some_and(|t| t.elapsed() < std::time::Duration::from_millis(300));

                        if is_recent_save {
                            continue;
                        }

                        match &event {
                            rustforge_workspace::FileEvent::Modified(_)
                            | rustforge_workspace::FileEvent::Created(_) => {
                                if !tab.editor.buffer.is_dirty() {
                                    if let Err(e) = tab.reload_from_disk() {
                                        tracing::warn!(
                                            "Failed to reload modified file {:?}: {}",
                                            tab_path,
                                            e
                                        );
                                    }
                                } else {
                                    tab.has_external_conflict = true;
                                }
                            }
                            rustforge_workspace::FileEvent::Deleted(_) => {
                                tab.has_external_conflict = true;
                            }
                        }
                    }
                }
            }
        }

        if refresh_workspace {
            self.workspace.refresh();
            self.refresh_git();
        }
    }

    // LSP Integration Methods
    pub fn run_async<F, R>(&self, f: F) -> Option<R>
    where
        F: std::future::Future<Output = R> + Send,
        R: Send,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                Some(tokio::task::block_in_place(|| handle.block_on(f)))
            } else {
                std::thread::scope(|s| {
                    s.spawn(|| {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .ok()?;
                        Some(rt.block_on(f))
                    })
                    .join()
                    .ok()
                    .flatten()
                })
            }
        } else if let Some(ref rt) = self.tokio_rt {
            Some(rt.block_on(f))
        } else {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            Some(rt.block_on(f))
        }
    }

    pub fn sync_lsp_diagnostics(&mut self) {
        let Some(ref lsp) = self.lsp_manager else {
            return;
        };
        let store = lsp.diagnostic_store();
        for tab in &mut self.tabs {
            if let Some(path) = tab.path() {
                let diags = store.get_diagnostics_for_path(path);
                if tab.editor.diagnostics != diags {
                    tab.editor.set_diagnostics(diags);
                }
            }
        }
    }

    pub fn trigger_completion(&mut self) {
        let active_idx = match self.active_tab_idx {
            Some(idx) if idx < self.tabs.len() => idx,
            _ => return,
        };
        let (path, pos) = {
            let tab = &self.tabs[active_idx];
            let Some(path) = tab.path() else { return; };
            (path.to_path_buf(), tab.editor.primary_position())
        };

        let Ok(uri) = rustforge_language_server::path_to_uri(&path) else {
            return;
        };

        let client = {
            let Some(ref lsp) = self.lsp_manager else {
                return;
            };
            if let Some(c) = lsp.get_client_for_path(&path) {
                Some(c)
            } else {
                let root_uri = self
                    .workspace
                    .root_path
                    .as_deref()
                    .and_then(|r| rustforge_language_server::path_to_uri(r).ok());
                self.run_async(lsp.start_client_for_path(&path, root_uri))
                    .and_then(|res| res.ok())
            }
        };

        let Some(client) = client else {
            return;
        };

        let client_arc = client.clone();
        let completion_res = self.run_async(async move { client_arc.completion(uri, pos).await });
        let items = match completion_res {
            Some(Ok(Some(lsp_types::CompletionResponse::Array(arr)))) => arr,
            Some(Ok(Some(lsp_types::CompletionResponse::List(list)))) => list.items,
            _ => Vec::new(),
        };

        if let Some(tab) = self.tabs.get_mut(active_idx) {
            tab.editor.set_completions(items);
            tab.editor.show_completion = true;
        }
    }

    pub fn trigger_hover(&mut self) {
        let active_idx = match self.active_tab_idx {
            Some(idx) if idx < self.tabs.len() => idx,
            _ => return,
        };
        let (path, pos) = {
            let tab = &self.tabs[active_idx];
            let Some(path) = tab.path() else { return; };
            (path.to_path_buf(), tab.editor.primary_position())
        };

        let Ok(uri) = rustforge_language_server::path_to_uri(&path) else {
            return;
        };

        let client = {
            let Some(ref lsp) = self.lsp_manager else {
                return;
            };
            if let Some(c) = lsp.get_client_for_path(&path) {
                Some(c)
            } else {
                let root_uri = self
                    .workspace
                    .root_path
                    .as_deref()
                    .and_then(|r| rustforge_language_server::path_to_uri(r).ok());
                self.run_async(lsp.start_client_for_path(&path, root_uri))
                    .and_then(|res| res.ok())
            }
        };

        let Some(client) = client else {
            return;
        };

        let client_arc = client.clone();
        let hover_res = self.run_async(async move { client_arc.hover(uri, pos).await });
        let hover_text = match hover_res {
            Some(Ok(Some(hover))) => {
                let text = format_hover_contents(&hover.contents);
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            _ => None,
        };

        if let Some(tab) = self.tabs.get_mut(active_idx) {
            tab.editor.hover_text = hover_text;
            tab.editor.show_hover = true;
        }
    }

    pub fn trigger_goto_definition(&mut self) {
        let active_idx = match self.active_tab_idx {
            Some(idx) if idx < self.tabs.len() => idx,
            _ => return,
        };
        let (path, pos) = {
            let tab = &self.tabs[active_idx];
            let Some(path) = tab.path() else { return; };
            (path.to_path_buf(), tab.editor.primary_position())
        };

        let Ok(uri) = rustforge_language_server::path_to_uri(&path) else {
            return;
        };

        let client = {
            let Some(ref lsp) = self.lsp_manager else {
                return;
            };
            if let Some(c) = lsp.get_client_for_path(&path) {
                Some(c)
            } else {
                let root_uri = self
                    .workspace
                    .root_path
                    .as_deref()
                    .and_then(|r| rustforge_language_server::path_to_uri(r).ok());
                self.run_async(lsp.start_client_for_path(&path, root_uri))
                    .and_then(|res| res.ok())
            }
        };

        let Some(client) = client else {
            return;
        };

        let client_arc = client.clone();
        let def_res = self.run_async(async move { client_arc.goto_definition(uri, pos).await });
        let target = match def_res {
            Some(Ok(Some(lsp_types::GotoDefinitionResponse::Scalar(loc)))) => {
                Some((loc.uri, loc.range))
            }
            Some(Ok(Some(lsp_types::GotoDefinitionResponse::Array(arr)))) => {
                arr.into_iter().next().map(|loc| (loc.uri, loc.range))
            }
            Some(Ok(Some(lsp_types::GotoDefinitionResponse::Link(links)))) => links
                .into_iter()
                .next()
                .map(|link| (link.target_uri, link.target_selection_range)),
            _ => None,
        };

        if let Some((target_uri, target_range)) = target {
            let target_path = rustforge_language_server::uri_to_path(&target_uri);
            self.open_file(&target_path);
            if let Some(idx) = self.active_tab_idx {
                if let Some(tab) = self.tabs.get_mut(idx) {
                    let target_pos = rustforge_core::Position::new(
                        target_range.start.line as usize,
                        target_range.start.character as usize,
                    );
                    let char_offset = tab.editor.buffer.position_to_char(target_pos);
                    tab.editor.cursors = vec![rustforge_core::Selection::point(char_offset)];
                    tab.editor.primary_cursor_idx = 0;
                    tab.editor.scroll_cursor_into_view();
                }
            }
        }
    }

    pub fn trigger_formatting(&mut self) {
        let active_idx = match self.active_tab_idx {
            Some(idx) if idx < self.tabs.len() => idx,
            _ => return,
        };
        let (path, tab_size) = {
            let tab = &self.tabs[active_idx];
            let Some(path) = tab.path() else { return; };
            (path.to_path_buf(), tab.editor.tab_size as u32)
        };

        let Ok(uri) = rustforge_language_server::path_to_uri(&path) else {
            return;
        };

        let client = {
            let Some(ref lsp) = self.lsp_manager else {
                return;
            };
            if let Some(c) = lsp.get_client_for_path(&path) {
                Some(c)
            } else {
                let root_uri = self
                    .workspace
                    .root_path
                    .as_deref()
                    .and_then(|r| rustforge_language_server::path_to_uri(r).ok());
                self.run_async(lsp.start_client_for_path(&path, root_uri))
                    .and_then(|res| res.ok())
            }
        };

        let Some(client) = client else {
            return;
        };

        let client_arc = client.clone();
        let format_res = self.run_async(async move { client_arc.formatting(uri, tab_size, true).await });
        if let Some(Ok(Some(edits))) = format_res {
            if let Some(tab) = self.tabs.get_mut(active_idx) {
                let mut sorted_edits = edits;
                sorted_edits.sort_by(|a, b| {
                    b.range
                        .start
                        .line
                        .cmp(&a.range.start.line)
                        .then_with(|| b.range.start.character.cmp(&a.range.start.character))
                });

                for edit in sorted_edits {
                    let start_pos = rustforge_core::Position::new(
                        edit.range.start.line as usize,
                        edit.range.start.character as usize,
                    );
                    let end_pos = rustforge_core::Position::new(
                        edit.range.end.line as usize,
                        edit.range.end.character as usize,
                    );
                    let start_char = tab.editor.buffer.position_to_char(start_pos);
                    let end_char = tab.editor.buffer.position_to_char(end_pos);

                    if start_char != end_char {
                        tab.editor.buffer.delete_range(
                            start_char,
                            end_char,
                            tab.editor.cursors.clone(),
                        );
                    }
                    if !edit.new_text.is_empty() {
                        tab.editor.buffer.insert(
                            start_char,
                            &edit.new_text,
                            tab.editor.cursors.clone(),
                        );
                    }
                }
                let total = tab.editor.buffer.total_chars();
                for cursor in &mut tab.editor.cursors {
                    cursor.anchor = cursor.anchor.min(total);
                    cursor.head = cursor.head.min(total);
                }
                tab.editor.wrap_dirty = true;
                tab.editor.update_minimap();
            }
        }
    }

    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Ctrl+Shift+P => Command Palette
        if input.modifiers.ctrl && input.modifiers.shift && input.key_pressed(Key::P) {
            self.dispatch_action(Action::CommandPalette);
        } else if input.modifiers.ctrl && input.modifiers.shift && input.key_pressed(Key::G) {
            // Ctrl+Shift+G => Source Control
            self.active_sidebar = SidebarTab::Git;
            self.sidebar_visible = true;
        } else if input.modifiers.ctrl && (input.key_pressed(Key::I) || (input.modifiers.alt && input.key_pressed(Key::H))) {
            // Ctrl+I or Alt+H => LSP Hover Documentation
            self.trigger_hover();
        } else if input.modifiers.ctrl && input.key_pressed(Key::Space) {
            // Ctrl+Space => LSP Autocomplete
            self.trigger_completion();
        } else if input.key_pressed(Key::F12) {
            // F12 => LSP Go to Definition
            self.trigger_goto_definition();
        } else if input.modifiers.shift && input.modifiers.alt && input.key_pressed(Key::F) {
            // Shift+Alt+F => Format Document
            self.trigger_formatting();
        } else if input.modifiers.ctrl && input.key_pressed(Key::P) {
            // Ctrl+P => Quick Open
            self.dispatch_action(Action::QuickOpen);
        } else if input.modifiers.ctrl && input.modifiers.shift && input.key_pressed(Key::S) {
            // Ctrl+Shift+S => Save As
            self.dispatch_action(Action::SaveFileAs);
        } else if input.modifiers.ctrl && input.key_pressed(Key::S) {
            // Ctrl+S => Save
            self.dispatch_action(Action::SaveFile);
        } else if input.modifiers.ctrl && input.key_pressed(Key::N) {
            // Ctrl+N => New File
            self.dispatch_action(Action::NewFile);
        } else if input.modifiers.ctrl && input.key_pressed(Key::O) {
            // Ctrl+O => Open File
            self.dispatch_action(Action::OpenFile);
        } else if input.modifiers.ctrl && input.key_pressed(Key::W) {
            // Ctrl+W => Close Tab
            self.dispatch_action(Action::CloseTab);
        } else if input.modifiers.ctrl && input.key_pressed(Key::F) {
            // Ctrl+F => Find
            self.dispatch_action(Action::Find);
        } else if input.modifiers.ctrl && input.key_pressed(Key::B) {
            // Ctrl+B => Toggle Sidebar
            self.dispatch_action(Action::ToggleSidebar);
        } else if input.modifiers.ctrl && input.key_pressed(Key::Z) {
            // Ctrl+Z => Undo
            self.dispatch_action(Action::Undo);
        } else if input.modifiers.ctrl && input.key_pressed(Key::Y) {
            // Ctrl+Y => Redo
            self.dispatch_action(Action::Redo);
        } else if input.modifiers.ctrl && input.key_pressed(Key::Backtick) {
            // Ctrl+` => Toggle Terminal
            self.dispatch_action(Action::ToggleTerminal);
        }
    }
}

fn format_hover_contents(contents: &lsp_types::HoverContents) -> String {
    match contents {
        lsp_types::HoverContents::Scalar(marked) => match marked {
            lsp_types::MarkedString::String(s) => s.clone(),
            lsp_types::MarkedString::LanguageString(ls) => {
                format!("```{}\n{}\n```", ls.language, ls.value)
            }
        },
        lsp_types::HoverContents::Array(arr) => arr
            .iter()
            .map(|marked| match marked {
                lsp_types::MarkedString::String(s) => s.clone(),
                lsp_types::MarkedString::LanguageString(ls) => {
                    format!("```{}\n{}\n```", ls.language, ls.value)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
        lsp_types::HoverContents::Markup(m) => m.value.clone(),
    }
}

fn paths_match(p1: &Path, p2: &Path) -> bool {
    if p1 == p2 {
        return true;
    }
    if let (Ok(c1), Ok(c2)) = (p1.canonicalize(), p2.canonicalize()) {
        return c1 == c2;
    }
    false
}

impl eframe::App for RustForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.egui_ctx.is_none() {
            self.egui_ctx = Some(ctx.clone());
        }
        self.drain_file_events();
        self.sync_lsp_diagnostics();
        self.handle_global_shortcuts(ctx);

        // 1. Top Menu Bar
        TopBottomPanel::top("top_menu_bar")
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_app)
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0)),
            )
            .show(ctx, |ui| {
                render_menu_bar(self, ui);
            });

        // 2. Bottom Status Bar
        TopBottomPanel::bottom("bottom_status_bar")
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_status_bar)
                    .inner_margin(egui::Margin::symmetric(10.0, 3.0)),
            )
            .show(ctx, |ui| {
                render_status_bar(self, ui);
            });

        // 3. Left Activity Bar (Thin icon column)
        SidePanel::left("activity_bar")
            .resizable(false)
            .exact_width(48.0)
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_activity_bar)
                    .stroke(egui::Stroke::new(1.0_f32, self.theme.border)),
            )
            .show(ctx, |ui| {
                render_activity_bar(self, ui);
            });

        // 4. Primary Sidebar (Explorer, Search, Git, Settings)
        if self.sidebar_visible {
            SidePanel::left("primary_sidebar")
                .resizable(true)
                .default_width(self.sidebar_width)
                .width_range(160.0..=500.0)
                .frame(
                    egui::Frame::none()
                        .fill(self.theme.bg_sidebar)
                        .stroke(egui::Stroke::new(1.0_f32, self.theme.border))
                        .inner_margin(egui::Margin::same(8.0)),
                )
                .show(ctx, |ui| {
                    render_sidebar(self, ui);
                });
        }

        // 5. Bottom Panel (Integrated Terminal)
        if self.terminal_visible {
            TopBottomPanel::bottom("terminal_bottom_panel")
                .resizable(true)
                .default_height(self.terminal_height)
                .height_range(100.0..=600.0)
                .frame(
                    egui::Frame::none()
                        .fill(self.theme.bg_terminal)
                        .stroke(egui::Stroke::new(1.0_f32, self.theme.border)),
                )
                .show(ctx, |ui| {
                    render_terminal_panel(self, ui);
                });
        }

        // 6. Central Editor Area
        CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.bg_editor))
            .show(ctx, |ui| {
                render_editor_area(self, ui);
            });

        // 7. Modals: Command Palette & Quick Open overlay
        render_command_palette(self, ctx);
    }
}
