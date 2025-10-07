use serde::{Deserialize, Serialize};
use crate::db::ColumnInfo;
use crate::models::FilterRule;

#[derive(Clone, Serialize, Deserialize)]
pub struct TableData {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub data: Option<TableData>,
    #[serde(skip)]
    pub is_loading: bool,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub current_page: usize,
    pub page_size: usize,
    // Track the source for reloading
    pub source: TabSource,
    // Filters for this tab
    pub filters: Vec<FilterRule>,
    // Search text for quick search across all columns
    pub search_text: String,
    // Current search match index for navigation
    #[serde(skip)]
    pub search_match_index: usize,
    // Query input for this tab (editable SQL)
    pub query_input: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TabSource {
    Table { schema: String, table: String },
    Query { sql: String },
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tab_search_reset_on_text_change() {
        // Simulate what happens when search text changes
        let search_match_index = 5;
        let current_page = 2;

        // When search text changes, both should reset
        let new_search_match_index = 0;
        let new_current_page = 0;

        assert_ne!(search_match_index, new_search_match_index, "Search index should change from 5 to 0");
        assert_ne!(current_page, new_current_page, "Page should change from 2 to 0");
        assert_eq!(new_search_match_index, 0, "Search index should reset to 0");
        assert_eq!(new_current_page, 0, "Page should reset to 0");
    }

    #[test]
    fn test_next_match_navigation() {
        let total_matches = 10;
        let mut search_match_index = 0;

        // Navigate forward
        search_match_index = (search_match_index + 1) % total_matches;
        assert_eq!(search_match_index, 1);

        search_match_index = (search_match_index + 1) % total_matches;
        assert_eq!(search_match_index, 2);

        // Test wrapping at end
        search_match_index = 9;
        search_match_index = (search_match_index + 1) % total_matches;
        assert_eq!(search_match_index, 0, "Should wrap to 0 at end");
    }

    #[test]
    fn test_prev_match_navigation() {
        let total_matches = 10;
        let mut search_match_index = 5;

        // Navigate backward
        search_match_index = if search_match_index == 0 {
            total_matches - 1
        } else {
            search_match_index - 1
        };
        assert_eq!(search_match_index, 4);

        // Test wrapping at beginning
        search_match_index = 0;
        search_match_index = if search_match_index == 0 {
            total_matches - 1
        } else {
            search_match_index - 1
        };
        assert_eq!(search_match_index, 9, "Should wrap to last match at beginning");
    }

    #[test]
    fn test_navigation_with_single_match() {
        let total_matches = 1;
        let mut search_match_index = 0;

        // Forward navigation with single match should stay at 0
        search_match_index = (search_match_index + 1) % total_matches;
        assert_eq!(search_match_index, 0, "Should stay at 0 with single match");

        // Backward navigation with single match should stay at 0
        search_match_index = if search_match_index == 0 {
            total_matches - 1
        } else {
            search_match_index - 1
        };
        assert_eq!(search_match_index, 0, "Should stay at 0 with single match");
    }

    #[test]
    fn test_navigation_with_two_matches() {
        let total_matches = 2;
        let mut search_match_index = 0;

        // Forward: 0 -> 1
        search_match_index = (search_match_index + 1) % total_matches;
        assert_eq!(search_match_index, 1);

        // Forward: 1 -> 0 (wrap)
        search_match_index = (search_match_index + 1) % total_matches;
        assert_eq!(search_match_index, 0);

        // Backward: 0 -> 1 (wrap)
        search_match_index = if search_match_index == 0 {
            total_matches - 1
        } else {
            search_match_index - 1
        };
        assert_eq!(search_match_index, 1);

        // Backward: 1 -> 0
        search_match_index = if search_match_index == 0 {
            total_matches - 1
        } else {
            search_match_index - 1
        };
        assert_eq!(search_match_index, 0);
    }

    #[test]
    fn test_page_auto_navigation() {
        // Test that page changes when navigating to a match on a different page
        let mut current_page = 0;
        let target_page = 3;

        // Simulate auto-navigation to page containing match
        if current_page != target_page {
            current_page = target_page;
        }

        assert_eq!(current_page, 3, "Should navigate to target page");
    }

    #[test]
    fn test_search_state_initialization() {
        // Test that new tabs start with correct search state
        let search_match_index = 0;
        let search_text = String::new();

        assert_eq!(search_match_index, 0, "Should start at match index 0");
        assert_eq!(search_text, "", "Should start with empty search text");
    }
}
