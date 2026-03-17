pub mod connection;
pub mod project;
pub mod settings;

use crate::config::connection::Connection;
use crate::settings::Settings;
use color_eyre::Result;
use color_eyre::eyre::bail;
use std::fs;
use std::path::PathBuf;

/// Get the config file path based on the OS. It returns the path to the directory
/// where the config file is stored.
pub fn get_config_dir_path_based_on_os() -> Result<Option<PathBuf>> {
    let config_path = match dirs::config_dir() {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    // Check if the config directory for the application exists
    let app_name = env!("CARGO_PKG_NAME");
    if !config_path.join(app_name).exists() {
        return Ok(None);
    }

    let config_file_path = match std::env::consts::OS {
        "windows" => config_path.join(app_name),
        "macos" => config_path.join(app_name),
        "linux" => config_path.join(app_name),
        _ => bail!("unsupported OS"),
    };

    return Ok(Some(config_file_path));
}

pub fn get_logging_dir_path_based_on_os() -> Result<Option<PathBuf>> {
    let config_path = match dirs::config_dir() {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    // Check if the config directory for the application exists
    let app_name = env!("CARGO_PKG_NAME");
    if !config_path.join(app_name).exists() {
        return Ok(None);
    }

    let logging_file_path = match std::env::consts::OS {
        "windows" => config_path.join(app_name).join("logs"),
        "macos" => config_path.join(app_name).join("logs"),
        "linux" => config_path.join(app_name).join("logs"),
        _ => bail!("unsupported OS"),
    };

    return Ok(Some(logging_file_path));
}

pub fn create_config_if_not_exists() -> Result<()> {
    if let None = get_config_dir_path_based_on_os()? {
        // Create the directory that holds the config files.
        let config_path = match dirs::config_dir() {
            Some(path) => path,
            None => bail!("Unable to get config dir"),
        };

        let app_name = env!("CARGO_PKG_NAME");
        let config_dir = config_path.join(app_name);

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        // Create the connections config file.
        let connections_file_path = config_dir.join("connections.json");
        let project_connections = Vec::<Connection>::new();
        let project_connections_json = serde_json::to_string_pretty(&project_connections)?;
        fs::write(connections_file_path, project_connections_json)?;

        // Create the settings config file.
        let settings_file_path = config_dir.join("settings.json");
        let project_settings = Settings::default();
        let project_settings_json = serde_json::to_string_pretty(&project_settings)?;
        fs::write(settings_file_path, project_settings_json)?;

        // Create an empty projects config dir
        std::fs::create_dir_all(&config_dir.join("projects"))?;
    }

    return Ok(());
}
