use crate::db::SchemaInfo;
use eframe::egui;
use std::collections::HashSet;

#[derive(Debug)]
pub enum DatabaseTreeEvent {
    TableClicked(String, String),
    TableRightClicked(String, String),
    SchemaToggled(String),
}

pub struct DatabaseTree;

impl DatabaseTree {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        schemas: &[SchemaInfo],
        expanded_schemas: &HashSet<String>,
        selected_table: &Option<(String, String)>,
    ) -> Option<DatabaseTreeEvent> {
        let mut event = None;

        egui::ScrollArea::vertical()
            .id_source("tables_sidebar")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for schema in schemas {
                    let is_expanded = expanded_schemas.contains(&schema.name);

                    // Schema row with expand/collapse arrow
                    ui.horizontal(|ui| {
                        let arrow = if is_expanded { "▼" } else { "▶" };
                        if ui.button(arrow).clicked() {
                            event = Some(DatabaseTreeEvent::SchemaToggled(schema.name.clone()));
                        }
                        ui.label(egui::RichText::new(&schema.name).strong());
                        ui.label(format!("({})", schema.tables.len()));
                    });

                    // Show tables if expanded
                    if is_expanded {
                        ui.indent(&schema.name, |ui| {
                            for table in &schema.tables {
                                let is_selected = selected_table.as_ref() == Some(&(schema.name.clone(), table.clone()));
                                let response = ui.selectable_label(is_selected, format!("📊 {}", table));

                                if response.clicked() {
                                    event = Some(DatabaseTreeEvent::TableClicked(schema.name.clone(), table.clone()));
                                }

                                response.context_menu(|ui| {
                                    if ui.button("View Data").clicked() {
                                        event = Some(DatabaseTreeEvent::TableRightClicked(schema.name.clone(), table.clone()));
                                        ui.close_menu();
                                    }
                                });
                            }
                        });
                    }
                }
            });

        event
    }
}
