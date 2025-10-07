use crate::db::SchemaInfo;
use eframe::egui;
use std::collections::HashSet;

#[derive(Debug)]
pub enum DatabaseTreeEvent {
    TableClicked(String, String),
    TableRightClicked(String, String),
    SchemaToggled(String),
    SearchChanged(String),
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
        search_query: &mut String,
    ) -> Option<DatabaseTreeEvent> {
        let mut event = None;

        // Search input
        ui.horizontal(|ui| {
            ui.label("🔍");
            let response = ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("Search...")
                    .desired_width(180.0)
            );

            if response.changed() {
                event = Some(DatabaseTreeEvent::SearchChanged(search_query.clone()));
            }

            if !search_query.is_empty() && ui.small_button("✖").clicked() {
                search_query.clear();
                event = Some(DatabaseTreeEvent::SearchChanged(String::new()));
            }
        });

        ui.separator();

        // Filter schemas and tables based on search query
        let search_lower = search_query.to_lowercase();
        let filtered_schemas: Vec<_> = if search_query.is_empty() {
            schemas.iter().map(|s| (s, s.tables.clone())).collect()
        } else {
            schemas
                .iter()
                .filter_map(|schema| {
                    let schema_matches = schema.name.to_lowercase().contains(&search_lower);
                    let filtered_tables: Vec<_> = schema
                        .tables
                        .iter()
                        .filter(|table| {
                            schema_matches || table.to_lowercase().contains(&search_lower)
                        })
                        .cloned()
                        .collect();

                    if !filtered_tables.is_empty() {
                        Some((schema, filtered_tables))
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Show results count if searching
        if !search_query.is_empty() {
            let total_tables: usize = filtered_schemas.iter().map(|(_, tables)| tables.len()).sum();
            ui.label(egui::RichText::new(format!("Found {} table(s) in {} schema(s)", total_tables, filtered_schemas.len()))
                .size(10.0)
                .color(egui::Color32::GRAY));
            ui.separator();
        }

        egui::ScrollArea::vertical()
            .id_source("tables_sidebar")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if filtered_schemas.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("No tables found")
                            .color(egui::Color32::GRAY));
                    });
                }

                for (schema, filtered_tables) in &filtered_schemas {
                    let is_expanded = expanded_schemas.contains(&schema.name) || !search_query.is_empty();

                    // Schema row with expand/collapse arrow
                    ui.horizontal(|ui| {
                        if search_query.is_empty() {
                            let arrow = if is_expanded { "▼" } else { "▶" };
                            if ui.button(arrow).clicked() {
                                event = Some(DatabaseTreeEvent::SchemaToggled(schema.name.clone()));
                            }
                        }

                        // Highlight schema name if it matches search
                        let schema_text = if !search_query.is_empty() && schema.name.to_lowercase().contains(&search_lower) {
                            egui::RichText::new(&schema.name).strong().color(egui::Color32::from_rgb(100, 200, 255))
                        } else {
                            egui::RichText::new(&schema.name).strong()
                        };

                        ui.label(schema_text);
                        ui.label(format!("({})", filtered_tables.len()));
                    });

                    // Show tables if expanded or searching
                    if is_expanded {
                        ui.indent(&schema.name, |ui| {
                            for table in filtered_tables {
                                let is_selected = selected_table.as_ref() == Some(&(schema.name.clone(), table.clone()));

                                // Highlight table name if it matches search
                                let table_text = if !search_query.is_empty() && table.to_lowercase().contains(&search_lower) {
                                    egui::RichText::new(format!("📊 {}", table)).color(egui::Color32::from_rgb(100, 200, 255))
                                } else {
                                    egui::RichText::new(format!("📊 {}", table))
                                };

                                let response = ui.selectable_label(is_selected, table_text);

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
