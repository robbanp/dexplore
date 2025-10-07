use serde::{Deserialize, Serialize};
use crate::db::ColumnInfo;

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
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TabSource {
    Table { schema: String, table: String },
    Query { sql: String },
}
