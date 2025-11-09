use color_eyre::Result;
use color_eyre::eyre::bail;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::EnvFilter;

use crate::config; // Needed for implementing the `parse` function on `AppArgs`.

pub fn setup_logging() -> Result<WorkerGuard> {
    let logging_dir = match config::get_logging_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Config directory does not exist. Please run `init` command to create it."),
    };

    let file_appender = RollingFileAppender::new(Rotation::DAILY, logging_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Set log level based on build mode
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(env_filter)
        .init();

    return Ok(guard);
}
