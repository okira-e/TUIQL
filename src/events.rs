use std::time::Duration;

use color_eyre::Result;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use futures::FutureExt;
use futures::StreamExt;

use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::CommandAction;
use crate::actions::DbAction;
use crate::actions::ExplorerAction;
use crate::actions::JsonViewAction;
use crate::actions::ResultsTableAction;
use crate::app::App;
use crate::app::View;

impl App {
    /// Handles all events, both from crossterm and internal async actions.
    pub async fn handle_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                if let Some(Ok(evt)) = event {
                    match evt {
                        Event::Key(key)
                        if key.kind == KeyEventKind::Press => {
                            self.handle_key(key).await?;
                        }
                        Event::Resize(w, h) => {
                            self.update(Action::App(AppAction::Resize(w, h))).await?;
                        }
                        _ => {}
                    }
                }
            }

            action = self.action_rx.recv() => {
                if let Some(action) = action {
                    self.update(action).await?;
                }
            }

            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                self.update(Action::App(AppAction::Tick)).await?;
            }
        }

        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        let focused_view = self.get_focused_view();
        match focused_view {
            View::ResultsTable => match (key.modifiers, key.code) {
                (_, KeyCode::Char('k') | KeyCode::Up)
                | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::MoveUp))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('j') | KeyCode::Down)
                | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::MoveDown))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('h') | KeyCode::Left) => {
                    self.update(Action::ResultsTable(ResultsTableAction::ScrollLeft))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('l') | KeyCode::Right) => {
                    self.update(Action::ResultsTable(ResultsTableAction::ScrollRight))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('0')) => {
                    self.update(Action::ResultsTable(
                        ResultsTableAction::GoToFirstHorizontally,
                    ))
                    .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('$')) => {
                    self.update(Action::ResultsTable(
                        ResultsTableAction::GoToLastHorizontally,
                    ))
                    .await?;
                    return Ok(());
                }

                (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::JumpUp))
                        .await?;
                    return Ok(());
                }

                (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::JumpDown))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('g')) => {
                    self.update(Action::ResultsTable(
                        ResultsTableAction::GoToFirstVertically,
                    ))
                    .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('G')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::GoToLastVertically))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('n')) => {
                    self.update(Action::Db(DbAction::NextPage)).await?;
                    return Ok(());
                }

                (_, KeyCode::Char('p')) => {
                    self.update(Action::Db(DbAction::PrevPage)).await?;
                    return Ok(());
                }

                (_, KeyCode::Enter) => {
                    self.update(Action::App(AppAction::ViewSelectedRowAsJson))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('y')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::YankSelection))
                        .await?;
                    return Ok(());
                }

                _ => {}
            },

            View::Explorer => match (key.modifiers, key.code) {
                (_, KeyCode::Char('k') | KeyCode::Up)
                | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    self.update(Action::Explorer(ExplorerAction::MoveUp))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('j') | KeyCode::Down)
                | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.update(Action::Explorer(ExplorerAction::MoveDown))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('h') | KeyCode::Left) => {
                    self.update(Action::Explorer(ExplorerAction::ExpandNextItemType))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Enter) => {
                    if let Some(item) = &self.explorer_model.focused_item {
                        self.update(Action::App(AppAction::SelectTable(item.name.clone())))
                            .await?;
                        return Ok(());
                    }
                }

                _ => {}
            },

            View::StatusLine => match (key.modifiers, key.code) {
                (_, KeyCode::Char(c)) => {
                    self.update(Action::Command(CommandAction::AddChar(c)))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Backspace) => {
                    self.update(Action::Command(CommandAction::PopChar)).await?;
                    return Ok(());
                }

                (_, KeyCode::Left) => {
                    self.update(Action::Command(CommandAction::MoveLeft))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Right) => {
                    self.update(Action::Command(CommandAction::MoveRight))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Enter) => {
                    self.update(Action::Command(CommandAction::Execute)).await?;
                    return Ok(());
                }

                _ => {}
            },

            View::JsonView => match (key.modifiers, key.code) {
                (_, KeyCode::Char('k') | KeyCode::Up)
                | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    self.update(Action::JsonView(JsonViewAction::MoveUp))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('j') | KeyCode::Down)
                | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.update(Action::JsonView(JsonViewAction::MoveDown))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Esc) => {
                    self.update(Action::App(AppAction::CloseJsonView))
                        .await?;
                    return Ok(());
                }

                (_, KeyCode::Char('y')) => {
                    self.update(Action::ResultsTable(ResultsTableAction::YankSelection))
                        .await?;
                    return Ok(());
                }

                _ => {}
            },
        }

        // Global keymaps
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.update(Action::App(AppAction::Quit)).await?;
                return Ok(());
            }
            (_, KeyCode::Tab) => {
                self.update(Action::App(AppAction::CyclePane)).await?;
                return Ok(());
            }
            (_, KeyCode::Char(':')) => {
                // self.update(Action::App(AppAction::SetCommandMode)).await?;
                return Ok(());
            }
            _ => {}
        }

        return Ok(());
    }
}
