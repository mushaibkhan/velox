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

enum GitAction {
    Stage(std::path::PathBuf),
    Unstage(std::path::PathBuf),
    Discard(std::path::PathBuf),
    OpenDiff(std::path::PathBuf),
}

fn render_git_tab(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("SOURCE CONTROL").strong().size(11.0).color(app.theme.fg_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("🔄").on_hover_text("Refresh Git Status").clicked() {
                app.refresh_git();
            }
            if app.git_repo.is_some()
                && ui.small_button("+").on_hover_text("Stage All Changes").clicked()
            {
                app.stage_all();
            }
        });
    });
    ui.separator();

    if app.git_repo.is_none() {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Not a git repository").color(app.theme.fg_muted).size(13.0));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Open a folder inside a git repository to track changes.")
                    .color(app.theme.fg_muted)
                    .size(11.0),
            );
        });
        return;
    }

    // Branch & Ahead/Behind display
    let (ahead, behind) = app.git_ahead_behind;
    let sync_text = match (ahead, behind) {
        (0, 0) => String::new(),
        (a, 0) => format!(" ↑{}", a),
        (0, b) => format!(" ↓{}", b),
        (a, b) => format!(" ↑{} ↓{}", a, b),
    };
    let branch_name = if app.git_branch.is_empty() { "HEAD" } else { &app.git_branch };
    ui.label(
        RichText::new(format!("🌿 {}{}", branch_name, sync_text))
            .color(app.theme.fg_primary)
            .strong()
            .size(12.0),
    );
    ui.add_space(6.0);

    // Commit Message & Button
    let commit_resp = ui.add(
        TextEdit::multiline(&mut app.git_commit_message)
            .hint_text("Message (Ctrl+Enter to commit)")
            .desired_rows(2)
            .desired_width(ui.available_width()),
    );
    let ctrl_enter = commit_resp.has_focus()
        && ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter));

    ui.add_space(4.0);

    let can_commit = !app.git_commit_message.trim().is_empty();
    let mut do_commit = false;

    ui.horizontal(|ui| {
        let btn = ui.add_enabled(
            can_commit,
            egui::Button::new(RichText::new("✔ Commit").size(11.0).color(egui::Color32::WHITE))
                .fill(if can_commit { app.theme.fg_accent } else { app.theme.bg_tab_inactive }),
        );
        if btn.clicked() || (ctrl_enter && can_commit) {
            do_commit = true;
        }
    });

    if do_commit {
        let _ = app.commit();
    }

    ui.add_space(6.0);
    ui.separator();

    let is_clean = app.git_status.as_ref().map(|s| s.is_clean()).unwrap_or(true);
    if is_clean {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Working tree clean").color(app.theme.fg_muted).size(12.0));
        });
        return;
    }

    let mut actions = Vec::new();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(status) = &app.git_status {
                // 1. Staged Changes
                if !status.staged.is_empty() {
                    egui::CollapsingHeader::new(
                        RichText::new(format!("STAGED CHANGES ({})", status.staged.len()))
                            .strong()
                            .size(11.0)
                            .color(app.theme.fg_secondary),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for item in &status.staged {
                            render_file_status_row(
                                ui,
                                &item.path,
                                Some(item.status),
                                true,  // is_staged
                                false, // is_untracked
                                &app.theme,
                                &mut actions,
                            );
                        }
                    });
                }

                // 2. Changes (Unstaged)
                if !status.unstaged.is_empty() {
                    egui::CollapsingHeader::new(
                        RichText::new(format!("CHANGES ({})", status.unstaged.len()))
                            .strong()
                            .size(11.0)
                            .color(app.theme.fg_secondary),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for item in &status.unstaged {
                            render_file_status_row(
                                ui,
                                &item.path,
                                Some(item.status),
                                false, // is_staged
                                false, // is_untracked
                                &app.theme,
                                &mut actions,
                            );
                        }
                    });
                }

                // 3. Untracked Files
                if !status.untracked.is_empty() {
                    egui::CollapsingHeader::new(
                        RichText::new(format!("UNTRACKED ({})", status.untracked.len()))
                            .strong()
                            .size(11.0)
                            .color(app.theme.fg_secondary),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for path in &status.untracked {
                            render_file_status_row(
                                ui,
                                path,
                                None,
                                false, // is_staged
                                true,  // is_untracked
                                &app.theme,
                                &mut actions,
                            );
                        }
                    });
                }
            }
        });

    for action in actions {
        match action {
            GitAction::Stage(p) => app.stage_file(&p),
            GitAction::Unstage(p) => app.unstage_file(&p),
            GitAction::Discard(p) => app.discard_file(&p),
            GitAction::OpenDiff(p) => app.open_diff_tab(&p),
        }
    }
}

fn render_file_status_row(
    ui: &mut Ui,
    path: &std::path::Path,
    change_type: Option<rustforge_git::FileChangeType>,
    is_staged: bool,
    is_untracked: bool,
    theme: &rustforge_themes::Theme,
    actions: &mut Vec<GitAction>,
) {
    ui.horizontal(|ui| {
        let (tag, tag_color) = if is_untracked {
            ("U", egui::Color32::from_rgb(115, 201, 145))
        } else {
            match change_type {
                Some(rustforge_git::FileChangeType::Added) => ("A", egui::Color32::from_rgb(115, 201, 145)),
                Some(rustforge_git::FileChangeType::Modified) => ("M", egui::Color32::from_rgb(229, 192, 123)),
                Some(rustforge_git::FileChangeType::Deleted) => ("D", egui::Color32::from_rgb(224, 108, 117)),
                Some(rustforge_git::FileChangeType::Renamed) => ("R", egui::Color32::from_rgb(97, 175, 239)),
                Some(rustforge_git::FileChangeType::Typechange) => ("T", egui::Color32::from_rgb(198, 120, 221)),
                None => (" ", theme.fg_muted),
            }
        };

        // Status badge
        ui.label(RichText::new(tag).strong().size(11.0).color(tag_color));

        // File display name and optional folder
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let parent = path.parent().and_then(|p| {
            let s = p.to_string_lossy();
            if s.is_empty() { None } else { Some(s) }
        });

        let display_text = if let Some(dir) = parent {
            format!("{} ({})", file_name, dir)
        } else {
            file_name.to_string()
        };

        let label_btn = ui.selectable_label(
            false,
            RichText::new(display_text).size(12.0).color(theme.fg_primary),
        ).on_hover_text("Click to view diff");
        if label_btn.clicked() {
            actions.push(GitAction::OpenDiff(path.to_path_buf()));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if is_staged {
                if ui.small_button("−").on_hover_text("Unstage Changes").clicked() {
                    actions.push(GitAction::Unstage(path.to_path_buf()));
                }
            } else if is_untracked {
                if ui.small_button("🗑").on_hover_text("Delete File").clicked() {
                    actions.push(GitAction::Discard(path.to_path_buf()));
                }
                if ui.small_button("+").on_hover_text("Stage File").clicked() {
                    actions.push(GitAction::Stage(path.to_path_buf()));
                }
            } else {
                if ui.small_button("↺").on_hover_text("Discard Changes").clicked() {
                    actions.push(GitAction::Discard(path.to_path_buf()));
                }
                if ui.small_button("+").on_hover_text("Stage Changes").clicked() {
                    actions.push(GitAction::Stage(path.to_path_buf()));
                }
            }
        });
    });
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
