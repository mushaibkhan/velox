use crate::app::RustForgeApp;
use egui::{Color32, RichText, Ui};

pub fn render_status_bar(app: &RustForgeApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;

        // Git branch indicator
        ui.label(
            RichText::new("🌿 master")
                .size(11.0)
                .color(Color32::WHITE),
        );

        if let Some(active_idx) = app.active_tab_idx {
            if let Some(tab) = app.tabs.get(active_idx) {
                let dirty = if tab.is_dirty() { "*" } else { "" };
                let file_path = tab
                    .path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| tab.title.clone());

                ui.label(
                    RichText::new(format!("{}{}", file_path, dirty))
                        .size(11.0)
                        .color(Color32::WHITE),
                );
            }
        }

        // Right side indicators
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            ui.label(
                RichText::new(app.theme.name)
                    .size(11.0)
                    .color(Color32::WHITE),
            );

            if let Some(active_idx) = app.active_tab_idx {
                if let Some(tab) = app.tabs.get(active_idx) {
                    ui.label(
                        RichText::new(tab.editor.language.name())
                            .size(11.0)
                            .color(Color32::WHITE),
                    );

                    ui.label(
                        RichText::new(format!("Spaces: {}", tab.editor.tab_size))
                            .size(11.0)
                            .color(Color32::WHITE),
                    );

                    ui.label(
                        RichText::new("UTF-8")
                            .size(11.0)
                            .color(Color32::WHITE),
                    );

                    let pos = tab.editor.primary_position();
                    let total_lines = tab.editor.buffer.line_count();
                    ui.label(
                        RichText::new(format!("Ln {}, Col {} ({})", pos.line + 1, pos.column + 1, total_lines))
                            .size(11.0)
                            .color(Color32::WHITE),
                    );
                }
            }
        });
    });
}
