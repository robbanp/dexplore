use crate::db::ColumnInfo;
use crate::models::{FilterRule, FilterOperator, FilterConjunction};
use eframe::egui;

#[derive(Debug)]
pub enum FilterBarEvent {
    FilterAdded,
    FilterRemoved(usize),
    FiltersChanged,
    FilterApplied,
}

pub struct FilterBar;

impl FilterBar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        filters: &mut Vec<FilterRule>,
        columns: &[ColumnInfo],
    ) -> Option<FilterBarEvent> {
        let mut event = None;

        ui.horizontal(|ui| {
            // Add filter button
            if ui.button("➕").on_hover_text("Add filter").clicked() {
                filters.push(FilterRule::new(0));
                event = Some(FilterBarEvent::FilterAdded);
            }

            // Remove filter button
            if !filters.is_empty() {
                if ui.button("➖").on_hover_text("Remove last filter").clicked() {
                    filters.pop();
                    event = Some(FilterBarEvent::FilterRemoved(filters.len()));
                }
            }

            ui.separator();

            // Apply/Search button
            if ui.button("🔍").on_hover_text("Apply filters").clicked() {
                event = Some(FilterBarEvent::FilterApplied);
            }

            ui.separator();

            // Filter count
            if !filters.is_empty() {
                ui.label(egui::RichText::new(format!("{} filter(s)", filters.len()))
                    .size(10.0)
                    .color(egui::Color32::GRAY));
            }
        });

        // Show filter rows
        let mut filters_changed = false;
        let mut filter_to_remove: Option<usize> = None;

        for (idx, filter) in filters.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // Conjunction (except for first filter)
                if idx > 0 {
                    egui::ComboBox::from_id_source(format!("conjunction_{}", idx))
                        .selected_text(filter.conjunction.as_str())
                        .width(60.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut filter.conjunction, FilterConjunction::And, "AND").clicked() {
                                filters_changed = true;
                            }
                            if ui.selectable_value(&mut filter.conjunction, FilterConjunction::Or, "OR").clicked() {
                                filters_changed = true;
                            }
                        });
                } else {
                    ui.add_space(70.0);
                }

                // Column selection
                let column_name = columns.get(filter.column_index)
                    .map(|c| c.name.as_str())
                    .unwrap_or("(select column)");

                egui::ComboBox::from_id_source(format!("column_{}", idx))
                    .selected_text(column_name)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for (col_idx, col) in columns.iter().enumerate() {
                            if ui.selectable_value(&mut filter.column_index, col_idx, &col.name).clicked() {
                                filters_changed = true;
                            }
                        }
                    });

                // Operator selection
                egui::ComboBox::from_id_source(format!("operator_{}", idx))
                    .selected_text(filter.operator.as_str())
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for op in FilterOperator::all() {
                            if ui.selectable_value(&mut filter.operator, op.clone(), op.as_str()).clicked() {
                                filters_changed = true;
                            }
                        }
                    });

                // Value input (if operator needs value)
                if filter.operator.needs_value() {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut filter.value)
                            .hint_text("value...")
                            .desired_width(200.0)
                    );

                    if response.changed() {
                        filters_changed = true;
                    }
                }

                // Remove this specific filter button
                if ui.small_button("✖").on_hover_text("Remove this filter").clicked() {
                    filter_to_remove = Some(idx);
                }
            });
        }

        // Remove filter if requested
        if let Some(idx) = filter_to_remove {
            filters.remove(idx);
            event = Some(FilterBarEvent::FilterRemoved(idx));
        } else if filters_changed {
            event = Some(FilterBarEvent::FiltersChanged);
        }

        event
    }
}
