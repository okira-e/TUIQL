use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::AppCmd;
use crate::actions::DbAction;
use crate::commander::Cmd;
use crate::commander::parse_cmd;
use crate::config::project::ProjectConfig;
use crate::drivers;
use crate::drivers::DbDriver;
use crate::events;
use crate::models::explorer_model::ExplorerItem;
use crate::models::explorer_model::ExplorerItemKind;
use crate::models::explorer_model::ExplorerModel;
use crate::models::help_view_model::HelpViewModel;
use crate::models::json_view_model::JsonViewModel;
use crate::models::statusline_model::MsgKind;
use crate::models::statusline_model::MsgLifetime;
use crate::models::statusline_model::StatusLineMode;
use crate::models::statusline_model::StatusLineModel;
use crate::models::statusline_model::StatusLineMsg;
use crate::models::table_model::QueryState;
use crate::models::table_model::TableModel;
use crate::render;
use crate::settings::Settings;
use crate::suggestor::CompletionContext;
use crate::suggestor::suggest;
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
    Help,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Pane {
    Left,
    Right,
    StatusLine,
}

pub enum RightView {
    /// Always open with an empty data message.
    ResultsTable,
    JsonView,
    Help,
}

#[derive(Default)]
pub struct WidgetsChunks {
    pub explorer_chunk: Rect,
    pub table_chunk: Rect,
    pub json_view_chunk: Rect,
    pub statusline_chunk: Rect,
    pub help_view_chunk: Rect,
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
    pub help_view_model: HelpViewModel,
    pub settings: Settings,
    /// None if the user connected directly without saving the connection.
    pub config: Option<ProjectConfig>,
    /// Since the driver is behind a mutex, we get automatic serialization of requests that
    /// throttles database actions to just one at a time.
    ///
    /// If we wanted to allow for concurrent requests to the database we could use a semaphore
    /// instead.
    pub db_driver: Arc<Mutex<DbDriver>>,
    pub theme: Theme,
    pub area: Rect,
    pub is_loading: bool,
    pub prev_focused_pane: Pane,
}

impl App {
    pub async fn new(settings: Settings, db_driver: drivers::DbDriver, config: Option<ProjectConfig>) -> App {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let theme: Theme = match settings.theme.parse() {
            Ok(t) => t,
            Err(_) => Theme::catppuccin_mocha(),
        };

        let table_model = TableModel::new(&settings);
        let project_name = if let Some(ref config) = config {
            Some(config.name.clone())
        } else {
            None
        };

        return App {
            settings: settings,
            config: config,
            running: false,
            event_stream: EventStream::new(),
            action_tx: action_tx,
            action_rx: action_rx,
            db_driver: Arc::new(Mutex::new(db_driver)),
            theme,
            focused_pane: Pane::Left,
            right_view: RightView::ResultsTable,
            widgets_chunks: WidgetsChunks::default(),
            table_model: table_model,
            explorer_model: ExplorerModel::new(project_name),
            statusline_model: StatusLineModel::default(),
            json_view_model: JsonViewModel::default(),
            area: Rect::default(),
            help_view_model: HelpViewModel::default(),
            is_loading: false,
            prev_focused_pane: Pane::Left,
        };
    }
}

pub async fn init(app: &mut App) -> Result<()> {
    // Populate the explorer state.
    populate_explorer_state(app).await?;

    return Ok(());
}

async fn populate_explorer_state(app: &mut App) -> Result<()> {
    let driver = app.db_driver.lock().await;
    let tables: Vec<String> = driver.get_tables().await?;
    let views: Vec<String> = driver.get_views().await?;
    let materialized: Vec<String> = driver.get_materialized_views().await?;
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

    app.explorer_model.items = items;
    if !app.explorer_model.items.is_empty() {
        app.explorer_model.focused_item = Some(app.explorer_model.items[0].clone());
        app.explorer_model.table_state.select(Some(0));
    }

    return Ok(());
}

pub async fn run(mut app: App, mut terminal: DefaultTerminal) -> Result<()> {
    app.running = true;
    let size = terminal.size()?;
    app.area = Rect::new(0, 0, size.width, size.height);
    calculate_widgets_chunks(&mut app);

    while app.running {
        events::handle_events(&mut app).await?;

        terminal.draw(|frame| {
            render::render(&mut app, frame);
        })?;
    }

    return Ok(());
}

pub fn quit(app: &mut App) {
    app.running = false;
}

