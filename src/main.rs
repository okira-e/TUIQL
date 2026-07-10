mod actions;
mod app;
mod cli;
mod commander;
mod config;
mod drivers;
mod events;
mod logging;
mod models;
mod render;
mod settings;
mod theme;
mod update;
mod utils;
mod views;
mod suggestor;

use clap::Parser;
use color_eyre::Result;
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    if let Err(err) = config::create_config_if_not_exists() {
        eprintln!("Error creating config files: {}", err);
        std::process::exit(1);
    }

    // Set up logging
    // Should live for the lifetime of the application.
    let _guard = logging::setup_logging()?;

    let args = cli::args::AppArgs::parse();
    if let Err(err) = cli::run(args).await {
        eprintln!("{}", err.to_string().red());
        std::process::exit(1);
    };

    return Ok(());
}
