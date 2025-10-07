use crate::config::{Config, DatabaseConnection};
use crate::db::{AsyncOperation, Database, SchemaInfo};
use crate::models::{AppState, Tab, TabSource, TableData};
use crate::ui::components::*;
use crate::ui::setup_styles;
use eframe::egui;
use poll_promise::Promise;
use std::collections::HashSet;
use std::sync::Arc;

pub struct DbClientApp {
    // Connection state
    pub config: Config,
    pub connection_string: String,
    pub database: Option<Arc<Database>>,
    pub connection_status: String,

    // Tokio runtime for async operations
    pub runtime: Arc<tokio::runtime::Runtime>,

    // UI state
    pub schemas: Vec<SchemaInfo>,
    pub expanded_schemas: HashSet<String>,
    pub selected_table: Option<(String, String)>, // (schema, table)

    // Tabs
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub next_tab_id: usize,

    // Query input
    pub query_input: String,
    pub show_query_panel: bool,

    // Async operations
    pub pending_operation: Option<AsyncOperation>,

    // Status
    pub status_message: String,

    // Settings dialog
    pub show_settings: bool,
    pub edit_connection: Option<DatabaseConnection>,
    pub edit_connection_index: Option<usize>,

    // UI Components
    menu_bar: MenuBar,
    status_bar: StatusBar,
    query_panel: QueryPanel,
    settings_dialog: SettingsDialog,
    connection_editor: ConnectionEditor,
    database_tree: DatabaseTree,
    tab_bar: TabBar,
    pagination: PaginationControls,
    data_grid: DataGrid,
}

impl DbClientApp {
    pub fn save_state(&self) {
        let state = AppState {
            tabs: self.tabs.clone(),
            active_tab: self.active_tab,
            next_tab_id: self.next_tab_id,
            expanded_schemas: self.expanded_schemas.clone(),
        };
        let _ = state.save(); // Ignore errors when saving state
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Setup monospace styles for better data display
        setup_styles(&cc.egui_ctx);

        let config = Config::load().unwrap_or_else(|_| Config::new());

        // Try to get connection from last saved connection, environment, or use default
        let connection_string = if let Some(conn) = config.get_last_connection() {
            conn.to_connection_string()
        } else {
            std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "host=localhost user=postgres password=postgres dbname=postgres".to_string())
        };

