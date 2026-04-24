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
    /// The type of database: "postgres" | "mysql" | "mariadb" | "sqlite".
    #[arg(long)]
    pub r#type: drivers::kinds::DbKind,

    /// Connection URL (required for postgres, mysql, mariadb).
    #[arg(long)]
    pub url: Option<String>,

    /// Path to the SQLite database file (required for sqlite).
    #[arg(long)]
    pub path: Option<String>,
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
pub struct SaveConnectionCmdArgs {
    /// The type of database: "postgres" | "mysql" | "mariadb" | "sqlite".
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
}
