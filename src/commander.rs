use crate::actions::Action;
use crate::actions::CmdAction;
use crate::app::App;
use std::fmt::Display;

pub enum Command {
    /// Returns the total count of the currently selected table
    Count,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Count => write!(f, "count"),
        }
    }
}

impl App {
    pub fn evaluate_cmd(&self, cmd: &str) -> Action {
        if let Some(cmd) = self.parse_cmd(cmd) {
            match cmd {
                Command::Count => {
                    return Action::Cmd(CmdAction::Count);
                }
            }
        };

        return Action::None;
    }

    fn parse_cmd(&self, cmd: &str) -> Option<Command> {
        return match cmd.trim().to_lowercase().as_str() {
            "count" | "c" => Some(Command::Count),
            _ => None,
        };
    }
}
