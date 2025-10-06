# PostgreSQL Database Client (TUI)

A terminal-based PostgreSQL database client built with Rust, featuring a modern TUI interface similar to GUI database clients.

## Features

- 📊 Browse database tables in a sidebar
- 🔍 View table data in a grid layout
- ⌨️ Execute custom SQL queries
- 🎨 Vim-style keyboard navigation
- ⚡ Fast and lightweight

## Setup

### Prerequisites

- Rust (1.70 or later)
- PostgreSQL database access

### Installation

```bash
cargo build --release
```

## Usage

### Setting Database Connection

Set the `DATABASE_URL` environment variable:

```bash
export DATABASE_URL="host=localhost user=postgres password=yourpassword dbname=yourdb"
```

Or let it use the default connection string (localhost, user=postgres, password=postgres, dbname=postgres).

### Running the Application

```bash
cargo run
```

Or run the compiled binary:

```bash
./target/release/db-client
```

## Keyboard Controls

### Navigation

- `Tab` - Switch between sidebar and data grid
- `↑/↓` or `k/j` - Navigate up/down in lists
- `Enter` - Load selected table data

### Query Mode

- `i` - Enter query mode
- Type your SQL query
- `Enter` - Execute query
- `Esc` - Cancel and return to normal mode

### General

- `q` - Quit application

## Layout

```
┌─────────────────────────────────────────────────────────┐
│ SQL Query Input                                          │
├────────────┬────────────────────────────────────────────┤
│ Tables     │ Data Grid                                   │
│            │                                             │
│ 📊 users   │ id │ name     │ email                      │
│ 📊 orders  │ 1  │ John Doe │ john@example.com           │
│ 📊 products│ 2  │ Jane     │ jane@example.com           │
│            │                                             │
├────────────┴────────────────────────────────────────────┤
│ Status                                                   │
└─────────────────────────────────────────────────────────┘
```

## Example Queries

```sql
SELECT * FROM users WHERE email LIKE '%@example.com';
SELECT COUNT(*) FROM orders;
SELECT * FROM products ORDER BY price DESC LIMIT 10;
```

## Architecture

- `src/db.rs` - Database connection and query execution
- `src/main.rs` - TUI application and event handling

## Dependencies

- `tokio-postgres` - Async PostgreSQL client
- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal manipulation
- `anyhow` - Error handling

## License

MIT
