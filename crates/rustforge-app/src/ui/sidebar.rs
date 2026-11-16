use crate::app::{RustForgeApp, SidebarTab};
use egui::{RichText, ScrollArea, TextEdit, Ui};
use rustforge_workspace::FileTreeNode;

pub fn render_sidebar(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        match app.active_sidebar {
            SidebarTab::Explorer => render_explorer_tab(app, ui),
            SidebarTab::Search => render_search_tab(app, ui),
            SidebarTab::Git => render_git_tab(app, ui),
            SidebarTab::Settings => render_settings_tab(app, ui),
        }
    });
}

fn render_explorer_tab(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("EXPLORER").strong().size(11.0).color(app.theme.fg_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("🔄").on_hover_text("Refresh Workspace").clicked() {
                app.workspace.refresh();
            }
            if ui.small_button("📂").on_hover_text("Open Folder").clicked() {
                app.open_folder_dialog();
            }
        });
    });

    ui.separator();

    let root_title = app.workspace.root_name();
    ui.label(RichText::new(format!("📁 {}", root_title)).strong().color(app.theme.fg_primary));

    ui.add_space(4.0);

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut file_to_open = None;
            let mut folder_to_toggle = None;

            if let Some(tree) = &app.workspace.file_tree {
                for child in &tree.children {
                    render_tree_node(child, 0, ui, &mut file_to_open, &mut folder_to_toggle, app);
                }
            } else {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No folder opened").color(app.theme.fg_muted));
                    ui.add_space(8.0);
                    if ui.button("Open Folder").clicked() {
                        app.open_folder_dialog();
                    }
                });
            }

            if let Some(folder_path) = folder_to_toggle {
                app.workspace.toggle_folder(&folder_path);
            }

            if let Some(path) = file_to_open {
                app.open_file(&path);
            }
        });
}

fn render_tree_node(
    node: &FileTreeNode,
    depth: usize,
    ui: &mut Ui,
    file_to_open: &mut Option<std::path::PathBuf>,
    folder_to_toggle: &mut Option<std::path::PathBuf>,
    app: &RustForgeApp,
) {
    let indent = depth as f32 * 12.0;

    ui.horizontal(|ui| {
        ui.add_space(indent);

        if node.is_dir {
            let icon = if node.is_expanded { "📂" } else { "📁" };
            let label = RichText::new(format!("{} {}", icon, node.name))
                .color(app.theme.fg_primary)
                .size(13.0);

            if ui.selectable_label(false, label).clicked() {
                *folder_to_toggle = Some(node.path.clone());
            }
        } else {
            let icon = get_file_icon(&node.name);
            let label = RichText::new(format!("{} {}", icon, node.name))
                .color(app.theme.fg_secondary)
                .size(13.0);

            if ui.selectable_label(false, label).clicked() {
                *file_to_open = Some(node.path.clone());
            }
        }
    });

    if node.is_dir && node.is_expanded {
        for child in &node.children {
            render_tree_node(child, depth + 1, ui, file_to_open, folder_to_toggle, app);
        }
    }
}

fn get_file_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.ends_with(".rs") {
        "🦀"
    } else if lower.ends_with(".py") {
        "🐍"
    } else if lower.ends_with(".js") || lower.ends_with(".ts") || lower.ends_with(".tsx") {
        "📜"
    } else if lower.ends_with(".json") || lower.ends_with(".toml") {
        "⚙"
    } else if lower.ends_with(".md") {
        "📝"
    } else if lower.ends_with(".html") || lower.ends_with(".css") {
        "🌐"
    } else {
        "📄"
    }
}

fn render_search_tab(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.label(RichText::new("SEARCH").strong().size(11.0).color(app.theme.fg_secondary));
    ui.separator();

    ui.horizontal(|ui| {
        let resp = ui.add(
            TextEdit::singleline(&mut app.search_query)
                .hint_text("Search in files...")
                .desired_width(ui.available_width() - 40.0),
        );
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            app.execute_workspace_search();
        }

        if ui.button("🔍").clicked() {
            app.execute_workspace_search();
        }
    });

    ui.add_space(8.0);

    let match_count = app.search_results.len();
    ui.label(
        RichText::new(format!("{} results found", match_count))
            .size(11.0)
            .color(app.theme.fg_muted),
    );

    ui.separator();

    ScrollArea::vertical().show(ui, |ui| {
        let mut jump_target = None;

        for m in &app.search_results {
            let file_name = m.file_path.file_name().unwrap_or_default().to_string_lossy();
            let label_text = format!("{}:{} - {}", file_name, m.line_number, m.line_text.trim());

            if ui
                .selectable_label(
                    false,
                    RichText::new(label_text).size(12.0).color(app.theme.fg_primary),
                )
                .clicked()
            {
                jump_target = Some((m.file_path.clone(), m.line_number));
            }
        }

        if let Some((path, line)) = jump_target {
            app.open_file_at_line(&path, line);
        }
    });
}

fn render_git_tab(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.label(RichText::new("SOURCE CONTROL").strong().size(11.0).color(app.theme.fg_secondary));
    ui.separator();

    ui.label(RichText::new("🌿 Branch: master / main").color(app.theme.fg_primary));
    ui.add_space(8.0);

    ui.label(RichText::new("Working Tree Clean").color(app.theme.fg_muted).size(12.0));
}

fn render_settings_tab(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.label(RichText::new("SETTINGS").strong().size(11.0).color(app.theme.fg_secondary));
    ui.separator();

    ScrollArea::vertical().show(ui, |ui| {
        ui.label(RichText::new("Appearance").strong().color(app.theme.fg_primary));
        ui.horizontal(|ui| {
            ui.label("Theme:");
            if ui.selectable_label(app.settings.theme_mode == rustforge_themes::ThemeMode::DarkModern, "Dark Modern").clicked() {
                app.set_theme(rustforge_themes::ThemeMode::DarkModern);
            }
            if ui.selectable_label(app.settings.theme_mode == rustforge_themes::ThemeMode::TokyoNight, "Tokyo Night").clicked() {
                app.set_theme(rustforge_themes::ThemeMode::TokyoNight);
            }
            if ui.selectable_label(app.settings.theme_mode == rustforge_themes::ThemeMode::MonokaiPro, "Monokai Pro").clicked() {
                app.set_theme(rustforge_themes::ThemeMode::MonokaiPro);
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Editor").strong().color(app.theme.fg_primary));
        
        ui.horizontal(|ui| {
            ui.label("Font Size:");
            ui.add(egui::Slider::new(&mut app.settings.font_size, 10.0..=24.0).step_by(1.0));
        });

        ui.horizontal(|ui| {
            ui.label("Tab Size:");
            ui.add(egui::Slider::new(&mut app.settings.tab_size, 2..=8));
        });

        ui.checkbox(&mut app.settings.show_line_numbers, "Show Line Numbers");
        ui.checkbox(&mut app.settings.word_wrap, "Word Wrap");
    });
}
