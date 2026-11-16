use crate::app::{FocusedPanel, RustForgeApp};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Key, Rect, RichText, ScrollArea, Sense,
    Ui,
};

pub fn render_terminal_panel(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        // Terminal Header Bar
        egui::Frame::none()
            .fill(app.theme.bg_panel)
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let title = app
                        .terminal
                        .as_ref()
                        .map(|t| t.title.as_str())
                        .unwrap_or("Terminal");

                    ui.label(
                        RichText::new(format!("⌨ {}", title))
                            .strong()
                            .size(12.0)
                            .color(app.theme.fg_primary),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("×").on_hover_text("Hide Terminal").clicked() {
                            app.terminal_visible = false;
                        }
                        if ui.small_button("🗑").on_hover_text("Clear").clicked() {
                            if let Some(t) = &mut app.terminal {
                                t.grid.write().clear_screen();
                            }
                        }
                        if ui.small_button("+").on_hover_text("New Session").clicked() {
                            let ctx = ui.ctx().clone();
                            app.new_terminal_session(Some(ctx));
                        }
                    });
                });
            });

        ui.separator();

        if let Some(terminal) = &mut app.terminal {
            let font_size = 13.0;
            let font_id = FontId::monospace(font_size);
            let line_height = ui.fonts(|f| f.row_height(&font_id)) + 2.0;
            let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' '));

            let is_focused = app.focused_panel == FocusedPanel::Terminal;

            if is_focused {
                let input = ui.input(|i| i.clone());

                // Key routing into PTY
                if input.modifiers.ctrl && input.key_pressed(Key::C) {
                    let _ = terminal.write_bytes(&[3]); // ETX
                } else if input.modifiers.ctrl && input.key_pressed(Key::D) {
                    let _ = terminal.write_bytes(&[4]); // EOT
                } else if input.modifiers.ctrl && input.key_pressed(Key::L) {
                    let _ = terminal.write_bytes(&[12]); // Form Feed
                } else if input.key_pressed(Key::Enter) {
                    let _ = terminal.write_bytes(b"\r");
                } else if input.key_pressed(Key::Backspace) {
                    let _ = terminal.write_bytes(&[8]);
                } else if input.key_pressed(Key::Tab) {
                    let _ = terminal.write_bytes(b"\t");
                } else if input.key_pressed(Key::ArrowUp) {
                    let _ = terminal.write_bytes(b"\x1b[A");
                } else if input.key_pressed(Key::ArrowDown) {
                    let _ = terminal.write_bytes(b"\x1b[B");
                } else if input.key_pressed(Key::ArrowRight) {
                    let _ = terminal.write_bytes(b"\x1b[C");
                } else if input.key_pressed(Key::ArrowLeft) {
                    let _ = terminal.write_bytes(b"\x1b[D");
                }

                for event in &input.events {
                    if let egui::Event::Text(t) = event {
                        if !input.modifiers.ctrl && !input.modifiers.command && !input.modifiers.alt {
                            let _ = terminal.write_str(t);
                        }
                    }
                }
            }

            let available_size = ui.available_size();
            let desired_cols = ((available_size.x / char_width).floor() as u16).max(20);
            let desired_rows = ((available_size.y / line_height).floor() as u16).max(5);

            let _ = terminal.resize(desired_cols, desired_rows);

            let mut clicked_terminal = false;

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let grid = terminal.grid.read();
                    let (rect, response) = ui.allocate_exact_size(
                        vec2(grid.cols as f32 * char_width, (grid.rows as f32 * line_height).max(available_size.y)),
                        Sense::click(),
                    );

                    if response.clicked() {
                        clicked_terminal = true;
                    }

                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, app.theme.bg_terminal);

                    for y in 0..grid.rows {
                        let row_y = rect.min.y + (y as f32 * line_height);

                        for (x, &cell) in grid.cells[y].iter().enumerate().take(grid.cols) {
                            let cell_x = rect.min.x + (x as f32 * char_width);

                            // Background color if not default black
                            if cell.bg != rustforge_terminal::Color::BLACK {
                                let bg_rect = Rect::from_min_size(
                                    pos2(cell_x, row_y),
                                    vec2(char_width, line_height),
                                );
                                painter.rect_filled(
                                    bg_rect,
                                    0.0,
                                    Color32::from_rgb(cell.bg.r, cell.bg.g, cell.bg.b),
                                );
                            }

                            if cell.ch != ' ' {
                                let fg_color = Color32::from_rgb(cell.fg.r, cell.fg.g, cell.fg.b);
                                painter.text(
                                    pos2(cell_x, row_y),
                                    Align2::LEFT_TOP,
                                    cell.ch.to_string(),
                                    font_id.clone(),
                                    fg_color,
                                );
                            }
                        }
                    }

                    // Cursor
                    if grid.cursor_y < grid.rows && grid.cursor_x < grid.cols {
                        let cur_x = rect.min.x + (grid.cursor_x as f32 * char_width);
                        let cur_y = rect.min.y + (grid.cursor_y as f32 * line_height);
                        let cur_rect = Rect::from_min_size(
                            pos2(cur_x, cur_y),
                            vec2(char_width, line_height),
                        );
                        painter.rect_filled(
                            cur_rect,
                            0.0,
                            Color32::from_rgba_premultiplied(220, 220, 220, 180),
                        );
                    }
                });

            if clicked_terminal {
                app.focused_panel = FocusedPanel::Terminal;
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(RichText::new("Terminal not running").color(app.theme.fg_muted));
                if ui.button("Start Terminal").clicked() {
                    let ctx = ui.ctx().clone();
                    app.new_terminal_session(Some(ctx));
                }
            });
        }
    });
}
