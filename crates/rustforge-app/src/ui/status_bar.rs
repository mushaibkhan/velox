use crate::app::RustForgeApp;
use crate::ui::icons::Icon;
use egui::{vec2, Color32, RichText, Sense, Ui};

pub fn render_status_bar(app: &RustForgeApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;

        // Git branch indicator
        if app.git_repo.is_some() {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let (icon_rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                Icon::GitBranch.paint(ui.painter(), icon_rect, app.theme.fg_accent);

                let name = if app.git_branch.is_empty() {
                    "HEAD"
                } else {
                    &app.git_branch
                };
                ui.label(RichText::new(name).size(11.0).color(app.theme.fg_primary));

                let (ahead, behind) = app.git_ahead_behind;
                if ahead > 0 {
                    ui.label(
                        RichText::new(format!("↑{}", ahead))
                            .size(10.0)
                            .color(Color32::from_rgb(34, 197, 94)),
                    );
                }
                if behind > 0 {
                    ui.label(
                        RichText::new(format!("↓{}", behind))
                            .size(10.0)
                            .color(Color32::from_rgb(245, 158, 11)),
                    );
                }
            });
        }

        // Active file status
        if let Some(active_idx) = app.active_tab_idx {
            if let Some(tab) = app.tabs.get(active_idx) {
                let dirty = if tab.is_dirty() { "● " } else { "" };
                let file_path = tab
                    .path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| tab.title.clone());

                ui.label(
                    RichText::new(format!("{}{}", dirty, file_path))
                        .size(11.0)
                        .color(app.theme.fg_secondary),
                );
            }
        }

        // Right side indicators
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            ui.label(
                RichText::new(app.theme.name)
                    .size(11.0)
                    .color(app.theme.fg_muted),
            );

            if let Some(active_idx) = app.active_tab_idx {
                if let Some(tab) = app.tabs.get(active_idx) {
                    ui.label(
                        RichText::new(tab.editor.language.name())
                            .size(11.0)
                            .color(app.theme.fg_secondary),
                    );

                    ui.label(
                        RichText::new(format!("Spaces: {}", tab.editor.tab_size))
                            .size(11.0)
                            .color(app.theme.fg_secondary),
                    );

                    ui.label(
                        RichText::new("UTF-8")
                            .size(11.0)
                            .color(app.theme.fg_secondary),
                    );

                    let pos = tab.editor.primary_position();
                    let total_lines = tab.editor.buffer.line_count();
                    ui.label(
                        RichText::new(format!(
                            "Ln {}, Col {} ({})",
                            pos.line + 1,
                            pos.column + 1,
                            total_lines
                        ))
                        .size(11.0)
                        .color(app.theme.fg_primary),
                    );

                    // LSP Diagnostic counts
                    let mut error_count = 0;
                    let mut warning_count = 0;
                    for d in &tab.editor.diagnostics {
                        match d.severity {
                            rustforge_language_server::DiagnosticSeverity::Error => error_count += 1,
                            rustforge_language_server::DiagnosticSeverity::Warning => warning_count += 1,
                            _ => {}
                        }
                    }

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        let err_color = if error_count > 0 {
                            Color32::from_rgb(239, 68, 68)
                        } else {
                            app.theme.fg_muted
                        };
                        let (err_rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        Icon::DiagnosticError.paint(ui.painter(), err_rect, err_color);
                        ui.label(RichText::new(error_count.to_string()).size(11.0).color(err_color));

                        let warn_color = if warning_count > 0 {
                            Color32::from_rgb(245, 158, 11)
                        } else {
                            app.theme.fg_muted
                        };
                        let (warn_rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        Icon::DiagnosticWarning.paint(ui.painter(), warn_rect, warn_color);
                        ui.label(RichText::new(warning_count.to_string()).size(11.0).color(warn_color));
                    });
                }
            }
        });
    });
}
