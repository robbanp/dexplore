use crate::config::Config;
use eframe::egui;

#[derive(Debug)]
pub enum SettingsDialogEvent {
    Connect(usize),
    Edit(usize),
    Delete(usize),
    NewConnection,
    Close,
}

pub struct SettingsDialog;

impl SettingsDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ctx: &egui::Context, config: &Config) -> Option<SettingsDialogEvent> {
        let mut event = None;

        egui::Window::new("Settings")
            .default_width(600.0)
            .show(ctx, |ui| {
                ui.heading("Database Connections");
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (idx, conn) in config.connections.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(&conn.name);
                                ui.label(format!("{}@{}/{}", conn.user, conn.host, conn.database));

                                if ui.button("Connect").clicked() {
                                    event = Some(SettingsDialogEvent::Connect(idx));
                                }
                                if ui.button("Edit").clicked() {
                                    event = Some(SettingsDialogEvent::Edit(idx));
                                }
                                if ui.button("Delete").clicked() {
                                    event = Some(SettingsDialogEvent::Delete(idx));
                                }
                            });
                            ui.separator();
                        }
                    });

                ui.separator();

                if ui.button("+ New Connection").clicked() {
                    event = Some(SettingsDialogEvent::NewConnection);
                }

                ui.separator();

                if ui.button("Close").clicked() {
                    event = Some(SettingsDialogEvent::Close);
                }
            });

        event
    }
}