        // Create a persistent tokio runtime for all async operations
        let runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime")
        );

        // Try to restore previous state
        let (tabs, active_tab, next_tab_id, expanded_schemas) = if let Ok(state) = AppState::load() {
            (state.tabs, state.active_tab, state.next_tab_id, state.expanded_schemas)
        } else {
            (Vec::new(), 0, 0, HashSet::new())
        };

        let mut app = Self {
            config,
            connection_string,
            database: None,
            connection_status: "Not connected".to_string(),
            runtime,
            schemas: Vec::new(),
            expanded_schemas,
            selected_table: None,
            tabs,
            active_tab,
            next_tab_id,
            query_input: String::new(),
            show_query_panel: false,
            pending_operation: None,
            status_message: "Ready".to_string(),
            show_settings: false,
            edit_connection: None,
            edit_connection_index: None,
            menu_bar: MenuBar::new(),
            status_bar: StatusBar::new(),
            query_panel: QueryPanel::new(),
            settings_dialog: SettingsDialog::new(),
            connection_editor: ConnectionEditor::new(),
            database_tree: DatabaseTree::new(),
            tab_bar: TabBar::new(),
            pagination: PaginationControls::new(),
            data_grid: DataGrid::new(),
        };

        // Auto-connect on startup
        app.connect_to_database();

        app
    }

    pub fn connect_to_database(&mut self) {
        let connection_string = self.connection_string.clone();
        self.connection_status = "Connecting...".to_string();
        let runtime = Arc::clone(&self.runtime);

        self.pending_operation = Some(AsyncOperation::LoadStructure(
            Promise::spawn_thread("load_structure", move || {
                runtime.block_on(async move {
                    let db = Database::connect(&connection_string).await?;
                    let schemas = db.list_schemas_with_tables().await?;
                    Ok((Arc::new(db), schemas))
                })
            })
        ));
    }

    pub fn load_table_data(&mut self, schema: String, table_name: String, tab_index: Option<usize>) {
        if let Some(db) = &self.database {
            self.status_message = format!("Loading table: {}.{}", schema, table_name);
            let db_clone = Arc::clone(db);
            let schema_clone = schema.clone();
            let table_name_clone = table_name.clone();
            let full_table_name = format!("{}.{}", schema, table_name);
            let runtime = Arc::clone(&self.runtime);

            let promise = Promise::spawn_thread("query_table", move || {
                runtime.block_on(async move {
                    db_clone.query_table(&full_table_name, 100000).await
                })
            });

            self.pending_operation = Some(AsyncOperation::LoadTableData(schema_clone, table_name_clone, promise, tab_index));
        }
    }

    pub fn execute_query(&mut self, tab_index: Option<usize>) {
        if let Some(db) = &self.database {
            let query = self.query_input.clone();
            if query.trim().is_empty() {
                return;
            }

            self.status_message = "Executing query...".to_string();
            let db_clone = Arc::clone(db);
            let query_clone = query.clone();
            let runtime = Arc::clone(&self.runtime);

            let promise = Promise::spawn_thread("execute_query", move || {
                runtime.block_on(async move {
                    db_clone.execute_query(&query_clone).await
                })
            });

            self.pending_operation = Some(AsyncOperation::ExecuteQuery(query, promise, tab_index));
        }
    }

    pub fn add_tab(&mut self, title: String, data: Option<TableData>, source: TabSource) {
        let tab = Tab {
            id: self.next_tab_id,
            title,
            data,
            is_loading: false,
            sort_column: None,
            sort_ascending: true,
            current_page: 0,
            page_size: 100,
            source,
        };
        self.next_tab_id += 1;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.save_state();
    }

    pub fn reload_current_tab(&mut self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            let source = tab.source.clone();
            let tab_index = self.active_tab;
            match source {
                TabSource::Table { schema, table } => {
                    self.load_table_data(schema, table, Some(tab_index));
                }
                TabSource::Query { sql } => {
                    self.query_input = sql;
                    self.execute_query(Some(tab_index));
                }
            }
        }
    }

    pub fn sort_tab_data(&mut self, tab_index: usize, column_index: usize) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            // Toggle sort direction if clicking same column
            if tab.sort_column == Some(column_index) {
                tab.sort_ascending = !tab.sort_ascending;
            } else {
                tab.sort_column = Some(column_index);
                tab.sort_ascending = true;
            }

            // Sort the data
            if let Some(data) = &mut tab.data {
                let ascending = tab.sort_ascending;
                data.rows.sort_by(|a, b| {
                    let a_val = a.get(column_index).map(|s| s.as_str()).unwrap_or("");
                    let b_val = b.get(column_index).map(|s| s.as_str()).unwrap_or("");

                    // Try to parse as numbers for numeric sorting
                    let cmp = match (a_val.parse::<f64>(), b_val.parse::<f64>()) {
                        (Ok(a_num), Ok(b_num)) => a_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal),
                        _ => a_val.cmp(b_val),
                    };

                    if ascending { cmp } else { cmp.reverse() }
                });
            }
            self.save_state();
        }
    }

    pub fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active_tab >= self.tabs.len() && self.active_tab > 0 {
                self.active_tab = self.tabs.len() - 1;
            }
            self.save_state();
        }
    }
}

