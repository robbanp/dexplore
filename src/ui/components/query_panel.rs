use eframe::egui;

#[derive(Debug)]
pub enum QueryPanelEvent {
    Execute,
    Clear,
    Close,
    SaveQuery,
    LoadQuery,
}

pub struct QueryPanel;

impl QueryPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, query_input: &mut String) -> Option<QueryPanelEvent> {
        let mut event = None;

        ui.vertical(|ui| {
            ui.label("SQL Query:");
            let response = ui.add(
                egui::TextEdit::multiline(query_input)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY)
            );

            ui.horizontal(|ui| {
                if ui.button("Execute").clicked() ||
                   (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command)) {
                    event = Some(QueryPanelEvent::Execute);
                }
                if ui.button("Clear").clicked() {
                    event = Some(QueryPanelEvent::Clear);
                }

                ui.separator();

                if ui.button("💾 Save").on_hover_text("Save current query").clicked() {
                    event = Some(QueryPanelEvent::SaveQuery);
                }
                if ui.button("📂 Load").on_hover_text("Load saved query").clicked() {
                    event = Some(QueryPanelEvent::LoadQuery);
                }

                ui.separator();

                if ui.button("Close").clicked() {
                    event = Some(QueryPanelEvent::Close);
                }
            });
        });

        event
    }
}
