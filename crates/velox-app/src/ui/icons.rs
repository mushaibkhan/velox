use egui::{pos2, Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Files,
    Search,
    SourceControl,
    Terminal,
    Settings,
    FolderClosed,
    FolderOpen,
    FileRust,
    FileToml,
    FileMarkdown,
    FileGeneric,
    ChevronRight,
    ChevronDown,
    Close,
    UnsavedDot,
    SplitHorizontal,
    SplitVertical,
    GitBranch,
    GitAdded,
    GitModified,
    GitRemoved,
    DiagnosticError,
    DiagnosticWarning,
    DiagnosticInfo,
}

impl Icon {
    pub fn ui(&self, ui: &mut Ui, size: Vec2, color: Color32) -> Response {
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            self.paint(ui.painter(), rect, color);
        }
        response
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }

        let min = rect.min;
        let w = rect.width();
        let h = rect.height();
        let pt = |x: f32, y: f32| -> Pos2 { pos2(min.x + x * w, min.y + y * h) };
        let stroke_w = (w * 0.08).clamp(1.0, 2.5);
        let stroke = Stroke::new(stroke_w, color);

        match self {
            Icon::Files => {
                // Document shape with folded corner
                painter.line_segment([pt(0.2, 0.15), pt(0.6, 0.15)], stroke);
                painter.line_segment([pt(0.6, 0.15), pt(0.8, 0.35)], stroke);
                painter.line_segment([pt(0.8, 0.35), pt(0.8, 0.85)], stroke);
                painter.line_segment([pt(0.8, 0.85), pt(0.2, 0.85)], stroke);
                painter.line_segment([pt(0.2, 0.85), pt(0.2, 0.15)], stroke);
                // Fold line
                painter.line_segment([pt(0.6, 0.15), pt(0.6, 0.35)], stroke);
                painter.line_segment([pt(0.6, 0.35), pt(0.8, 0.35)], stroke);
            }
            Icon::Search => {
                let center = pt(0.42, 0.42);
                let radius = w * 0.24;
                painter.circle_stroke(center, radius, stroke);
                painter.line_segment([pt(0.6, 0.6), pt(0.82, 0.82)], Stroke::new(stroke_w * 1.2, color));
            }
            Icon::SourceControl => {
                let r = w * 0.1;
                painter.circle_filled(pt(0.3, 0.3), r, color);
                painter.circle_filled(pt(0.3, 0.7), r, color);
                painter.circle_filled(pt(0.7, 0.45), r, color);
                painter.line_segment([pt(0.3, 0.3), pt(0.3, 0.7)], stroke);
                // Curved branch to third node
                painter.line_segment([pt(0.3, 0.55), pt(0.7, 0.45)], stroke);
            }
            Icon::Terminal => {
                painter.line_segment([pt(0.2, 0.3), pt(0.45, 0.5)], stroke);
                painter.line_segment([pt(0.45, 0.5), pt(0.2, 0.7)], stroke);
                painter.line_segment([pt(0.55, 0.72), pt(0.82, 0.72)], stroke);
            }
            Icon::Settings => {
                let center = pt(0.5, 0.5);
                let radius = w * 0.28;
                painter.circle_stroke(center, radius, stroke);
                painter.circle_filled(center, w * 0.1, color);
                // 4 teeth
                painter.line_segment([pt(0.5, 0.1), pt(0.5, 0.22)], stroke);
                painter.line_segment([pt(0.5, 0.78), pt(0.5, 0.9)], stroke);
                painter.line_segment([pt(0.1, 0.5), pt(0.22, 0.5)], stroke);
                painter.line_segment([pt(0.78, 0.5), pt(0.9, 0.5)], stroke);
            }
            Icon::FolderClosed => {
                // Folder tab + body
                painter.line_segment([pt(0.15, 0.3), pt(0.4, 0.3)], stroke);
                painter.line_segment([pt(0.4, 0.3), pt(0.5, 0.4)], stroke);
                painter.line_segment([pt(0.5, 0.4), pt(0.85, 0.4)], stroke);
                painter.line_segment([pt(0.85, 0.4), pt(0.85, 0.75)], stroke);
                painter.line_segment([pt(0.85, 0.75), pt(0.15, 0.75)], stroke);
                painter.line_segment([pt(0.15, 0.75), pt(0.15, 0.3)], stroke);
            }
            Icon::FolderOpen => {
                painter.line_segment([pt(0.15, 0.3), pt(0.4, 0.3)], stroke);
                painter.line_segment([pt(0.4, 0.3), pt(0.5, 0.4)], stroke);
                painter.line_segment([pt(0.5, 0.4), pt(0.85, 0.4)], stroke);
                // Slanted open flap
                painter.line_segment([pt(0.15, 0.5), pt(0.85, 0.5)], stroke);
                painter.line_segment([pt(0.85, 0.5), pt(0.75, 0.8)], stroke);
                painter.line_segment([pt(0.75, 0.8), pt(0.25, 0.8)], stroke);
                painter.line_segment([pt(0.25, 0.8), pt(0.15, 0.5)], stroke);
            }
            Icon::FileRust => {
                painter.circle_stroke(pt(0.5, 0.5), w * 0.32, stroke);
                painter.line_segment([pt(0.38, 0.35), pt(0.38, 0.65)], stroke);
                painter.line_segment([pt(0.38, 0.35), pt(0.58, 0.35)], stroke);
                painter.line_segment([pt(0.58, 0.35), pt(0.58, 0.5)], stroke);
                painter.line_segment([pt(0.58, 0.5), pt(0.38, 0.5)], stroke);
                painter.line_segment([pt(0.48, 0.5), pt(0.62, 0.65)], stroke);
            }
            Icon::FileToml => {
                // [ ] brackets
                painter.line_segment([pt(0.35, 0.25), pt(0.25, 0.25)], stroke);
                painter.line_segment([pt(0.25, 0.25), pt(0.25, 0.75)], stroke);
                painter.line_segment([pt(0.25, 0.75), pt(0.35, 0.75)], stroke);

                painter.line_segment([pt(0.65, 0.25), pt(0.75, 0.25)], stroke);
                painter.line_segment([pt(0.75, 0.25), pt(0.75, 0.75)], stroke);
                painter.line_segment([pt(0.75, 0.75), pt(0.65, 0.75)], stroke);
            }
            Icon::FileMarkdown => {
                // M + arrow down
                painter.line_segment([pt(0.2, 0.65), pt(0.2, 0.35)], stroke);
                painter.line_segment([pt(0.2, 0.35), pt(0.35, 0.5)], stroke);
                painter.line_segment([pt(0.35, 0.5), pt(0.5, 0.35)], stroke);
                painter.line_segment([pt(0.5, 0.35), pt(0.5, 0.65)], stroke);

                painter.line_segment([pt(0.7, 0.35), pt(0.7, 0.65)], stroke);
                painter.line_segment([pt(0.6, 0.55), pt(0.7, 0.65)], stroke);
                painter.line_segment([pt(0.8, 0.55), pt(0.7, 0.65)], stroke);
            }
            Icon::FileGeneric => {
                painter.line_segment([pt(0.25, 0.2), pt(0.75, 0.2)], stroke);
                painter.line_segment([pt(0.75, 0.2), pt(0.75, 0.8)], stroke);
                painter.line_segment([pt(0.75, 0.8), pt(0.25, 0.8)], stroke);
                painter.line_segment([pt(0.25, 0.8), pt(0.25, 0.2)], stroke);
            }
            Icon::ChevronRight => {
                painter.line_segment([pt(0.35, 0.25), pt(0.65, 0.5)], stroke);
                painter.line_segment([pt(0.65, 0.5), pt(0.35, 0.75)], stroke);
            }
            Icon::ChevronDown => {
                painter.line_segment([pt(0.25, 0.35), pt(0.5, 0.65)], stroke);
                painter.line_segment([pt(0.5, 0.65), pt(0.75, 0.35)], stroke);
            }
            Icon::Close => {
                painter.line_segment([pt(0.25, 0.25), pt(0.75, 0.75)], stroke);
                painter.line_segment([pt(0.75, 0.25), pt(0.25, 0.75)], stroke);
            }
            Icon::UnsavedDot => {
                let center = pt(0.5, 0.5);
                painter.circle_filled(center, w * 0.25, color);
            }
            Icon::SplitHorizontal => {
                painter.rect_stroke(rect.shrink(2.0), 0.0, stroke);
                painter.line_segment([pt(0.5, 0.2), pt(0.5, 0.8)], stroke);
            }
            Icon::SplitVertical => {
                painter.rect_stroke(rect.shrink(2.0), 0.0, stroke);
                painter.line_segment([pt(0.2, 0.5), pt(0.8, 0.5)], stroke);
            }
            Icon::GitBranch => {
                let r = w * 0.1;
                painter.circle_filled(pt(0.3, 0.3), r, color);
                painter.circle_filled(pt(0.3, 0.7), r, color);
                painter.circle_filled(pt(0.7, 0.4), r, color);
                painter.line_segment([pt(0.3, 0.3), pt(0.3, 0.7)], stroke);
                painter.line_segment([pt(0.3, 0.6), pt(0.7, 0.4)], stroke);
            }
            Icon::GitAdded => {
                painter.line_segment([pt(0.5, 0.25), pt(0.5, 0.75)], stroke);
                painter.line_segment([pt(0.25, 0.5), pt(0.75, 0.5)], stroke);
            }
            Icon::GitModified => {
                painter.line_segment([pt(0.25, 0.5), pt(0.45, 0.35)], stroke);
                painter.line_segment([pt(0.45, 0.35), pt(0.55, 0.65)], stroke);
                painter.line_segment([pt(0.55, 0.65), pt(0.75, 0.5)], stroke);
            }
            Icon::GitRemoved => {
                painter.line_segment([pt(0.25, 0.5), pt(0.75, 0.5)], stroke);
            }
            Icon::DiagnosticError => {
                let center = pt(0.5, 0.5);
                painter.circle_filled(center, w * 0.38, color);
                let inv_stroke = Stroke::new(stroke_w, Color32::BLACK);
                painter.line_segment([pt(0.35, 0.35), pt(0.65, 0.65)], inv_stroke);
                painter.line_segment([pt(0.65, 0.35), pt(0.35, 0.65)], inv_stroke);
            }
            Icon::DiagnosticWarning => {
                // Triangle
                let p1 = pt(0.5, 0.18);
                let p2 = pt(0.18, 0.82);
                let p3 = pt(0.82, 0.82);
                painter.line_segment([p1, p2], stroke);
                painter.line_segment([p2, p3], stroke);
                painter.line_segment([p3, p1], stroke);
                painter.line_segment([pt(0.5, 0.42), pt(0.5, 0.62)], stroke);
                painter.circle_filled(pt(0.5, 0.72), stroke_w * 0.6, color);
            }
            Icon::DiagnosticInfo => {
                let center = pt(0.5, 0.5);
                painter.circle_stroke(center, w * 0.35, stroke);
                painter.circle_filled(pt(0.5, 0.35), stroke_w * 0.7, color);
                painter.line_segment([pt(0.5, 0.48), pt(0.5, 0.68)], stroke);
            }
        }
    }
}
