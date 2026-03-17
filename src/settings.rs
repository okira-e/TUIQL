use crate::app::App;
use crate::config::settings::update_settings;
use crate::models::table_model::QueryState;
use crate::utils::serde_utils::default_false;
use crate::utils::serde_utils::default_limit;
use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    #[serde(default = "default_false")]
    pub transparent_background: bool,
    #[serde(default = "default_limit")]
    pub default_limit: u16,
}

impl App {
    pub fn update_settings(&mut self, key: String, value_input: Option<String>) -> Result<()> {
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

                self.settings.transparent_background = value;
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

                self.settings.default_limit = value;
            }
            _ => bail!("Unknown settings key: {}", key),
        }

        self.settings = update_settings(&self.settings)?;

        return Ok(());
    }
}
