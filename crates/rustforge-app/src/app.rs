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
use rustforge_terminal::TerminalSession;
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
    pub terminal: Option<TerminalSession>,
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

        let mut app = Self {
            theme,
            settings,
            workspace,
            tabs: Vec::new(),
            active_tab_idx: None,
            terminal: None,
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
        };

        // Try to start default terminal session with egui repaint callback
        app.new_terminal_session(Some(cc.egui_ctx.clone()));

        // Create an initial welcome/untitled file
        app.new_file();

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
        // If already open, switch to it
        for (idx, tab) in self.tabs.iter().enumerate() {
            if let Some(p) = tab.path() {
                if p == path {
                    self.active_tab_idx = Some(idx);
                    return;
                }
            }
        }

        if let Ok(buffer) = TextBuffer::from_file(path) {
            let editor = EditorState::new(buffer);
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
            self.workspace.open(&folder);
        }
    }

    pub fn open_file_dialog(&mut self) {
        if let Some(file) = rfd::FileDialog::new().pick_file() {
            self.open_file(&file);
        }
    }

    pub fn save_current_file(&mut self) {
        if let Some(idx) = self.active_tab_idx {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if tab.path().is_some() {
                    let _ = tab.editor.buffer.save();
                    tab.update_title();
                } else {
                    self.save_current_file_as();
                }
            }
        }
    }

    pub fn save_current_file_as(&mut self) {
        if let Some(idx) = self.active_tab_idx {
            if let Some(dest) = rfd::FileDialog::new().save_file() {
                if let Some(tab) = self.tabs.get_mut(idx) {
                    let _ = tab.editor.buffer.save_as(&dest);
                    tab.update_title();
                }
            }
        }
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
            Ok(session) => self.terminal = Some(session),
            Err(e) => tracing::warn!("Failed to spawn terminal session: {}", e),
        }
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
            Action::Quit => std::process::exit(0),
            _ => {}
        }
    }

    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Ctrl+Shift+P => Command Palette
        if input.modifiers.ctrl && input.modifiers.shift && input.key_pressed(Key::P) {
            self.dispatch_action(Action::CommandPalette);
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

impl eframe::App for RustForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
