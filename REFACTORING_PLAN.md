# Code Refactoring Plan

## Current State Analysis

### File Structure
- `src/main.rs` (1069 lines) - **TOO LARGE**
- `src/db.rs` (269 lines) - Reasonable size
- `src/config.rs` (114 lines) - Good size

### Problems
1. `main.rs` contains everything: models, UI, state, logic
2. No clear separation of concerns
3. Hard to test individual components
4. Difficult to navigate and maintain
5. UI code mixed with business logic

## Target Structure

```
src/
├── main.rs                    (minimal, ~50 lines)
├── app.rs                     (main app coordinator, ~200 lines)
├── config.rs                  (existing, keep as-is)
├── db/
│   ├── mod.rs                 (re-exports)
│   ├── client.rs              (Database struct, queries)
│   ├── models.rs              (ColumnInfo, SchemaInfo)
│   └── operations.rs          (async operations enum)
├── models/
│   ├── mod.rs                 (re-exports)
│   ├── tab.rs                 (Tab, TabSource, TableData)
│   └── state.rs               (AppState)
├── ui/
│   ├── mod.rs                 (re-exports)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── menu_bar.rs        (top menu)
│   │   ├── status_bar.rs      (bottom status)
│   │   ├── query_panel.rs     (SQL input)
│   │   ├── settings_dialog.rs (connection settings)
│   │   ├── connection_editor.rs (edit connection form)
│   │   ├── database_tree.rs   (left sidebar)
│   │   ├── tab_bar.rs         (tab management)
│   │   ├── data_grid.rs       (table view)
│   │   └── pagination.rs      (pagination controls)
│   └── styles.rs              (font configs, colors)
└── utils/
    └── async_helpers.rs       (Promise wrappers if needed)
```

## Refactoring Steps (Order matters!)

### Phase 1: Extract Models (Safe, no UI changes)

**Step 1.1: Create models module structure**
- Create `src/models/mod.rs`
- Create `src/models/tab.rs`
- Create `src/models/state.rs`

**Step 1.2: Move Tab-related structs**
- Move `Tab` struct to `models/tab.rs`
- Move `TabSource` enum to `models/tab.rs`
- Move `TableData` struct to `models/tab.rs`
- Update imports in `main.rs`
- **TEST**: Build and run, verify nothing breaks

**Step 1.3: Move AppState**
- Move `AppState` struct to `models/state.rs`
- Keep implementation with the struct
- Update imports in `main.rs`
- **TEST**: Build and run, verify state saves/loads correctly

### Phase 2: Extract Database Models

**Step 2.1: Create db module structure**
- Create `src/db/mod.rs`
- Create `src/db/models.rs`
- Create `src/db/client.rs`
- Create `src/db/operations.rs`

**Step 2.2: Move database models**
- Move `ColumnInfo` to `db/models.rs`
- Move `SchemaInfo` to `db/models.rs`
- Update re-exports in `db/mod.rs`
- **TEST**: Build and run

**Step 2.3: Move Database implementation**
- Move `Database` struct to `db/client.rs`
- Move all query methods
- Move helper functions (like `row_value_to_string`)
- Update re-exports
- Delete old `src/db.rs`
- **TEST**: Build and run, test database queries

**Step 2.4: Move AsyncOperation**
- Move `AsyncOperation` enum to `db/operations.rs`
- Update imports in `main.rs`
- **TEST**: Build and run

### Phase 3: Extract Application Logic

**Step 3.1: Create app.rs**
- Create `src/app.rs`
- Move `DbClientApp` struct definition (fields only)
- Keep methods in main.rs temporarily

**Step 3.2: Move application methods (in groups)**

Group A - State management:
- Move `save_state()`
- Move `new()`
- **TEST**: Build and run

Group B - Data loading:
- Move `connect_to_database()`
- Move `load_table_data()`
- Move `execute_query()`
- Move `reload_current_tab()`
- **TEST**: Build and run, test loading tables

Group C - Tab management:
- Move `add_tab()`
- Move `close_tab()`
- Move `sort_tab_data()`
- **TEST**: Build and run, test tab operations

### Phase 4: Extract UI Components

**Step 4.1: Create UI module structure**
- Create `src/ui/mod.rs`
- Create `src/ui/components/mod.rs`
- Create `src/ui/styles.rs`

**Step 4.2: Extract styles**
- Move font configuration code to `styles.rs`
- Create `setup_styles(ctx: &egui::Context)` function
- Call from `app.rs::new()`
- **TEST**: Build and run, verify fonts look correct

