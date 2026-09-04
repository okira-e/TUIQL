use crate::app::App;
use crate::config;
use crate::models::table_model::QueryState;
use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use color_eyre::eyre::eyre;
use serde::Deserialize;
use serde::Serialize;

// @Settings
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    #[serde(default = "default_false")]
    pub transparent_background: bool,
    #[serde(default = "default_limit")]
    pub default_limit: u16,
    /// "asc" or "desc"
    #[serde(default = "default_sort")]
    pub default_sort: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

pub fn update_settings(app: &mut App, key: String, value_input: Option<String>) -> Result<()> {
    match key.as_str() {
        "transparent_background" => {
            let value = match value_input {
                None => true,
                Some(input) => {
                    if let Ok(parsed) = input.parse::<bool>() {
                        parsed
                    } else {
                        bail!("Expected values: true, false");
                    }
                }
            };

            app.settings.transparent_background = value;
        }
        "default_limit" => {
            let value = match value_input {
                None => QueryState::default().limit as u16,
                Some(input) => {
                    if let Ok(parsed) = input.parse::<u16>() {
                        parsed
                    } else {
                        bail!("Expected the value to be a positive number");
                    }
                }
            };

            app.settings.default_limit = value;
        }
        "default_sort" => {
            let value = match value_input {
                None => default_sort(),
                Some(input) => match input.as_str() {
                    "asc" | "desc" => input,
                    _ => bail!("Expected values: asc, desc"),
                },
            };

            app.settings.default_sort = value;
        }
        "theme" => {
            let value = match value_input {
                Some(input) => input,
                None => bail!("Missing theme name"),
            };

            app.settings.theme = value;
        }
        _ => bail!("Unknown settings key: {}", key),
    }

    app.settings = config::settings::update_settings(&app.settings)?;

    return Ok(());
}

pub fn save_preset(app: &mut App, name: String, mut query_state: QueryState) -> Result<()> {
    let table_name = selected_table_name(app)?;
    query_state.offset = 0;

    match &mut app.config {
        None => bail!("Open this database as a saved project to use this feature"),
        Some(config) => {
            if query_state == QueryState::new(&app.settings) {
                bail!("No filters applied. Add some to save.")
            }

            config::project::save_preset(config, &table_name, name, query_state)?;

            return Ok(());
        }
    }
}

pub fn load_preset(app: &mut App, name: String) -> Result<QueryState> {
    let table_name = selected_table_name(app)?;

    match &app.config {
        None => bail!("Open this database as a saved project to use this feature"),
        Some(config) => {
            return config::project::load_preset(config, &table_name, &name);
        }
    }
}

pub fn remove_preset(app: &mut App, name: String) -> Result<()> {
    let table_name = selected_table_name(app)?;

    match &mut app.config {
        None => bail!("Open this database as a saved project to use this feature"),
        Some(config) => {
            return config::project::remove_preset(config, &table_name, &name);
        }
    }
}

fn selected_table_name(app: &App) -> Result<String> {
    return app
        .table_model
        .table_name
        .clone()
        .ok_or_else(|| eyre!("No table is currently selected"));
}

pub fn default_false() -> bool {
    return false;
}

pub fn default_limit() -> u16 {
    return 200;
}

pub fn default_sort() -> String {
    return String::from("asc");
}

pub fn default_theme() -> String {
    return String::from("catppuccin-mocha");
}
