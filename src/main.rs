use eframe::egui;

mod app;
mod data;
mod fs;
mod gui;
mod utils;
mod visualization;

use app::DiskUsageApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DiskUsage",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(DiskUsageApp::new(cc))
        }),
    )
}