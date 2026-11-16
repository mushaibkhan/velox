use crate::app::RustForgeApp;
use egui::{
    pos2, vec2, Align2, FontId, Key, Rect, RichText, ScrollArea, Sense, Stroke,
    TextEdit, Ui,
};
use rustforge_editor::Direction;

pub fn render_editor_area(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        render_tab_bar(app, ui);

        if let Some(active_idx) = app.active_tab_idx {
            if active_idx < app.tabs.len() {
                render_active_editor(app, active_idx, ui);
            } else {
                render_empty_editor(app, ui);
            }
        } else {
            render_empty_editor(app, ui);
        }
    });
}

fn render_tab_bar(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;

        let mut tab_to_close = None;
        let mut tab_to_switch = None;

        for (idx, tab) in app.tabs.iter().enumerate() {
            let is_active = Some(idx) == app.active_tab_idx;
            let bg_color = if is_active {
                app.theme.bg_tab_active
            } else {
                app.theme.bg_tab_inactive
            };
            let text_color = if is_active {
                app.theme.fg_primary
            } else {
                app.theme.fg_secondary
            };

            let dirty_mark = if tab.is_dirty() { " ●" } else { "" };
            let title_text = format!("{}{}", tab.title, dirty_mark);

            let frame = egui::Frame::none()
                .fill(bg_color)
                .inner_margin(egui::Margin::symmetric(10.0, 6.0));

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(is_active, RichText::new(title_text).color(text_color).size(12.0))
                        .clicked()
                    {
                        tab_to_switch = Some(idx);
                    }

                    if ui
                        .small_button("×")
                        .on_hover_text("Close Tab")
                        .clicked()
                    {
                        tab_to_close = Some(idx);
                    }
                });
            });
        }

        if let Some(idx) = tab_to_switch {
            app.active_tab_idx = Some(idx);
        }

        if let Some(idx) = tab_to_close {
            app.close_tab(idx);
        }
    });
}

