use crate::config::get_config_dir_path_based_on_os;
use color_eyre::Result;
use color_eyre::eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

const HISTORY_CAP: usize = 100;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
    pub commands: ProjectConfigCommands,
}

impl ProjectConfig {
    pub fn new(name: &str) -> Self {
        return Self { name: name.to_string(), commands: Default::default() };
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

fn project_config_path(project: &str) -> Result<PathBuf> {
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
