#![allow(dead_code)]

use egui::{
    pos2, vec2, Align2, Color32, FontId, Rect, RichText, ScrollArea, Sense, Stroke, Ui,
};
use rustforge_git::{DiffLine, FileDiff, LineChange};
use rustforge_themes::Theme;

/// Visual diff viewer component displaying file differences.
#[derive(Debug, Clone)]
pub struct DiffView {
    pub side_by_side: bool,
    pub old_label: String,
    pub new_label: String,
}

impl DiffView {
    pub fn new(old_label: impl Into<String>, new_label: impl Into<String>) -> Self {
        Self {
            side_by_side: false,
            old_label: old_label.into(),
            new_label: new_label.into(),
        }
    }

    pub fn with_side_by_side(mut self, side_by_side: bool) -> Self {
        self.side_by_side = side_by_side;
        self
    }

    pub fn render(&mut self, ui: &mut Ui, theme: &Theme, diff: &FileDiff) {
        render_diff_view_impl(
            ui,
            theme,
            diff,
            &self.old_label,
            &self.new_label,
            Some(&mut self.side_by_side),
        );
    }
}

/// Standalone function to render a visual diff view.
pub fn render_diff_view(
    ui: &mut Ui,
    theme: &Theme,
    diff: &FileDiff,
    old_label: &str,
    new_label: &str,
    side_by_side: bool,
) {
    let id = ui.make_persistent_id("diff_view_side_by_side");
    let mut sbs = ui.data_mut(|d| *d.get_temp_mut_or(id, side_by_side));
    let prev_sbs = sbs;
    render_diff_view_impl(ui, theme, diff, old_label, new_label, Some(&mut sbs));
    if sbs != prev_sbs {
        ui.data_mut(|d| d.insert_temp(id, sbs));
    }
}

/// Safely count character offset and length within a byte-indexed span without UTF-8 slicing.
fn safe_char_count(s: &str, byte_start: usize, byte_end: usize) -> (usize, usize) {
    let mut col_start = 0;
    let mut col_len = 0;
    for (byte_idx, _) in s.char_indices() {
        if byte_idx < byte_start {
            col_start += 1;
        } else if byte_idx < byte_end {
            col_len += 1;
        }
    }
    (col_start, col_len)
}

fn diff_colors(change: LineChange, theme: &Theme) -> (Color32, Color32, Color32) {
    match change {
        LineChange::Added => (
            Color32::from_rgba_unmultiplied(75, 175, 80, 45),   // line bg
            Color32::from_rgba_unmultiplied(75, 175, 80, 120),  // inline span bg
            Color32::from_rgb(120, 220, 130),                   // fg text
        ),
        LineChange::Deleted => (
            Color32::from_rgba_unmultiplied(230, 80, 80, 45),   // line bg
            Color32::from_rgba_unmultiplied(230, 80, 80, 120),  // inline span bg
            Color32::from_rgb(245, 120, 120),                   // fg text
        ),
        LineChange::Modified => (
            Color32::from_rgba_unmultiplied(60, 140, 230, 40),  // line bg
            Color32::from_rgba_unmultiplied(60, 140, 230, 115), // inline span bg
            Color32::from_rgb(110, 180, 250),                   // fg text
        ),
        LineChange::Unchanged => (
            Color32::TRANSPARENT,
            Color32::TRANSPARENT,
            theme.fg_primary,
        ),
    }
}

fn render_diff_view_impl(
    ui: &mut Ui,
    theme: &Theme,
    diff: &FileDiff,
    old_label: &str,
    new_label: &str,
    mut sbs_opt: Option<&mut bool>,
) {
    ui.vertical(|ui| {
        // --- Diff Header ---
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            ui.label(
                RichText::new(old_label)
                    .color(theme.fg_secondary)
                    .monospace(),
            );
            ui.label(RichText::new("➔").color(theme.fg_muted));
            ui.label(
                RichText::new(new_label)
                    .color(theme.fg_primary)
                    .strong()
                    .monospace(),
            );

            ui.separator();

            // Statistics (+X, -Y, ~Z)
            ui.label(
                RichText::new(format!("+{}", diff.additions))
                    .color(Color32::from_rgb(75, 175, 80))
                    .strong(),
            );
            ui.label(
                RichText::new(format!("-{}", diff.deletions))
                    .color(Color32::from_rgb(230, 80, 80))
                    .strong(),
            );
            if diff.modifications > 0 {
                ui.label(
                    RichText::new(format!("~{}", diff.modifications))
                        .color(Color32::from_rgb(60, 140, 230))
                        .strong(),
                );
            }

            // View mode toggle
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(ref mut sbs) = sbs_opt {
                    let cur_sbs = **sbs;
                    if ui.selectable_label(cur_sbs, "Side-by-side").clicked() {
                        **sbs = true;
                    }
                    if ui.selectable_label(!cur_sbs, "Unified").clicked() {
                        **sbs = false;
                    }
                }
            });
        });

        ui.separator();

        if diff.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new(format!("No changes between {old_label} and {new_label}"))
                        .color(theme.fg_muted)
                        .size(13.0),
                );
            });
            return;
        }

        let is_side_by_side = sbs_opt.as_ref().map(|s| **s).unwrap_or(false);

        if is_side_by_side {
            render_side_by_side_view(ui, theme, diff);
        } else {
            render_unified_view(ui, theme, diff);
        }
    });
}

