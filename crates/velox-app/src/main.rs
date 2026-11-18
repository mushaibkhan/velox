use eframe::NativeOptions;
use velox_app::VeloxApp;
use velox_core::init_logging;

fn main() -> eframe::Result<()> {
    init_logging();

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Velox")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_drag_and_drop(true),
        vsync: false,
        ..Default::default()
    };

    eframe::run_native(
        "Velox",
        native_options,
        Box::new(|cc| Ok(Box::new(VeloxApp::new(cc)))),
    )
}
