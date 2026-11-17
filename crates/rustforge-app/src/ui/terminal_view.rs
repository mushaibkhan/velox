use crate::app::{FocusedPanel, RustForgeApp};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Key, Rect, RichText, ScrollArea, Sense,
    Ui,
};
use rustforge_terminal::{SplitMode, TerminalSession};

pub fn render_terminal_panel(app: &mut RustForgeApp, ui: &mut Ui) {
    // 1. Remove any dead sessions and ensure valid focus index
    app.terminal_manager.remove_dead_sessions();
    if app.terminal_manager.is_empty() {
        app.focused_terminal_idx = None;
    } else if app
        .focused_terminal_idx
        .is_none_or(|i| i >= app.terminal_manager.session_count())
    {
        app.focused_terminal_idx = Some(app.terminal_manager.active_idx());
    }

    ui.vertical(|ui| {
        // 2. Terminal Header Bar (Tabs + Action Buttons)
        let mut tab_to_switch = None;
        let mut tab_to_close = None;
        let mut new_session_requested = false;
        let mut clear_requested = false;
        let mut toggle_split_requested = false;
        let mut hide_panel_requested = false;

        egui::Frame::none()
            .fill(app.theme.bg_panel)
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    let session_count = app.terminal_manager.session_count();
                    let active_idx = app.terminal_manager.active_idx();
                    let split_idx = app.terminal_manager.split_idx();
                    let split_mode = app.terminal_manager.split_mode();
                    let focused_idx = app.focused_terminal_idx;

                    // Interactive Tab Bar
                    for idx in 0..session_count {
                        let is_active = idx == active_idx;
                        let is_split = split_mode != SplitMode::None && split_idx == Some(idx);
                        let is_focused = focused_idx == Some(idx);

                        let bg_color = if is_focused {
                            app.theme.bg_tab_active
                        } else if is_active || is_split {
                            app.theme.bg_tab_active.linear_multiply(0.8)
                        } else {
                            app.theme.bg_tab_inactive
                        };

                        let text_color = if is_focused || is_active {
                            app.theme.fg_primary
                        } else {
                            app.theme.fg_secondary
                        };

                        let title = app
                            .terminal_manager
                            .tab_title(idx)
                            .unwrap_or_else(|| format!("{}: term", idx + 1));

                        let frame = egui::Frame::none()
                            .fill(bg_color)
                            .inner_margin(egui::Margin::symmetric(6.0, 3.0));

                        frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .selectable_label(
                                        is_active,
                                        RichText::new(&title).size(11.0).color(text_color),
                                    )
                                    .clicked()
                                {
                                    tab_to_switch = Some(idx);
                                }

                                if ui
                                    .small_button("✕")
                                    .on_hover_text("Close Session")
                                    .clicked()
                                {
                                    tab_to_close = Some(idx);
                                }
                            });
                        });
                    }

                    // Action Buttons (Right-aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("×").on_hover_text("Hide Terminal").clicked() {
                            hide_panel_requested = true;
                        }
                        if ui.small_button("🗑").on_hover_text("Clear").clicked() {
                            clear_requested = true;
                        }
                        let (split_icon, split_tip) = match app.terminal_manager.split_mode() {
                            SplitMode::None => ("◫", "Split Terminal (Vertical)"),
                            SplitMode::Vertical => ("❚❚", "Split Terminal (Horizontal)"),
                            SplitMode::Horizontal => ("☰", "Unsplit Terminal"),
                        };
                        if ui.small_button(split_icon).on_hover_text(split_tip).clicked() {
                            toggle_split_requested = true;
                        }
                        if ui.small_button("+").on_hover_text("New Session").clicked() {
                            new_session_requested = true;
                        }
                    });
                });
            });

        // 3. Process Header Actions
        if hide_panel_requested {
            app.terminal_visible = false;
        }
        if let Some(idx) = tab_to_switch {
            app.terminal_manager.set_active_idx(idx);
            app.focused_terminal_idx = Some(idx);
            app.focused_panel = FocusedPanel::Terminal;
        }
        if let Some(idx) = tab_to_close {
            app.close_terminal_session(idx);
        }
        if new_session_requested {
            let ctx = ui.ctx().clone();
            app.new_terminal_session(Some(ctx));
        }
        if toggle_split_requested {
            app.terminal_manager.toggle_split();
            if app.terminal_manager.split_mode() != SplitMode::None
                && app.terminal_manager.session_count() < 2
            {
                let ctx = ui.ctx().clone();
                app.new_terminal_session(Some(ctx));
            }
        }
        if clear_requested {
            let target_idx = app
                .focused_terminal_idx
                .unwrap_or_else(|| app.terminal_manager.active_idx());
            if let Some(t) = app.terminal_manager.get_mut(target_idx) {
                t.grid.write().clear_screen();
            }
        }

        ui.separator();

        // 4. Empty State
        if app.terminal_manager.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(RichText::new("No active terminal sessions").color(app.theme.fg_muted));
                if ui.button("Start Terminal").clicked() {
                    let ctx = ui.ctx().clone();
                    app.new_terminal_session(Some(ctx));
                }
            });
            return;
        }

        // 5. Route Keyboard Inputs to Focused Session
        let is_terminal_focused = app.focused_panel == FocusedPanel::Terminal;
        if is_terminal_focused {
            let target_idx = app
                .focused_terminal_idx
                .unwrap_or_else(|| app.terminal_manager.active_idx());
            if let Some(session) = app.terminal_manager.get_mut(target_idx) {
                route_terminal_input(ui, session);
            }
        }

        // 6. Pane Rendering (Single or Split)
        let split_mode = app.terminal_manager.split_mode();
        let active_idx = app.terminal_manager.active_idx();
        let focused_idx = app.focused_terminal_idx;
        let theme = app.theme.clone();
        let available_size = ui.available_size();

        let mut clicked_pane_idx: Option<usize> = None;

        match split_mode {
            SplitMode::None => {
                if let Some(session) = app.terminal_manager.active_session_mut() {
                    let is_pane_focused = is_terminal_focused && focused_idx == Some(active_idx);
                    if render_single_pane(
                        ui,
                        session,
                        active_idx,
                        is_pane_focused,
                        available_size,
                        &theme,
                    ) {
                        clicked_pane_idx = Some(active_idx);
                    }
                }
            }
            SplitMode::Vertical => {
                let split_idx = app
                    .terminal_manager
                    .split_idx()
                    .unwrap_or(if active_idx == 0 { 1 } else { 0 });
                let (active_opt, split_opt) = app.terminal_manager.active_and_split_mut();

                if let (Some(active_session), Some(split_session)) = (active_opt, split_opt) {
                    let sep_w = 4.0;
                    let pane_w = ((available_size.x - sep_w) / 2.0).max(40.0);
                    let pane_h = available_size.y;
                    let pane_size = vec2(pane_w, pane_h);

                    ui.horizontal(|ui| {
                        // Left Pane: Active Session
                        let left_res = ui.allocate_ui_with_layout(
                            pane_size,
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                let is_left_focused =
                                    is_terminal_focused && focused_idx == Some(active_idx);
                                render_single_pane(
                                    ui,
                                    active_session,
                                    active_idx,
                                    is_left_focused,
                                    pane_size,
                                    &theme,
                                )
                            },
                        );
                        if left_res.inner {
                            clicked_pane_idx = Some(active_idx);
                        }

                        ui.separator();

                        // Right Pane: Split Session
                        let right_res = ui.allocate_ui_with_layout(
                            pane_size,
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                let is_right_focused =
                                    is_terminal_focused && focused_idx == Some(split_idx);
                                render_single_pane(
                                    ui,
                                    split_session,
                                    split_idx,
                                    is_right_focused,
                                    pane_size,
                                    &theme,
                                )
                            },
                        );
                        if right_res.inner {
                            clicked_pane_idx = Some(split_idx);
                        }
                    });
                } else if let Some(active_session) = app.terminal_manager.active_session_mut() {
                    let is_pane_focused = is_terminal_focused && focused_idx == Some(active_idx);
                    if render_single_pane(
                        ui,
                        active_session,
                        active_idx,
                        is_pane_focused,
                        available_size,
                        &theme,
                    ) {
                        clicked_pane_idx = Some(active_idx);
                    }
                }
            }
            SplitMode::Horizontal => {
                let split_idx = app
                    .terminal_manager
                    .split_idx()
                    .unwrap_or(if active_idx == 0 { 1 } else { 0 });
                let (active_opt, split_opt) = app.terminal_manager.active_and_split_mut();

                if let (Some(active_session), Some(split_session)) = (active_opt, split_opt) {
                    let sep_h = 4.0;
                    let pane_h = ((available_size.y - sep_h) / 2.0).max(30.0);
                    let pane_w = available_size.x;
                    let pane_size = vec2(pane_w, pane_h);

                    ui.vertical(|ui| {
                        // Top Pane: Active Session
                        let top_res = ui.allocate_ui_with_layout(
                            pane_size,
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                let is_top_focused =
                                    is_terminal_focused && focused_idx == Some(active_idx);
                                render_single_pane(
                                    ui,
                                    active_session,
                                    active_idx,
                                    is_top_focused,
                                    pane_size,
                                    &theme,
                                )
                            },
                        );
                        if top_res.inner {
                            clicked_pane_idx = Some(active_idx);
                        }

                        ui.separator();

                        // Bottom Pane: Split Session
                        let bot_res = ui.allocate_ui_with_layout(
                            pane_size,
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                let is_bot_focused =
                                    is_terminal_focused && focused_idx == Some(split_idx);
                                render_single_pane(
                                    ui,
                                    split_session,
                                    split_idx,
                                    is_bot_focused,
                                    pane_size,
                                    &theme,
                                )
                            },
                        );
                        if bot_res.inner {
                            clicked_pane_idx = Some(split_idx);
                        }
                    });
                } else if let Some(active_session) = app.terminal_manager.active_session_mut() {
                    let is_pane_focused = is_terminal_focused && focused_idx == Some(active_idx);
                    if render_single_pane(
                        ui,
                        active_session,
                        active_idx,
                        is_pane_focused,
                        available_size,
                        &theme,
                    ) {
                        clicked_pane_idx = Some(active_idx);
                    }
                }
            }
        }

        if let Some(idx) = clicked_pane_idx {
            app.focused_terminal_idx = Some(idx);
            app.focused_panel = FocusedPanel::Terminal;
        }
    });
}

