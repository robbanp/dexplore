mod app;
mod config;
mod db;
mod models;
mod ui;

use app::DbClientApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("PostgreSQL Database Client"),
        ..Default::default()
    };

    eframe::run_native(
        "DB Client",
        options,
        Box::new(|cc| Box::new(DbClientApp::new(cc))),
    )
}
