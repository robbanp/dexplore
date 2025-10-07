# Module Specifications

Detailed interface specifications for each module after refactoring.

## models/tab.rs

```rust
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
    pub source: TabSource,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TabSource {
    Table { schema: String, table: String },
    Query { sql: String },
}
```

## models/state.rs

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use anyhow::Result;
use crate::models::Tab;

#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub next_tab_id: usize,
    pub expanded_schemas: HashSet<String>,
}

impl AppState {
    pub fn save_path() -> Result<PathBuf>;
    pub fn save(&self) -> Result<()>;
    pub fn load() -> Result<Self>;
}
```

## db/models.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub name: String,
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
}
```

## db/client.rs

```rust
use anyhow::Result;
use tokio_postgres::Client;
use crate::db::{ColumnInfo, SchemaInfo};

pub struct Database {
    client: Client,
}

impl Database {
    pub async fn connect(connection_string: &str) -> Result<Self>;
    pub async fn list_schemas_with_tables(&self) -> Result<Vec<SchemaInfo>>;
    pub async fn query_table(&self, table: &str, limit: i64) -> Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>;
    pub async fn execute_query(&self, query: &str) -> Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>;
}
```

## db/operations.rs

```rust
use poll_promise::Promise;
use anyhow::Result;
use std::sync::Arc;
use crate::db::{Database, ColumnInfo, SchemaInfo};

pub enum AsyncOperation {
    LoadStructure(Promise<Result<(Arc<Database>, Vec<SchemaInfo>)>>),
    LoadTableData(String, String, Promise<Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>>, Option<usize>),
    ExecuteQuery(String, Promise<Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>>, Option<usize>),
}
```

## app.rs

```rust
use std::sync::Arc;
use std::collections::HashSet;
use eframe::egui;
use crate::config::{Config, DatabaseConnection};
use crate::db::{Database, SchemaInfo, AsyncOperation};
use crate::models::{Tab, TabSource, TableData};

pub struct DbClientApp {
    // Connection state
    pub config: Config,
    pub connection_string: String,
    pub database: Option<Arc<Database>>,
    pub connection_status: String,

    // Runtime
    pub runtime: Arc<tokio::runtime::Runtime>,

    // UI state
    pub schemas: Vec<SchemaInfo>,
    pub expanded_schemas: HashSet<String>,
    pub selected_table: Option<(String, String)>,
    pub selected_row: Option<usize>,

    // Tabs
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub next_tab_id: usize,

    // Query
    pub query_input: String,
    pub show_query_panel: bool,

    // Async
    pub pending_operation: Option<AsyncOperation>,

    // Status
    pub status_message: String,

    // Dialogs
    pub show_settings: bool,
    pub edit_connection: Option<DatabaseConnection>,
    pub edit_connection_index: Option<usize>,
}

impl DbClientApp {
    pub fn new(cc: &eframe::CreationContext) -> Self;
    pub fn save_state(&self);

    // Database operations
    pub fn connect_to_database(&mut self);
    pub fn load_table_data(&mut self, schema: String, table: String, tab_index: Option<usize>);
    pub fn execute_query(&mut self, tab_index: Option<usize>);
    pub fn reload_current_tab(&mut self);

    // Tab operations
    pub fn add_tab(&mut self, title: String, data: Option<TableData>, source: TabSource);
    pub fn close_tab(&mut self, index: usize);
    pub fn sort_tab_data(&mut self, tab_index: usize, column_index: usize);
}

impl eframe::App for DbClientApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage);
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame);
}
```

## ui/styles.rs

```rust
use eframe::egui;

pub fn setup_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(11.0, egui::FontFamily::Monospace)
    );
    // ... other styles

    ctx.set_style(style);
}
```

## ui/components/menu_bar.rs

```rust
use eframe::egui;

pub enum MenuBarEvent {
    ShowSettings,
    ToggleQuery,
    Refresh,
    Quit,
}

pub struct MenuBar {
    pub connection_status: String,
}

impl MenuBar {
    pub fn new(connection_status: String) -> Self;

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<MenuBarEvent>;

    pub fn update_connection_status(&mut self, status: String);
}
```

## ui/components/status_bar.rs

