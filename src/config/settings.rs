use crate::config::get_config_dir_path_based_on_os;
use crate::utils::default_false;
use color_eyre::Result;
use color_eyre::eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    #[serde(default = "default_false")]
    pub transparent_background: bool,
}

pub fn load_settings() -> Result<Settings> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let file_path = config_path.join("settings.json");
    if !file_path.exists() {
        bail!("Settings file does not exist");
    }

    let file = match File::open(file_path) {
        Ok(res) => res,
        Err(err) => bail!("Failed to open settings file: {}", err),
    };

    let settings: Settings = match serde_json::from_reader(file) {
        Ok(res) => res,
        Err(err) => bail!("Failed to parse settings file: {}", err),
    };

    return Ok(settings);
}

pub fn update_settings(settings: &Settings) -> Result<Settings> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let file_path = config_path.join("settings.json");
    let file = match File::create(file_path) {
        Ok(res) => res,
        Err(err) => bail!("Failed to open settings file for writing: {}", err),
    };

    serde_json::to_writer_pretty(file, &settings)?;

    return load_settings();
}