**Step 4.3: Extract menu bar**
- Create `src/ui/components/menu_bar.rs`
- Create struct `MenuBar` with method `show()`
- Extract menu bar rendering code
- Returns events (ShowSettings, Quit, ToggleQuery, Refresh)
- Call from main `update()` loop
- **TEST**: Build and run, test menu clicks

**Step 4.4: Extract status bar**
- Create `src/ui/components/status_bar.rs`
- Create struct `StatusBar` with method `show()`
- Extract status bar rendering
- Takes status message and row count as params
- **TEST**: Build and run

**Step 4.5: Extract query panel**
- Create `src/ui/components/query_panel.rs`
- Create struct `QueryPanel` with method `show()`
- Extract query panel rendering
- Returns events (Execute, Clear, Close)
- Manages query_input as mutable reference
- **TEST**: Build and run, test query execution

**Step 4.6: Extract settings dialog**
- Create `src/ui/components/settings_dialog.rs`
- Create struct `SettingsDialog` with method `show()`
- Extract settings window rendering
- Returns events (Connect, Edit, Delete, NewConnection, Close)
- **TEST**: Build and run, test settings dialog

**Step 4.7: Extract connection editor**
- Create `src/ui/components/connection_editor.rs`
- Create struct `ConnectionEditor` with method `show()`
- Extract connection edit window
- Returns events (Save, Cancel)
- **TEST**: Build and run, test connection editing

**Step 4.8: Extract database tree**
- Create `src/ui/components/database_tree.rs`
- Create struct `DatabaseTree` with method `show()`
- Extract sidebar rendering
- Returns events (TableClicked, TableRightClicked, SchemaToggled)
- **TEST**: Build and run, test tree navigation

**Step 4.9: Extract tab bar**
- Create `src/ui/components/tab_bar.rs`
- Create struct `TabBar` with method `show()`
- Extract tab rendering
- Returns events (TabActivated, TabClosed)
- **TEST**: Build and run, test tab switching

**Step 4.10: Extract pagination**
- Create `src/ui/components/pagination.rs`
- Create struct `PaginationControls` with method `show()`
- Extract pagination controls
- Returns events (Reload, PageSizeChanged, PageChanged)
- **TEST**: Build and run, test pagination

**Step 4.11: Extract data grid**
- Create `src/ui/components/data_grid.rs`
- Create struct `DataGrid` with method `show()`
- Extract table rendering (largest component)
- Returns events (ColumnSorted, RowSelected)
- **TEST**: Build and run, test data viewing and sorting

### Phase 5: Cleanup and Polish

**Step 5.1: Clean up main.rs**
- Should only contain:
  - main() function
  - Minimal eframe setup
  - Import statements
- Move everything else to appropriate modules

**Step 5.2: Review and optimize imports**
- Use `pub use` in mod.rs files for clean API
- Remove unused imports
- Group imports logically

**Step 5.3: Add module documentation**
- Add doc comments to each module
- Document public interfaces
- Add examples where helpful

**Step 5.4: Final testing**
- Full integration test
- Test all features:
  - Connect to database
  - Browse schemas/tables
  - Open multiple tabs
  - Sort columns
  - Execute queries
  - Reload data
  - Close and reopen app (state persistence)
  - Multiple connections

## Event-Driven Architecture Pattern

To decouple UI from logic, use an event system:

```rust
// In each UI component
pub enum MenuBarEvent {
    ShowSettings,
    Quit,
    ToggleQuery,
    Refresh,
}

// In component
impl MenuBar {
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<MenuBarEvent> {
        // Render UI
        // Return event if user clicked something
    }
}

// In main update loop
if let Some(event) = menu_bar.show(ui) {
    match event {
        MenuBarEvent::ShowSettings => app.show_settings = true,
        // ... handle other events
    }
}
```

## Testing Strategy

After each step:
1. `cargo build` - Must compile without errors
2. `cargo run` - Must start successfully
3. Manual testing of affected features
4. Check state persistence still works

## Rollback Plan

- Each step is in git commit
- If something breaks, revert the last commit
- Fix the issue before proceeding

## Benefits After Refactoring

1. **Maintainability**: Each file has single responsibility
2. **Testability**: Components can be tested in isolation
3. **Readability**: Easy to find where code lives
4. **Scalability**: Easy to add new features
5. **Team Work**: Multiple people can work on different components
6. **Performance**: Can optimize individual components

## Estimated Time

- Phase 1: 30 minutes
- Phase 2: 45 minutes
- Phase 3: 1 hour
- Phase 4: 3-4 hours (largest phase)
- Phase 5: 30 minutes

**Total: ~6-7 hours** of careful, methodical refactoring

## Notes

- Do NOT skip testing steps
- Do NOT combine multiple steps
- Commit after each successful step
- Keep a backup of working code
- If you get stuck, ask for help rather than forcing it
