use crate::models::{TableData, FilterRule, FilterConjunction};
use eframe::egui;
use std::cell::Cell;

#[derive(Debug)]
pub enum DataGridEvent {
    ColumnSorted(usize),
    RowSelected(Option<usize>),
}

pub struct DataGrid {
    selected_row: Option<usize>,
}

impl DataGrid {
    pub fn new() -> Self {
        Self {
            selected_row: None,
        }
    }

    fn apply_filters(rows: &[Vec<String>], filters: &[FilterRule]) -> Vec<usize> {
        if filters.is_empty() {
            return (0..rows.len()).collect();
        }

        rows.iter()
            .enumerate()
            .filter(|(_, row)| {
                let mut result = filters[0].matches_row(row);

                for filter in filters.iter().skip(1) {
                    let matches = filter.matches_row(row);
                    result = match filter.conjunction {
                        FilterConjunction::And => result && matches,
                        FilterConjunction::Or => result || matches,
                    };
                }

                result
            })
            .map(|(idx, _)| idx)
            .collect()
    }

    fn apply_search(rows: &[Vec<String>], indices: Vec<usize>, search_text: &str) -> Vec<usize> {
        if search_text.is_empty() {
            return indices;
        }

        let search_lower = search_text.to_lowercase();
        indices
            .into_iter()
            .filter(|&idx| {
                // Search across all columns in the row
                rows[idx].iter().any(|cell| cell.to_lowercase().contains(&search_lower))
            })
            .collect()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        data: &TableData,
        sort_column: Option<usize>,
        sort_ascending: bool,
        current_page: usize,
        page_size: usize,
        filters: &[FilterRule],
        search_text: &str,
    ) -> Option<DataGridEvent> {
        let column_to_sort = Cell::new(None);

        // Apply filters to get indices of matching rows
        let filtered_indices = Self::apply_filters(&data.rows, filters);

        // Apply search on filtered results
        let searched_indices = Self::apply_search(&data.rows, filtered_indices, search_text);

        // Calculate pagination on filtered and searched data
        let total_rows = searched_indices.len();
        let start_row = current_page * page_size;
        let end_row = (start_row + page_size).min(total_rows);

        let available_height = ui.available_height();
        egui::ScrollArea::both()
            .id_source("data_grid")
            .max_height(available_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                use egui_extras::{Column, TableBuilder};

                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .vscroll(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::initial(50.0).at_least(40.0).resizable(false)) // Line number column
                    .columns(Column::initial(120.0).at_least(80.0).resizable(true).clip(true), data.columns.len())
                    .min_scrolled_height(available_height);

                table
                    .header(22.0, |mut header| {
                        // Line number header
                        header.col(|ui| {
                            ui.vertical(|ui| {
                                ui.strong("#");
                                ui.add_space(2.0);
                                ui.separator();
                            });
                        });

                        // Data column headers
                        for (col_index, column) in data.columns.iter().enumerate() {
                            header.col(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        // Add key indicator
                                        if column.is_primary_key {
                                            ui.label(egui::RichText::new("🔑").color(egui::Color32::from_rgb(255, 215, 0)));
                                        } else if column.is_foreign_key {
                                            ui.label(egui::RichText::new("🔗").color(egui::Color32::from_rgb(150, 150, 255)));
                                        }

                                        // Create clickable header with sort indicator
                                        let sort_indicator = if sort_column == Some(col_index) {
                                            if sort_ascending { " ▲" } else { " ▼" }
                                        } else {
                                            ""
                                        };

                                        // Column name (strong)
                                        let header_text = format!("{}{}", column.name, sort_indicator);
                                        if ui.button(egui::RichText::new(header_text).strong()).clicked() {
                                            column_to_sort.set(Some(col_index));
                                        }
                                    });

                                    // Data type (gray, smaller text)
                                    ui.label(egui::RichText::new(&column.data_type)
                                        .size(9.0)
                                        .color(egui::Color32::from_rgb(150, 150, 150)));

                                    ui.add_space(2.0);
                                    ui.separator();
                                });
                            });
                        }
                    })
                    .body(|mut body| {
                        // Only show rows for current page from searched indices
                        let page_indices = &searched_indices[start_row..end_row];
                        for (page_row_index, &original_row_index) in page_indices.iter().enumerate() {
                            let row = &data.rows[original_row_index];
                            let actual_row_index = start_row + page_row_index;
                            let is_selected = self.selected_row == Some(actual_row_index);

                            body.row(18.0, |mut row_ui| {
                                // Line number cell
                                row_ui.col(|ui| {
                                    let rect = ui.available_rect_before_wrap();

                                    // Add background color for selected row
                                    if is_selected {
                                        ui.painter().rect_filled(
                                            rect,
                                            0.0,
                                            egui::Color32::from_rgb(200, 200, 200)
                                        );
                                    }

                                    // Interact with entire cell area for row selection
                                    let cell_response = ui.interact(rect, ui.id().with(actual_row_index), egui::Sense::click());

                                    // Left click anywhere in cell to select row
                                    if cell_response.clicked() {
                                        if is_selected {
                                            self.selected_row = None;
                                        } else {
                                            self.selected_row = Some(actual_row_index);
                                        }
                                    }

                                    // Display line number (1-indexed)
                                    ui.label(egui::RichText::new(format!("{}", actual_row_index + 1))
                                        .color(egui::Color32::from_rgb(150, 150, 150)));
                                });

                                // Data cells
                                for cell in row {
                                    row_ui.col(|ui| {
                                        // Get the full cell rect
                                        let rect = ui.available_rect_before_wrap();

                                        // Add background color for selected row
                                        if is_selected {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                egui::Color32::from_rgb(200, 200, 200)
                                            );
                                        }

                                        // Interact with entire cell area for row selection
                                        let cell_response = ui.interact(rect, ui.id().with(actual_row_index), egui::Sense::click());

                                        // Left click anywhere in cell to select row
                                        if cell_response.clicked() {
                                            if is_selected {
                                                self.selected_row = None;
                                            } else {
                                                self.selected_row = Some(actual_row_index);
                                            }
                                        }

                                        ui.style_mut().wrap = Some(false);

                                        let label_response = ui.add(
                                            egui::Label::new(cell)
                                                .truncate(true)
                                                .selectable(true)
                                        );

                                        // Right click context menu to copy cell value
                                        label_response.context_menu(|ui| {
                                            if ui.button("Copy Cell Value").clicked() {
                                                ui.output_mut(|o| o.copied_text = cell.clone());
                                                ui.close_menu();
                                            }
                                        });
                                    });
                                }
                            });
                        }
                    });
            });

        // Handle column sort after the immutable borrow is released
        if let Some(col_index) = column_to_sort.get() {
            return Some(DataGridEvent::ColumnSorted(col_index));
        }

        None
    }
}
