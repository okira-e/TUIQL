use crate::config::get_config_dir_path_based_on_os;
use crate::models::table_model::QueryState;
use color_eyre::Result;
use color_eyre::eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

const HISTORY_CAP: usize = 100;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
    pub commands: ProjectConfigCommands,
    #[serde(default = "HashMap::new")]
    pub presets: HashMap<String, QueryState>,
}

impl ProjectConfig {
    pub fn new(name: &str) -> Self {
        return Self {
            name: name.to_string(),
            commands: Default::default(),
            presets: HashMap::new(),
        };
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ProjectConfigCommands {
    pub history: VecDeque<String>,
}

pub fn create_new_project_config(name: &str) -> Result<()> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let config_path = config_path.join(format!("projects/{}.json", name));
    let project_config = ProjectConfig::new(name);
    let project_config_json = serde_json::to_string_pretty(&project_config)?;
    fs::write(config_path, project_config_json)?;

    return Ok(());
}

pub fn load_project_config(project: &str) -> Result<ProjectConfig> {
    let file_path = project_config_path(project)?;
    let file = std::fs::File::open(file_path)?;
    let obj = serde_json::from_reader(file)?;

    return Ok(obj);
}

pub fn rename_project_config(current_name: &str, new_name: &str) -> Result<()> {
    validate_project_name(current_name)?;
    validate_project_name(new_name)?;

    if current_name == new_name {
        bail!("The new project name must be different from the current name");
    }

    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };
    let current_path = project_config_path(current_name)?;
    let new_path = config_path.join("projects").join(format!("{}.json", new_name));

    if new_path.exists() {
        bail!("A project named \"{}\" already exists", new_name);
    }

    let file = fs::File::open(&current_path)?;
    let mut project_config: ProjectConfig = serde_json::from_reader(file)?;
    project_config.name = new_name.to_string();

    let project_config_json = serde_json::to_vec_pretty(&project_config)?;
    let mut new_file = OpenOptions::new().write(true).create_new(true).open(&new_path)?;
    if let Err(err) = new_file.write_all(&project_config_json) {
        drop(new_file);
        let _ = fs::remove_file(&new_path);
        return Err(err.into());
    }
    drop(new_file);

    if let Err(err) = fs::remove_file(&current_path) {
        let _ = fs::remove_file(&new_path);
        return Err(err.into());
    }

    return Ok(());
}

pub fn append_history(config: &mut ProjectConfig, command: String) -> Result<()> {
    if config.commands.history.len() >= HISTORY_CAP {
        config.commands.history.pop_back();
    }

    config.commands.history.push_front(command);

    let file_path = project_config_path(&config.name)?;
    let file = std::fs::File::create(file_path)?;
    serde_json::to_writer_pretty(file, config)?;

    return Ok(());
}

pub fn save_preset(config: &mut ProjectConfig, name: String, query_state: QueryState) -> Result<()> {
    if config.presets.contains_key(&name) {
        bail!(format!("Preset with the name \"{}\" already exists", name));
    }

    config.presets.insert(name, query_state);

    return Ok(());
}

pub fn load_preset(config: &ProjectConfig, name: &str) -> Result<QueryState> {
    return match config.presets.get(name) {
        None => bail!("Preset with the name \"{name}\" does not exist"),
        Some(v) => Ok(v.clone()),
    };
}

pub fn remove_preset(config: &mut ProjectConfig, name: &str) -> Result<()> {
    if config.presets.remove(name).is_none() {
        bail!("Preset with the name \"{name}\" does not exist");
    }

    return Ok(());
}

fn project_config_path(project: &str) -> Result<PathBuf> {
    validate_project_name(project)?;

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

pub fn validate_project_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name == ".." || name.contains(['/', '\\']) {
        bail!("Project names cannot be empty or contain path separators");
    }

    return Ok(());
}
