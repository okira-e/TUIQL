use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::DefaultTerminal;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

use crate::{
    actions::{Action, AppAction, DbAction, ResultsTableAction},
    config::Settings,
    db::{self, DbConnection},
    query_state::Query,
    theme::{Flavor, Theme},
    ui::{Pane, UI},
};

pub struct App {
    pub running: bool,
    pub event_stream: EventStream,
    pub action_tx: mpsc::UnboundedSender<Action>,
    pub action_rx: mpsc::UnboundedReceiver<Action>,
    pub ui: UI,
    pub results: db::QueryResult,
    pub query_state: Query,
    settings: Settings,
    db_conn: Arc<dyn DbConnection>,
    theme: Theme,
}

impl App {
    pub async fn new(
        settings: Settings,
        db_conn: Arc<dyn db::DbConnection>,
    ) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let theme = Theme::catppuccin(Flavor::Mocha);

        let ui = UI::new();
        
        return Self {
            settings,
            running: false,
            event_stream: EventStream::new(),
            action_tx,
            action_rx,
            db_conn,
            ui,
            results: db::QueryResult::default(),
            query_state: Query::new(),
            theme,
        };
    }

    pub async fn init(&mut self) -> Result<()> {
        let tables: Vec<String> = self.db_conn.get_tables().await?;
        let views: Vec<String> = self.db_conn.get_views().await?;

        self.ui.explorer.set_items(tables, views);

        return Ok(());
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;

        while self.running {
            terminal.draw(|frame| {
                self.ui.draw(&self.theme, frame);
            })?;

            self.handle_events().await?;
        }

        return Ok(());
    }

    pub async fn update(&mut self, action: Action) -> Result<()> {
        debug!("Received action: {:?}", action);

        match action {
            Action::App(action) => self.handle_app_action(action).await?,
            Action::Db(action) => self.handle_db_action(action).await?,
            Action::None => {},
            _ => self.ui.update(action),
        };
        
        return Ok(());
    }
    
    async fn handle_app_action(&mut self, action: AppAction) -> Result<()> {
        match action {
            AppAction::Quit => {
                self.quit();
            }
            AppAction::CyclePane => {
                if self.ui.focused_pane == Pane::Left {
                    self.ui.focused_pane = Pane::Right;
                } else {
                    self.ui.focused_pane = Pane::Left;
                }
            },
            AppAction::SelectTable(name) => {
                self.query_state = Query::new();
                self.handle_db_action(DbAction::QueryTable(name)).await?;
            },
        }
        
        return Ok(());
    }

    async fn handle_db_action(&mut self, action: DbAction) -> Result<()> {
        match action {
            DbAction::QueryTable(name) => {
                // Sort by the primary key(s) by default. If no order by was specified
                // by the user.
                if self.query_state.order_by.is_empty() {
                    let pk_cols = self.db_conn.get_pk_columns(&name).await?;
                    if pk_cols.is_empty() {
                        self.query_state.order_by = String::new()
                    } else {
                        self.query_state.order_by = format!(" ORDER BY {}", pk_cols.join(", "));
                    }
                }

                self.results = self.db_conn.query_table(&name, &self.query_state).await?;
                self.ui.focused_pane = Pane::Right;
                self.ui.update(Action::ResultsTable(ResultsTableAction::SetResults(self.results.clone())));
            },
            DbAction::QueryStatement(_) => todo!(),
        };

        return Ok(());
    }
    
    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
