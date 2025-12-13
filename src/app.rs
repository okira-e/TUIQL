use crate::{
    actions::Action,
    config::Settings,
    drivers::{self, DbDriver, QueryResult},
    models::{
        explorer_model::{ExplorerItem, ExplorerModel},
        results_table_model::ResultsTableModel,
        statusline_model::StatusLineModel,
    },
    theme::{Flavor, Theme},
};
use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::{DefaultTerminal, layout::Rect};
use tokio::sync::mpsc;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum View {
    Explorer,
    ResultsTable,
    StatusLine,
}

pub struct App {
    pub running: bool,
    pub event_stream: EventStream,
    pub action_tx: mpsc::UnboundedSender<Action>,
    pub action_rx: mpsc::UnboundedReceiver<Action>,
    pub focused_view: View,
    pub results_table_model: ResultsTableModel,
    pub explorer_model: ExplorerModel,
    pub statusline_model: StatusLineModel,
    pub query_result: QueryResult,
    pub settings: Settings,
    pub db_driver: Box<dyn DbDriver>,
    pub theme: Theme,
    pub selected_table: Option<String>,
    pub area: Rect,
}

impl App {
    pub async fn new(
        settings: Settings,
        db_driver: Box<dyn drivers::DbDriver>,
    ) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let theme = Theme::catppuccin(Flavor::Mocha);
        
        return Self {
            settings,
            running: false,
            event_stream: EventStream::new(),
            action_tx,
            action_rx,
            db_driver,
            theme,
            selected_table: None,
            focused_view: View::Explorer,
            query_result: QueryResult::default(),
            results_table_model: ResultsTableModel::default(),
            explorer_model: ExplorerModel::default(),
            statusline_model: StatusLineModel::new(),
            area: Rect::default(),
        };
    }

    pub async fn init(&mut self) -> Result<()> {
        // Populate the explorer state.
        let tables: Vec<String> = self.db_driver.get_tables().await?;
        let views: Vec<String> = self.db_driver.get_views().await?;

        let tables: Vec<ExplorerItem> = tables.iter().enumerate().map(|(i, name)| ExplorerItem {
            name: name.clone(),
            kind: "table".to_string(),
            index: i,
        }).collect();

        let views: Vec<ExplorerItem> = views.iter().enumerate().map(|(i, name)| ExplorerItem {
            name: name.clone(),
            kind: "view".to_string(),
            index: i,
        }).collect();
        
        let items: Vec<_> = tables.into_iter().chain(views).collect();
        self.explorer_model.items = items;
        self.explorer_model.focused_item = Some(
            self.explorer_model.items[0].clone()
        );

        return Ok(());
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        let size = terminal.size()?;
        self.area = Rect::new(0, 0, size.width, size.height);

        while self.running {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            self.handle_events().await?;
        }

        return Ok(());
    }
    
    pub fn quit(&mut self) {
        self.running = false;
    }
}
