use eframe::egui;
use crate::config::SavedQueries;

#[derive(Debug)]
pub enum SavedQueriesDialogEvent {
    Load(usize),
    Delete(usize),
    Close,
}

pub struct SavedQueriesDialog;

impl SavedQueriesDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ctx: &egui::Context, saved_queries: &SavedQueries) -> Option<SavedQueriesDialogEvent> {
        let mut event = None;
        let mut is_open = true;

        egui::Window::new("📂 Saved Queries")
            .open(&mut is_open)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Your Saved Queries");
                    ui.separator();

                    if saved_queries.queries.is_empty() {
                        ui.label("No saved queries yet.");
                        ui.label("Save queries from the SQL editor to see them here.");
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for (index, query) in saved_queries.queries.iter().enumerate() {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.strong(&query.name);
                                                ui.label(egui::RichText::new(&query.created_at)
                                                    .size(10.0)
                                                    .color(egui::Color32::GRAY));
                                            });

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("🗑 Delete").clicked() {
                                                    event = Some(SavedQueriesDialogEvent::Delete(index));
                                                }
                                                if ui.button("📥 Load").clicked() {
                                                    event = Some(SavedQueriesDialogEvent::Load(index));
                                                }
                                            });
                                        });

                                        // Show SQL preview
                                        ui.add_space(5.0);
                                        let preview = if query.sql.len() > 150 {
                                            format!("{}...", &query.sql[..150])
                                        } else {
                                            query.sql.clone()
                                        };
                                        ui.label(egui::RichText::new(preview)
                                            .size(10.0)
                                            .color(egui::Color32::DARK_GRAY)
                                            .family(egui::FontFamily::Monospace));
                                    });
                                    ui.add_space(5.0);
                                }
                            });
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            event = Some(SavedQueriesDialogEvent::Close);
                        }
                    });
                });
            });

        if !is_open {
            event = Some(SavedQueriesDialogEvent::Close);
        }

        event
    }
}

#[derive(Debug)]
pub enum SaveQueryDialogEvent {
    Save(String),
    Cancel,
}

pub struct SaveQueryDialog {
    query_name: String,
}

impl SaveQueryDialog {
    pub fn new() -> Self {
        Self {
            query_name: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<SaveQueryDialogEvent> {
        let mut event = None;
        let mut is_open = true;

        egui::Window::new("💾 Save Query")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("Enter a name for this query:");
                    ui.add_space(5.0);

                    let response = ui.text_edit_singleline(&mut self.query_name);

                    // Auto-focus the text field
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.query_name.trim().is_empty() {
                            event = Some(SaveQueryDialogEvent::Save(self.query_name.clone()));
                        }
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if !self.query_name.trim().is_empty() {
                                event = Some(SaveQueryDialogEvent::Save(self.query_name.clone()));
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            event = Some(SaveQueryDialogEvent::Cancel);
                        }
                    });
                });
            });

        if !is_open {
            event = Some(SaveQueryDialogEvent::Cancel);
        }

        event
    }
}
