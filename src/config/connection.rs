use crate::config;
use crate::config::get_config_dir_path_based_on_os;
use crate::config::project::create_new_project_config;
use crate::config::project::rename_project_config;
use crate::drivers::kinds::DbKind;
use color_eyre::Result;
use color_eyre::eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
pub struct Connection {
    pub name: String,
    pub kind: DbKind,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
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
    config::project::validate_project_name(&conn.name)?;

    //
    // Get connections
    //

    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let connections_file_path = config_path.join("connections.json");
    assert!(connections_file_path.exists());

    let connections_file = match File::open(connections_file_path) {
        Ok(val) => val,
        Err(err) => bail!("Failed to open connection file: {}", err),
    };

    let mut connections: Vec<Connection> = match serde_json::from_reader(connections_file) {
        Ok(val) => val,
        Err(err) => bail!("Failed to parse connection file: {}", err),
    };

    // Check if a connection with the same name exists
    for c in connections.iter() {
        if conn.name == c.name {
            bail!("A connection with the same name already exists!");
        }
    }

    let connection_name = conn.name.clone();

    //
    // Add project config file
    //

    create_new_project_config(&connection_name)?;

    //
    // Add connection to connections.json
    //

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

pub fn remove_connection(name: &str) -> Result<()> {
    let mut connections = load_connections()?;

    let initial_len = connections.len();
    connections.retain(|c| c.name != name);

    if connections.len() == initial_len {
        bail!("Connection \"{}\" not found", name);
    }

    // Remove the connection from the connections file

    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let connections_file = match File::create(config_path.join("connections.json")) {
        Ok(val) => val,
        Err(err) => bail!("Failed to open connection file: {}", err),
    };

    match serde_json::to_writer_pretty(connections_file, &connections) {
        Ok(_) => {}
        Err(err) => bail!("Failed to write connection file: {}", err),
    };

    // Remove the project file

    fs::remove_file(config_path.join("projects").join(format!("{}.json", name)))?;

    return Ok(());
}

pub fn rename_project(current_name: &str, new_name: &str) -> Result<()> {
    if current_name == new_name {
        bail!("The new project name must be different from the current name");
    }

    let mut connections = load_connections()?;

    let mut connection_index: Option<usize> = None;
    for (i, con) in connections.iter().enumerate() {
        if con.name == current_name {
            connection_index = Some(i);
        }
    }

    let connection_index = match connection_index {
        Some(i) => i,
        None => bail!("Project \"{}\" not found", current_name),
    };

    if connections.iter().any(|connection| connection.name == new_name) {
        bail!("A project named \"{}\" already exists", new_name);
    }

    rename_project_config(current_name, new_name)?;

    connections[connection_index].name = new_name.to_string();

    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => {
            let _ = rename_project_config(new_name, current_name);
            bail!("Unable to get config dir");
        }
    };

    let connections_json = serde_json::to_string_pretty(&connections)?;
    if let Err(err) = std::fs::write(config_path.join("connections.json"), connections_json) {
        if let Err(rollback_err) = rename_project_config(new_name, current_name) {
            bail!(
                "Failed to save renamed project: {}. Also failed to restore its project config: {}",
                err,
                rollback_err
            );
        }
        return Err(err.into());
    }

    return Ok(());
}
