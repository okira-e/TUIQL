use crate::cli;
use crate::drivers;
use clap::Args;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
pub struct AppArgs {
    /// See where the config files are stored.
    #[arg(long, default_value_t = false)]
    pub config_path: bool,

    #[command(subcommand)]
    pub command: Option<cli::Commands>,
}

#[derive(Debug, Args)]
pub struct ConnectCmdArgs {
    /// The type of database: "postgres" | "mysql" | "mariadb" | "sqlite" | "turso".
    #[arg(long)]
    pub r#type: drivers::kinds::DbKind,

    /// Connection URL (required for postgres, mysql, mariadb, turso).
    #[arg(long)]
    pub url: Option<String>,

    /// Path to the SQLite database file (required for sqlite).
    #[arg(long)]
    pub path: Option<String>,

    /// Auth token (required for turso). Falls back to TURSO_AUTH_TOKEN env var.
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Debug, Args)]
pub struct EditConnectionCmdArgs {
    /// The name of the saved connection to edit.
    pub connection_name: String,

    /// Replace the host (postgres, mysql, mariadb).
    #[arg(long)]
    pub host: Option<String>,

    /// Replace the user (postgres, mysql, mariadb).
    #[arg(long)]
    pub user: Option<String>,

    /// Replace the port (postgres, mysql, mariadb).
    #[arg(long)]
    pub port: Option<u16>,

    /// Replace the database name (postgres, mysql, mariadb).
    #[arg(long)]
    pub database: Option<String>,

    /// Replace the SQLite database path.
    #[arg(long)]
    pub path: Option<String>,

    /// Replace the Turso connection URL.
    #[arg(long)]
    pub url: Option<String>,

    /// Prompt for a replacement database password.
    #[arg(long, default_value_t = false)]
    pub password: bool,

    /// Prompt for a replacement Turso auth token.
    #[arg(long, default_value_t = false)]
    pub token: bool,
}

#[derive(Debug, Args)]
pub struct OpenCmdArgs {
    /// The name of the connection to open.
    pub connection_name: String,
}

#[derive(Debug, Args)]
pub struct RemoveConnectionCmdArgs {
    /// The name of the connection to remove.
    pub connection_name: String,
}

#[derive(Debug, Args)]
pub struct RenameProjectCmdArgs {
    /// The current name of the saved project.
    pub current_name: String,

    /// The new name for the saved project.
    pub new_name: String,
}

#[derive(Debug, Args)]
pub struct SaveConnectionCmdArgs {
    /// The type of database: "postgres" | "mysql" | "mariadb" | "sqlite" | "turso".
    #[arg(long)]
    pub r#type: drivers::kinds::DbKind,
    /// The name of the connection.
    #[arg(long)]
    pub name: String,
    /// The host of the database (required for postgres, mysql, mariadb).
    #[arg(long)]
    pub host: Option<String>,
    /// The user of the database (required for postgres, mysql, mariadb).
    #[arg(long)]
    pub user: Option<String>,
    /// The port of the database (required for postgres, mysql, mariadb).
    #[arg(long)]
    pub port: Option<u16>,
    /// The database name (required for postgres, mysql, mariadb).
    #[arg(long)]
    pub database: Option<String>,
    /// Path to the SQLite database file (required for sqlite).
    #[arg(long)]
    pub path: Option<String>,
    /// Connection URL (required for turso, e.g. libsql://my-db.turso.io).
    #[arg(long)]
    pub url: Option<String>,
}
