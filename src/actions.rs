use crate::drivers::QueryResult;
use color_eyre::eyre;

#[derive(Debug)]
pub enum Action {
    App(AppAction),
    Explorer(ExplorerAction),
    ResultsTable(ResultsTableAction),
    Db(DbAction),
    JsonView(JsonViewAction),
    Command(CommandAction),
    None,
}

#[derive(Debug)]
pub enum AppAction {
    Quit,
    Tick,
    CyclePane,
    SelectTable(String),
    Resize(u16, u16),
    ViewSelectedRowAsJson,
    CloseJsonView,
    SetCommandMode,
    ReportError(eyre::Report),
    StopLoading,
}

#[derive(Debug, Clone)]
pub enum ExplorerAction {
    MoveUp,
    MoveDown,
    NextTab,
    PrevTab,
    GoToFirst,
    GoToLast,
}

#[derive(Debug, Clone)]
pub enum ResultsTableAction {
    MoveUp,
    MoveDown,
    ScrollLeft,
    ScrollRight,
    JumpUp,
    JumpDown,
    GoToFirstVertically,
    GoToLastVertically,
    GoToFirstHorizontally,
    GoToLastHorizontally,
    YankSelection,
}

#[derive(Debug, Clone)]
pub enum DbAction {
    QueryTable(String),
    QueryTableComplete(String, QueryResult, usize),
    NextPage,
    NextPageComplete(String, QueryResult, usize, usize),
    PrevPage,
    PrevPageComplete(String, QueryResult, usize, usize),
}

#[derive(Debug, Clone)]
pub enum JsonViewAction {
    MoveUp,
    MoveDown,
    GoToFirst,
}

#[derive(Debug, Clone)]
pub enum CommandAction {
    AddChar(char),
    PopChar,
    MoveLeft,
    MoveRight,
    Execute,
}