```rust
use eframe::egui;

pub struct StatusBar {
    pub status_message: String,
}

impl StatusBar {
    pub fn new(status_message: String) -> Self;

    pub fn show(&mut self, ui: &mut egui::Ui, current_tab_rows: Option<usize>);

    pub fn update_message(&mut self, message: String);
}
```

## ui/components/query_panel.rs

```rust
use eframe::egui;

pub enum QueryPanelEvent {
    Execute,
    Clear,
    Close,
}

pub struct QueryPanel;

impl QueryPanel {
    pub fn new() -> Self;

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        query_input: &mut String
    ) -> Option<QueryPanelEvent>;
}
```

## ui/components/settings_dialog.rs

```rust
use eframe::egui;
use crate::config::{Config, DatabaseConnection};

pub enum SettingsEvent {
    Connect(usize),
    Edit(usize),
    Delete(usize),
    NewConnection,
    Close,
}

pub struct SettingsDialog;

impl SettingsDialog {
    pub fn new() -> Self;

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        config: &Config
    ) -> Option<SettingsEvent>;
}
```

## ui/components/connection_editor.rs

```rust
use eframe::egui;
use crate::config::DatabaseConnection;

pub enum ConnectionEditorEvent {
    Save(DatabaseConnection),
    Cancel,
}

pub struct ConnectionEditor;

impl ConnectionEditor {
    pub fn new() -> Self;

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        connection: &mut DatabaseConnection
    ) -> Option<ConnectionEditorEvent>;
}
```

## ui/components/database_tree.rs

```rust
use eframe::egui;
use std::collections::HashSet;
use crate::db::SchemaInfo;

pub enum DatabaseTreeEvent {
    TableClicked { schema: String, table: String },
    TableRightClicked { schema: String, table: String },
    SchemaToggled { schema: String },
}

pub struct DatabaseTree;

impl DatabaseTree {
    pub fn new() -> Self;

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        schemas: &[SchemaInfo],
        expanded_schemas: &HashSet<String>,
        selected_table: &Option<(String, String)>
    ) -> Option<DatabaseTreeEvent>;
}
```

## ui/components/tab_bar.rs

```rust
use eframe::egui;
use crate::models::Tab;

pub enum TabBarEvent {
    TabActivated(usize),
    TabClosed(usize),
}

pub struct TabBar;

impl TabBar {
    pub fn new() -> Self;

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        tabs: &[Tab],
        active_tab: usize
    ) -> Option<TabBarEvent>;
}
```

## ui/components/pagination.rs

```rust
use eframe::egui;

pub enum PaginationEvent {
    Reload,
    PageSizeChanged(usize),
    PageChanged(usize),
}

pub struct PaginationControls;

impl PaginationControls {
    pub fn new() -> Self;

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        current_page: usize,
        page_size: usize,
        total_rows: usize
    ) -> Option<PaginationEvent>;
}
```

## ui/components/data_grid.rs

```rust
use eframe::egui;
use crate::models::TableData;

pub enum DataGridEvent {
    ColumnSorted(usize),
    RowSelected(Option<usize>),
}

pub struct DataGrid;

impl DataGrid {
    pub fn new() -> Self;

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        data: &TableData,
        sort_column: Option<usize>,
        sort_ascending: bool,
        current_page: usize,
        page_size: usize,
        selected_row: Option<usize>
    ) -> Option<DataGridEvent>;
}
```

## Dependencies Between Modules

```
main.rs
  └─> app.rs
       ├─> models/*
       ├─> db/*
       ├─> ui/*
       └─> config.rs

ui/components/*
  └─> models/* (read-only)
  └─> db/models.rs (read-only)

app.rs
  └─> db/operations.rs (async handling)

db/client.rs
  └─> db/models.rs
```

## Key Design Principles

1. **Unidirectional data flow**: Events flow up, data flows down
2. **Immutable borrows in UI**: UI components take immutable refs
3. **Event-driven**: UI returns events, app handles them
4. **No circular dependencies**: Clear module hierarchy
5. **Public API through mod.rs**: Clean re-exports

## Migration Checklist Per Module

- [ ] Create module file
- [ ] Move code
- [ ] Update imports
- [ ] Add re-exports to mod.rs
- [ ] Update main.rs imports
- [ ] Build successfully
- [ ] Test functionality
- [ ] Commit changes