// ---------------------------------------------------------------------------
// Unified View Rendering
// ---------------------------------------------------------------------------

enum UnifiedRow<'a> {
    Header(&'a str),
    Line(&'a DiffLine),
}

fn render_unified_view(ui: &mut Ui, theme: &Theme, diff: &FileDiff) {
    let font_id = FontId::monospace(12.5);
    let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' '));
    let line_height = ui.fonts(|f| f.row_height(&font_id)) + 4.0;

    let mut rows = Vec::new();
    let mut max_chars = 40;

    for hunk in &diff.hunks {
        rows.push(UnifiedRow::Header(&hunk.header));
        for line in &hunk.lines {
            max_chars = max_chars.max(line.content.len());
            rows.push(UnifiedRow::Line(line));
        }
    }

    let gutter_width = 10.0 * char_width + 16.0; // 4 old + 4 new + 2 sign + padding
    let content_width = ((max_chars + 16) as f32 * char_width).max(ui.available_width());
    let total_height = rows.len() as f32 * line_height;

    ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (rect, _response) = ui.allocate_exact_size(
                vec2(content_width, total_height),
                Sense::hover(),
            );
            let painter = ui.painter_at(rect);

            // Background
            painter.rect_filled(rect, 0.0, theme.bg_editor);

            let clip = ui.clip_rect();
            let start_idx = ((clip.min.y - rect.min.y) / line_height).floor().max(0.0) as usize;
            let end_idx = (((clip.max.y - rect.min.y) / line_height).ceil() as usize).min(rows.len());

            for (idx, row) in rows.iter().enumerate().take(end_idx).skip(start_idx) {
                let row_y = rect.min.y + (idx as f32 * line_height);
                let row_rect = Rect::from_min_size(
                    pos2(rect.min.x, row_y),
                    vec2(content_width, line_height),
                );

                match row {
                    UnifiedRow::Header(header_text) => {
                        // Hunk header styling
                        let header_bg = Color32::from_rgba_unmultiplied(60, 140, 230, 25);
                        painter.rect_filled(row_rect, 0.0, header_bg);
                        painter.text(
                            pos2(rect.min.x + 8.0, row_y + 2.0),
                            Align2::LEFT_TOP,
                            *header_text,
                            font_id.clone(),
                            theme.syntax_constant,
                        );
                    }
                    UnifiedRow::Line(line) => {
                        let (bg_color, span_bg, fg_color) = diff_colors(line.change, theme);

                        // Line background tint
                        if bg_color != Color32::TRANSPARENT {
                            painter.rect_filled(row_rect, 0.0, bg_color);
                        }

                        // Gutter line numbers (old and new)
                        let old_str = line.old_line_no.map(|n| format!("{:>4}", n)).unwrap_or_else(|| "    ".to_string());
                        let new_str = line.new_line_no.map(|n| format!("{:>4}", n)).unwrap_or_else(|| "    ".to_string());
                        let sign_str = match line.change {
                            LineChange::Added => "+ ",
                            LineChange::Deleted => "- ",
                            LineChange::Modified => "~ ",
                            LineChange::Unchanged => "  ",
                        };

                        painter.text(
                            pos2(rect.min.x + 4.0, row_y + 2.0),
                            Align2::LEFT_TOP,
                            old_str,
                            font_id.clone(),
                            theme.fg_muted,
                        );

                        painter.text(
                            pos2(rect.min.x + 4.0 + (4.5 * char_width), row_y + 2.0),
                            Align2::LEFT_TOP,
                            new_str,
                            font_id.clone(),
                            theme.fg_muted,
                        );

                        let sign_color = match line.change {
                            LineChange::Added => Color32::from_rgb(75, 175, 80),
                            LineChange::Deleted => Color32::from_rgb(230, 80, 80),
                            LineChange::Modified => Color32::from_rgb(60, 140, 230),
                            LineChange::Unchanged => theme.fg_muted,
                        };

                        painter.text(
                            pos2(rect.min.x + 4.0 + (9.0 * char_width), row_y + 2.0),
                            Align2::LEFT_TOP,
                            sign_str,
                            font_id.clone(),
                            sign_color,
                        );

                        // Line content and inline spans
                        let text_x = rect.min.x + gutter_width;
                        let clean_content = line.content.trim_end_matches(['\r', '\n']);

                        // Highlight inline character spans
                        for span in &line.inline_spans {
                            if span.start < clean_content.len() {
                                let (col_start, col_len) =
                                    safe_char_count(clean_content, span.start, span.end);

                                let span_x = text_x + (col_start as f32 * char_width);
                                let span_w = (col_len as f32 * char_width).max(char_width * 0.5);
                                let span_rect = Rect::from_min_size(
                                    pos2(span_x, row_y + 1.0),
                                    vec2(span_w, line_height - 2.0),
                                );
                                painter.rect_filled(span_rect, 2.0, span_bg);
                            }
                        }

                        // Content text
                        painter.text(
                            pos2(text_x, row_y + 2.0),
                            Align2::LEFT_TOP,
                            clean_content,
                            font_id.clone(),
                            fg_color,
                        );
                    }
                }
            }
        });
}

