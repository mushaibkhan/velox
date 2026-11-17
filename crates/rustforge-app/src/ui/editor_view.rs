use crate::app::RustForgeApp;
use egui::{
    pos2, vec2, Align2, FontId, Key, Rect, RichText, ScrollArea, Sense, Stroke,
    TextEdit, Ui,
};
use rustforge_editor::{
    buffer_pos_to_visual_row, click_to_scroll_line, thumb_y_to_scroll_line,
    visual_row_to_buffer_pos, Direction,
};

pub fn render_editor_area(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        render_tab_bar(app, ui);

        if let Some(active_idx) = app.active_tab_idx {
            if active_idx < app.tabs.len() {
                if app.tabs[active_idx].has_external_conflict {
                    render_conflict_banner(app, active_idx, ui);
                }
                render_active_editor(app, active_idx, ui);
            } else {
                render_empty_editor(app, ui);
            }
        } else {
            render_empty_editor(app, ui);
        }
    });
}

fn render_conflict_banner(app: &mut RustForgeApp, active_idx: usize, ui: &mut Ui) {
    let warning_bg = egui::Color32::from_rgb(52, 38, 16);
    let warning_border = egui::Color32::from_rgb(190, 130, 30);
    let frame = egui::Frame::none()
        .fill(warning_bg)
        .stroke(egui::Stroke::new(1.0_f32, warning_border))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0));

    let mut action_reload = false;
    let mut action_keep = false;

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⚠ File has been modified on disk.")
                    .color(egui::Color32::from_rgb(255, 195, 80))
                    .strong()
                    .size(12.0),
            );

            ui.add_space(8.0);

            if ui
                .button(
                    RichText::new("Reload from Disk")
                        .color(egui::Color32::WHITE)
                        .size(12.0),
                )
                .on_hover_text("Discard editor changes and load latest disk contents")
                .clicked()
            {
                action_reload = true;
            }

            if ui
                .button(
                    RichText::new("Keep Buffer Changes")
                        .color(egui::Color32::WHITE)
                        .size(12.0),
                )
                .on_hover_text("Ignore disk changes and retain current buffer contents")
                .clicked()
            {
                action_keep = true;
            }
        });
    });

    if action_reload {
        let _ = app.tabs[active_idx].reload_from_disk();
    } else if action_keep {
        app.tabs[active_idx].keep_buffer_changes();
    }
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
            let conflict_mark = if tab.has_external_conflict { " ⚠" } else { "" };
            let title_text = format!("{}{}{}", tab.title, dirty_mark, conflict_mark);

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
            app.focused_panel = crate::app::FocusedPanel::Editor;
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

    let theme = &app.theme;
    let show_line_numbers = app.settings.show_line_numbers;
    let show_minimap_setting = app.settings.show_minimap;
    let editor = &mut app.tabs[active_idx].editor;

    // Handle keyboard input for active editor if focused
    let is_focused = app.focused_panel == crate::app::FocusedPanel::Editor;
    let mut nav_key_pressed = false;
    let mut set_editor_focus = false;
    if is_focused {
        let input = ui.input(|i| i.clone());

        // Navigation and editing keystrokes
        if input.key_pressed(Key::ArrowLeft) {
            editor.move_cursor(Direction::Left, input.modifiers.shift);
            nav_key_pressed = true;
        } else if input.key_pressed(Key::ArrowRight) {
            editor.move_cursor(Direction::Right, input.modifiers.shift);
            nav_key_pressed = true;
        } else if input.key_pressed(Key::ArrowUp) {
            editor.move_cursor(Direction::Up, input.modifiers.shift);
            nav_key_pressed = true;
        } else if input.key_pressed(Key::ArrowDown) {
            editor.move_cursor(Direction::Down, input.modifiers.shift);
            nav_key_pressed = true;
        } else if input.key_pressed(Key::Home) {
            editor.move_to_line_start(input.modifiers.shift);
            nav_key_pressed = true;
        } else if input.key_pressed(Key::End) {
            editor.move_to_line_end(input.modifiers.shift);
            nav_key_pressed = true;
        } else if input.key_pressed(Key::Backspace) {
            editor.backspace();
            nav_key_pressed = true;
        } else if input.key_pressed(Key::Delete) {
            editor.delete_forward();
            nav_key_pressed = true;
        } else if input.key_pressed(Key::Enter) {
            editor.insert_newline();
            nav_key_pressed = true;
        } else if input.key_pressed(Key::Tab) {
            let spaces = " ".repeat(editor.tab_size);
            editor.insert_text(&spaces);
            nav_key_pressed = true;
        }

        // Text typing event handling
        for event in &input.events {
            if let egui::Event::Text(t) = event {
                if !input.modifiers.ctrl && !input.modifiers.command && !input.modifiers.alt {
                    editor.insert_text(t);
                    nav_key_pressed = true;
                }
            }
        }
    }

    // Viewport rendering metrics
    let total_lines = editor.buffer.line_count();
    let gutter_digits = total_lines.to_string().len().max(3);
    let gutter_width = if show_line_numbers {
        (gutter_digits as f32 + 2.0) * char_width + 12.0
    } else {
        8.0
    };

    let available_height = ui.available_height();
    let visible_lines = ((available_height / line_height).ceil() as usize).max(5);
    editor.visible_lines = visible_lines;

    let primary_pos = editor.primary_position();

    let show_minimap = editor.minimap.enabled && show_minimap_setting;
    let minimap_width = if show_minimap { editor.minimap.width } else { 0.0 };
    let editor_width = (ui.available_width() - minimap_width).max(80.0);
    let is_minimap_dragging = editor.minimap.is_dragging;

    // Synchronize word wrap setting with editor state
    editor.word_wrap = app.settings.word_wrap;

    // Word wrap layouts if enabled (cached in EditorState)
    let content_width = (editor_width - gutter_width - 16.0).max(10.0);
    let max_chars_per_row = ((content_width / char_width).floor() as usize).max(10);
    if editor.word_wrap {
        editor.update_wrapped_layouts(max_chars_per_row);
    }
    let total_visual_rows: usize = if editor.word_wrap {
        editor.cached_wrapped_layouts.iter().map(|l| l.rows.len().max(1)).sum()
    } else {
        total_lines
    };

    // Consume any pending scroll request from minimap interaction
    let minimap_target = editor.minimap.scroll_target.take();
    let has_scroll_target = minimap_target.is_some();
    if let Some(target) = minimap_target {
        editor.scroll_line = target;
    }

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        // 1. Main Editor Canvas
        ui.allocate_ui_with_layout(
            vec2(editor_width, available_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                let mut scroll_area = ScrollArea::vertical()
                    .auto_shrink([false, false]);

                if is_minimap_dragging || nav_key_pressed || has_scroll_target {
                    let scroll_v_row = if editor.word_wrap {
                        buffer_pos_to_visual_row(
                            rustforge_core::Position::new(editor.scroll_line, 0),
                            &editor.cached_wrapped_layouts,
                        )
                        .0
                    } else {
                        editor.scroll_line
                    };
                    scroll_area = scroll_area.vertical_scroll_offset(scroll_v_row as f32 * line_height);
                }

                scroll_area.show(ui, |ui| {
                    let canvas_height = (total_visual_rows as f32 * line_height).max(available_height);
                    let (rect, response) = ui.allocate_exact_size(
                        vec2(ui.available_width(), canvas_height),
                        Sense::click_and_drag(),
                    );

                    let painter = ui.painter_at(rect);

                    // Background
                    painter.rect_filled(rect, 0.0, theme.bg_editor);

                    // Gutter background
                    let gutter_rect = Rect::from_min_size(rect.min, vec2(gutter_width, rect.height()));
                    painter.rect_filled(gutter_rect, 0.0, theme.bg_gutter);
                    painter.line_segment(
                        [gutter_rect.right_top(), gutter_rect.right_bottom()],
                        Stroke::new(1.0_f32, theme.border),
                    );

                    // Active line highlight
                    let (cur_cursor_v_row, cur_cursor_v_col) = if editor.word_wrap {
                        buffer_pos_to_visual_row(primary_pos, &editor.cached_wrapped_layouts)
                    } else {
                        (primary_pos.line, primary_pos.column)
                    };

                    let active_line_y = rect.min.y + (cur_cursor_v_row as f32 * line_height);
                    let active_line_rect = Rect::from_min_size(
                        pos2(rect.min.x + gutter_width, active_line_y),
                        vec2(rect.width() - gutter_width, line_height),
                    );
                    painter.rect_filled(active_line_rect, 0.0, theme.line_highlight);

                    if editor.word_wrap {
                        let start_v_row = ((ui.clip_rect().min.y - rect.min.y) / line_height).floor().max(0.0) as usize;
                        let end_v_row = ((ui.clip_rect().max.y - rect.min.y) / line_height).ceil() as usize;
                        let end_v_row = end_v_row.min(total_visual_rows);

                        if !is_minimap_dragging && !has_scroll_target {
                            let top_pos = visual_row_to_buffer_pos(start_v_row, 0, &editor.cached_wrapped_layouts);
                            editor.scroll_line = top_pos.line;
                        }

                        let mut v_row_acc = 0;
                        for layout in &editor.cached_wrapped_layouts {
                            if v_row_acc >= end_v_row {
                                break;
                            }
                            let line_idx = layout.line_index;
                            let layout_len = layout.rows.len().max(1);

                            if v_row_acc + layout_len > start_v_row && v_row_acc < end_v_row {
                                let tokens = editor.highlight_line(line_idx);
                                let line_start_char = editor.buffer.line_to_char(line_idx);
                                let clean_line_opt = editor.buffer.line_to_string(line_idx);
                                let clean_line = clean_line_opt.as_deref().map(|s| s.trim_end_matches(['\r', '\n'])).unwrap_or("");

                                for (sub_idx, row_range) in layout.rows.iter().enumerate() {
                                    let v_row = v_row_acc + sub_idx;
                                    if v_row >= start_v_row && v_row < end_v_row {
                                        let row_y = rect.min.y + (v_row as f32 * line_height);

                                        // Git gutter marker strip
                                        if let Some(change) = editor.line_git_change(line_idx + 1) {
                                            if let Some(color) = git_gutter_color(change) {
                                                let strip_rect = Rect::from_min_size(
                                                    pos2(gutter_rect.right() - 3.0, row_y),
                                                    vec2(3.0, line_height),
                                                );
                                                painter.rect_filled(strip_rect, 0.0, color);
                                            }
                                        }

                                        // 1. Line numbers and soft continuation marker
                                        if show_line_numbers {
                                            if sub_idx == 0 {
                                                let num_str = format!("{:>width$}", line_idx + 1, width = gutter_digits);
                                                let num_color = if line_idx == primary_pos.line {
                                                    theme.fg_primary
                                                } else {
                                                    theme.fg_muted
                                                };
                                                painter.text(
                                                    pos2(rect.min.x + 8.0, row_y + 1.0),
                                                    Align2::LEFT_TOP,
                                                    num_str,
                                                    font_id.clone(),
                                                    num_color,
                                                );
                                            } else {
                                                let marker_str = format!("{:>width$}", "·", width = gutter_digits);
                                                painter.text(
                                                    pos2(rect.min.x + 8.0, row_y + 1.0),
                                                    Align2::LEFT_TOP,
                                                    marker_str,
                                                    font_id.clone(),
                                                    theme.fg_muted,
                                                );
                                            }
                                        }

                                        // 2. Syntax-highlighted text tokens in this sub-row
                                        for token in &tokens {
                                            let token_start_in_line = token.start.saturating_sub(line_start_char);
                                            let token_end_in_line = token.end.saturating_sub(line_start_char).min(clean_line.chars().count());

                                            let seg_start = token_start_in_line.max(row_range.start);
                                            let seg_end = token_end_in_line.min(row_range.end);
                                            if seg_start < seg_end {
                                                let text_segment: String = clean_line
                                                    .chars()
                                                    .skip(seg_start)
                                                    .take(seg_end - seg_start)
                                                    .collect();

                                                let col_offset = seg_start - row_range.start;
                                                let token_x = rect.min.x + gutter_width + 8.0 + (col_offset as f32 * char_width);
                                                let color = theme.get_token_color(token.token_type);

                                                painter.text(
                                                    pos2(token_x, row_y + 1.0),
                                                    Align2::LEFT_TOP,
                                                    text_segment,
                                                    font_id.clone(),
                                                    color,
                                                );
                                            }
                                        }

                                        // 3. Selection highlight for this sub-row
                                        for cursor in &editor.cursors {
                                            if !cursor.is_empty() {
                                                let sel_start = cursor.start();
                                                let sel_end = cursor.end();
                                                let row_start_char = line_start_char + row_range.start;
                                                let row_end_char = line_start_char + row_range.end;

                                                if sel_end > row_start_char && sel_start < row_end_char {
                                                    let sub_sel_start = sel_start.max(row_start_char) - row_start_char;
                                                    let sub_sel_end = sel_end.min(row_end_char) - row_start_char;

                                                    let sel_x = rect.min.x + gutter_width + 8.0 + (sub_sel_start as f32 * char_width);
                                                    let sel_w = (sub_sel_end - sub_sel_start) as f32 * char_width;

                                                    let sel_rect = Rect::from_min_size(
                                                        pos2(sel_x, row_y),
                                                        vec2(sel_w, line_height),
                                                    );
                                                    painter.rect_filled(sel_rect, 0.0, theme.selection);
                                                }
                                            }
                                        }

                                        // 4. Cursor rendering
                                        if v_row == cur_cursor_v_row {
                                            let cursor_x = rect.min.x + gutter_width + 8.0 + (cur_cursor_v_col as f32 * char_width);
                                            painter.line_segment(
                                                [pos2(cursor_x, row_y), pos2(cursor_x, row_y + line_height - 1.0)],
                                                Stroke::new(2.0_f32, theme.fg_primary),
                                            );
                                        }
                                    }
                                }
                            }
                            v_row_acc += layout_len;
                        }
                    } else {
                        // Viewport lines only (unwrapped)
                        let start_line = ((ui.clip_rect().min.y - rect.min.y) / line_height).floor().max(0.0) as usize;
                        let end_line = ((ui.clip_rect().max.y - rect.min.y) / line_height).ceil() as usize;
                        let end_line = end_line.min(total_lines);

                        if !is_minimap_dragging && !has_scroll_target {
                            editor.scroll_line = start_line;
                        }

                        for line_idx in start_line..end_line {
                            let line_y = rect.min.y + (line_idx as f32 * line_height);

                            // Git gutter marker strip
                            if let Some(change) = editor.line_git_change(line_idx + 1) {
                                if let Some(color) = git_gutter_color(change) {
                                    let strip_rect = Rect::from_min_size(
                                        pos2(gutter_rect.right() - 3.0, line_y),
                                        vec2(3.0, line_height),
                                    );
                                    painter.rect_filled(strip_rect, 0.0, color);
                                }
                            }

                            // 1. Line numbers
                            if show_line_numbers {
                                let num_str = format!("{:>width$}", line_idx + 1, width = gutter_digits);
                                let num_color = if line_idx == primary_pos.line {
                                    theme.fg_primary
                                } else {
                                    theme.fg_muted
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
                                        let color = theme.get_token_color(token.token_type);

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
                                            painter.rect_filled(sel_rect, 0.0, theme.selection);
                                        }
                                    }
                                }
                            }

                            // 4. Cursor rendering
                            if line_idx == primary_pos.line {
                                let cursor_x = rect.min.x + gutter_width + 8.0 + (primary_pos.column as f32 * char_width);
                                painter.line_segment(
                                    [pos2(cursor_x, line_y), pos2(cursor_x, line_y + line_height - 1.0)],
                                    Stroke::new(2.0_f32, theme.fg_primary),
                                );
                            }
                        }
                    }

                    // Click inside editor to position cursor and grant focus
                    if response.clicked() {
                        set_editor_focus = true;
                        if let Some(mouse_pos) = response.interact_pointer_pos() {
                            if editor.word_wrap {
                                let clicked_v_row = ((mouse_pos.y - rect.min.y) / line_height).floor().max(0.0) as usize;
                                let clicked_v_row = clicked_v_row.min(total_visual_rows.saturating_sub(1));
                                let clicked_col = ((mouse_pos.x - (rect.min.x + gutter_width + 8.0)) / char_width).round().max(0.0) as usize;

                                let buf_pos = visual_row_to_buffer_pos(clicked_v_row, clicked_col, &editor.cached_wrapped_layouts);
                                let char_offset = editor.buffer.position_to_char(buf_pos);
                                editor.cursors = vec![rustforge_core::Selection::point(char_offset)];
                                editor.primary_cursor_idx = 0;
                            } else {
                                let clicked_line = ((mouse_pos.y - rect.min.y) / line_height).floor().max(0.0) as usize;
                                let clicked_line = clicked_line.min(total_lines.saturating_sub(1));
                                let clicked_col = ((mouse_pos.x - (rect.min.x + gutter_width + 8.0)) / char_width).round().max(0.0) as usize;

                                let char_offset = editor.buffer.position_to_char(rustforge_core::Position::new(clicked_line, clicked_col));
                                editor.cursors = vec![rustforge_core::Selection::point(char_offset)];
                                editor.primary_cursor_idx = 0;
                            }
                        }
                    }
                });
            },
        );

        // 2. Minimap Gutter (100px wide sleek bar on the right side)
        if show_minimap {
            let minimap_interacted = render_minimap(editor, theme, ui, available_height, line_height);
            if minimap_interacted {
                set_editor_focus = true;
            }
        }
    });

    if set_editor_focus {
        app.focused_panel = crate::app::FocusedPanel::Editor;
    }
}

