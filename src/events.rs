use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::CmdLineAction;
use crate::actions::DbAction;
use crate::actions::ExplorerAction;
use crate::actions::JsonViewAction;
use crate::actions::ResultsTableAction;
use crate::app::App;
use crate::app::View;
use color_eyre::Result;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use futures::StreamExt;
use std::time::Duration;

impl App {
    /// Handles all events, both from crossterm and internal actions.
    pub async fn handle_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next() => {
                if let Some(Ok(evt)) = event {
                    match evt {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            self.handle_key(key);
                        }
                        Event::Resize(w, h) => {
                            self.update(Action::App(AppAction::Resize(w, h)));
                        }
                        _ => {}
                    }
                }
            }

            action = self.action_rx.recv() => {
                if let Some(action) = action {
                    self.update(action);
                }
            }

            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                self.update(Action::App(AppAction::Tick));
            }
        }

        return Ok(());
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let focused_view = self.get_focused_view();
        match focused_view {
            View::ResultsTable => match (key.modifiers, key.code) {
                (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::MoveUp));
                    return;
                }

                (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::MoveDown));
                    return;
                }

                (_, KeyCode::Char('h') | KeyCode::Left) => {
                    self.update(Action::ResultsTable(ResultsTableAction::ScrollLeft));
                    return;
                }

                (_, KeyCode::Char('l') | KeyCode::Right) => {
                    self.update(Action::ResultsTable(ResultsTableAction::ScrollRight));
                    return;
                }

                (_, KeyCode::Char('0')) => {
                    self.update(Action::ResultsTable(
                        ResultsTableAction::GoToFirstHorizontally,
                    ));
                    return;
                }

                (_, KeyCode::Char('$')) => {
                    self.update(Action::ResultsTable(
                        ResultsTableAction::GoToLastHorizontally,
                    ));
                    return;
                }

                (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::JumpUp));
                    return;
                }

                (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::JumpDown));
                    return;
                }

                (_, KeyCode::Char('g')) => {
                    self.update(Action::ResultsTable(
                        ResultsTableAction::GoToFirstVertically,
                    ));
                    return;
                }

                (_, KeyCode::Char('G')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::GoToLastVertically));
                    return;
                }

                (_, KeyCode::Char('n')) => {
                    self.update(Action::Db(DbAction::NextPage));
                    return;
                }

                (_, KeyCode::Char('p')) => {
                    self.update(Action::Db(DbAction::PrevPage));
                    return;
                }

                (_, KeyCode::Enter) => {
                    self.update(Action::App(AppAction::ViewSelectedRowAsJson));
                    return;
                }

                (_, KeyCode::Char('y')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::YankSelection));
                    return;
                }

                _ => {}
            },

            View::Explorer => match (key.modifiers, key.code) {
                (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    self.update(Action::Explorer(ExplorerAction::MoveUp));
                    return;
                }

                (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.update(Action::Explorer(ExplorerAction::MoveDown));
                    return;
                }

                (_, KeyCode::Char(']')) => {
                    self.update(Action::Explorer(ExplorerAction::NextTab));
                    return;
                }

                (_, KeyCode::Char('[')) => {
                    self.update(Action::Explorer(ExplorerAction::PrevTab));
                    return;
                }

                (_, KeyCode::Char('g')) => {
                    self.update(Action::Explorer(ExplorerAction::GoToFirst));
                    return;
                }

                (_, KeyCode::Char('G')) => {
                    self.update(Action::Explorer(ExplorerAction::GoToLast));
                    return;
                }

                (_, KeyCode::Enter) => {
                    if let Some(item) = &self.explorer_model.focused_item {
                        self.update(Action::App(AppAction::SelectTable(item.name.clone())));
                        return;
                    }
                }

                _ => {}
            },

            View::StatusLine => match (key.modifiers, key.code) {
                (_, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    self.update(Action::CmdLine(CmdLineAction::TogglePrevCommand));
                    return;
                }

                (_, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.update(Action::CmdLine(CmdLineAction::ToggleNextCommand));
                    return;
                }

                (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    self.update(Action::CmdLine(CmdLineAction::Exit));
                    return;
                }

                (_, KeyCode::Backspace) => {
                    self.update(Action::CmdLine(CmdLineAction::PopChar));
                    return;
                }

                (_, KeyCode::Left) => {
                    self.update(Action::CmdLine(CmdLineAction::MoveLeft));
                    return;
                }

                (_, KeyCode::Right) => {
                    self.update(Action::CmdLine(CmdLineAction::MoveRight));
                    return;
                }

                (_, KeyCode::Enter) => {
                    self.update(Action::CmdLine(CmdLineAction::Execute));
                    return;
                }

                (_, KeyCode::Char(c)) => {
                    self.update(Action::CmdLine(CmdLineAction::AddChar(c)));
                    return;
                }

                _ => {}
            },

            View::JsonView => match (key.modifiers, key.code) {
                (_, KeyCode::Char('g')) => {
                    self.update(Action::JsonView(JsonViewAction::GoToFirst));
                    return;
                }

                (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    self.update(Action::JsonView(JsonViewAction::MoveUp));
                    return;
                }

                (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.update(Action::JsonView(JsonViewAction::MoveDown));
                    return;
                }

                (_, KeyCode::Esc) => {
                    self.update(Action::App(AppAction::CloseJsonView));
                    return;
                }

                (_, KeyCode::Char('y')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::YankSelection));
                    return;
                }

                _ => {}
            },
        }

        // Global keymaps
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.update(Action::App(AppAction::Quit));
                return;
            }
            (_, KeyCode::Tab) => {
                self.update(Action::App(AppAction::CyclePane));
                return;
            }
            (_, KeyCode::Char(':')) => {
                self.update(Action::App(AppAction::SetCommandMode));
                return;
            }
            _ => {}
        }

        return;
    }
}
