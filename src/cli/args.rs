use clap::{Args, Parser};

use crate::{cli, drivers};

#[derive(Debug, Parser)]
#[command(version)]
pub struct AppArgs {
    /// See where the config files are stored.
    #[arg(long, default_value_t = false)]
    pub config_path: bool,

    #[command(subcommand)]
    pub command: Option<cli::commands::Commands>,
}

#[derive(Debug, Args)]
pub struct ConnectCmdArgs {
    /// If provided along with a url, opens the connection directly.
    #[arg(long)]
    pub r#type: drivers::kinds::DbKinds,

    #[arg(long)]
    pub url: String,
}

#[derive(Debug, Args)]
pub struct OpenCmdArgs {
    /// The name of the connection to open.
    pub connection_name: String,
}