fn render_minimap(
    editor: &mut rustforge_editor::EditorState,
    theme: &rustforge_themes::Theme,
    ui: &mut Ui,
    height: f32,
    _line_height: f32,
) -> bool {
    let minimap_width = editor.minimap.width.max(60.0);
    let total_lines = editor.buffer.line_count();
    let primary_pos = editor.primary_position();
    let mut user_interacted = false;

    let (minimap_rect, response) = ui.allocate_exact_size(
        vec2(minimap_width, height),
        Sense::click_and_drag(),
    );

    let painter = ui.painter_at(minimap_rect);

    // 1. Background and left border separator
    painter.rect_filled(minimap_rect, 0.0, theme.bg_gutter);
    painter.line_segment(
        [minimap_rect.left_top(), minimap_rect.left_bottom()],
        Stroke::new(1.0_f32, theme.border),
    );

    // 2. Harmonized effective track height
    let track_height = editor.minimap.effective_height(total_lines, height);

    // 3. Code density bars with zero-allocation slice iteration
    if total_lines > 0 {
        let lines = editor.minimap.cached_lines();
        let scale_y = track_height / total_lines as f32;
        let minimap_line_h = (scale_y * 0.8).clamp(1.0, 3.0);
        let line_step = scale_y;

        let content_width = (minimap_width - 12.0).max(10.0);
        let mut last_drawn_y = -10.0_f32;

        for line in lines.iter().take(total_lines) {
            if line.length > 0 && !line.is_empty() {
                let y = minimap_rect.min.y + (line.line as f32 * line_step);
                if y >= minimap_rect.min.y && y + minimap_line_h <= minimap_rect.max.y + 2.0 {
                    // Skip redundant sub-pixel lines on huge files unless it's the active line
                    if y - last_drawn_y < 1.0 && line.line != primary_pos.line {
                        continue;
                    }
                    last_drawn_y = y;

                    let bar_x = minimap_rect.min.x + 6.0 + (line.indent_ratio * content_width);
                    let bar_w = (line.content_width_ratio() * content_width).max(2.0);
                    let bar_w = bar_w.min((minimap_rect.max.x - bar_x - 4.0).max(2.0));

                    let bar_color = if line.line == primary_pos.line {
                        egui::Color32::from_rgba_unmultiplied(
                            theme.fg_primary.r(),
                            theme.fg_primary.g(),
                            theme.fg_primary.b(),
                            160,
                        )
                    } else {
                        egui::Color32::from_rgba_unmultiplied(
                            theme.fg_secondary.r(),
                            theme.fg_secondary.g(),
                            theme.fg_secondary.b(),
                            80,
                        )
                    };

                    painter.rect_filled(
                        Rect::from_min_size(pos2(bar_x, y), vec2(bar_w, minimap_line_h)),
                        0.5,
                        bar_color,
                    );
                }
            }
        }
    }

    // 4. Draggable semi-transparent viewport thumb
    let thumb = editor.minimap.compute_thumb(
        editor.scroll_line,
        editor.visible_lines,
        total_lines,
        track_height,
    );
    let thumb_rect = Rect::from_min_size(
        pos2(minimap_rect.min.x, minimap_rect.min.y + thumb.y),
        vec2(minimap_width, thumb.height),
    );

    let is_hovered = response.hover_pos().is_some_and(|p| thumb_rect.contains(p));
    let is_dragging = editor.minimap.is_dragging;

    let (thumb_fill, thumb_stroke) = if is_dragging {
        (
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 45),
            Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 80)),
        )
    } else if is_hovered {
        (
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
        )
    } else {
        (
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
        )
    };

    painter.rect(thumb_rect, 2.0, thumb_fill, thumb_stroke);

    // 5. Interactivity: Click and Drag to update scroll_line with persistence
    if response.drag_started() {
        editor.minimap.is_dragging = true;
        user_interacted = true;
        if let Some(pos) = response.interact_pointer_pos() {
            let rel_y = pos.y - minimap_rect.min.y;
            if !thumb_rect.contains(pos) {
                let target = click_to_scroll_line(
                    rel_y,
                    total_lines,
                    editor.visible_lines,
                    track_height,
                );
                editor.scroll_line = target;
                editor.minimap.scroll_target = Some(target);
            }
        }
        ui.ctx().request_repaint();
    }

    if response.dragged() {
        editor.minimap.is_dragging = true;
        user_interacted = true;
        if let Some(pos) = response.interact_pointer_pos() {
            let rel_y = pos.y - minimap_rect.min.y;
            let target_thumb_y = rel_y - (thumb.height / 2.0);
            let target = thumb_y_to_scroll_line(
                target_thumb_y,
                thumb.height,
                total_lines,
                editor.visible_lines,
                track_height,
            );
            editor.scroll_line = target;
            editor.minimap.scroll_target = Some(target);
        }
        ui.ctx().request_repaint();
    }

    if response.drag_stopped() {
        editor.minimap.is_dragging = false;
        ui.ctx().request_repaint();
    }

    if response.clicked() {
        user_interacted = true;
        if let Some(pos) = response.interact_pointer_pos() {
            let rel_y = pos.y - minimap_rect.min.y;
            let target = click_to_scroll_line(
                rel_y,
                total_lines,
                editor.visible_lines,
                track_height,
            );
            editor.scroll_line = target;
            editor.minimap.scroll_target = Some(target);
            ui.ctx().request_repaint();
        }
    }

    user_interacted
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

fn git_gutter_color(change: rustforge_git::LineChange) -> Option<egui::Color32> {
    match change {
        rustforge_git::LineChange::Added => Some(egui::Color32::from_rgb(75, 175, 80)),
        rustforge_git::LineChange::Modified => Some(egui::Color32::from_rgb(60, 140, 230)),
        rustforge_git::LineChange::Deleted => Some(egui::Color32::from_rgb(230, 80, 80)),
        rustforge_git::LineChange::Unchanged => None,
    }
}
