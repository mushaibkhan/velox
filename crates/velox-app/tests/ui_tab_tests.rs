use egui::Vec2;
use velox_app::ui::icons::Icon;

#[test]
fn test_tab_icon_sizing() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let res_dot = Icon::UnsavedDot.ui(ui, Vec2::splat(12.0), egui::Color32::from_rgb(56, 189, 248));
            assert_eq!(res_dot.rect.size(), Vec2::splat(12.0));

            let res_close = Icon::Close.ui(ui, Vec2::splat(12.0), egui::Color32::WHITE);
            assert_eq!(res_close.rect.size(), Vec2::splat(12.0));
        });
    });
}
