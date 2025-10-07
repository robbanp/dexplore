# db-client

Simple PostgreSQL client I threw together in Rust. Uses egui for the GUI.

## What it does

Browse your postgres databases, run queries, view tables. Nothing fancy but gets the job done.

- Tree view of schemas and tables on the left
- Click a table to open it in a new tab
- Tabs stay open between sessions (saves to ~/.config/db-client/state.json)
- Sort columns by clicking headers
- Reload data with the refresh button
- Execute custom SQL queries
- Multiple connection configs

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

First time you run it, go to File → Settings and add your database connections. It'll remember which schemas you had expanded and what tabs were open.

If you don't set anything up, it tries to connect to localhost with user=postgres, password=postgres, dbname=postgres. You can also set DATABASE_URL env var.

Connection configs saved to `~/.config/db-client/config.json`

## UI stuff

- Column headers show data types in smaller gray text
- 🔑 icon = primary key
- 🔗 icon = foreign key
- Click column headers to sort
- Page through large tables with the pagination controls
- Right-click tables for context menu

## Query panel

Click "Query" button or go to View → Show Query Panel. Type your SQL and hit Execute (or Cmd+Enter). Results open in a new tab.

## Keys used

- eframe/egui for the GUI
- tokio-postgres for database stuff
- serde for saving state

## License

MIT probably? Do whatever you want with it.
