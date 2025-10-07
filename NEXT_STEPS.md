# Next Steps for Refactoring

## Quick Summary

✅ **Completed:** Phases 1-2 (Models & Database layer)
⏸️ **Paused at:** Phase 3 (Application logic extraction)
📦 **Code status:** Compiles ✓, All features work ✓

## What to Do Next

### Immediate Next Step: Phase 3 (1 hour)

**Goal:** Move `DbClientApp` to `src/app.rs`

This will reduce `main.rs` from 997 lines to ~50 lines!

#### Quick Steps:

```bash
# 1. Create app.rs
touch src/app.rs

# 2. Add to app.rs header:
cat > src/app.rs << 'EOF'
use std::sync::Arc;
use std::collections::HashSet;
use std::cell::Cell;
use eframe::egui;
use poll_promise::Promise;
use crate::config::{Config, DatabaseConnection};
use crate::db::{AsyncOperation, Database, SchemaInfo};
use crate::models::{AppState, Tab, TabSource, TableData};

pub struct DbClientApp {
    // Copy struct definition from main.rs lines 30-65
}

impl DbClientApp {
    // Move methods from main.rs in groups
}

impl eframe::App for DbClientApp {
    // Move the entire impl block from main.rs
}
EOF

# 3. Update main.rs to import
# Replace line 1 with:
# mod app;
#
# Replace line 7 with:
# use app::DbClientApp;
#
# Delete lines 30-963 (everything between main() and end of file)

# 4. Build and test
cargo build
cargo run
```

#### Detailed Instructions

See `MIGRATION_GUIDE.md` Phase 3 section for:
- Complete code examples
- Import management
- Step-by-step method migration
- Testing checklist

### After Phase 3: UI Components (3-4 hours)

Extract UI into components following event-driven pattern.

**Start with easy ones:**
1. `ui/styles.rs` - Font setup
2. `ui/components/status_bar.rs` - Just displays status
3. `ui/components/menu_bar.rs` - Returns MenuBarEvent

**Then medium complexity:**
4. `ui/components/query_panel.rs`
5. `ui/components/tab_bar.rs`
6. `ui/components/pagination.rs`

**Finally complex:**
7. `ui/components/database_tree.rs`
8. `ui/components/settings_dialog.rs`
9. `ui/components/data_grid.rs` (the biggest!)

Each component:
- Takes `&self` references for data
- Returns `Option<SomeEvent>` for actions
- App handles events in main update loop

See `MODULE_SPEC.md` for exact interfaces.

## Files to Reference

### Planning Documents
- **REFACTORING_PLAN.md** - Overall strategy, all 5 phases
- **REFACTORING_STATUS.md** - Current progress, what's left
- **MODULE_SPEC.md** - Interface specs for each module
- **MIGRATION_GUIDE.md** - Step-by-step code examples

### Current Code Structure
```
src/
├── main.rs (997 lines) ← Move most of this to app.rs
├── config.rs ✓
├── models/ ✓
│   ├── mod.rs
│   ├── state.rs
│   └── tab.rs
└── db/ ✓
    ├── mod.rs
    ├── client.rs
    ├── models.rs
    └── operations.rs
```

### Target Structure (After Complete Refactoring)
```
src/
├── main.rs (~50 lines)
├── app.rs (~300 lines)
├── config.rs
├── models/
│   ├── mod.rs
│   ├── state.rs
│   └── tab.rs
├── db/
│   ├── mod.rs
│   ├── client.rs
│   ├── models.rs
│   └── operations.rs
└── ui/
    ├── mod.rs
    ├── styles.rs
    └── components/
        ├── mod.rs
        ├── menu_bar.rs
        ├── status_bar.rs
        ├── query_panel.rs
        ├── settings_dialog.rs
        ├── connection_editor.rs
        ├── database_tree.rs
        ├── tab_bar.rs
        ├── pagination.rs
        └── data_grid.rs
```

## Common Pitfalls

### 1. Borrow Checker Issues
**Problem:** Can't call `&mut self` methods from inside UI closures

**Solution:** Use event pattern
```rust
// BAD
ui.button("Click").clicked() {
    self.do_something(); // ERROR
}

// GOOD
let mut clicked = false;
if ui.button("Click").clicked() {
    clicked = true;
}
// After UI closure:
if clicked {
    self.do_something(); // OK
}
```

### 2. Import Management
After moving code, you'll need to add imports. Common ones:
```rust
use std::sync::Arc;
use std::collections::HashSet;
use eframe::egui;
use crate::models::*;
use crate::db::*;
```

### 3. Module Visibility
Make sure to use `pub` on items that need to be exported:
```rust
pub struct MyComponent;  // Visible outside module
struct Internal;         // Only visible inside module
```

## Testing After Each Step

```bash
# Always do this after moving code:
cargo build          # Must succeed
cargo run            # Must start
# Then manually test:
# - Connect to DB
# - Browse tables
# - Open tabs
# - Execute queries
# - Reload data
# - Close and reopen (state persists)
```

If anything breaks, `git log` shows recent commits, `git revert` can undo.

## Estimated Time Remaining

- **Phase 3:** 1 hour (move app logic to app.rs)
- **Phase 4:** 3-4 hours (extract UI components)
- **Phase 5:** 30 minutes (polish and cleanup)

**Total:** ~5-6 hours of focused work

Can be done incrementally over multiple sessions!

## Questions?

Read the docs:
1. Start with `REFACTORING_STATUS.md` - understand current state
2. Check `REFACTORING_PLAN.md` - see the big picture
3. Use `MIGRATION_GUIDE.md` - follow step-by-step examples
4. Reference `MODULE_SPEC.md` - see exact interfaces needed

## Ready to Continue?

```bash
# Verify current state works
cargo build && cargo run

# Start Phase 3
# Read MIGRATION_GUIDE.md Phase 3 section
# Create src/app.rs
# Move code in small chunks
# Build and test after each chunk
```

Good luck! The hard part (planning) is done. Now it's just systematic execution.
