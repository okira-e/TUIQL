use crate::commander::GotoCmd;
use crate::drivers::QueryResult;
use crate::models::statusline::MsgKind;
use crate::models::statusline::MsgLifetime;
use color_eyre::Result;
use color_eyre::eyre;

#[derive(Debug)]
pub enum Action {
    App(AppAction),
    Explorer(ExplorerAction),
    ResultsTable(ResultsTableAction),
    Db(DbAction),
    JsonView(JsonViewAction),
    CmdLine(CmdLineAction),
    Cmd(AppCmd),
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
    ReportMessage(String, MsgKind, MsgLifetime),
    ReportError(eyre::Report),
    StartLoading,
    StopLoading,
}

#[derive(Debug)]
pub enum ExplorerAction {
    MoveUp,
    MoveDown,
    NextTab,
    PrevTab,
    GoToFirst,
    GoToLast,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum DbAction {
    QueryTable(String),
    QueryCount,
    QueryCountComplete(Result<usize>),
    QueryTableComplete(String, QueryResult),
    NextPage,
    NextPageComplete(QueryResult),
    PrevPage,
    PrevPageComplete(QueryResult),
    GotoPageComplete(QueryResult, usize),
}

#[derive(Debug)]
pub enum JsonViewAction {
    MoveUp,
    MoveDown,
    GoToFirst,
}

#[derive(Debug)]
pub enum CmdLineAction {
    AddChar(char),
    PopChar,
    MoveLeft,
    MoveRight,
    Execute,
    Exit,
}

#[derive(Debug)]
pub enum AppCmd {
    Count,
    Goto(GotoCmd),
}
