use eframe::egui;

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, status_message: &str, row_count: Option<usize>) {
        ui.horizontal(|ui| {
            ui.label(status_message);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(count) = row_count {
                    ui.label(format!("{} rows", count));
                }
            });
        });
    }
}
