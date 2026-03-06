use crate::config::get_config_dir_path_based_on_os;
use color_eyre::Result;
use color_eyre::eyre::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {}

pub fn load_settings() -> Result<Settings> {
    let config_path = match get_config_dir_path_based_on_os()? {
        Some(path) => path,
        None => bail!("Unable to get config dir"),
    };

    let file_path = config_path.join("settings.json");
    if !file_path.exists() {
        bail!("Settings file does not exist");
    }

    let file = std::fs::File::open(file_path)?;
    let obj = serde_json::from_reader(file)?;

    return Ok(obj);
}
