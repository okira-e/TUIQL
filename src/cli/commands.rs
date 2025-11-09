use clap::Subcommand;

use crate::cli::args::{ConnectCmdArgs, OpenCmdArgs};


#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize the config files.
    Init,
    /// Connect to a database directly without saving it.
    Connect(ConnectCmdArgs),
    /// Open a saved connection to a database.
    Open(OpenCmdArgs),
    /// List all saved connections.
    ListConnections,
}