// ---------------------------------------------------------------------------
// Side-by-side View Rendering
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum SideBySideRow<'a> {
    Header(&'a str),
    Pair {
        left: Option<&'a DiffLine>,
        right: Option<&'a DiffLine>,
    },
}

pub fn build_side_by_side_rows<'a>(diff: &'a FileDiff) -> Vec<SideBySideRow<'a>> {
    let mut rows = Vec::new();
    for hunk in &diff.hunks {
        rows.push(SideBySideRow::Header(&hunk.header));

        let mut i = 0;
        let lines = &hunk.lines;
        while i < lines.len() {
            if lines[i].change == LineChange::Unchanged {
                rows.push(SideBySideRow::Pair {
                    left: Some(&lines[i]),
                    right: Some(&lines[i]),
                });
                i += 1;
            } else {
                let mut deletes = Vec::new();
                let mut additions = Vec::new();

                while i < lines.len() && lines[i].change != LineChange::Unchanged {
                    match lines[i].change {
                        LineChange::Deleted => deletes.push(&lines[i]),
                        LineChange::Added | LineChange::Modified => additions.push(&lines[i]),
                        LineChange::Unchanged => unreachable!(),
                    }
                    i += 1;
                }

                let max_count = deletes.len().max(additions.len());
                for k in 0..max_count {
                    rows.push(SideBySideRow::Pair {
                        left: deletes.get(k).copied(),
                        right: additions.get(k).copied(),
                    });
                }
            }
        }
    }
    rows
}

