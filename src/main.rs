mod config;
mod db;
mod models;

use anyhow::Result;
use config::{Config, DatabaseConnection};
use db::{ColumnInfo, Database, SchemaInfo};
use models::{AppState, Tab, TabSource, TableData};
use eframe::egui;
use poll_promise::Promise;
use std::collections::HashSet;
use std::sync::Arc;
use std::cell::Cell;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("PostgreSQL Database Client"),
        ..Default::default()
    };

    eframe::run_native(
        "DB Client",
        options,
        Box::new(|cc| Box::new(DbClientApp::new(cc))),
    )
}

enum AsyncOperation {
    LoadStructure(Promise<Result<(Arc<Database>, Vec<SchemaInfo>)>>),
    LoadTableData(String, String, Promise<Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>>, Option<usize>), // schema, table, promise, optional tab_index for reload
    ExecuteQuery(String, Promise<Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>>, Option<usize>), // query, promise, optional tab_index for reload
}

struct DbClientApp {
    // Connection state
    config: Config,
    connection_string: String,
    database: Option<Arc<Database>>,
    connection_status: String,

    // Tokio runtime for async operations
    runtime: Arc<tokio::runtime::Runtime>,

    // UI state
    schemas: Vec<SchemaInfo>,
    expanded_schemas: HashSet<String>,
    selected_table: Option<(String, String)>, // (schema, table)
    selected_row: Option<usize>,

    // Tabs
    tabs: Vec<Tab>,
    active_tab: usize,
    next_tab_id: usize,

    // Query input
    query_input: String,
    show_query_panel: bool,

    // Async operations
    pending_operation: Option<AsyncOperation>,

    // Status
    status_message: String,

    // Settings dialog
    show_settings: bool,
    edit_connection: Option<DatabaseConnection>,
    edit_connection_index: Option<usize>,
}

