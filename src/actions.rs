use crate::commander::GotoCmd;
use crate::drivers::QueryResult;
use crate::models::statusline_model::MsgKind;
use crate::models::statusline_model::MsgLifetime;
use color_eyre::Result;
use color_eyre::eyre;
use std::fmt;

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
    /// Updates the where and the order_by in the query state respectively
    UpdateQueryState(Option<String>, Option<String>),
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

pub enum DbAction {
    QueryTable,
    /// Takes a flag for ignoring the applied query state filters
    QueryCount(bool),
    /// First flag is for if we ignored the applied query state filters
    QueryCountComplete(bool, Result<usize>),
    QueryTableComplete(QueryResult),
    NextPage,
    NextPageComplete(QueryResult, usize),
    PrevPage,
    PrevPageComplete(QueryResult, usize),
    GotoPageComplete(QueryResult, usize, usize),
}

#[derive(Debug)]
pub enum JsonViewAction {
    MoveUp,
    MoveDown,
    GoToFirst,
}

/// This is key actions for the statusline when the user has it on focus.
#[derive(Debug)]
pub enum CmdLineAction {
    AddChar(char),
    PopChar,
    PopWord,
    PopLine,
    MoveLeft,
    MoveRight,
    MoveLeftWord,
    MoveRightWord,
    SetText(String),
    TogglePrevCommand,
    ToggleNextCommand,
    Execute,
    Exit,
}

#[derive(Debug)]
pub enum AppCmd {
    Count,
    TotalCount,
    Goto(GotoCmd),
    OrderBy(Option<String>),
    Where(Option<String>),
    Limit(usize),
    SettingChange(String, Option<String>),
}

// Implementing this so that we don't get the entire result object in the log file with enum variants
// that have QueryResult in their payloads
impl fmt::Debug for DbAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbAction::QueryTable => f.write_str("QueryTable"),
            DbAction::QueryCount(ignore_filters) => f.debug_tuple("QueryCount").field(&ignore_filters).finish(),
            DbAction::QueryCountComplete(ignored_filters, r) => f
                .debug_tuple("QueryCountComplete")
                .field(&ignored_filters)
                .field(&r)
                .finish(),

            DbAction::QueryTableComplete(_) => f.debug_tuple("QueryTableComplete").field(&"<QueryResult>").finish(),
            DbAction::NextPage => f.write_str("NextPage"),
            DbAction::NextPageComplete(_, page) => f
                .debug_tuple("NextPageComplete")
                .field(&"<QueryResult>")
                .field(page)
                .finish(),
            DbAction::PrevPage => f.write_str("PrevPage"),
            DbAction::PrevPageComplete(_, offset) => f
                .debug_tuple("PrevPageComplete")
                .field(&"<QueryResult>")
                .field(offset)
                .finish(),
            DbAction::GotoPageComplete(_, page, total) => f
                .debug_tuple("GotoPageComplete")
                .field(&"<QueryResult>")
                .field(page)
                .field(total)
                .finish(),
        }
    }
}
