use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::DefaultTerminal;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::actions::Action;
use crate::config::Settings;
use crate::drivers::DbDriver;
use crate::drivers::{self};
use crate::models::explorer::ExplorerItem;
use crate::models::explorer::ExplorerItemKind;
use crate::models::explorer::ExplorerModel;
use crate::models::json_view::JsonViewModel;
use crate::models::statusline::MsgKind;
use crate::models::statusline::MsgLifetime;
use crate::models::statusline::StatusLineMode;
use crate::models::statusline::StatusLineModel;
use crate::models::statusline::StatusLineMsg;
use crate::models::table::TableModel;
use crate::theme::Flavor;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum View {
    Explorer,
    ResultsTable,
    StatusLine,
    JsonView,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Pane {
    Left,
    Right,
}

pub enum RightView {
    ResultsTable,
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
    pub focused_pane: Pane,
    pub right_view: RightView,
    pub widgets_chunks: WidgetsChunks,
    pub table_model: TableModel,
    pub explorer_model: ExplorerModel,
    pub statusline_model: StatusLineModel,
    pub json_view_model: JsonViewModel,
    pub settings: Settings,
    pub db_driver: Arc<Mutex<Box<dyn DbDriver>>>,
    pub theme: Theme,
    pub selected_table: Option<String>,
    pub area: Rect,
}

impl App {
    pub async fn new(settings: Settings, db_driver: Box<dyn drivers::DbDriver>) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let theme = Theme::catppuccin(Flavor::Mocha);

        return Self {
            settings,
            running: false,
            event_stream: EventStream::new(),
            action_tx,
            action_rx,
            db_driver: Arc::new(Mutex::new(db_driver)),
            theme,
            selected_table: None,
            focused_pane: Pane::Left,
            right_view: RightView::ResultsTable,
            // active_temporary_widget: None,
            widgets_chunks: WidgetsChunks::default(),
            table_model: TableModel::default(),
            explorer_model: ExplorerModel::default(),
            statusline_model: StatusLineModel::default(),
            json_view_model: JsonViewModel::default(),
            area: Rect::default(),
        };
    }

    pub async fn init(&mut self) -> Result<()> {
        // Populate the explorer state.
        let driver = self.db_driver.lock().await;
        let tables: Vec<String> = driver.get_tables().await?;
        let views: Vec<String> = driver.get_views().await?;
        drop(driver);

        let tables: Vec<ExplorerItem> = tables
            .iter()
            .enumerate()
            .map(|(i, name)| ExplorerItem {
                name: name.clone(),
                kind: ExplorerItemKind::Table,
                index: i,
            })
            .collect();

        let views: Vec<ExplorerItem> = views
            .iter()
            .enumerate()
            .map(|(i, name)| ExplorerItem {
                name: name.clone(),
                kind: ExplorerItemKind::View,
                index: i,
            })
            .collect();

        let items: Vec<_> = tables.into_iter().chain(views).collect();
        self.explorer_model.items = items;
        self.explorer_model.focused_item = Some(self.explorer_model.items[0].clone());

        return Ok(());
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        let size = terminal.size()?;
        self.area = Rect::new(0, 0, size.width, size.height);
        self.calculate_widgets_chunks();

        while self.running {
            self.handle_events().await?;

            terminal.draw(|frame| {
                self.render(frame);
            })?;
        }

        return Ok(());
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn calculate_widgets_chunks(&mut self) {
        let app_statusline_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(1)])
            .split(self.area);

        let explorer_table_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Min(1)])
            .split(app_statusline_split[0]);

        let (explorer_split, table_split) = (explorer_table_split[0], explorer_table_split[1]);

        self.widgets_chunks.explorer_chunk = explorer_split;
        self.widgets_chunks.table_chunk = table_split;
        self.widgets_chunks.json_view_chunk = table_split;
        self.widgets_chunks.statusline_chunk = app_statusline_split[1];
    }

    pub fn report_message(
        &mut self,
        text: impl Into<String>,
        kind: MsgKind,
        lifetime: MsgLifetime,
    ) {
        self.statusline_model.mode = StatusLineMode::Status;
        self.statusline_model.msg = StatusLineMsg {
            text: text.into(),
            kind,
            lifetime,
            created_at: std::time::Instant::now(),
        };
    }

    pub fn get_focused_view(&self) -> View {
        return match self.focused_pane {
            Pane::Left => View::Explorer,
            Pane::Right => match self.right_view {
                RightView::ResultsTable => View::ResultsTable,
                RightView::JsonView => View::JsonView,
            },
        };
    }
}
