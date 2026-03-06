use crate::config::get_config_dir_path_based_on_os;
use crate::config::project::create_new_project_config;
use crate::drivers::kinds::DbKind;
use color_eyre::Result;
use color_eyre::eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Connection {
    pub name: String,
    pub kind: DbKind,
    pub url: String,
}

/// Loads all database connections stored by the user.
pub fn load_connections() -> Result<Vec<Connection>> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let connections_file_path = config_path.join("connections.json");
    if !connections_file_path.exists() {
        bail!("Connection file does not exist");
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

    // Add connection

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

    let connection_name = conn.name.clone();
    connections.push(conn);

    let connections_file = match File::create(config_path.join("connections.json")) {
        Ok(val) => val,
        Err(err) => bail!("Failed to open connection file: {}", err),
    };

    match serde_json::to_writer_pretty(connections_file, &connections) {
        Ok(_) => {}
        Err(err) => bail!("Failed to write connection file: {}", err),
    };

    // Add project config file

    create_new_project_config(&connection_name)?;

    return Ok(());
}

fn connections_config_path(project: &str) -> Result<PathBuf> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let path = config_path.join(format!("projects/{}.json", project));
    if !path.exists() {
        bail!("Project config file does not exist");
    }

    return Ok(path);
}
