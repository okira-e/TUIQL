use crate::{
    actions::Action,
    config::Settings,
    drivers::{self, DbDriver},
    models::{
        explorer::{ExplorerItem, ExplorerModel}, json_view::JsonViewModel, statusline::{MsgLifetime, StatusLineMode, StatusLineModel, StatusLineMsg, MsgKind}, table::TableModel
    },
    theme::{Flavor, Theme},
};
use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::{DefaultTerminal, layout::{Constraint, Direction, Layout, Rect}};
use tokio::sync::mpsc;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum View {
    Explorer,
    ResultsTable,
    StatusLine,
    JsonView,
}

#[derive(Default)]
pub struct WidgetsChunks {
    pub explorer_chunk: Rect,
    pub table_chunk: Rect,
    pub json_view_chunk: Rect,
    pub statusline_chunk: Rect,
}

pub struct App {
    pub running: bool,
    pub event_stream: EventStream,
    pub action_tx: mpsc::UnboundedSender<Action>,
    pub action_rx: mpsc::UnboundedReceiver<Action>,
    pub focused_view: View,
    pub widgets_chunks: WidgetsChunks,
    pub table_model: TableModel,
    pub explorer_model: ExplorerModel,
    pub statusline_model: StatusLineModel,
    pub json_view_model: JsonViewModel,
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
            widgets_chunks: WidgetsChunks::default(),
            table_model: TableModel::default(),
            explorer_model: ExplorerModel::default(),
            statusline_model: StatusLineModel::new(),
            json_view_model: JsonViewModel::default(),
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
        self.calculate_widgets_chunks();

        while self.running {
            terminal.draw(|frame| {
                self.render(frame);
            })?;

            self.handle_events().await?;
        }

        return Ok(());
    }
    
    pub fn quit(&mut self) {
        self.running = false;
    }
    
    pub fn calculate_widgets_chunks(&mut self) {
        let app_statusline_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Max(1)
            ])
            .split(self.area);
        
        let explorer_table_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Min(1)
            ])
            .split(app_statusline_split[0]);

        let (explorer_split, table_split) = (explorer_table_split[0], explorer_table_split[1]);

        self.widgets_chunks.explorer_chunk = explorer_split;
        self.widgets_chunks.table_chunk = table_split;
        self.widgets_chunks.json_view_chunk = table_split;
        self.widgets_chunks.statusline_chunk = app_statusline_split[1];
    }
    
    pub fn report_message(&mut self, text: impl Into<String>, kind: MsgKind, lifetime: MsgLifetime) {
        self.statusline_model.mode = StatusLineMode::Status;
        self.statusline_model.msg = StatusLineMsg {
            text: text.into(),
            kind,
            lifetime,
            created_at: std::time::Instant::now(),
        };
    }
}
