use clap::Subcommand;

use crate::cli::args::ConnectCmdArgs;
use crate::cli::args::OpenCmdArgs;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Connect to a database directly without saving it.
    Connect(ConnectCmdArgs),
    /// Open a saved connection to a database.
    Open(OpenCmdArgs),
    /// List all saved connections.
    ListConnections,
}
