use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::CmdLineAction;
use crate::actions::DbAction;
use crate::actions::ExplorerAction;
use crate::actions::HelpViewAction;
use crate::actions::JsonViewAction;
use crate::actions::ResultsTableAction;
use crate::app;
use crate::app::App;
use crate::app::View;
use crate::events;
use crate::update;
use color_eyre::Result;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use futures::StreamExt;
use std::time::Duration;

/// Handles all events, both from crossterm and internal actions.
pub async fn handle_events(app: &mut App) -> Result<()> {
    tokio::select! {
        event = app.event_stream.next() => {
            if let Some(Ok(evt)) = event {
                match evt {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        events::handle_key(app, key);
                    }
                    Event::Resize(w, h) => {
                        update::update(app, Action::App(AppAction::Resize(w, h)));
                    }
                    _ => {}
                }
            }
        }

        action = app.action_rx.recv() => {
            if let Some(action) = action {
                update::update(app, action);
            }
        }

        _ = tokio::time::sleep(Duration::from_millis(100)) => {
            update::update(app, Action::App(AppAction::Tick));
        }
    }

    return Ok(());
}

fn handle_key(app: &mut App, key: KeyEvent) {
    let focused_view = app::get_focused_view(app);
    match focused_view {
        View::ResultsTable => match (key.modifiers, key.code) {
            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::MoveUp));
                return;
            }

            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::MoveDown));
                return;
            }

            (_, KeyCode::Char('h') | KeyCode::Left) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::ScrollLeft));
                return;
            }

            (_, KeyCode::Char('l') | KeyCode::Right) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::ScrollRight));
                return;
            }

            (_, KeyCode::Char('0')) => {
                update::update(
                    app,
                    Action::ResultsTable(ResultsTableAction::GoToFirstHorizontally),
                );
                return;
            }

            (_, KeyCode::Char('$')) => {
                update::update(
                    app,
                    Action::ResultsTable(ResultsTableAction::GoToLastHorizontally),
                );
                return;
            }

            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::JumpUp));
                return;
            }

            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::JumpDown));
                return;
            }

            (_, KeyCode::Char('g')) => {
                update::update(
                    app,
                    Action::ResultsTable(ResultsTableAction::GoToFirstVertically),
                );
                return;
            }

            (_, KeyCode::Char('G')) => {
                update::update(
                    app,
                    Action::ResultsTable(ResultsTableAction::GoToLastVertically),
                );
                return;
            }

            (_, KeyCode::Char('n')) => {
                update::update(app, Action::Db(DbAction::NextPage));
                return;
            }

            (_, KeyCode::Char('p')) => {
                update::update(app, Action::Db(DbAction::PrevPage));
                return;
            }

            (_, KeyCode::Char('r')) => {
                update::update(app, Action::Db(DbAction::QueryTable));
                return;
            }

            (_, KeyCode::Char('w')) => {
                update::update(app, Action::CmdLine(CmdLineAction::ToggleWhereClause));
                return;
            }

            (_, KeyCode::Char('o')) => {
                update::update(app, Action::CmdLine(CmdLineAction::ToggleOrderByClause));
                return;
            }

            (_, KeyCode::Enter) => {
                update::update(app, Action::App(AppAction::ViewSelectedRowAsJson));
                return;
            }

            (_, KeyCode::Char('y')) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::YankSelection));
                return;
            }

            _ => {}
        },
        View::Explorer => match (key.modifiers, key.code) {
            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                update::update(app, Action::Explorer(ExplorerAction::MoveUp));
                return;
            }

            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                update::update(app, Action::Explorer(ExplorerAction::MoveDown));
                return;
            }

            (_, KeyCode::Char(']')) => {
                update::update(app, Action::Explorer(ExplorerAction::NextTab));
                return;
            }

            (_, KeyCode::Char('[')) => {
                update::update(app, Action::Explorer(ExplorerAction::PrevTab));
                return;
            }

            (_, KeyCode::Char('g')) => {
                update::update(app, Action::Explorer(ExplorerAction::GoToFirst));
                return;
            }

            (_, KeyCode::Char('G')) => {
                update::update(app, Action::Explorer(ExplorerAction::GoToLast));
                return;
            }

            (_, KeyCode::Enter) => {
                if let Some(item) = &app.explorer_model.focused_item {
                    update::update(app, Action::App(AppAction::SelectTable(item.name.clone())));
                    return;
                }
            }

            _ => {}
        },
        View::StatusLine => match (key.modifiers, key.code) {
            (_, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                update::update(app, Action::CmdLine(CmdLineAction::TogglePrevCommand));
                return;
            }

            (_, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                update::update(app, Action::CmdLine(CmdLineAction::ToggleNextCommand));
                return;
            }

            (_, KeyCode::Tab) => {
                update::update(app, Action::CmdLine(CmdLineAction::NextSuggestion));
                return;
            }

            (_, KeyCode::BackTab) => {
                update::update(app, Action::CmdLine(CmdLineAction::PrevSuggestion));
                return;
            }

            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                update::update(app, Action::CmdLine(CmdLineAction::Exit));
                return;
            }

            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                update::update(app, Action::CmdLine(CmdLineAction::PopLine));
                return;
            }

            (KeyModifiers::ALT, KeyCode::Backspace) => {
                update::update(app, Action::CmdLine(CmdLineAction::PopWord));
                return;
            }

            (KeyModifiers::ALT | KeyModifiers::CONTROL, KeyCode::Left) | (KeyModifiers::ALT, KeyCode::Char('b')) => {
                update::update(app, Action::CmdLine(CmdLineAction::MoveLeftWord));
                return;
            }

            (KeyModifiers::ALT | KeyModifiers::CONTROL, KeyCode::Right) | (KeyModifiers::ALT, KeyCode::Char('f')) => {
                update::update(app, Action::CmdLine(CmdLineAction::MoveRightWord));
                return;
            }

            (_, KeyCode::Backspace) => {
                update::update(app, Action::CmdLine(CmdLineAction::PopChar));
                return;
            }

            (_, KeyCode::Left) => {
                update::update(app, Action::CmdLine(CmdLineAction::MoveLeft));
                return;
            }

            (_, KeyCode::Right) => {
                update::update(app, Action::CmdLine(CmdLineAction::MoveRight));
                return;
            }

            (_, KeyCode::Enter) => {
                update::update(app, Action::CmdLine(CmdLineAction::Execute));
                return;
            }

            (_, KeyCode::Char(c)) => {
                update::update(app, Action::CmdLine(CmdLineAction::AddChar(c)));
                return;
            }

            _ => {}
        },
        View::JsonView => match (key.modifiers, key.code) {
            (_, KeyCode::Char('g')) => {
                update::update(app, Action::JsonView(JsonViewAction::GoToFirst));
                return;
            }

            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                update::update(app, Action::JsonView(JsonViewAction::MoveUp));
                return;
            }

            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                update::update(app, Action::JsonView(JsonViewAction::MoveDown));
                return;
            }

            (_, KeyCode::Esc) => {
                update::update(app, Action::App(AppAction::CloseJsonView));
                return;
            }

            (_, KeyCode::Char('y')) => {
                update::update(app, Action::ResultsTable(ResultsTableAction::YankSelection));
                return;
            }

            _ => {}
        },
        View::Help => match (key.modifiers, key.code) {
            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                update::update(app, Action::HelpView(HelpViewAction::MoveUp));
                return;
            }

            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                update::update(app, Action::HelpView(HelpViewAction::MoveDown));
                return;
            }

            (_, KeyCode::Char('g')) => {
                update::update(app, Action::HelpView(HelpViewAction::GoToFirst));
                return;
            }

            (_, KeyCode::Enter) => {
                update::update(app, Action::HelpView(HelpViewAction::ActivateAction));
                return;
            }

            (_, KeyCode::Char('G')) => {
                update::update(app, Action::HelpView(HelpViewAction::GoToLast));
                return;
            }

            (_, KeyCode::Esc) => {
                update::update(app, Action::App(AppAction::CloseHelp));
                return;
            }

            _ => {}
        },
    }

    // Global keymaps
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            update::update(app, Action::App(AppAction::Quit));
            return;
        }
        (_, KeyCode::Tab) => {
            update::update(app, Action::App(AppAction::CyclePane));
            return;
        }
        (_, KeyCode::Char(':')) => {
            update::update(app, Action::App(AppAction::SetCommandMode));
            return;
        }
        (_, KeyCode::Char('?')) => {
            update::update(app, Action::App(AppAction::OpenHelp));
            return;
        }
        _ => {}
    }

    return;
}
