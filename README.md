<div align="center">
  <img src="https://raw.githubusercontent.com/debba/tabularis/main/public/logo-sm.png" width="120" height="120" />
</div>

# tabularis-google-sheets-plugin

<p align="center">

![](https://img.shields.io/github/release/debba/tabularis-google-sheets-plugin.svg?style=flat)
![](https://img.shields.io/github/downloads/debba/tabularis-google-sheets-plugin/total.svg?style=flat)
![Build & Release](https://github.com/debba/tabularis-google-sheets-plugin/workflows/Release/badge.svg)
[![Discord](https://img.shields.io/discord/1470772941296894128?color=5865F2&logo=discord&logoColor=white)](https://discord.gg/YrZPHAwMSG)

</p>

A Google Sheets plugin for [Tabularis](https://github.com/debba/tabularis), the lightweight database management tool.

This plugin turns **any Google spreadsheet into a queryable database**. Each tab becomes a table, the first row defines the column headers, and Tabularis lets you browse and edit the cells from the grid. Authentication is handled via OAuth2 — sign in once from the settings panel and your token gets refreshed automatically.

**No external dependencies** — ships as a single compiled Rust binary that talks directly to the Sheets REST API.

**Discord** - [Join our discord server](https://discord.gg/YrZPHAwMSG) and chat with the maintainers.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Automatic (via Tabularis)](#automatic-via-tabularis)
  - [Manual Installation](#manual-installation)
- [How It Works](#how-it-works)
- [Authentication](#authentication)
- [SQL Support](#sql-support)
- [Example Queries](#example-queries)
- [Supported Operations](#supported-operations)
- [Limitations](#limitations)
- [Development](#development)
- [Changelog](#changelog)
- [License](#license)

## Features

- **OAuth2 sign-in** — one-click "Connect with Google" button inside the plugin settings; refresh tokens are stored and rotated for you.
- **Any spreadsheet** — paste the spreadsheet URL (or the raw ID) into the connection form and you're in.
- **Tabs as tables** — every sheet tab shows up in the sidebar explorer as a standalone table.
- **Type inference** — column types (`TEXT`, `INTEGER`, `REAL`) are guessed from the first 100 rows of each tab.
- **Read and write** — `SELECT`, `INSERT`, `UPDATE`, `DELETE` all map to real Sheets API calls. Row edits from the grid work out of the box.
- **Synthetic `_row` primary key** — every row exposes its 1-based sheet row number as an integer PK, so Tabularis can edit and delete individual records.
- **Schema Inspection** — browse tables and columns in the sidebar explorer.
- **Cross-platform** — prebuilt binaries for Linux (x64/arm64), macOS (x64/arm64), and Windows (x64).

## Installation

### Automatic (via Tabularis)

Open **Settings → Available Plugins** in Tabularis and install **Google Sheets** from the plugin registry.

### Manual Installation

1. Download the latest release archive for your platform from the [Releases page](https://github.com/TabularisDB/tabularis-google-sheets-plugin/releases).
2. Extract the archive.
3. Copy `google-sheets-plugin` (or `google-sheets-plugin.exe` on Windows), `manifest.json`, and the `ui/dist/` folder into the Tabularis plugins directory:

| OS | Plugins Directory |
|---|---|
| **Linux** | `~/.local/share/tabularis/plugins/google-sheets/` |
| **macOS** | `~/Library/Application Support/com.debba.tabularis/plugins/google-sheets/` |
| **Windows** | `%APPDATA%\com.debba.tabularis\plugins\google-sheets\` |

4. Make the binary executable (Linux/macOS):

```bash
chmod +x ~/.local/share/tabularis/plugins/google-sheets/google-sheets-plugin
```

5. Restart Tabularis.

## How It Works

The plugin is a compiled Rust binary that communicates with Tabularis through **JSON-RPC 2.0 over stdio**:

1. Tabularis spawns `google-sheets-plugin` as a child process.
2. Requests are sent as newline-delimited JSON-RPC messages to the plugin's `stdin`.
3. Responses are written to `stdout` in the same format.

On each call that needs fresh data (`get_tables`, `get_columns`, `execute_query`, row edits…) the plugin hits the Google Sheets REST API directly using the stored access token. If the token has expired, it's refreshed in-place using the saved refresh token.

There is no local cache: data is read live from the spreadsheet. This keeps the plugin simple and always in sync, at the cost of one HTTP round-trip per operation.

All debug output is written to `stderr` and appears in Tabularis's log viewer — `stdout` is reserved exclusively for JSON-RPC responses.

## Authentication

The first time you open the plugin settings you'll see a **Connect with Google** button (provided by the bundled UI extension). Clicking it launches the standard Google OAuth consent flow and, on success, stores the client id, client secret, access token and refresh token in the plugin settings.

From that point on:

- `access_token`, `refresh_token` and `token_expiry` are **auto-managed** — don't edit them by hand.
- When a request hits a 401 or the token is about to expire, the plugin silently calls the refresh endpoint and retries.
- To revoke access, sign out from the plugin settings or remove the app at <https://myaccount.google.com/permissions>.

The scope requested is `https://www.googleapis.com/auth/spreadsheets` (read + write access to spreadsheets you open).

## SQL Support

The plugin implements a **tiny SQL subset** parsed in-process and flattened into Sheets REST calls. It is intentionally narrow — Sheets is not a relational engine, and translating arbitrary SQL to range operations would be lying to the user.

Supported:

- `SELECT <cols | *> FROM <tab> [WHERE ...] [LIMIT n [OFFSET m]]`
- `SELECT COUNT(*) FROM <tab> [WHERE ...]`
- `INSERT INTO <tab> (cols...) VALUES (...)` — appends a row
- `UPDATE <tab> SET col = val, ... WHERE _row = N` — targets a specific sheet row
- `DELETE FROM <tab> WHERE _row = N`

`WHERE` clauses are evaluated in Rust against the fetched rows, so any column can be referenced but only basic comparison operators are supported. `UPDATE` and `DELETE` require a `WHERE _row = <n>` predicate because Sheets addresses rows by position.

Not supported: `JOIN`, subqueries, aggregates other than `COUNT(*)`, window functions, CTEs, `GROUP BY`, `ORDER BY`. `EXPLAIN` returns an explicit error.

## Example Queries

```sql
-- First 100 rows of a tab
SELECT * FROM Sales LIMIT 100;
```

```sql
-- Count rows matching a predicate
SELECT COUNT(*) FROM Sales WHERE region = 'EU';
```

```sql
-- Project a few columns with pagination
SELECT _row, customer, total
FROM Orders
WHERE status = 'paid'
LIMIT 50 OFFSET 100;
```

```sql
-- Append a new row
INSERT INTO Contacts (name, email, tag)
VALUES ('Jane Doe', 'jane@example.com', 'lead');
```

```sql
-- Update a specific sheet row (row 7 of the tab)
UPDATE Contacts SET tag = 'customer' WHERE _row = 7;
```

```sql
-- Remove a specific sheet row
DELETE FROM Contacts WHERE _row = 7;
```

## Supported Operations

| Method | Description |
|---|---|
| `test_connection` | Calls the Sheets API with the saved token to validate the spreadsheet ID and auth |
| `get_databases` | Returns the spreadsheet ID as the single database |
| `get_tables` | Lists the tabs of the spreadsheet |
| `get_columns` | Returns the header row plus the synthetic `_row` PK, with inferred types |
| `get_schema_snapshot` | Full schema dump in one call (used for ER diagrams) |
| `get_all_columns_batch` | All columns for all tabs in one call |
| `execute_query` | Runs the SQL subset against the sheet |
| `insert_record` / `update_record` / `delete_record` | Row-level edits from the grid |
| `get_schemas`, `get_indexes`, `get_foreign_keys`, `get_views*`, `get_routines*` | Return empty — Sheets has no concept of these |
| `explain_query`, `create_view`, `alter_view`, `drop_view`, DDL generators | Return `-32601` (not applicable to Sheets) |

## Limitations

- No `JOIN` across tabs. If you need that, export the sheets to CSV and use the CSV plugin, or pull them into DuckDB.
- No server-side filtering: `WHERE` clauses are evaluated after fetching the full sheet range. Fine for sheets with a few thousand rows, unnecessarily slow past that.
- Row identity is positional (`_row` = sheet row number). Deleting or inserting a row shifts every `_row` below it — don't cache these values across sessions.
- Types are best-effort. A column with `1`, `2`, `3.5` will be `REAL`; add a non-numeric value and it becomes `TEXT`.
- The driver targets spreadsheets whose first row is a header row. Tabs without headers will show up with no columns.

## Development

### Building

```bash
just build      # debug build
just release    # optimized release build (what CI ships)
```

The UI extensions (OAuth button, spreadsheet-ID connection field) live in `ui/` and are bundled with Vite:

```bash
just ui-install
just ui-build
```

### Testing the Plugin

Run the full build + install into the local Tabularis plugins folder:

```bash
just dev-install        # debug
just release-install    # release
```

You can also poke the binary from the shell without opening Tabularis:

```bash
# List the tabs of a spreadsheet (requires a valid token already stored)
echo '{"jsonrpc":"2.0","method":"get_tables","params":{"params":{"driver":"google-sheets","database":"<spreadsheet-id>"}},"id":1}' \
  | ./target/debug/google-sheets-plugin
```

Unit tests:

```bash
just test
```

### Layout

```
src/
├── main.rs            stdio loop
├── rpc.rs             JSON-RPC dispatch + response helpers
├── auth.rs            OAuth2 state + automatic token refresh
├── sheets.rs          thin wrapper over the Sheets REST API
├── sql.rs             tiny SQL parser (SELECT / INSERT / UPDATE / DELETE)
├── client.rs          shared reqwest client
├── error.rs           plugin error type
├── models.rs          ConnectionParams + common shapes
├── handlers/
│   ├── init.rs        initialize — hydrates auth state from settings
│   ├── metadata.rs    databases, tables, columns, schema snapshot
│   ├── query.rs       test_connection, execute_query
│   ├── crud.rs        insert_record, update_record, delete_record
│   └── ddl.rs         unsupported DDL — returns -32601
└── utils/
    ├── identifiers.rs quote_identifier(name) + tests
    └── pagination.rs  paginate(query, page, size) + tests
ui/
└── src/               React-based UI extensions (Vite build)
```

### Tech Stack

- **Language:** Rust
- **HTTP client:** reqwest (blocking, rustls-tls)
- **Auth:** Google OAuth2 (Installed App flow)
- **Data source:** [Google Sheets API v4](https://developers.google.com/sheets/api)
- **UI extensions:** Vite + `@tabularis/plugin-api`
- **Protocol:** JSON-RPC 2.0 over stdio

## [Changelog](./CHANGELOG.md)

## License

Apache License 2.0