fn render_side_by_side_view(ui: &mut Ui, theme: &Theme, diff: &FileDiff) {
    let font_id = FontId::monospace(12.5);
    let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' '));
    let line_height = ui.fonts(|f| f.row_height(&font_id)) + 4.0;

    let rows = build_side_by_side_rows(diff);

    let mut max_chars = 30;
    for hunk in &diff.hunks {
        for line in &hunk.lines {
            max_chars = max_chars.max(line.content.len());
        }
    }

    let col_gutter_width = 6.0 * char_width + 8.0; // 4 line_no + 2 sign + padding
    let min_col_width = (max_chars as f32 * char_width) + col_gutter_width + 16.0;
    let available_w = ui.available_width();
    let col_width = min_col_width.max(available_w * 0.5);
    let content_width = col_width * 2.0 + 2.0; // + 2 for separator
    let total_height = rows.len() as f32 * line_height;

    ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (rect, _response) = ui.allocate_exact_size(
                vec2(content_width, total_height),
                Sense::hover(),
            );
            let painter = ui.painter_at(rect);

            // Full background
            painter.rect_filled(rect, 0.0, theme.bg_editor);

            let clip = ui.clip_rect();
            let start_idx = ((clip.min.y - rect.min.y) / line_height).floor().max(0.0) as usize;
            let end_idx = (((clip.max.y - rect.min.y) / line_height).ceil() as usize).min(rows.len());

            let divider_x = rect.min.x + col_width;
            let metrics = DiffColumnMetrics {
                gutter_width: col_gutter_width,
                char_width,
                line_height,
                font_id: &font_id,
                theme,
            };

            for (idx, row) in rows.iter().enumerate().take(end_idx).skip(start_idx) {
                let row_y = rect.min.y + (idx as f32 * line_height);

                match row {
                    SideBySideRow::Header(header_text) => {
                        let header_rect = Rect::from_min_size(
                            pos2(rect.min.x, row_y),
                            vec2(content_width, line_height),
                        );
                        let header_bg = Color32::from_rgba_unmultiplied(60, 140, 230, 25);
                        painter.rect_filled(header_rect, 0.0, header_bg);
                        painter.text(
                            pos2(rect.min.x + 8.0, row_y + 2.0),
                            Align2::LEFT_TOP,
                            *header_text,
                            font_id.clone(),
                            theme.syntax_constant,
                        );
                    }
                    SideBySideRow::Pair { left, right } => {
                        // Left column (Old)
                        let left_rect = Rect::from_min_size(
                            pos2(rect.min.x, row_y),
                            vec2(col_width, line_height),
                        );
                        render_diff_column_cell(
                            &painter,
                            left_rect,
                            *left,
                            true,
                            &metrics,
                        );

                        // Right column (New)
                        let right_rect = Rect::from_min_size(
                            pos2(divider_x + 2.0, row_y),
                            vec2(col_width, line_height),
                        );
                        render_diff_column_cell(
                            &painter,
                            right_rect,
                            *right,
                            false,
                            &metrics,
                        );
                    }
                }
            }

            // Center vertical divider
            painter.line_segment(
                [pos2(divider_x, rect.min.y), pos2(divider_x, rect.max.y)],
                Stroke::new(1.0_f32, theme.border),
            );
        });
}

struct DiffColumnMetrics<'a> {
    gutter_width: f32,
    char_width: f32,
    line_height: f32,
    font_id: &'a FontId,
    theme: &'a Theme,
}

