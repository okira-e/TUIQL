use crate::config::get_config_dir_path_based_on_os;
use crate::models::table_model::QueryState;
use color_eyre::Result;
use color_eyre::eyre::bail;
use color_eyre::eyre::eyre;
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
    #[serde(default = "Default::default")]
    pub presets: Vec<Preset>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Preset {
    pub table_name: String,
    #[serde(default = "HashMap::new")]
    pub presets: HashMap<String, QueryState>,
}

impl ProjectConfig {
    pub fn new(name: &str) -> Self {
        return Self {
            name: name.to_string(),
            commands: Default::default(),
            presets: Default::default(),
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

pub fn save_preset(
    config: &mut ProjectConfig,
    table_name: &str,
    name: String,
    mut query_state: QueryState,
) -> Result<()> {
    query_state.offset = 0;

    if let Some(table_presets) = config.presets.iter_mut().find(|preset| preset.table_name == table_name) {
        if table_presets.presets.contains_key(&name) {
            bail!("Preset with the name \"{name}\" already exists for table \"{table_name}\"");
        }

        table_presets.presets.insert(name, query_state);
    } else {
        let mut presets = HashMap::new();
        presets.insert(name, query_state);
        config
            .presets
            .push(Preset { table_name: table_name.to_string(), presets });
    }

    return Ok(());
}

pub fn load_preset(config: &ProjectConfig, table_name: &str, name: &str) -> Result<QueryState> {
    let table_presets = config
        .presets
        .iter()
        .find(|preset| preset.table_name == table_name)
        .ok_or_else(|| eyre!("Preset with the name \"{name}\" does not exist for table \"{table_name}\""))?;

    let mut query_state = table_presets
        .presets
        .get(name)
        .cloned()
        .ok_or_else(|| eyre!("Preset with the name \"{name}\" does not exist for table \"{table_name}\""))?;
    query_state.offset = 0;

    return Ok(query_state);
}

pub fn remove_preset(config: &mut ProjectConfig, table_name: &str, name: &str) -> Result<()> {
    let table_index = config
        .presets
        .iter()
        .position(|preset| preset.table_name == table_name)
        .ok_or_else(|| eyre!("Preset with the name \"{name}\" does not exist for table \"{table_name}\""))?;

    let table_presets = &mut config.presets[table_index];
    if table_presets.presets.remove(name).is_none() {
        bail!("Preset with the name \"{name}\" does not exist for table \"{table_name}\"");
    }

    if table_presets.presets.is_empty() {
        config.presets.remove(table_index);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn query_state(offset: usize, where_clause: &str) -> QueryState {
        return QueryState {
            offset,
            limit: 50,
            order_by_clause: Some(String::from("id desc")),
            where_clause: Some(where_clause.to_string()),
        };
    }

    #[test]
    fn presets_are_scoped_by_table_and_reset_offset() {
        let mut config = ProjectConfig::new("test");

        save_preset(
            &mut config,
            "users",
            String::from("active"),
            query_state(100, "active = true"),
        )
        .unwrap();
        save_preset(
            &mut config,
            "orders",
            String::from("active"),
            query_state(200, "paid = true"),
        )
        .unwrap();

        let users = load_preset(&config, "users", "active").unwrap();
        let orders = load_preset(&config, "orders", "active").unwrap();

        assert_eq!(users.offset, 0);
        assert_eq!(users.where_clause.as_deref(), Some("active = true"));
        assert_eq!(orders.offset, 0);
        assert_eq!(orders.where_clause.as_deref(), Some("paid = true"));
    }

    #[test]
    fn duplicate_preset_names_are_rejected_only_within_the_same_table() {
        let mut config = ProjectConfig::new("test");
        save_preset(
            &mut config,
            "users",
            String::from("active"),
            query_state(0, "active = true"),
        )
        .unwrap();

        let result = save_preset(
            &mut config,
            "users",
            String::from("active"),
            query_state(0, "active = false"),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn removing_the_last_preset_removes_its_table_group() {
        let mut config = ProjectConfig::new("test");
        save_preset(
            &mut config,
            "users",
            String::from("active"),
            query_state(0, "active = true"),
        )
        .unwrap();

        remove_preset(&mut config, "users", "active").unwrap();

        assert!(config.presets.is_empty());
    }
}
