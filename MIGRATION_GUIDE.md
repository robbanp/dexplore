# Step-by-Step Migration Guide

Practical examples for each refactoring step.

## Quick Reference Commands

```bash
# After each step, run:
cargo build          # Must succeed
cargo run            # Test manually
git add -A
git commit -m "Step X.Y: Description"
```

## Phase 1 Examples

### Step 1.1: Create models module

```bash
mkdir -p src/models
touch src/models/mod.rs
touch src/models/tab.rs
touch src/models/state.rs
```

**src/models/mod.rs**
```rust
mod tab;
mod state;

pub use tab::{Tab, TabSource, TableData};
pub use state::AppState;
```

### Step 1.2: Move Tab structs

**Before (main.rs):**
```rust
mod config;
mod db;

use anyhow::Result;
// ... imports

#[derive(Clone, Serialize, Deserialize)]
struct TableData {
    name: String,
    columns: Vec<ColumnInfo>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Tab {
    // ... fields
}

#[derive(Clone, Serialize, Deserialize)]
enum TabSource {
    // ... variants
}
```

**After (main.rs):**
```rust
mod config;
mod db;
mod models;  // NEW

use anyhow::Result;
use models::{Tab, TabSource, TableData};  // NEW
// ... other imports
```

**NEW (src/models/tab.rs):**
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

**Test:**
```bash
cargo build
cargo run
# Open a table, verify it works
# Close and reopen, verify tabs persist
```

### Step 1.3: Move AppState

**NEW (src/models/state.rs):**
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
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
    pub fn save_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".config").join("db-client").join("state.json"))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::save_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Self::save_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let state: AppState = serde_json::from_str(&content)?;
            Ok(state)
        } else {
            Err(anyhow::anyhow!("State file does not exist"))
        }
    }
}
```

**Update main.rs:**
```rust
use models::{AppState, Tab, TabSource, TableData};
```

Remove the AppState definition from main.rs.

**Test:**
```bash
cargo build
cargo run
# Create some tabs
# Close app
# Reopen - verify tabs are restored
```

## Phase 2 Examples

### Step 2.1-2.3: Database refactoring

```bash
mkdir -p src/db
touch src/db/mod.rs
touch src/db/models.rs
touch src/db/client.rs
touch src/db/operations.rs
```

**src/db/mod.rs**
```rust
mod models;
mod client;
mod operations;

pub use models::{ColumnInfo, SchemaInfo};
pub use client::Database;
pub use operations::AsyncOperation;
```

**src/db/models.rs**
- Copy ColumnInfo and SchemaInfo from old db.rs
- Make fields pub

**src/db/client.rs**
- Copy Database struct and impl from old db.rs
- Copy helper functions like row_value_to_string
- Update imports

**src/db/operations.rs**
- Cut AsyncOperation enum from main.rs
- Paste here
- Update imports

**Update main.rs:**
```rust
mod config;
mod db;  // Now points to db/mod.rs
mod models;

use db::{AsyncOperation, Database, SchemaInfo};
```

Delete old `src/db.rs` file.

**Test:**
```bash
cargo build
cargo run
# Connect to database
# Browse schemas
# Open tables
```

## Phase 3 Examples

### Step 3.1: Create app.rs skeleton

**src/app.rs**
```rust
use std::sync::Arc;
use std::collections::HashSet;
use eframe::egui;
use crate::config::{Config, DatabaseConnection};
use crate::db::{Database, SchemaInfo, AsyncOperation};
use crate::models::{Tab, TabSource, TableData, AppState};

pub struct DbClientApp {
    // Copy ALL fields from main.rs
    pub config: Config,
    pub connection_string: String,
    // ... etc
}

// Methods will be moved here gradually
impl DbClientApp {
}

impl eframe::App for DbClientApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Will be implemented later
        todo!()
    }
}
```

**Update main.rs:**
```rust
mod app;
mod config;
mod db;
mod models;

use app::DbClientApp;

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
```

### Step 3.2: Move methods gradually

**Move in groups, test after each group:**

1. First: `save_state()`, `new()`
2. Then: `connect_to_database()`, `load_table_data()`, `execute_query()`
3. Then: `add_tab()`, `close_tab()`, `sort_tab_data()`, `reload_current_tab()`
4. Finally: `update()` method

**Example - Moving new():**

Cut from main.rs, paste into app.rs:

```rust
impl DbClientApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up styles
        let mut style = (*cc.egui_ctx.style()).clone();
        // ... all the style setup code
        cc.egui_ctx.set_style(style);

        // ... rest of initialization
    }
}
```

## Phase 4 Examples

### Step 4.2: Extract styles (simplest UI component)

**src/ui/styles.rs**
```rust
use eframe::egui;

pub fn setup_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

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

    ctx.set_style(style);
}
```

**Update app.rs new():**
```rust
use crate::ui::styles;

