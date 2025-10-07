# db-client

Simple PostgreSQL client I threw together in Rust. Uses egui for the GUI.

## Features

### Database Navigation
- **Tree view** of schemas and tables in the left sidebar
- **Collapsible schemas** - expand/collapse to show/hide tables
- **Search bar** in database tree to filter tables
- **Context menu** on tables for quick actions
- **Connection management** - save and switch between multiple database connections
- **Auto-reconnect** - remembers your last connection on startup

### Data Viewing
- **Multi-tab interface** - open multiple tables and query results simultaneously
- **Persistent tabs** - tabs restore between sessions (saved to ~/.config/db-client/state.json)
- **Column sorting** - click headers to sort ascending/descending
- **Pagination** - configurable page size (50, 100, 200, 500 rows per page)
- **Column metadata**:
  - 🔑 Primary key indicator
  - 🔗 Foreign key indicator
  - Data type display in column headers
- **Row selection** - click to select, visual highlighting
- **Copy cell values** - right-click context menu

### Search & Filter
- **Quick search** - search across all columns in the current table
- **Real-time highlighting** - all matches highlighted in yellow
- **Match navigation** - use ◀ ▶ arrows to jump between search results
- **Match counter** - shows "Match X of Y"
- **Current match emphasis** - active match highlighted in orange
- **Auto-scroll** - automatically scrolls to bring matches into view
- **Case-insensitive** - searches ignore case by default
- **Advanced filtering** - filter bar with multiple conditions (AND/OR logic)
- **Per-column filters** - filter by specific columns with operators (equals, contains, greater than, etc.)

### Query Execution
- **SQL query panel** - write and execute custom SQL queries
- **Keyboard shortcut** - Cmd+Enter (Mac) / Ctrl+Enter (Windows/Linux) to execute
- **Query results in tabs** - results open in new tabs just like tables
- **Per-tab queries** - each tab has its own query, switch between tabs to work on different queries
- **Query display** - SQL query is shown above results with copy and edit buttons
- **Query persistence** - can reload/refresh query results
- **Save queries** - save frequently used queries with names
- **Load queries** - quickly load saved queries into the editor
- **Query library** - manage your saved queries (view, load, delete)
- **Timestamps** - saved queries include creation timestamps

### Session Management
- **State persistence** - remembers:
  - Open tabs and their content
  - Active tab
  - Expanded schemas
  - Column sort preferences
  - Page position
  - Filters and search text
- **Configuration storage** - database connections saved securely
- **Reload functionality** - refresh button to reload current tab data

## Building

You need Rust installed.

```bash
cargo build --release
```

## Running

```bash
cargo run
```

Or just run the binary after building:

```bash
./target/release/db-client
```

## Configuration

First time you run it, go to **File → Settings** and add your database connections. It'll remember which schemas you had expanded and what tabs were open.

**Default connection**: If you don't set anything up, it tries to connect to:
- Host: localhost
- User: postgres
- Password: postgres
- Database: postgres

You can also set the `DATABASE_URL` environment variable.

**Config files:**
- Connection configs: `~/.config/db-client/config.json`
- Session state: `~/.config/db-client/state.json`
- Saved queries: `~/.config/db-client/queries.json`

## Quick Start

1. Launch the application
2. Go to **File → Settings** to add your database connection
3. Click **Connect** to connect to your database
4. Browse tables in the left sidebar
5. Click a table to open it in a tab
6. Use the search bar above the grid to find data
7. Use **◀ ▶** arrows to navigate between search matches
8. Open **View → Show Query Panel** to execute custom SQL
9. Click **💾 Save** to save frequently used queries
10. Click **📂 Load** to access your saved queries

## Usage Tips

### SQL Query Editor
- Write SQL queries in the query panel (**View → Show Query Panel**)
- Execute with **Cmd/Ctrl + Enter** or click **Execute**
- Each tab has its own query - switch between tabs to work on different queries
- The executed SQL query is displayed above the results
- Click **📋 Copy** to copy the query to clipboard
- Click **✏ Edit** to open it in the query panel
- Save queries you use often with the **💾 Save** button
- Load saved queries with the **📂 Load** button

### Search & Navigation
- Type in the search box to highlight all matches
- Use **◀ ▶** arrows to jump between matches
- Match counter shows your position (e.g., "Match 3 of 12")
- View automatically scrolls to show the current match

### Keyboard Shortcuts
- **Cmd/Ctrl + Enter** - Execute query in query panel
- Click column headers to sort
- Right-click cells to copy values
- Right-click tables for context menu

## Technology Stack

- **eframe/egui** - Cross-platform GUI framework
- **tokio-postgres** - Async PostgreSQL driver
- **serde** - Serialization for state persistence
- **egui_extras** - Table components for data grid
- **poll-promise** - Async task management

## Testing

Run the test suite:

```bash
cargo test
```

The project includes comprehensive unit tests for:
- Search match counting and navigation
- Page and row position calculations
- Match navigation with edge cases
- Filter and search state management

## License

MIT probably? Do whatever you want with it.