fn render_active_editor(app: &mut RustForgeApp, active_idx: usize, ui: &mut Ui) {
    let font_size = app.settings.font_size;
    let font_id = FontId::monospace(font_size);
    let line_height = ui.fonts(|f| f.row_height(&font_id)) + 3.0;
    let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' '));

    // Handle in-editor Find bar if opened
    if app.tabs[active_idx].editor.show_find {
        render_find_bar(app, active_idx, ui);
    }

    let editor = &mut app.tabs[active_idx].editor;

    // Handle keyboard input for active editor
    let input = ui.input(|i| i.clone());

    // Navigation and editing keystrokes
    if input.key_pressed(Key::ArrowLeft) {
        editor.move_cursor(Direction::Left, input.modifiers.shift);
    } else if input.key_pressed(Key::ArrowRight) {
        editor.move_cursor(Direction::Right, input.modifiers.shift);
    } else if input.key_pressed(Key::ArrowUp) {
        editor.move_cursor(Direction::Up, input.modifiers.shift);
    } else if input.key_pressed(Key::ArrowDown) {
        editor.move_cursor(Direction::Down, input.modifiers.shift);
    } else if input.key_pressed(Key::Home) {
        editor.move_to_line_start(input.modifiers.shift);
    } else if input.key_pressed(Key::End) {
        editor.move_to_line_end(input.modifiers.shift);
    } else if input.key_pressed(Key::Backspace) {
        editor.backspace();
    } else if input.key_pressed(Key::Delete) {
        editor.delete_forward();
    } else if input.key_pressed(Key::Enter) {
        editor.insert_newline();
    } else if input.key_pressed(Key::Tab) {
        let spaces = " ".repeat(editor.tab_size);
        editor.insert_text(&spaces);
    }

    // Text typing event handling
    for event in &input.events {
        if let egui::Event::Text(t) = event {
            if !input.modifiers.ctrl && !input.modifiers.command && !input.modifiers.alt {
                editor.insert_text(t);
            }
        }
    }

    // Viewport rendering
    let total_lines = editor.buffer.line_count();
    let gutter_digits = total_lines.to_string().len().max(3);
    let gutter_width = if app.settings.show_line_numbers {
        (gutter_digits as f32 + 2.0) * char_width + 12.0
    } else {
        8.0
    };

    let available_height = ui.available_height();
    let visible_lines = ((available_height / line_height).ceil() as usize).max(5);
    editor.visible_lines = visible_lines;

    let primary_pos = editor.primary_position();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                vec2(ui.available_width(), (total_lines as f32 * line_height).max(available_height)),
                Sense::click_and_drag(),
            );

            let painter = ui.painter_at(rect);

            // Background
            painter.rect_filled(rect, 0.0, app.theme.bg_editor);

            // Gutter background
            let gutter_rect = Rect::from_min_size(rect.min, vec2(gutter_width, rect.height()));
            painter.rect_filled(gutter_rect, 0.0, app.theme.bg_gutter);
            painter.line_segment(
                [gutter_rect.right_top(), gutter_rect.right_bottom()],
                Stroke::new(1.0_f32, app.theme.border),
            );

            // Active line highlight
            let active_line_y = rect.min.y + (primary_pos.line as f32 * line_height);
            let active_line_rect = Rect::from_min_size(
                pos2(rect.min.x + gutter_width, active_line_y),
                vec2(rect.width() - gutter_width, line_height),
            );
            painter.rect_filled(active_line_rect, 0.0, app.theme.line_highlight);

            // Viewport lines only
            let start_line = ((ui.clip_rect().min.y - rect.min.y) / line_height).floor().max(0.0) as usize;
            let end_line = ((ui.clip_rect().max.y - rect.min.y) / line_height).ceil() as usize;
            let end_line = end_line.min(total_lines);

            let _primary_cursor = editor.primary_cursor();

            for line_idx in start_line..end_line {
                let line_y = rect.min.y + (line_idx as f32 * line_height);

                // 1. Line numbers
                if app.settings.show_line_numbers {
                    let num_str = format!("{:>width$}", line_idx + 1, width = gutter_digits);
                    let num_color = if line_idx == primary_pos.line {
                        app.theme.fg_primary
                    } else {
                        app.theme.fg_muted
                    };
                    painter.text(
                        pos2(rect.min.x + 8.0, line_y + 1.0),
                        Align2::LEFT_TOP,
                        num_str,
                        font_id.clone(),
                        num_color,
                    );
                }

                // 2. Syntax-highlighted text tokens
                let tokens = editor.highlight_line(line_idx);
                let line_start_char = editor.buffer.line_to_char(line_idx);

                if let Some(line_str) = editor.buffer.line_to_string(line_idx) {
                    let clean_line = line_str.trim_end_matches(['\r', '\n']);

                    for token in tokens {
                        let token_start_in_line = token.start.saturating_sub(line_start_char);
                        let token_end_in_line = token.end.saturating_sub(line_start_char).min(clean_line.chars().count());

                        if token_start_in_line < token_end_in_line {
                            let text_segment: String = clean_line
                                .chars()
                                .skip(token_start_in_line)
                                .take(token_end_in_line - token_start_in_line)
                                .collect();

                            let token_x = rect.min.x + gutter_width + 8.0 + (token_start_in_line as f32 * char_width);
                            let color = app.theme.get_token_color(token.token_type);

                            painter.text(
                                pos2(token_x, line_y + 1.0),
                                Align2::LEFT_TOP,
                                text_segment,
                                font_id.clone(),
                                color,
                            );
                        }
                    }

                    // 3. Selection highlight for this line
                    for cursor in &editor.cursors {
                        if !cursor.is_empty() {
                            let sel_start = cursor.start();
                            let sel_end = cursor.end();
                            let line_end_char = line_start_char + clean_line.chars().count();

                            if sel_end > line_start_char && sel_start < line_end_char {
                                let line_sel_start = sel_start.max(line_start_char).saturating_sub(line_start_char);
                                let line_sel_end = sel_end.min(line_end_char).saturating_sub(line_start_char);

                                let sel_x = rect.min.x + gutter_width + 8.0 + (line_sel_start as f32 * char_width);
                                let sel_w = (line_sel_end - line_sel_start) as f32 * char_width;

                                let sel_rect = Rect::from_min_size(
                                    pos2(sel_x, line_y),
                                    vec2(sel_w, line_height),
                                );
                                painter.rect_filled(sel_rect, 0.0, app.theme.selection);
                            }
                        }
                    }
                }

                // 4. Cursor rendering
                if line_idx == primary_pos.line {
                    let cursor_x = rect.min.x + gutter_width + 8.0 + (primary_pos.column as f32 * char_width);
                    painter.line_segment(
                        [pos2(cursor_x, line_y), pos2(cursor_x, line_y + line_height - 1.0)],
                        Stroke::new(2.0_f32, app.theme.fg_primary),
                    );
                }
            }

            // Click inside editor to position cursor
            if response.clicked() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    let clicked_line = ((mouse_pos.y - rect.min.y) / line_height).floor().max(0.0) as usize;
                    let clicked_line = clicked_line.min(total_lines.saturating_sub(1));
                    let clicked_col = ((mouse_pos.x - (rect.min.x + gutter_width + 8.0)) / char_width).round().max(0.0) as usize;

                    let char_offset = editor.buffer.position_to_char(rustforge_core::Position::new(clicked_line, clicked_col));
                    editor.cursors = vec![rustforge_core::Selection::point(char_offset)];
                    editor.primary_cursor_idx = 0;
                }
            }
        });
}

