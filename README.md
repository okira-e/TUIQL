# tuiql

A fully-featured SQL database client that lives in your terminal. Fast, keyboard-driven, and designed to replace heavyweight GUI clients.

![promo](./assets/demo.gif)

## Why tuiql?

Most terminal database tools are either too basic or painful to use. tuiql gives you the power of a native database client, table browsing, filtering, sorting, pagination, JSON viewing, theming, and more without ever leaving the terminal.

No mouse required. No Electron. No waiting.

## Supported Databases

- PostgreSQL
- MySQL
- MariaDB
- SQLite
- Turso

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

Rename a saved project (including its query history):

```sh
tuiql rename mydb production
```

Edit one or more details of a saved connection:

```sh
tuiql edit production --host db.internal --port 5433 --database analytics
```

Use `--password` or `--token` to securely prompt for a replacement credential.

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

| Command            | Shorthand | Action                                                       |
| ------------------ | --------- | ------------------------------------------------------------ |
| help               | h         | Open the help view                                           |
| quit               | q         | Quit the application                                         |
| count              | c         | Count fetched rows                                           |
| total-count        | tc        | Count total rows in the selected table regardless of filters |
| goto page 5        | g p       | Jump to a specific page                                      |
| goto table_name    | g         | Jump to a table by name                                      |
| limit 1000\|1k     | l         | Set rows per page                                            |
| refresh            | r         | Re-fetch data with current filters                           |
| order-by col desc  | ob        | Add ORDER BY (no args to reset)                              |
| where col = 'val'  | w         | Add WHERE clause (no args to reset)                          |
| save-preset name   |           | Save the current query state as a named preset               |
| load-preset name   |           | Apply a preset and re-fetch the selected table               |
| remove-preset name |           | Remove a saved preset                                        |
| set key value      |           | Change a setting at runtime                                  |
| theme name         |           | Change the current theme at runtime                          |

### Query Presets

Query presets save the current pagination and filtering state: the page offset, row limit, `WHERE` clause, and `ORDER BY` clause. Presets are stored in the saved project's configuration alongside its command history, so they are available the next time the project is opened.

Presets are only available when the database was opened as a saved project with `tuiql open`. Preset names cannot contain spaces, and the current query state must differ from the defaults before it can be saved. Saving a duplicate name, or loading or removing a name that does not exist, reports an error.

For example:

```text
:where status = 'active'
:order-by created_at desc
:limit 100
:save-preset active-users

:load-preset active-users
:remove-preset active-users
```

## Features

- Built with Rust for instant startup and minimal resource usage
- Vim-style navigation throughout the entire interface
- Inline WHERE and ORDER BY filtering without writing full queries
- Named query presets for saved projects
- Paginated table browsing with configurable page sizes
- JSON cell viewer for inspecting complex data
- Customizable themes

## Configuration

Run `tuiql --config-path` to see where your config files are stored.

### Settings

| Setting                | Options             | Default          | Description                                      |
| ---------------------- | ------------------- | ---------------- | ------------------------------------------------ |
| transparent_background | `true`, `false`     | `false`          | Use terminal background instead of theme color   |
| default_limit          | number              | `200`            | Sets the default query limit on every table      |
| default_sort           | "asc", "desc"       | "asc"            | Sets the default sorting direction on table load |
| theme                  | supported thme name | catppuccin-mocha | Sets the default theme                           |

## License

This project is licensed under the MIT license - see the [LICENSE](LICENSE) file for details.

## Testing

Testing this app is done with the following free sample databases:

- Postgres: [pagila](https://github.com/devrimgunduz/pagila)
- Mysql/Mariadb: [sakila](https://dev.mysql.com/doc/sakila/en/sakila-installation.html)