pub fn calculate_widgets_chunks(app: &mut App) {
    let app_statusline_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Max(1)])
        .split(Rect { x: 0, y: 0, width: app.area.width, height: app.area.height });

    let explorer_table_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Min(1)])
        .split(app_statusline_split[0]);

    let (explorer_split, table_split) = (explorer_table_split[0], explorer_table_split[1]);

    app.widgets_chunks.explorer_chunk = explorer_split;
    app.widgets_chunks.table_chunk = table_split;
    app.widgets_chunks.json_view_chunk = table_split;
    app.widgets_chunks.help_view_chunk = table_split;
    app.widgets_chunks.statusline_chunk = app_statusline_split[1];
}

pub fn report_message(app: &mut App, text: &str, kind: MsgKind, lifetime: MsgLifetime) {
    app.statusline_model.mode = StatusLineMode::Status;
    app.statusline_model.msg = StatusLineMsg {
        text: text.into(),
        kind,
        lifetime,
        created_at: std::time::Instant::now(),
    };
}

pub fn get_focused_view(app: &App) -> View {
    return match app.focused_pane {
        Pane::Left => View::Explorer,
        Pane::Right => match app.right_view {
            RightView::ResultsTable => View::ResultsTable,
            RightView::JsonView => View::JsonView,
            RightView::Help => View::Help,
        },
        Pane::StatusLine => View::StatusLine,
    };
}

pub fn evaluate_app_action_from_cmd(cmd: &str) -> Result<Action> {
    return match parse_cmd(cmd)? {
        Cmd::Quit => Ok(Action::App(AppAction::Quit)),
        Cmd::Count => Ok(Action::Cmd(AppCmd::Count)),
        Cmd::TotalCount => Ok(Action::Cmd(AppCmd::TotalCount)),
        Cmd::Goto(sub_cmd) => Ok(Action::Cmd(AppCmd::Goto(sub_cmd))),
        Cmd::OrderBy(clause) => Ok(Action::Cmd(AppCmd::OrderBy(clause))),
        Cmd::Where(clause) => Ok(Action::Cmd(AppCmd::Where(clause))),
        Cmd::Limit(limit) => Ok(Action::Cmd(AppCmd::Limit(limit))),
        Cmd::RefreshTable => Ok(Action::Db(DbAction::QueryTable)),
        Cmd::Set(key, value) => Ok(Action::Cmd(AppCmd::SettingChange(key, value))),
        Cmd::ChangeTheme(value) => Ok(Action::Cmd(AppCmd::ChangeTheme(value))),
        Cmd::OpenHelp => Ok(Action::App(AppAction::OpenHelp)),
        Cmd::SavePreset(name) => Ok(Action::Cmd(AppCmd::SavePreset(name))),
        Cmd::LoadPreset(name) => Ok(Action::Cmd(AppCmd::LoadPreset(name))),
        Cmd::RemovePreset(name) => Ok(Action::Cmd(AppCmd::RemovePreset(name))),
    };
}

pub fn focus_pane(app: &mut App, pane: Pane) {
    app.prev_focused_pane = app.focused_pane;
    app.focused_pane = pane;
}

pub fn select_table(app: &mut App, name: String) {
    app.table_model.table_name = Some(name.clone());
    app.table_model.reset_ui(Some(0));
    app.table_model.query_state = QueryState::new(&app.settings);

    let _ = app.action_tx.send(Action::Db(DbAction::QueryTable));
}

pub fn refresh_suggestions(app: &mut App) {
    let mut tables: Vec<&'_ str> = Vec::with_capacity(app.explorer_model.items.len());
    for item in app.explorer_model.items.iter() {
        if item.kind == ExplorerItemKind::Table {
            tables.push(item.name.as_str());
        }
    }

    let mut columns: Vec<&'_ str> = Vec::with_capacity(app.table_model.query_result.columns.len());
    for col in app.table_model.query_result.columns.iter() {
        columns.push(col.name.as_str());
    }

    let mut presets_for_table = vec![];

    if let Some(config) = &app.config {
        if let Some(table_name) = &app.table_model.table_name {
            for preset in config.presets.iter() {
                if &preset.table_name == table_name {
                    presets_for_table = preset.presets.keys().cloned().collect();
                }
            }
        }
    }

    let ctx = CompletionContext {
        tables: tables.as_ref(),
        columns: columns.as_ref(),
        preset_names: presets_for_table,
    };

    app.statusline_model.completion.candidates = suggest(&ctx, &app.statusline_model.cmd.text);
    app.statusline_model.completion.selected = None;
}