fn render_diff_column_cell(
    painter: &egui::Painter,
    cell_rect: Rect,
    line_opt: Option<&DiffLine>,
    is_left: bool,
    metrics: &DiffColumnMetrics<'_>,
) {
    if let Some(line) = line_opt {
        let (bg_color, span_bg, fg_color) = diff_colors(line.change, metrics.theme);
        if bg_color != Color32::TRANSPARENT {
            painter.rect_filled(cell_rect, 0.0, bg_color);
        }

        let line_no_str = if is_left {
            line.old_line_no.map(|n| format!("{:>4}", n)).unwrap_or_else(|| "    ".to_string())
        } else {
            line.new_line_no.map(|n| format!("{:>4}", n)).unwrap_or_else(|| "    ".to_string())
        };

        let sign_str = match line.change {
            LineChange::Added => "+ ",
            LineChange::Deleted => "- ",
            LineChange::Modified => "~ ",
            LineChange::Unchanged => "  ",
        };

        let sign_color = match line.change {
            LineChange::Added => Color32::from_rgb(75, 175, 80),
            LineChange::Deleted => Color32::from_rgb(230, 80, 80),
            LineChange::Modified => Color32::from_rgb(60, 140, 230),
            LineChange::Unchanged => metrics.theme.fg_muted,
        };

        // Gutter line number & sign
        painter.text(
            pos2(cell_rect.min.x + 4.0, cell_rect.min.y + 2.0),
            Align2::LEFT_TOP,
            line_no_str,
            metrics.font_id.clone(),
            metrics.theme.fg_muted,
        );
        painter.text(
            pos2(cell_rect.min.x + 4.0 + (4.5 * metrics.char_width), cell_rect.min.y + 2.0),
            Align2::LEFT_TOP,
            sign_str,
            metrics.font_id.clone(),
            sign_color,
        );

        // Content
        let text_x = cell_rect.min.x + metrics.gutter_width;
        let clean_content = line.content.trim_end_matches(['\r', '\n']);

        // Inline span highlights
        for span in &line.inline_spans {
            if span.start < clean_content.len() {
                let (col_start, col_len) =
                    safe_char_count(clean_content, span.start, span.end);

                let span_x = text_x + (col_start as f32 * metrics.char_width);
                let span_w = (col_len as f32 * metrics.char_width).max(metrics.char_width * 0.5);
                let span_rect = Rect::from_min_size(
                    pos2(span_x, cell_rect.min.y + 1.0),
                    vec2(span_w, metrics.line_height - 2.0),
                );
                painter.rect_filled(span_rect, 2.0, span_bg);
            }
        }

        // Text
        painter.text(
            pos2(text_x, cell_rect.min.y + 2.0),
            Align2::LEFT_TOP,
            clean_content,
            metrics.font_id.clone(),
            fg_color,
        );
    } else {
        // Blank slot (filler background)
        let blank_bg = Color32::from_rgba_unmultiplied(40, 40, 40, 30);
        painter.rect_filled(cell_rect, 0.0, blank_bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustforge_git::compute_diff;

    #[test]
    fn test_diff_view_side_by_side_pairing() {
        let old_text = "line 1\nline 2\nline 3\n";
        let new_text = "line 1\nline 2 modified\nline 2.5 added\nline 3\n";
        let diff = compute_diff(old_text, new_text);

        let rows = build_side_by_side_rows(&diff);
        assert!(!rows.is_empty());

        // First row must be header
        assert!(matches!(rows[0], SideBySideRow::Header(_)));

        // Unchanged line 1
        if let SideBySideRow::Pair { left, right } = &rows[1] {
            assert!(left.is_some());
            assert!(right.is_some());
            assert_eq!(left.unwrap().old_line_no, Some(1));
            assert_eq!(right.unwrap().new_line_no, Some(1));
        } else {
            panic!("expected pair row");
        }
    }

    #[test]
    fn test_diff_view_construction() {
        let view = DiffView::new("old.rs", "new.rs").with_side_by_side(true);
        assert!(view.side_by_side);
        assert_eq!(view.old_label, "old.rs");
        assert_eq!(view.new_label, "new.rs");
    }

    #[test]
    fn test_diff_view_pairing_pure_additions_and_deletions() {
        // Pure deletion
        let diff_del = compute_diff("alpha\nbeta\n", "alpha\n");
        let rows_del = build_side_by_side_rows(&diff_del);
        assert!(matches!(rows_del[0], SideBySideRow::Header(_)));
        // row 1: unchanged alpha
        if let SideBySideRow::Pair { left, right } = &rows_del[1] {
            assert!(left.is_some());
            assert!(right.is_some());
        } else {
            panic!("expected pair");
        }
        // row 2: deleted beta on left, None on right
        if let SideBySideRow::Pair { left, right } = &rows_del[2] {
            assert_eq!(left.unwrap().change, LineChange::Deleted);
            assert!(right.is_none());
        } else {
            panic!("expected pair");
        }

        // Pure addition
        let diff_add = compute_diff("alpha\n", "alpha\ngamma\n");
        let rows_add = build_side_by_side_rows(&diff_add);
        // row 2: None on left, added gamma on right
        if let SideBySideRow::Pair { left, right } = &rows_add[2] {
            assert!(left.is_none());
            assert_eq!(right.unwrap().change, LineChange::Added);
        } else {
            panic!("expected pair");
        }
    }

    #[test]
    fn test_diff_colors_coverage() {
        let theme = Theme::dark_modern();
        let (add_bg, add_span, add_fg) = diff_colors(LineChange::Added, &theme);
        assert_ne!(add_bg, Color32::TRANSPARENT);
        assert_ne!(add_span, Color32::TRANSPARENT);
        assert_ne!(add_fg, Color32::TRANSPARENT);

        let (del_bg, del_span, del_fg) = diff_colors(LineChange::Deleted, &theme);
        assert_ne!(del_bg, Color32::TRANSPARENT);
        assert_ne!(del_span, Color32::TRANSPARENT);
        assert_ne!(del_fg, Color32::TRANSPARENT);

        let (mod_bg, mod_span, mod_fg) = diff_colors(LineChange::Modified, &theme);
        assert_ne!(mod_bg, Color32::TRANSPARENT);
        assert_ne!(mod_span, Color32::TRANSPARENT);
        assert_ne!(mod_fg, Color32::TRANSPARENT);

        let (unc_bg, unc_span, unc_fg) = diff_colors(LineChange::Unchanged, &theme);
        assert_eq!(unc_bg, Color32::TRANSPARENT);
        assert_eq!(unc_span, Color32::TRANSPARENT);
        assert_eq!(unc_fg, theme.fg_primary);
    }
}
