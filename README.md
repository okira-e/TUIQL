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

| Key | Action                            |
| --- | --------------------------------- |
| w   | Add a WHERE clause                |
| o   | Add an ORDER BY clause            |
| r   | Refresh query result              |
| y   | Copy highlighted row to clipboard |

### Command Mode

Press `:` to enter command mode.

| Command           | Shorthand | Action                                                       |
| ----------------- | --------- | ------------------------------------------------------------ |
| help              | h         | Open the help view                                           |
| quit              | q         | Quit the application                                         |
| count             | c         | Count fetched rows                                           |
| total-count       | tc        | Count total rows in the selected table regardless of filters |
| goto page 5       | g p       | Jump to a specific page                                      |
| goto table_name   | g         | Jump to a table by name                                      |
| limit 1000\|1k    | l         | Set rows per page                                            |
| refresh           | r         | Re-fetch data with current filters                           |
| order-by col desc | ob        | Add ORDER BY (no args to reset)                              |
| where col = 'val' | w         | Add WHERE clause (no args to reset)                          |
| set key value     |           | Change a setting at runtime                                  |

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

| Setting                | Options         | Default | Description                                      |
| ---------------------- | --------------- | ------- | ------------------------------------------------ |
| transparent_background | `true`, `false` | `false` | Use terminal background instead of theme color   |
| default_limit          | number          | `200`   | Sets the default query limit on every table      |
| default_sort           | "asc", "desc"   | "asc"   | Sets the default sorting direction on table load |

## License

This project is licensed under the MIT license - see the [LICENSE](LICENSE) file for details.

## Testing

Testing this app is done with the following free sample databases:

- Postgres: [pagila](https://github.com/devrimgunduz/pagila)
- Mysql/Mariadb: [sakila](https://dev.mysql.com/doc/sakila/en/sakila-installation.html)
