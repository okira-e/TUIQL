use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};

use crate::{actions::{Action, AppAction}, app::App, ui::ViewId};


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
                        Event::Resize(_, _) => {}
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
            _ => {}
        }
        
        // Keymap for specific panes.
        let action = match self.ui.get_view_by_pane(self.ui.focused_pane) {
            ViewId::Explorer => self.ui.explorer.handle_key_event(key.modifiers, key.code),
            ViewId::ResultsTable => self.ui.results_table.handle_key_event(key.modifiers, key.code),
        };

        _ = self.action_tx.send(action);
    }

    /// Handle internal actions
    async fn handle_action(&mut self, action: Action) -> Result<()> {
        self.update(action).await?;
        
        return Ok(());
    }
}
