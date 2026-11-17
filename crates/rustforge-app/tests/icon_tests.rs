use egui::{Color32, Pos2, Rect, Vec2};
use rustforge_app::ui::icons::Icon;

#[test]
fn test_all_icons_paint_without_panic() {
    let icons = [
        Icon::Files,
        Icon::Search,
        Icon::SourceControl,
        Icon::Terminal,
        Icon::Settings,
        Icon::FolderClosed,
        Icon::FolderOpen,
        Icon::FileRust,
        Icon::FileToml,
        Icon::FileMarkdown,
        Icon::FileGeneric,
        Icon::ChevronRight,
        Icon::ChevronDown,
        Icon::Close,
        Icon::UnsavedDot,
        Icon::SplitHorizontal,
        Icon::SplitVertical,
        Icon::GitBranch,
        Icon::GitAdded,
        Icon::GitModified,
        Icon::GitRemoved,
        Icon::DiagnosticError,
        Icon::DiagnosticWarning,
        Icon::DiagnosticInfo,
    ];

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let rects = [
                Rect::from_min_size(Pos2::ZERO, Vec2::ZERO), // edge case: zero size
                Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(16.0, 16.0)),
                Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(24.0, 24.0)),
                Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(32.0, 32.0)),
            ];

            for icon in &icons {
                for rect in &rects {
                    icon.paint(painter, *rect, Color32::WHITE);
                }
            }
        });
    });
}
