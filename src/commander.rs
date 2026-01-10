use color_eyre::eyre::Result;

use crate::app::App;
use crate::models::statusline::MsgKind;
use crate::models::statusline::MsgLifetime;

pub enum Command {
    /// Returns the total count of the currently selected table
    Count,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Count => write!(f, "count"),
        }
    }
}

impl App {
    pub async fn evaluate_user_command(&mut self, command: &str) -> Result<()> {
        match self.parse_command_from_str(command) {
            None => {}
            Some(cmd) => match cmd {
                Command::Count => {
                    let mut driver = self.db_driver.lock().await;
                    let message = driver
                        .query_count(&self.table_model.table_name.clone())
                        .await?;
                    drop(driver);

                    self.report_message(
                        format!("Count: {}", message.to_string()),
                        MsgKind::Success,
                        MsgLifetime::Long,
                    );
                }
            },
        };

        return Ok(());
    }

    fn parse_command_from_str(&self, command: &str) -> Option<Command> {
        return match command.trim().to_lowercase().as_str() {
            "count" => Some(Command::Count),
            _ => None,
        };
    }
}