fn render_find_bar(app: &mut RustForgeApp, active_idx: usize, ui: &mut Ui) {
    let editor = &mut app.tabs[active_idx].editor;

    egui::Frame::none()
        .fill(app.theme.bg_panel)
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Find:").size(12.0).color(app.theme.fg_secondary));
                let resp = ui.add(
                    TextEdit::singleline(&mut editor.find_state.query)
                        .desired_width(180.0)
                        .hint_text("Search term..."),
                );

                if resp.changed() {
                    let entire_text = editor.buffer.slice_to_string(0, editor.buffer.total_chars());
                    editor.find_state.execute_search(&entire_text);
                }

                let matches_len = editor.find_state.matches.len();
                let match_info = if matches_len > 0 {
                    let current = editor.find_state.current_match_idx.unwrap_or(0) + 1;
                    format!("{}/{}", current, matches_len)
                } else {
                    "0/0".to_string()
                };

                ui.label(RichText::new(match_info).size(11.0).color(app.theme.fg_muted));

                if ui.small_button("▲").on_hover_text("Previous Match").clicked() {
                    if let Some(m) = editor.find_state.prev_match() {
                        editor.cursors = vec![rustforge_core::Selection::new(m.start, m.end)];
                        editor.scroll_cursor_into_view();
                    }
                }

                if ui.small_button("▼").on_hover_text("Next Match").clicked() {
                    if let Some(m) = editor.find_state.next_match() {
                        editor.cursors = vec![rustforge_core::Selection::new(m.start, m.end)];
                        editor.scroll_cursor_into_view();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("×").clicked() {
                        editor.show_find = false;
                    }
                });
            });
        });
}

fn render_empty_editor(app: &RustForgeApp, ui: &mut Ui) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, app.theme.bg_editor);

    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.label(
                RichText::new("RustForge")
                    .size(28.0)
                    .strong()
                    .color(app.theme.fg_primary),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Next-Generation Cross-Platform Code Editor")
                    .size(14.0)
                    .color(app.theme.fg_muted),
            );
            ui.add_space(20.0);

            ui.label(RichText::new("Ctrl+N  •  New File").color(app.theme.fg_secondary));
            ui.label(RichText::new("Ctrl+O  •  Open File").color(app.theme.fg_secondary));
            ui.label(RichText::new("Ctrl+P  •  Quick Open").color(app.theme.fg_secondary));
            ui.label(RichText::new("Ctrl+Shift+P  •  Command Palette").color(app.theme.fg_secondary));
            ui.label(RichText::new("Ctrl+`  •  Integrated Terminal").color(app.theme.fg_secondary));
        });
    });
}
