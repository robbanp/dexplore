use eframe::egui;
use crate::sql_editor::SqlEditor;

#[derive(Debug)]
pub enum QueryPanelEvent {
    Execute,
    Clear,
    Close,
    SaveQuery,
    LoadQuery,
}

pub struct QueryPanel {
    sql_editor: SqlEditor,
}

impl QueryPanel {
    pub fn new() -> Self {
        Self {
            sql_editor: SqlEditor::new(),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        query_input: &mut String,
        tables: &[String],
        columns: &[String],
    ) -> Option<QueryPanelEvent> {
        let mut event = None;

        ui.vertical(|ui| {
            ui.label("SQL Query:");

            let editor_response = self.sql_editor.show(ui, query_input, tables, columns);

            if editor_response.execute {
                event = Some(QueryPanelEvent::Execute);
            }

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("Execute").clicked() {
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