fn route_terminal_input(ui: &mut Ui, session: &mut TerminalSession) {
    let input = ui.input(|i| i.clone());

    if input.modifiers.ctrl && input.key_pressed(Key::C) {
        let _ = session.write_bytes(&[3]); // ETX
    } else if input.modifiers.ctrl && input.key_pressed(Key::D) {
        let _ = session.write_bytes(&[4]); // EOT
    } else if input.modifiers.ctrl && input.key_pressed(Key::L) {
        let _ = session.write_bytes(&[12]); // Form Feed
    } else if input.key_pressed(Key::Enter) {
        let _ = session.write_bytes(b"\r");
    } else if input.key_pressed(Key::Backspace) {
        let _ = session.write_bytes(&[8]);
    } else if input.key_pressed(Key::Tab) {
        let _ = session.write_bytes(b"\t");
    } else if input.key_pressed(Key::ArrowUp) {
        let _ = session.write_bytes(b"\x1b[A");
    } else if input.key_pressed(Key::ArrowDown) {
        let _ = session.write_bytes(b"\x1b[B");
    } else if input.key_pressed(Key::ArrowRight) {
        let _ = session.write_bytes(b"\x1b[C");
    } else if input.key_pressed(Key::ArrowLeft) {
        let _ = session.write_bytes(b"\x1b[D");
    } else if input.key_pressed(Key::PageUp) {
        let _ = session.write_bytes(b"\x1b[5~");
    } else if input.key_pressed(Key::PageDown) {
        let _ = session.write_bytes(b"\x1b[6~");
    } else if input.key_pressed(Key::Home) {
        let _ = session.write_bytes(b"\x1b[H");
    } else if input.key_pressed(Key::End) {
        let _ = session.write_bytes(b"\x1b[F");
    }

    for event in &input.events {
        if let egui::Event::Text(t) = event {
            if !input.modifiers.ctrl && !input.modifiers.command && !input.modifiers.alt {
                let _ = session.write_str(t);
            }
        }
    }
}

