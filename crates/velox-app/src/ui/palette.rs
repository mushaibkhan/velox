use crate::app::VeloxApp;
use crate::ui::icons::Icon;
use egui::{pos2, vec2, Color32, Key, RichText, ScrollArea, Sense, Stroke, TextEdit, Ui, Window};
use velox_search::SearchEngine;

fn get_file_icon_from_path(path: &std::path::Path) -> Icon {
    let name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_lowercase();
    if name.ends_with(".rs") {
        Icon::FileRust
    } else if name.ends_with(".toml") {
        Icon::FileToml
    } else if name.ends_with(".md") {
        Icon::FileMarkdown
    } else {
        Icon::FileGeneric
    }
}

pub fn render_command_palette(app: &mut VeloxApp, ctx: &egui::Context) {
    if !app.command_palette_open && !app.quick_open_open {
        return;
    }

    let screen_rect = ctx.screen_rect();
    let modal_width = 560.0_f32.min(screen_rect.width() - 40.0);
    let modal_height = 360.0_f32.min(screen_rect.height() - 80.0);
    let modal_pos = pos2(
        screen_rect.center().x - (modal_width / 2.0),
        screen_rect.min.y + 60.0,
    );

    // Dark backdrop scrim
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("modal_backdrop"),
    ));
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));

    let title = if app.command_palette_open {
        "Command Palette"
    } else {
        "Quick Open"
    };

    Window::new(title)
        .fixed_pos(modal_pos)
        .fixed_size(vec2(modal_width, modal_height))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(app.theme.bg_panel)
                .rounding(6.0)
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(42, 44, 56)))
                .inner_margin(egui::Margin::same(12.0)),
        )
        .show(ctx, |ui| {
            let input = ui.input(|i| i.clone());

            if input.key_pressed(Key::Escape) {
                app.command_palette_open = false;
                app.quick_open_open = false;
                app.palette_query.clear();
                return;
            }

            let hint = if app.command_palette_open {
                "> Type a command..."
            } else {
                "Search files by name..."
            };

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (s_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                Icon::Search.paint(ui.painter(), s_rect, app.theme.fg_accent);

                let text_edit = TextEdit::singleline(&mut app.palette_query)
                    .hint_text(hint)
                    .font(egui::TextStyle::Heading)
                    .desired_width(ui.available_width());

                let resp = ui.add(text_edit);
                resp.request_focus();
            });

            ui.separator();

            if app.command_palette_open {
                render_command_list(app, ui, &input);
            } else {
                render_file_list(app, ui, &input);
            }
        });
}

fn render_command_list(app: &mut VeloxApp, ui: &mut Ui, input: &egui::InputState) {
    let matches: Vec<&velox_core::CommandItem> = app
        .commands
        .iter()
        .filter(|cmd| {
            if app.palette_query.is_empty() {
                true
            } else {
                SearchEngine::fuzzy_match(&app.palette_query, &cmd.title).is_some()
                    || SearchEngine::fuzzy_match(&app.palette_query, &cmd.category).is_some()
            }
        })
        .collect();

    let total = matches.len();
    if total == 0 {
        ui.label(RichText::new("No matching commands").color(app.theme.fg_muted));
        return;
    }

    if input.key_pressed(Key::ArrowDown) {
        app.palette_selected_idx = (app.palette_selected_idx + 1) % total;
    } else if input.key_pressed(Key::ArrowUp) {
        app.palette_selected_idx = if app.palette_selected_idx == 0 {
            total - 1
        } else {
            app.palette_selected_idx - 1
        };
    }

    if app.palette_selected_idx >= total {
        app.palette_selected_idx = 0;
    }

    let mut action_to_dispatch = None;

    if input.key_pressed(Key::Enter) {
        if let Some(cmd) = matches.get(app.palette_selected_idx) {
            action_to_dispatch = Some(cmd.action.clone());
        }
    }

    ScrollArea::vertical().show(ui, |ui| {
        for (idx, cmd) in matches.iter().enumerate() {
            let is_selected = idx == app.palette_selected_idx;
            let bg_color = if is_selected {
                Color32::from_rgb(30, 41, 59)
            } else {
                Color32::TRANSPARENT
            };
            let text_color = if is_selected {
                Color32::WHITE
            } else {
                app.theme.fg_primary
            };

            egui::Frame::none()
                .fill(bg_color)
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(6.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let full_title = format!("{}: {}", cmd.category, cmd.title);
                        if ui
                            .selectable_label(
                                is_selected,
                                RichText::new(full_title).size(13.0).color(text_color),
                            )
                            .clicked()
                        {
                            action_to_dispatch = Some(cmd.action.clone());
                        }

                        if let Some(shortcut) = &cmd.shortcut {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    RichText::new(shortcut)
                                        .size(11.0)
                                        .color(app.theme.fg_secondary),
                                );
                            });
                        }
                    });
                });
        }
    });

    if let Some(action) = action_to_dispatch {
        app.command_palette_open = false;
        app.palette_query.clear();
        app.dispatch_action(action);
    }
}

fn render_file_list(app: &mut VeloxApp, ui: &mut Ui, input: &egui::InputState) {
    let matches: Vec<&std::path::PathBuf> = app
        .workspace
        .file_cache
        .iter()
        .filter(|path| {
            if app.palette_query.is_empty() {
                true
            } else {
                let name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
                SearchEngine::fuzzy_match(&app.palette_query, name).is_some()
            }
        })
        .collect();

    let total = matches.len();
    if total == 0 {
        ui.label(RichText::new("No matching files").color(app.theme.fg_muted));
        return;
    }

    if input.key_pressed(Key::ArrowDown) {
        app.palette_selected_idx = (app.palette_selected_idx + 1) % total;
    } else if input.key_pressed(Key::ArrowUp) {
        app.palette_selected_idx = if app.palette_selected_idx == 0 {
            total - 1
        } else {
            app.palette_selected_idx - 1
        };
    }

    if app.palette_selected_idx >= total {
        app.palette_selected_idx = 0;
    }

    let mut path_to_open = None;

    if input.key_pressed(Key::Enter) {
        if let Some(p) = matches.get(app.palette_selected_idx) {
            path_to_open = Some((*p).clone());
        }
    }

    ScrollArea::vertical().show(ui, |ui| {
        for (idx, path) in matches.iter().enumerate() {
            let is_selected = idx == app.palette_selected_idx;
            let bg_color = if is_selected {
                Color32::from_rgb(30, 41, 59)
            } else {
                Color32::TRANSPARENT
            };
            let text_color = if is_selected {
                Color32::WHITE
            } else {
                app.theme.fg_primary
            };

            egui::Frame::none()
                .fill(bg_color)
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(6.0, 4.0))
                .show(ui, |ui| {
                    let name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
                    let dir = path.parent().and_then(|p| p.to_str()).unwrap_or("");

                    ui.horizontal(|ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        let file_icon = get_file_icon_from_path(path);
                        file_icon.paint(
                            ui.painter(),
                            icon_rect,
                            if is_selected {
                                app.theme.fg_primary
                            } else {
                                app.theme.fg_accent
                            },
                        );

                        if ui
                            .selectable_label(
                                is_selected,
                                RichText::new(name).size(13.0).color(text_color),
                            )
                            .clicked()
                        {
                            path_to_open = Some((*path).clone());
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(dir).size(10.0).color(app.theme.fg_muted));
                        });
                    });
                });
        }
    });

    if let Some(path) = path_to_open {
        app.quick_open_open = false;
        app.palette_query.clear();
        app.open_file(&path);
    }
}
