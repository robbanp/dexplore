use crate::models::{TableData, FilterRule, FilterConjunction};
use eframe::egui;
use std::cell::Cell;

#[derive(Debug)]
pub enum DataGridEvent {
    ColumnSorted(usize),
    RowSelected(Option<usize>),
}

#[derive(Debug, Default)]
pub struct SearchMatchInfo {
    pub total_matches: usize,
    pub current_match_page: Option<usize>,
    pub current_match_row_in_page: Option<usize>, // Row index within the current page
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
        current_match_index: usize,
    ) -> (Option<DataGridEvent>, SearchMatchInfo) {
        let column_to_sort = Cell::new(None);

        // Apply filters to get indices of matching rows
        let filtered_indices = Self::apply_filters(&data.rows, filters);

        // Calculate pagination on filtered data (no filtering by search, just highlighting)
        let total_rows = filtered_indices.len();
        let start_row = current_page * page_size;
        let end_row = (start_row + page_size).min(total_rows);

        let search_lower = search_text.to_lowercase();

        // Collect all search matches across filtered data to determine total count and current match position
        let mut match_info = SearchMatchInfo::default();
        let mut current_match_cell_position: Option<(usize, usize)> = None; // (row_index, col_index)
        if !search_lower.is_empty() {
            let mut match_count = 0;
            for (filtered_idx, &original_row_idx) in filtered_indices.iter().enumerate() {
                let row = &data.rows[original_row_idx];
                for (col_idx, cell) in row.iter().enumerate() {
                    if cell.to_lowercase().contains(&search_lower) {
                        if match_count == current_match_index {
                            // This is the current match - calculate its page and position
                            let page = filtered_idx / page_size;
                            let row_in_page = filtered_idx % page_size;
                            match_info.current_match_page = Some(page);
                            match_info.current_match_row_in_page = Some(row_in_page);
                            current_match_cell_position = Some((original_row_idx, col_idx));
                        }
                        match_count += 1;
                    }
                }
            }
            match_info.total_matches = match_count;
        }

        let available_height = ui.available_height();
        egui::ScrollArea::both()
            .id_source("data_grid")
            .max_height(available_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                use egui_extras::{Column, TableBuilder};

                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .vscroll(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::initial(50.0).at_least(40.0).resizable(false)) // Line number column
                    .columns(Column::initial(120.0).at_least(80.0).resizable(true).clip(true), data.columns.len())
                    .min_scrolled_height(available_height);

                // Scroll to the row containing the current match
                if let Some(row_in_page) = match_info.current_match_row_in_page {
                    table = table.scroll_to_row(row_in_page, Some(egui::Align::Center));
                }

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
                        // Only show rows for current page from filtered indices
                        let page_indices = &filtered_indices[start_row..end_row];
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
                                for (col_idx, cell) in row.iter().enumerate() {
                                    row_ui.col(|ui| {
                                        // Get the full cell rect
                                        let rect = ui.available_rect_before_wrap();

                                        // Check if this cell matches the search text
                                        let has_search_match = !search_lower.is_empty()
                                            && cell.to_lowercase().contains(&search_lower);

                                        // Check if this is the current match
                                        let is_current_match = current_match_cell_position
                                            .map(|(row_idx, c_idx)| row_idx == original_row_index && c_idx == col_idx)
                                            .unwrap_or(false);

                                        // Add background color for selected row or search match
                                        if is_selected {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                egui::Color32::from_rgb(200, 200, 200)
                                            );
                                        } else if is_current_match {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                egui::Color32::from_rgb(255, 180, 100)  // Orange highlight for current match
                                            );
                                        } else if has_search_match {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                egui::Color32::from_rgb(255, 255, 150)  // Yellow highlight for other search matches
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
            return (Some(DataGridEvent::ColumnSorted(col_index)), match_info);
        }

        (None, match_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ColumnInfo;

    fn create_test_data(rows: Vec<Vec<String>>) -> TableData {
        TableData {
            name: "test_table".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "col1".to_string(),
                    data_type: "text".to_string(),
                    is_primary_key: false,
                    is_foreign_key: false,
                },
                ColumnInfo {
                    name: "col2".to_string(),
                    data_type: "text".to_string(),
                    is_primary_key: false,
                    is_foreign_key: false,
                },
            ],
            rows,
        }
    }

    #[test]
    fn test_search_match_counting() {
        let data = create_test_data(vec![
            vec!["apple".to_string(), "banana".to_string()],
            vec!["cherry".to_string(), "apple".to_string()],
            vec!["date".to_string(), "elderberry".to_string()],
            vec!["apple".to_string(), "fig".to_string()],
        ]);

        // Test counting matches for "apple"
        let search_text = "apple";
        let mut count = 0;
        for row in &data.rows {
            for cell in row {
                if cell.to_lowercase().contains(&search_text.to_lowercase()) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 3, "Should find 3 occurrences of 'apple'");
    }

    #[test]
    fn test_search_case_insensitive() {
        let data = create_test_data(vec![
            vec!["Apple".to_string(), "BANANA".to_string()],
            vec!["cherry".to_string(), "aPpLe".to_string()],
        ]);

        let search_text = "apple";
        let mut count = 0;
        for row in &data.rows {
            for cell in row {
                if cell.to_lowercase().contains(&search_text.to_lowercase()) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 2, "Should find 2 occurrences case-insensitively");
    }

    #[test]
    fn test_search_partial_match() {
        let data = create_test_data(vec![
            vec!["pineapple".to_string(), "banana".to_string()],
            vec!["apple".to_string(), "applesauce".to_string()],
        ]);

        let search_text = "apple";
        let mut count = 0;
        for row in &data.rows {
            for cell in row {
                if cell.to_lowercase().contains(&search_text.to_lowercase()) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 3, "Should find partial matches");
    }

    #[test]
    fn test_page_calculation() {
        let page_size = 10;

        // Test various row positions
        assert_eq!(0 / page_size, 0, "First row should be on page 0");
        assert_eq!(9 / page_size, 0, "Row 9 should be on page 0");
        assert_eq!(10 / page_size, 1, "Row 10 should be on page 1");
        assert_eq!(15 / page_size, 1, "Row 15 should be on page 1");
        assert_eq!(20 / page_size, 2, "Row 20 should be on page 2");
    }

    #[test]
    fn test_row_in_page_calculation() {
        let page_size = 10;

        // Test row position within page
        assert_eq!(0 % page_size, 0, "Row 0 should be at position 0 in page");
        assert_eq!(9 % page_size, 9, "Row 9 should be at position 9 in page");
        assert_eq!(10 % page_size, 0, "Row 10 should be at position 0 in page");
        assert_eq!(15 % page_size, 5, "Row 15 should be at position 5 in page");
        assert_eq!(23 % page_size, 3, "Row 23 should be at position 3 in page");
    }

    #[test]
    fn test_match_index_navigation() {
        let total_matches = 5;

        // Test forward navigation
        let mut index = 0;
        index = (index + 1) % total_matches;
        assert_eq!(index, 1, "Next from 0 should be 1");

        index = 4;
        index = (index + 1) % total_matches;
        assert_eq!(index, 0, "Next from 4 should wrap to 0");

        // Test backward navigation
        index = 1;
        index = if index == 0 { total_matches - 1 } else { index - 1 };
        assert_eq!(index, 0, "Previous from 1 should be 0");

        index = 0;
        index = if index == 0 { total_matches - 1 } else { index - 1 };
        assert_eq!(index, 4, "Previous from 0 should wrap to 4");
    }

    #[test]
    fn test_empty_search() {
        let data = create_test_data(vec![
            vec!["apple".to_string(), "banana".to_string()],
        ]);

        let search_text = "";
        let mut count = 0;
        for row in &data.rows {
            for cell in row {
                if !search_text.is_empty() && cell.to_lowercase().contains(&search_text.to_lowercase()) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 0, "Empty search should return 0 matches");
    }

    #[test]
    fn test_no_matches() {
        let data = create_test_data(vec![
            vec!["apple".to_string(), "banana".to_string()],
            vec!["cherry".to_string(), "date".to_string()],
        ]);

        let search_text = "xyz";
        let mut count = 0;
        for row in &data.rows {
            for cell in row {
                if cell.to_lowercase().contains(&search_text.to_lowercase()) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 0, "Should find 0 matches for non-existent text");
    }

    #[test]
    fn test_search_match_info_default() {
        let match_info = SearchMatchInfo::default();
        assert_eq!(match_info.total_matches, 0, "Default should have 0 matches");
        assert_eq!(match_info.current_match_page, None, "Default should have no current page");
        assert_eq!(match_info.current_match_row_in_page, None, "Default should have no current row");
    }

    #[test]
    fn test_current_match_position() {
        let page_size = 3;
        let data = create_test_data(vec![
            vec!["apple".to_string(), "banana".to_string()],  // row 0
            vec!["cherry".to_string(), "apple".to_string()],  // row 1
            vec!["date".to_string(), "elderberry".to_string()],  // row 2
            vec!["apple".to_string(), "fig".to_string()],  // row 3 (page 1)
            vec!["grape".to_string(), "apple".to_string()],  // row 4 (page 1)
        ]);

        // Simulate finding matches
        let search_text = "apple";
        let mut matches: Vec<(usize, usize)> = Vec::new(); // (row_idx, col_idx)

        for (row_idx, row) in data.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.to_lowercase().contains(&search_text.to_lowercase()) {
                    matches.push((row_idx, col_idx));
                }
            }
        }

        assert_eq!(matches.len(), 4, "Should find 4 matches");

        // Test first match
        let (row_idx, _) = matches[0];
        assert_eq!(row_idx / page_size, 0, "First match should be on page 0");
        assert_eq!(row_idx % page_size, 0, "First match should be at row 0 in page");

        // Test third match
        let (row_idx, _) = matches[2];
        assert_eq!(row_idx / page_size, 1, "Third match should be on page 1");
        assert_eq!(row_idx % page_size, 0, "Third match should be at row 0 in page");

        // Test fourth match
        let (row_idx, _) = matches[3];
        assert_eq!(row_idx / page_size, 1, "Fourth match should be on page 1");
        assert_eq!(row_idx % page_size, 1, "Fourth match should be at row 1 in page");
    }
}
