use crate::drivers::kinds::DbKind;
use color_eyre::Result;
use color_eyre::eyre::bail;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Settings {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Connection {
    pub name: String,
    pub kind: DbKind,
    pub url: String,
}

/// Loads the user settings stored in the config file.
pub fn load_settings() -> Result<Settings> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let settings_file_path = config_path.join("settings.json");
    if !settings_file_path.exists() {
        bail!("Settings file does not exist"); // Shouldn't happen.
    }

    let settings_file = std::fs::File::open(settings_file_path)?;
    let settings: Settings = serde_json::from_reader(settings_file)?;

    return Ok(settings);
}

/// Loads all database connections stored by the user.
pub fn load_connections() -> Result<Vec<Connection>> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let connections_file_path = config_path.join("connections.json");
    if !connections_file_path.exists() {
        bail!("Connection file does not exist"); // Shouldn't happen.
    }

    let connections_file = match std::fs::File::open(connections_file_path) {
        Ok(val) => val,
        Err(err) => bail!("Failed to open connection file: {}", err),
    };

    let connections: Vec<Connection> = match serde_json::from_reader(connections_file) {
        Ok(val) => val,
        Err(err) => bail!("Failed to parse connection file: {}", err),
    };

    return Ok(connections);
}

pub fn add_connection(conn: Connection) -> Result<()> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let connections_file_path = config_path.join("connections.json");
    if !connections_file_path.exists() {
        bail!("Connection file does not exist"); // Shouldn't happen.
    }

    let connections_file = match File::open(connections_file_path) {
        Ok(val) => val,
        Err(err) => bail!("Failed to open connection file: {}", err),
    };

    let mut connections: Vec<Connection> = match serde_json::from_reader(connections_file) {
        Ok(val) => val,
        Err(err) => bail!("Failed to parse connection file: {}", err),
    };

    connections.push(conn);

    let connections_file = match File::create(config_path.join("connections.json")) {
        Ok(val) => val,
        Err(err) => bail!("Failed to open connection file: {}", err),
    };

    match serde_json::to_writer_pretty(connections_file, &connections) {
        Ok(_) => {}
        Err(err) => bail!("Failed to write connection file: {}", err),
    };

    return Ok(());
}

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

/// Create the config directory if it does not exist.
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
        if !connections_file_path.exists() {
            let connections_config_file = std::fs::File::create(&connections_file_path)?;
            const DEFAULT_CONNECTIONS_CONFIG: &str = include_str!("../assets/config/default_connections_config.json");
            let default_config: Value = serde_json::from_str(DEFAULT_CONNECTIONS_CONFIG)?;
            serde_json::to_writer(connections_config_file, &default_config)?;
        }

        // Create the settings config file.
        let settings_file_path = config_dir.join("settings.json");
        if !settings_file_path.exists() {
            let settings_config_file = std::fs::File::create(&settings_file_path)?;
            const DEFAULT_SETTINGS_CONFIG: &str = include_str!("../assets/config/default_settings_config.json");
            let default_config: Value = serde_json::from_str(DEFAULT_SETTINGS_CONFIG)?;
            serde_json::to_writer(settings_config_file, &default_config)?;
        }
    }

    return Ok(());
}