impl DbClientApp {
    fn save_state(&self) {
        let state = AppState {
            tabs: self.tabs.clone(),
            active_tab: self.active_tab,
            next_tab_id: self.next_tab_id,
            expanded_schemas: self.expanded_schemas.clone(),
        };
        let _ = state.save(); // Ignore errors when saving state
    }

    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set default text style to use monospace for better data display
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(11.0, egui::FontFamily::Monospace)
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(11.0, egui::FontFamily::Monospace)
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(14.0, egui::FontFamily::Monospace)
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(9.0, egui::FontFamily::Monospace)
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(11.0, egui::FontFamily::Monospace)
        );
        cc.egui_ctx.set_style(style);

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
            selected_row: None,
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
        };

        // Auto-connect on startup
        app.connect_to_database();

        app
    }

    fn connect_to_database(&mut self) {
        let connection_string = self.connection_string.clone();
        self.connection_status = "Connecting...".to_string();
        let runtime = Arc::clone(&self.runtime);

        // We'll handle the result in the update loop
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

    fn load_table_data(&mut self, schema: String, table_name: String, tab_index: Option<usize>) {
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

    fn execute_query(&mut self, tab_index: Option<usize>) {
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

    fn add_tab(&mut self, title: String, data: Option<TableData>, source: TabSource) {
        let tab = Tab {
            id: self.next_tab_id,
            title,
            data,
            is_loading: false,
            sort_column: None,
            sort_ascending: true,
            current_page: 0,
            page_size: 100, // Default to 100 rows per page
            source,
        };
        self.next_tab_id += 1;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.save_state();
    }

    fn reload_current_tab(&mut self) {
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

    fn sort_tab_data(&mut self, tab_index: usize, column_index: usize) {
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

    fn close_tab(&mut self, index: usize) {
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
        // Save state when app closes
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle pending async operations
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

                                // Store the database connection for future queries
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
                                    // Update existing tab
                                    if let Some(tab) = self.tabs.get_mut(*idx) {
                                        tab.data = Some(data);
                                    }
                                    new_status = Some(format!("Reloaded {} rows from {}.{}", rows.len(), schema, table_name));
                                } else {
                                    // Create new tab
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
                                    // Update existing tab
                                    if let Some(tab) = self.tabs.get_mut(*idx) {
                                        tab.data = Some(data);
                                    }
                                    new_status = Some(format!("Reloaded query: {} rows", rows.len()));
                                } else {
                                    // Create new tab
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

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Settings...").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Show Query Panel").clicked() {
                        self.show_query_panel = !self.show_query_panel;
                    }
                });

                ui.separator();

                if ui.button("🔄 Refresh").clicked() {
                    self.connect_to_database();
                }

                if ui.button("📝 Query").clicked() {
                    self.show_query_panel = !self.show_query_panel;
                }

                ui.separator();
                ui.label(&self.connection_status);
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        if let Some(data) = &tab.data {
                            ui.label(format!("{} rows", data.rows.len()));
                        }
                    }
                });
            });
        });

        // Query panel (if shown)
        if self.show_query_panel {
            egui::TopBottomPanel::top("query_panel").show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("SQL Query:");
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.query_input)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                    );

                    ui.horizontal(|ui| {
                        if ui.button("Execute").clicked() ||
                           (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command)) {
                            self.execute_query(None);
                        }
                        if ui.button("Clear").clicked() {
                            self.query_input.clear();
                        }
                        if ui.button("Close").clicked() {
                            self.show_query_panel = false;
                        }
                    });
                });
            });
        }

        // Settings dialog
        if self.show_settings {
            egui::Window::new("Settings")
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.heading("Database Connections");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            let mut connection_to_delete: Option<usize> = None;
                            let mut connection_to_edit: Option<usize> = None;
                            let mut connection_to_connect: Option<usize> = None;

                            for (idx, conn) in self.config.connections.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(&conn.name);
                                    ui.label(format!("{}@{}/{}", conn.user, conn.host, conn.database));

                                    if ui.button("Connect").clicked() {
                                        connection_to_connect = Some(idx);
                                    }
                                    if ui.button("Edit").clicked() {
                                        connection_to_edit = Some(idx);
                                    }
                                    if ui.button("Delete").clicked() {
                                        connection_to_delete = Some(idx);
                                    }
                                });
                                ui.separator();
                            }

                            if let Some(idx) = connection_to_connect {
                                if let Some(conn) = self.config.get_connection(idx) {
                                    self.connection_string = conn.to_connection_string();
                                    self.config.last_connection_index = Some(idx);
                                    let _ = self.config.save();
                                    self.connect_to_database();
                                    self.show_settings = false;
                                }
                            }

                            if let Some(idx) = connection_to_edit {
                                if let Some(conn) = self.config.get_connection(idx) {
                                    self.edit_connection = Some(conn.clone());
                                    self.edit_connection_index = Some(idx);
                                }
                            }

                            if let Some(idx) = connection_to_delete {
                                self.config.delete_connection(idx);
                                let _ = self.config.save();
                            }
                        });

                    ui.separator();

                    if ui.button("+ New Connection").clicked() {
                        self.edit_connection = Some(DatabaseConnection::new());
                        self.edit_connection_index = None;
                    }

                    ui.separator();

                    if ui.button("Close").clicked() {
                        self.show_settings = false;
                    }
                });
        }

        // Connection edit dialog
        if let Some(ref mut conn) = self.edit_connection {
            let mut save_connection = false;
            let mut cancel_edit = false;

            egui::Window::new("Connection Details")
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut conn.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Host:");
                        ui.text_edit_singleline(&mut conn.host);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Port:");
                        ui.add(egui::DragValue::new(&mut conn.port).clamp_range(1..=65535));
                    });

                    ui.horizontal(|ui| {
                        ui.label("User:");
                        ui.text_edit_singleline(&mut conn.user);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(egui::TextEdit::singleline(&mut conn.password).password(true));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Database:");
                        ui.text_edit_singleline(&mut conn.database);
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            save_connection = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_edit = true;
                        }
                    });
                });

            if save_connection {
                if let Some(idx) = self.edit_connection_index {
                    // Update existing
                    self.config.update_connection(idx, conn.clone());
                } else {
                    // Add new
                    self.config.add_connection(conn.clone());
                }
                let _ = self.config.save();
                self.edit_connection = None;
                self.edit_connection_index = None;
            }

            if cancel_edit {
                self.edit_connection = None;
                self.edit_connection_index = None;
            }
        }

        // Left sidebar - Schema/Tables tree
        let mut table_clicked: Option<(String, String)> = None;
        let mut table_right_clicked: Option<(String, String)> = None;
        let mut schema_toggled: Option<String> = None;

        egui::SidePanel::left("database_structure_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(200.0)
            .max_width(600.0)
            .show(ctx, |ui| {
                ui.heading("Database Structure");
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_source("tables_sidebar")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for schema in &self.schemas.clone() {
                            let is_expanded = self.expanded_schemas.contains(&schema.name);

                            // Schema row with expand/collapse arrow
                            ui.horizontal(|ui| {
                                let arrow = if is_expanded { "▼" } else { "▶" };
                                if ui.button(arrow).clicked() {
                                    schema_toggled = Some(schema.name.clone());
                                }
                                ui.label(egui::RichText::new(&schema.name).strong());
                                ui.label(format!("({})", schema.tables.len()));
                            });

                            // Show tables if expanded
                            if is_expanded {
                                ui.indent(&schema.name, |ui| {
                                    for table in &schema.tables {
                                        let is_selected = self.selected_table.as_ref() == Some(&(schema.name.clone(), table.clone()));
                                        let response = ui.selectable_label(is_selected, format!("📊 {}", table));

                                        if response.clicked() {
                                            table_clicked = Some((schema.name.clone(), table.clone()));
                                        }

                                        response.context_menu(|ui| {
                                            if ui.button("View Data").clicked() {
                                                table_right_clicked = Some((schema.name.clone(), table.clone()));
                                                ui.close_menu();
                                            }
                                        });
                                    }
                                });
                            }
                        }
                    });
            });

        // Handle schema toggle
        if let Some(schema_name) = schema_toggled {
            if self.expanded_schemas.contains(&schema_name) {
                self.expanded_schemas.remove(&schema_name);
            } else {
                self.expanded_schemas.insert(schema_name);
            }
            self.save_state();
        }

        // Handle table click
        if let Some((schema, table)) = table_clicked {
            self.selected_table = Some((schema.clone(), table.clone()));
            self.load_table_data(schema, table, None);
        }

        // Handle table right-click (View Data)
        if let Some((schema, table)) = table_right_clicked {
            self.selected_table = Some((schema.clone(), table.clone()));
            self.load_table_data(schema, table, None);
        }

        // Main content area - Tabs and data grid
        egui::CentralPanel::default().show(ctx, |ui| {
                    // Tab bar
                    if !self.tabs.is_empty() {
                        let mut tab_to_activate: Option<usize> = None;
                        let mut tab_to_close: Option<usize> = None;

                        ui.horizontal(|ui| {
                            for (i, tab) in self.tabs.iter().enumerate() {
                                let is_active = i == self.active_tab;
                                let tab_label = egui::RichText::new(&tab.title).strong();

                                if ui.selectable_label(is_active, tab_label).clicked() {
                                    tab_to_activate = Some(i);
                                }

                                if ui.small_button("✖").clicked() {
                                    tab_to_close = Some(i);
                                }

                                ui.separator();
                            }
                        });

                        if let Some(i) = tab_to_activate {
                            self.active_tab = i;
                            self.save_state();
                        }
                        if let Some(i) = tab_to_close {
                            self.close_tab(i);
                        }

                        ui.separator();
                    }

                    // Data grid
                    let column_to_sort = Cell::new(None);
                    let mut page_size_changed = None;
                    let mut page_changed = None;
                    let mut reload_requested = false;

                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        if let Some(data) = &tab.data {
                            let sort_column = tab.sort_column;
                            let sort_ascending = tab.sort_ascending;
                            let current_page = tab.current_page;
                            let page_size = tab.page_size;

                            // Calculate pagination
                            let total_rows = data.rows.len();
                            let total_pages = (total_rows + page_size - 1) / page_size;
                            let start_row = current_page * page_size;
                            let end_row = (start_row + page_size).min(total_rows);

                            // Pagination controls
                            ui.horizontal(|ui| {
                                if ui.button("🔄 Reload").clicked() {
                                    reload_requested = true;
                                }

                                ui.separator();

                                ui.label("Rows per page:");

                                for size in [50, 100, 500, 1000, 5000] {
                                    let is_selected = page_size == size;
                                    if ui.selectable_label(is_selected, format!("{}", size)).clicked() {
                                        page_size_changed = Some(size);
                                    }
                                }

                                ui.separator();

                                if ui.button("◀ Previous").clicked() && current_page > 0 {
                                    page_changed = Some(current_page - 1);
                                }

                                ui.label(format!("Page {} of {} ({}-{} of {} rows)",
                                    current_page + 1,
                                    total_pages.max(1),
                                    start_row + 1,
                                    end_row,
                                    total_rows
                                ));

                                if ui.button("Next ▶").clicked() && current_page + 1 < total_pages {
                                    page_changed = Some(current_page + 1);
                                }
                            });

                            ui.separator();

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
                                            // Only show rows for current page
                                            let page_rows = &data.rows[start_row..end_row];
                                            for (page_row_index, row) in page_rows.iter().enumerate() {
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
                        } else if tab.is_loading {
                            ui.centered_and_justified(|ui| {
                                ui.spinner();
                                ui.label("Loading...");
                            });
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select a table to view data");
                        });
                    }

                    // Handle column sort after the immutable borrow is released
                    if let Some(col_index) = column_to_sort.get() {
                        self.sort_tab_data(self.active_tab, col_index);
                    }

                    // Handle page size changes
                    if let Some(new_size) = page_size_changed {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            tab.page_size = new_size;
                            tab.current_page = 0; // Reset to first page when changing page size
                            self.save_state();
                        }
                    }

                    // Handle page navigation
                    if let Some(new_page) = page_changed {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            tab.current_page = new_page;
                            self.save_state();
                        }
                    }

                    // Handle reload request
                    if reload_requested {
                        self.reload_current_tab();
                    }
        });

        // Request repaint if we're waiting for async operations
        if self.pending_operation.is_some() {
            ctx.request_repaint();
        }
    }
}