impl DbClientApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        styles::setup_styles(&cc.egui_ctx);

        // ... rest of code
    }
}
```

### Step 4.3: Extract menu bar (event pattern)

**src/ui/components/menu_bar.rs**
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
    pub fn new(connection_status: String) -> Self {
        Self { connection_status }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<MenuBarEvent> {
        let mut event = None;

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Settings...").clicked() {
                    event = Some(MenuBarEvent::ShowSettings);
                    ui.close_menu();
                }
                if ui.button("Quit").clicked() {
                    event = Some(MenuBarEvent::Quit);
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Show Query Panel").clicked() {
                    event = Some(MenuBarEvent::ToggleQuery);
                }
            });

            ui.separator();

            if ui.button("🔄 Refresh").clicked() {
                event = Some(MenuBarEvent::Refresh);
            }

            if ui.button("📝 Query").clicked() {
                event = Some(MenuBarEvent::ToggleQuery);
            }

            ui.separator();
            ui.label(&self.connection_status);
        });

        event
    }

    pub fn update_connection_status(&mut self, status: String) {
        self.connection_status = status;
    }
}
```

**Update app.rs update():**
```rust
use crate::ui::components::{MenuBar, MenuBarEvent};

pub struct DbClientApp {
    // Add new field
    menu_bar: MenuBar,
    // ... other fields
}

impl DbClientApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // ...
        let menu_bar = MenuBar::new("Not connected".to_string());

        Self {
            menu_bar,
            // ... other fields
        }
    }
}

impl eframe::App for DbClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if let Some(event) = self.menu_bar.show(ui) {
                match event {
                    MenuBarEvent::ShowSettings => {
                        self.show_settings = true;
                    }
                    MenuBarEvent::Quit => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    MenuBarEvent::ToggleQuery => {
                        self.show_query_panel = !self.show_query_panel;
                    }
                    MenuBarEvent::Refresh => {
                        self.connect_to_database();
                    }
                }
            }
        });

        // ... rest of UI
    }
}
```

## Common Patterns

### Pattern 1: Simple UI Component (no state)

```rust
// Component file
pub struct MyComponent;

impl MyComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&self, ui: &mut egui::Ui, data: &SomeData) {
        // Render using data
        ui.label(&data.text);
    }
}
```

### Pattern 2: UI Component with Events

```rust
// Component file
pub enum MyComponentEvent {
    ButtonClicked,
    ValueChanged(String),
}

pub struct MyComponent;

impl MyComponent {
    pub fn show(&self, ui: &mut egui::Ui, data: &SomeData) -> Option<MyComponentEvent> {
        let mut event = None;

        if ui.button("Click me").clicked() {
            event = Some(MyComponentEvent::ButtonClicked);
        }

        event
    }
}
```

### Pattern 3: UI Component with Mutable Data

```rust
pub struct MyComponent;

impl MyComponent {
    pub fn show(&self, ui: &mut egui::Ui, data: &mut String) -> bool {
        let changed = ui.text_edit_singleline(data).changed();
        changed
    }
}
```

## Testing Checklist

After EACH step:

- [ ] `cargo build` succeeds
- [ ] `cargo run` starts the app
- [ ] Can connect to database
- [ ] Can browse schemas
- [ ] Can open tables
- [ ] Can sort columns
- [ ] Can execute queries
- [ ] Can reload data
- [ ] Can close and reopen (state persists)
- [ ] All tabs work correctly

## Troubleshooting

### Problem: Borrow checker errors

**Solution:** Use events to decouple. Don't call methods that need `&mut self` from inside UI closures that borrow `&self`.

```rust
// BAD
ui.button("Click").clicked() {
    self.do_something_mut();  // ERROR: already borrowed
}

// GOOD
let mut clicked = false;
if ui.button("Click").clicked() {
    clicked = true;
}
// After UI rendering:
if clicked {
    self.do_something_mut();  // OK
}
```

### Problem: Too many imports

**Solution:** Use module re-exports

```rust
// In mod.rs
pub use menu_bar::{MenuBar, MenuBarEvent};
pub use status_bar::StatusBar;
// etc.

// In app.rs
use crate::ui::components::{MenuBar, MenuBarEvent, StatusBar};
```

### Problem: Circular dependencies

**Solution:** Move shared types to models/, make them pub

## Final Structure Check

After all phases, you should have:

```
src/
├── main.rs           (~50 lines)
├── app.rs            (~300 lines)
├── config.rs         (unchanged)
├── db/
│   ├── mod.rs
│   ├── client.rs     (~200 lines)
│   ├── models.rs     (~30 lines)
│   └── operations.rs (~20 lines)
├── models/
│   ├── mod.rs
│   ├── tab.rs        (~50 lines)
│   └── state.rs      (~50 lines)
└── ui/
    ├── mod.rs
    ├── styles.rs     (~30 lines)
    └── components/
        ├── mod.rs
        ├── menu_bar.rs         (~80 lines)
        ├── status_bar.rs       (~40 lines)
        ├── query_panel.rs      (~60 lines)
        ├── settings_dialog.rs  (~100 lines)
        ├── connection_editor.rs (~80 lines)
        ├── database_tree.rs    (~100 lines)
        ├── tab_bar.rs          (~60 lines)
        ├── pagination.rs       (~60 lines)
        └── data_grid.rs        (~200 lines)
```

**Total: ~1,500 lines across 20+ files instead of 1,069 lines in 1 file!**
