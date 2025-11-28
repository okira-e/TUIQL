use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;
use tracing::debug;

use crate::{
    actions::{Action, AppAction, DbAction, ResultsTableAction},
    config::Settings,
    drivers::{self, DbDriver},
    theme::{Flavor, Theme},
    ui::{Pane, UI},
};

pub struct App {
    pub running: bool,
    pub event_stream: EventStream,
    pub action_tx: mpsc::UnboundedSender<Action>,
    pub action_rx: mpsc::UnboundedReceiver<Action>,
    pub ui: UI,
    // pub results: drivers::QueryResult,
    settings: Settings,
    db_driver: Box<dyn DbDriver>,
    theme: Theme,
    selected_table: Option<String>,
    selected_table_row_count: usize,
}

impl App {
    pub async fn new(
        settings: Settings,
        db_driver: Box<dyn drivers::DbDriver>,
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
            db_driver,
            ui,
            // results: drivers::QueryResult::default(),
            theme,
            selected_table: None,
            selected_table_row_count: 0,
        };
    }

    pub async fn init(&mut self) -> Result<()> {
        let tables: Vec<String> = self.db_driver.get_tables().await?;
        let views: Vec<String> = self.db_driver.get_views().await?;

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
    
    pub fn quit(&mut self) {
        self.running = false;
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
                self.db_driver.reset_query_state();
                self.selected_table_row_count = 0;
                self.handle_db_action(DbAction::QueryTable(name)).await?;
            },
        }
        
        return Ok(());
    }

    async fn handle_db_action(&mut self, action: DbAction) -> Result<()> {
        match action {
            DbAction::QueryTable(table_name) => {
                self.selected_table = Some(table_name.to_owned());

                let results = self.db_driver.query(&table_name).await?;

                if self.selected_table_row_count == 0 {
                    self.selected_table_row_count = self.db_driver.query_count(&table_name).await?;
                }
                
                self.ui.focused_pane = Pane::Right;
                self.ui.update(
                    Action::ResultsTable(
                        ResultsTableAction::SetResults(
                            results,
                            self.selected_table_row_count,
                            self.db_driver.get_current_page(&table_name).await?,
                        )
                    )
                );
            },
            DbAction::NextPage => {
                if let Some(selected_table) = &self.selected_table {
                    self.db_driver.next_page(
                        &selected_table,
                        self.selected_table_row_count,
                    ).await?;
                    
                    let results = self.db_driver.query(selected_table).await?;
                    
                    // @Todo: Actions sent this way do not go through the actions channel which means
                    // we can't apply logging to them for example. We need a fix for recursive calls here.
                    if !results.rows.is_empty() {
                        self.ui.update(
                            Action::ResultsTable(
                                ResultsTableAction::SetResults(
                                    results,
                                    self.selected_table_row_count,
                                    self.db_driver.get_current_page(&selected_table).await?,
                                )
                            )
                        );
                    }
                }
            }
            DbAction::PrevPage => {
                if let Some(selected_table) = &self.selected_table {
                    self.db_driver.prev_page(&selected_table).await?;

                    let results = self.db_driver.query(&selected_table).await?;

                    if !results.rows.is_empty() {
                        self.ui.update(
                            Action::ResultsTable(
                                ResultsTableAction::SetResults(
                                    results,
                                    self.selected_table_row_count,
                                    self.db_driver.get_current_page(&selected_table).await?,
                                )
                            )
                        );
                    }
                }
            }
            DbAction::QueryStatement(_) => todo!(),
        };

        return Ok(());
    }
}