impl eframe::App for DbClientApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle pending async operations
        self.handle_async_operations();

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if let Some(event) = self.menu_bar.show(ui, &self.connection_status) {
                match event {
                    MenuBarEvent::ShowSettings => self.show_settings = true,
                    MenuBarEvent::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    MenuBarEvent::ToggleQueryPanel => self.show_query_panel = !self.show_query_panel,
                    MenuBarEvent::Refresh => self.connect_to_database(),
                }
            }
        });

        // Status bar
        let row_count = self.tabs.get(self.active_tab)
            .and_then(|tab| tab.data.as_ref())
            .map(|data| data.rows.len());

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status_bar.show(ui, &self.status_message, row_count);
        });

        // Query panel (if shown)
        if self.show_query_panel {
            egui::TopBottomPanel::top("query_panel").show(ctx, |ui| {
                if let Some(event) = self.query_panel.show(ui, &mut self.query_input) {
                    match event {
                        QueryPanelEvent::Execute => self.execute_query(None),
                        QueryPanelEvent::Clear => self.query_input.clear(),
                        QueryPanelEvent::Close => self.show_query_panel = false,
                    }
                }
            });
        }

        // Settings dialog
        if self.show_settings {
            if let Some(event) = self.settings_dialog.show(ctx, &self.config) {
                match event {
                    SettingsDialogEvent::Connect(idx) => {
                        if let Some(conn) = self.config.get_connection(idx) {
                            self.connection_string = conn.to_connection_string();
                            self.config.last_connection_index = Some(idx);
                            let _ = self.config.save();
                            self.connect_to_database();
                            self.show_settings = false;
                        }
                    }
                    SettingsDialogEvent::Edit(idx) => {
                        if let Some(conn) = self.config.get_connection(idx) {
                            self.edit_connection = Some(conn.clone());
                            self.edit_connection_index = Some(idx);
                        }
                    }
                    SettingsDialogEvent::Delete(idx) => {
                        self.config.delete_connection(idx);
                        let _ = self.config.save();
                    }
                    SettingsDialogEvent::NewConnection => {
                        self.edit_connection = Some(DatabaseConnection::new());
                        self.edit_connection_index = None;
                    }
                    SettingsDialogEvent::Close => self.show_settings = false,
                }
            }
        }

        // Connection editor dialog
        if let Some(ref mut conn) = self.edit_connection {
            if let Some(event) = self.connection_editor.show(ctx, conn) {
                match event {
                    ConnectionEditorEvent::Save => {
                        if let Some(idx) = self.edit_connection_index {
                            self.config.update_connection(idx, conn.clone());
                        } else {
                            self.config.add_connection(conn.clone());
                        }
                        let _ = self.config.save();
                        self.edit_connection = None;
                        self.edit_connection_index = None;
                    }
                    ConnectionEditorEvent::Cancel => {
                        self.edit_connection = None;
                        self.edit_connection_index = None;
                    }
                }
            }
        }

        // Left sidebar - Database tree
        egui::SidePanel::left("database_structure_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(200.0)
            .max_width(600.0)
            .show(ctx, |ui| {
                ui.heading("Database Structure");
                ui.separator();

                if let Some(event) = self.database_tree.show(ui, &self.schemas, &self.expanded_schemas, &self.selected_table) {
                    match event {
                        DatabaseTreeEvent::TableClicked(schema, table) => {
                            self.selected_table = Some((schema.clone(), table.clone()));
                            self.load_table_data(schema, table, None);
                        }
                        DatabaseTreeEvent::TableRightClicked(schema, table) => {
                            self.selected_table = Some((schema.clone(), table.clone()));
                            self.load_table_data(schema, table, None);
                        }
                        DatabaseTreeEvent::SchemaToggled(schema_name) => {
                            if self.expanded_schemas.contains(&schema_name) {
                                self.expanded_schemas.remove(&schema_name);
                            } else {
                                self.expanded_schemas.insert(schema_name);
                            }
                            self.save_state();
                        }
                    }
                }
            });

        // Main content area - Tabs and data grid
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar
            if let Some(event) = self.tab_bar.show(ui, &self.tabs, self.active_tab) {
                match event {
                    TabBarEvent::TabActivated(i) => {
                        self.active_tab = i;
                        self.save_state();
                    }
                    TabBarEvent::TabClosed(i) => {
                        self.close_tab(i);
                    }
                }
            }

            // Data grid with pagination
            // Extract values to avoid borrow checker issues
            let (has_data, is_loading, sort_column, sort_ascending, current_page, page_size, total_rows) =
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    if let Some(data) = &tab.data {
                        (true, false, tab.sort_column, tab.sort_ascending, tab.current_page, tab.page_size, Some(data.rows.len()))
                    } else {
                        (false, tab.is_loading, None, true, 0, 100, None)
                    }
                } else {
                    (false, false, None, true, 0, 100, None)
                };

            if has_data {
                // Pagination controls
                if let Some(event) = self.pagination.show(ui, current_page, page_size, total_rows.unwrap()) {
                    match event {
                        PaginationEvent::Reload => self.reload_current_tab(),
                        PaginationEvent::PageSizeChanged(size) => {
                            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                tab.page_size = size;
                                tab.current_page = 0;
                                self.save_state();
                            }
                        }
                        PaginationEvent::PageChanged(page) => {
                            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                tab.current_page = page;
                                self.save_state();
                            }
                        }
                    }
                }

                // Data grid
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    if let Some(data) = &tab.data {
                        if let Some(event) = self.data_grid.show(ui, data, sort_column, sort_ascending, current_page, page_size) {
                            match event {
                                DataGridEvent::ColumnSorted(col_index) => {
                                    self.sort_tab_data(self.active_tab, col_index);
                                }
                                DataGridEvent::RowSelected(_) => {
                                    // Row selection handled by data_grid internally
                                }
                            }
                        }
                    }
                }
            } else if is_loading {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Loading...");
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a table to view data");
                });
            }
        });

        // Request repaint if we're waiting for async operations
        if self.pending_operation.is_some() {
            ctx.request_repaint();
        }
    }
}

