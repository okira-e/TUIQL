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
    /// If provided along with a url, opens the connection directly.
    #[arg(long)]
    pub r#type: drivers::kinds::DbKind,

    #[arg(long)]
    pub url: String,
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
    /// The name of the connection to open.
    #[arg(long)]
    pub name: String,
    /// The host of the database.
    #[arg(long)]
    pub host: String,
    /// The user of the database.
    #[arg(long)]
    pub user: String,
    /// The port of the database.
    #[arg(long)]
    pub port: u16,
    /// The database name.
    #[arg(long)]
    pub database: String,
}
