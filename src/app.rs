use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::AppCmd;
use crate::commander::Cmd;
use crate::commander::parse_cmd;
use crate::config::Settings;
use crate::drivers;
use crate::drivers::DbDriver;
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
use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::DefaultTerminal;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

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
    StatusLine,
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
    /// Since the driver is behind a mutex, we get automatic serialization of requests that
    /// throttles database actions to just one at a time.
    ///
    /// If we wanted to allow for concurrent requests to the database we could use a semaphore
    /// instead.
    pub db_driver: Arc<Mutex<Box<dyn DbDriver>>>,
    pub theme: Theme,
    pub selected_table: Option<String>,
    pub area: Rect,
    pub is_loading: bool,
    pub prev_pane: Pane,
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
            is_loading: false,
            prev_pane: Pane::Left,
        };
    }

    pub async fn init(&mut self) -> Result<()> {
        // Populate the explorer state.
        self.populate_explorer_state().await?;

        return Ok(());
    }

    async fn populate_explorer_state(&mut self) -> Result<()> {
        let driver = self.db_driver.lock().await;
        let tables: Vec<String> = driver.get_tables().await?;
        let views: Vec<String> = driver.get_views().await?;
        let materialized: Vec<String> = driver.get_mateialized_views().await?;
        drop(driver); // unlock the mutex

        let tables: Vec<ExplorerItem> = tables
            .iter()
            .enumerate()
            .map(|(i, name)| ExplorerItem { name: name.clone(), kind: ExplorerItemKind::Table, index: i })
            .collect();

        let views: Vec<ExplorerItem> = views
            .iter()
            .enumerate()
            .map(|(i, name)| ExplorerItem { name: name.clone(), kind: ExplorerItemKind::View, index: i })
            .collect();

        let materialized: Vec<ExplorerItem> = materialized
            .iter()
            .enumerate()
            .map(|(i, name)| ExplorerItem {
                name: name.clone(),
                kind: ExplorerItemKind::MaterializedView,
                index: i,
            })
            .collect();

        let items: Vec<_> = tables.into_iter().chain(views).chain(materialized).collect();

        self.explorer_model.items = items;
        if !self.explorer_model.items.is_empty() {
            self.explorer_model.focused_item = Some(self.explorer_model.items[0].clone());
            self.explorer_model.table_state.select(Some(0));
        }

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

    pub fn report_message(&mut self, text: &str, kind: MsgKind, lifetime: MsgLifetime) {
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
            Pane::StatusLine => View::StatusLine,
        };
    }

    pub fn evaluate_app_action_from_cmd(&mut self, cmd: &str) -> Result<Action> {
        return match parse_cmd(cmd)? {
            Cmd::Quit => Ok(Action::App(AppAction::Quit)),
            Cmd::Count => Ok(Action::Cmd(AppCmd::Count)),
            Cmd::Goto(sub_cmd) => Ok(Action::Cmd(AppCmd::Goto(sub_cmd))),
            Cmd::Sort(column, direction) => Ok(Action::Cmd(AppCmd::Sort(column, direction.into()))),
            Cmd::Limit(limit) => Ok(Action::Cmd(AppCmd::Limit(limit))),
        };
    }

    pub fn focus_pane(&mut self, pane: Pane) {
        self.prev_pane = self.focused_pane;
        self.focused_pane = pane;
    }
}
