use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};

use crate::{actions::{Action, AppAction, DbAction, ExplorerAction, ResultsTableAction}, app::View, app::App};


impl App {
    /// Handles all events, both from crossterm and internal actions.
    pub async fn handle_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                if let Some(Ok(evt)) = event {
                    match evt {
                        Event::Key(key)
                        if key.kind == KeyEventKind::Press => self.on_key_event(key),
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
                // Sleep for a short duration to avoid busy waiting.
            }
        }

        return Ok(());
    }

    pub fn on_key_event(&mut self, key: KeyEvent) {
        // Global keymap.
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                let _ = self.action_tx.send(Action::App(AppAction::Quit));
            }
            (_, KeyCode::Tab) => {
                let _ = self.action_tx.send(Action::App(AppAction::CyclePane));
            }
            _ => {
                let action = self.handle_key_event(key.modifiers, key.code);
                _ = self.action_tx.send(action);
            }
        }
    }

    /// Handle internal actions
    async fn handle_action(&mut self, action: Action) -> Result<()> {
        self.update(action).await?;
        
        return Ok(());
    }
    
    pub fn handle_key_event(&self, modifier: KeyModifiers, code: KeyCode) -> Action {
        return match self.focused_view {
            View::ResultsTable => {
                match (modifier, code) {
                    (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                        Action::ResultsTable(ResultsTableAction::MoveUp)
                    }
                    (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                        Action::ResultsTable(ResultsTableAction::MoveDown)
                    }
                    (_, KeyCode::Char('h') | KeyCode::Left) => {
                        Action::ResultsTable(ResultsTableAction::ScrollLeft)
                    }
                    // @TODO: Scroll bar appears when it should on storage_tiers but I can't scroll right.
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
                    _ => Action::None,
                }
            }
            View::Explorer => {
                match (modifier, code) {
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
        };
    }
}
