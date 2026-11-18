use egui::Vec2;
use velox_app::ui::icons::Icon;

#[test]
fn test_activity_bar_icons_exist() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let icons = [
                Icon::Files,
                Icon::Search,
                Icon::SourceControl,
                Icon::Terminal,
                Icon::Settings,
                Icon::GitBranch,
                Icon::DiagnosticError,
                Icon::DiagnosticWarning,
            ];
            for icon in icons {
                let res = icon.ui(ui, Vec2::splat(18.0), egui::Color32::WHITE);
                assert_eq!(res.rect.width(), 18.0);
            }
        });
    });
}
