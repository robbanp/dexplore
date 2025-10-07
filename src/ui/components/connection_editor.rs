use crate::config::DatabaseConnection;
use eframe::egui;

#[derive(Debug)]
pub enum ConnectionEditorEvent {
    Save,
    Cancel,
}

pub struct ConnectionEditor;

impl ConnectionEditor {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ctx: &egui::Context, conn: &mut DatabaseConnection) -> Option<ConnectionEditorEvent> {
        let mut event = None;

        egui::Window::new("Connection Details")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut conn.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut conn.host);
                });

                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.add(egui::DragValue::new(&mut conn.port).clamp_range(1..=65535));
                });

                ui.horizontal(|ui| {
                    ui.label("User:");
                    ui.text_edit_singleline(&mut conn.user);
                });

                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut conn.password).password(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Database:");
                    ui.text_edit_singleline(&mut conn.database);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        event = Some(ConnectionEditorEvent::Save);
                    }
                    if ui.button("Cancel").clicked() {
                        event = Some(ConnectionEditorEvent::Cancel);
                    }
                });
            });

        event
    }
}
