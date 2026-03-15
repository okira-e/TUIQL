# tuiql

A fully-featured SQL database client that lives in your terminal. Fast, keyboard-driven, and designed to replace heavyweight GUI clients.

## Why tuiql?

Most terminal database tools are either too basic or painful to use. tuiql gives you the power of a native database client — table browsing, filtering, sorting, pagination, JSON viewing, theming — without ever leaving the terminal.

No mouse required. No Electron. No waiting.

## Supported Databases

- PostgreSQL
- MySQL
- MariaDB
- SQLite

## Install

```sh
cargo install tuiql
```

## Quick Start

Connect directly:

```sh
tuiql connect --type postgres --url "postgres://user:pass@localhost:5432/mydb"
```

Save a connection for later:

```sh
tuiql add --type postgres --name mydb --host localhost --port 5432 --user admin --pass secret --database mydb
```

Then open it by name:

```sh
tuiql open mydb
```

List saved connections:

```sh
tuiql ls
```

## Keybindings

### Navigation

| Key                 | Action               |
| ------------------- | -------------------- |
| j \| Down \| Ctrl-n | Move down            |
| k \| Up \| Ctrl-p   | Move up              |
| Ctrl-d              | Scroll 10 rows down  |
| Ctrl-u              | Scroll 10 rows up    |
| g                   | Go to top            |
| G                   | Go to bottom         |
| n                   | Next page            |
| p                   | Previous page        |
| Tab                 | Switch between panes |

### Table

| Key | Action                 |
| --- | ---------------------- |
| w   | Add a WHERE clause     |
| o   | Add an ORDER BY clause |

### Command Mode

Press `:` to enter command mode.

| Command               | Action                              |
| --------------------- | ----------------------------------- |
| (q)uit                | Quit the application                |
| (c)ount               | Count rows in the selected table    |
| (g)oto (p)age 5       | Jump to a specific page             |
| (l)imit 1000\|1k      | Set rows per page                   |
| (r)efresh             | Re-fetch data with current filters  |
| (o)rder-(b)y col desc | Add ORDER BY (no args to reset)     |
| (w)here col = 'val'   | Add WHERE clause (no args to reset) |
| (s)et key value       | Change a setting at runtime         |

Most commands support short aliases (`:q`, `:c`, etc).

## Features

- Built with Rust for instant startup and minimal resource usage
- Vim-style navigation throughout the entire interface
- Inline WHERE and ORDER BY filtering without writing full queries
- Paginated table browsing with configurable page sizes
- JSON cell viewer for inspecting complex data
- Customizable themes

## Configuration

Run `tuiql --config-path` to see where your config files are stored.

### Settings

| Setting                | Options         | Description                                    |
| ---------------------- | --------------- | ---------------------------------------------- |
| transparent_background | `true`, `false` | Use terminal background instead of theme color |

## License

MIT