impl DbClientApp {
    fn handle_async_operations(&mut self) {
        let mut should_clear_operation = false;
        let mut tab_to_add: Option<(String, Option<TableData>, TabSource)> = None;
        let mut new_schemas: Option<Vec<SchemaInfo>> = None;
        let mut new_database: Option<Arc<Database>> = None;
        let mut new_status = None;
        let mut new_connection_status = None;
        let mut close_query_panel = false;

        if let Some(operation) = &self.pending_operation {
            match operation {
                AsyncOperation::LoadStructure(promise) => {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok((db, schemas)) => {
                                let total_tables: usize = schemas.iter().map(|s| s.tables.len()).sum();
                                new_schemas = Some(schemas.clone());
                                new_connection_status = Some(format!("Connected - {} schemas, {} tables", schemas.len(), total_tables));
                                new_status = Some(format!("Loaded {} schemas with {} tables", schemas.len(), total_tables));
                                new_database = Some(Arc::clone(db));
                            }
                            Err(e) => {
                                new_connection_status = Some(format!("Connection failed: {}", e));
                                new_status = Some(format!("Error: {}", e));
                            }
                        }
                        should_clear_operation = true;
                    }
                }
                AsyncOperation::LoadTableData(schema, table_name, promise, tab_index) => {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok((columns, rows)) => {
                                let data = TableData {
                                    name: format!("{}.{}", schema, table_name),
                                    columns: columns.clone(),
                                    rows: rows.clone(),
                                };

                                if let Some(idx) = tab_index {
                                    if let Some(tab) = self.tabs.get_mut(*idx) {
                                        tab.data = Some(data);
                                    }
                                    new_status = Some(format!("Reloaded {} rows from {}.{}", rows.len(), schema, table_name));
                                } else {
                                    let source = TabSource::Table {
                                        schema: schema.clone(),
                                        table: table_name.clone(),
                                    };
                                    tab_to_add = Some((format!("{}.{}", schema, table_name), Some(data), source));
                                    new_status = Some(format!("Loaded {} rows from {}.{}", rows.len(), schema, table_name));
                                }
                            }
                            Err(e) => {
                                new_status = Some(format!("Error loading table: {}", e));
                            }
                        }
                        should_clear_operation = true;
                    }
                }
                AsyncOperation::ExecuteQuery(query, promise, tab_index) => {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok((columns, rows)) => {
                                let data = TableData {
                                    name: "Query Result".to_string(),
                                    columns: columns.clone(),
                                    rows: rows.clone(),
                                };

                                if let Some(idx) = tab_index {
                                    if let Some(tab) = self.tabs.get_mut(*idx) {
                                        tab.data = Some(data);
                                    }
                                    new_status = Some(format!("Reloaded query: {} rows", rows.len()));
                                } else {
                                    let source = TabSource::Query {
                                        sql: query.clone(),
                                    };
                                    tab_to_add = Some(("Query Result".to_string(), Some(data), source));
                                    new_status = Some(format!("Query returned {} rows", rows.len()));
                                    close_query_panel = true;
                                }
                            }
                            Err(e) => {
                                new_status = Some(format!("Query error: {}", e));
                            }
                        }
                        should_clear_operation = true;
                    }
                }
            }
        }

        // Apply state changes
        if should_clear_operation {
            self.pending_operation = None;
        }
        if let Some((title, data, source)) = tab_to_add {
            self.add_tab(title, data, source);
        }
        if let Some(schemas) = new_schemas {
            self.schemas = schemas;
        }
        if let Some(db) = new_database {
            self.database = Some(db);
        }
        if let Some(status) = new_status {
            self.status_message = status;
        }
        if let Some(conn_status) = new_connection_status {
            self.connection_status = conn_status;
        }
        if close_query_panel {
            self.show_query_panel = false;
        }
    }
}