fn render_single_pane(
    ui: &mut Ui,
    session: &mut TerminalSession,
    session_idx: usize,
    is_focused: bool,
    available_size: egui::Vec2,
    theme: &rustforge_themes::Theme,
) -> bool {
    let font_size = 13.0;
    let font_id = FontId::monospace(font_size);
    let line_height = ui.fonts(|f| f.row_height(&font_id)) + 2.0;
    let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' '));

    let desired_cols = ((available_size.x / char_width).floor() as u16).max(20);
    let desired_rows = ((available_size.y / line_height).floor() as u16).max(5);

    let _ = session.resize(desired_cols, desired_rows);

    let mut clicked = false;

    ui.push_id(session_idx, |ui| {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let grid = session.grid.read();
                let content_w = (grid.cols as f32 * char_width).max(available_size.x);
                let content_h = (grid.rows as f32 * line_height).max(available_size.y);

                let (rect, response) = ui.allocate_exact_size(
                    vec2(content_w, content_h),
                    Sense::click(),
                );

                if response.clicked() {
                    clicked = true;
                }

                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 0.0, theme.bg_terminal);

                if is_focused {
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(1.0_f32, theme.fg_accent.linear_multiply(0.6)),
                    );
                }

                for y in 0..grid.rows {
                    let row_y = rect.min.y + (y as f32 * line_height);

                    for (x, &cell) in grid.cells[y].iter().enumerate().take(grid.cols) {
                        let cell_x = rect.min.x + (x as f32 * char_width);

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
                    let cursor_color = if is_focused {
                        Color32::from_rgba_premultiplied(220, 220, 220, 180)
                    } else {
                        Color32::from_rgba_premultiplied(140, 140, 140, 100)
                    };
                    painter.rect_filled(cur_rect, 0.0, cursor_color);
                }
            });
    });

    clicked
}
