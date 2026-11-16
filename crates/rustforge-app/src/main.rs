mod app;
mod tab;
mod ui;

use app::RustForgeApp;
use eframe::NativeOptions;
use rustforge_core::init_logging;

fn main() -> eframe::Result<()> {
    init_logging();

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RustForge")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_drag_and_drop(true),
        vsync: false,
        ..Default::default()
    };

    eframe::run_native(
        "RustForge",
        native_options,
        Box::new(|cc| Ok(Box::new(RustForgeApp::new(cc)))),
    )
}
