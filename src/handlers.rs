use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};

use crate::{actions::{Action, AppAction, DbAction, ExplorerAction, JsonViewAction, ResultsTableAction}, app::{App, View}};


impl App {
    /// Handles all events, both from crossterm and internal actions.
    pub async fn handle_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                if let Some(Ok(evt)) = event {
                    match evt {
                        Event::Key(key)
                        if key.kind == KeyEventKind::Press => self.handle_key_event(key),
                        Event::Mouse(_) => {}
                        Event::Resize(w, h) => {
                            let _ = self.action_tx.send(Action::App(AppAction::Resize(w, h)));
                        }
                        _ => {}
                    }
                }
            }
            action = self.action_rx.recv() => {
                if let Some(action) = action {
                    self.handle_action(action).await?;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                self.action_tx.send(Action::App(AppAction::Tick))?;
            }
        }

        return Ok(());
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Global keymap
        let action = match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                Action::App(AppAction::Quit)
            }
            (_, KeyCode::Tab) => {
                Action::App(AppAction::CyclePane)
            }
            (_, KeyCode::Esc) => {
                Action::App(AppAction::ClosePopup)
            }
            // View specific
            _ => {
                match self.focused_view {
                    View::ResultsTable => {
                        match (key.modifiers, key.code) {
                            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                                Action::ResultsTable(ResultsTableAction::MoveUp)
                            }
                            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                                Action::ResultsTable(ResultsTableAction::MoveDown)
                            }
                            (_, KeyCode::Char('h') | KeyCode::Left) => {
                                Action::ResultsTable(ResultsTableAction::ScrollLeft)
                            }
                            (_, KeyCode::Char('l') | KeyCode::Right) => {
                                Action::ResultsTable(ResultsTableAction::ScrollRight)
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                                Action::ResultsTable(ResultsTableAction::JumpUp)
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                                Action::ResultsTable(ResultsTableAction::JumpDown)
                            }
                            (_, KeyCode::Char('g')) => {
                                Action::ResultsTable(ResultsTableAction::GoToFirst)
                            }
                            (_, KeyCode::Char('G')) => {
                                Action::ResultsTable(ResultsTableAction::GoToLast)
                            }
                            (_, KeyCode::Char('n')) => {
                                Action::Db(DbAction::NextPage)
                            }
                            (_, KeyCode::Char('p')) => {
                                Action::Db(DbAction::PrevPage)
                            }
                            (_, KeyCode::Enter) => {
                                Action::App(AppAction::ViewSelectedRowAsJson)
                            }
                            (_, KeyCode::Char('y')) => {
                                Action::ResultsTable(ResultsTableAction::YankSelection)
                            }
                            _ => Action::None,
                        }
                    }
                    View::Explorer => {
                        match (key.modifiers, key.code) {
                            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                                Action::Explorer(ExplorerAction::MoveUp)
                            }
                            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                                Action::Explorer(ExplorerAction::MoveDown)
                            }
                            (_, KeyCode::Char('h') | KeyCode::Left) => {
                                Action::Explorer(ExplorerAction::ExpandNextItemType)
                            }
                            (_, KeyCode::Enter) => {
                                if let Some(item) = &self.explorer_model.focused_item {
                                    Action::App(AppAction::SelectTable(item.name.clone()))
                                } else {
                                    Action::None
                                }
                            }
                            _ => Action::None,
                        }
                    },
                    View::StatusLine => {
                        todo!()
                    },
                    View::JsonView => {
                        match (key.modifiers, key.code) {
                            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                                Action::JsonView(JsonViewAction::MoveUp)
                            }
                            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                                Action::JsonView(JsonViewAction::MoveDown)
                            }
                            (_, KeyCode::Char('g')) => {
                                Action::JsonView(JsonViewAction::GoToFirst)
                            }
                            (_, KeyCode::Char('G')) => {
                                Action::JsonView(JsonViewAction::GoToFirst)
                            }
                            (_, KeyCode::Char('y')) => {
                                Action::ResultsTable(ResultsTableAction::YankSelection)
                            }
                            _ => Action::None,
                        }
                    },
                }
            }
        };
        _ = self.action_tx.send(action);
    }

    /// Handle internal actions
    async fn handle_action(&mut self, action: Action) -> Result<()> {
        self.update(action).await?;
        
        return Ok(());
    }
}
