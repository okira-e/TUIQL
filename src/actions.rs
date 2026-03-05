use std::fmt;

use crate::commander::GotoCmd;
use crate::drivers::OrderByDirection;
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

pub enum DbAction {
    QueryTable(String),
    QueryCount,
    QueryCountComplete(Result<usize>),
    QueryTableComplete(String, QueryResult),
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
    MoveLeft,
    MoveRight,
    Execute,
    Exit,
}

#[derive(Debug)]
pub enum AppCmd {
    Count,
    Goto(GotoCmd),
    Sort(Option<String>, OrderByDirection),
    Limit(usize),
}

// Implementing this so that we don't get the entire result object in the log file with enum variants
// that have QueryResult in their payloads
impl fmt::Debug for DbAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbAction::QueryTable(s) => f.debug_tuple("QueryTable").field(s).finish(),
            DbAction::QueryCount => f.write_str("QueryCount"),
            DbAction::QueryCountComplete(r) => f.debug_tuple("QueryCountComplete").field(r).finish(),

            DbAction::QueryTableComplete(name, _) => f
                .debug_tuple("QueryTableComplete")
                .field(name)
                .field(&"<QueryResult>")
                .finish(),
            DbAction::NextPage => f.write_str("NextPage"),
            DbAction::NextPageComplete(_, page) => f
                .debug_tuple("NextPageComplete")
                .field(&"<QueryResult>")
                .field(page)
                .finish(),
            DbAction::PrevPage => f.write_str("PrevPage"),
            DbAction::PrevPageComplete(_, offset) => f.debug_tuple("PrevPageComplete").field(&"<QueryResult>").field(offset).finish(),
            DbAction::GotoPageComplete(_, page, total) => f
                .debug_tuple("GotoPageComplete")
                .field(&"<QueryResult>")
                .field(page)
                .field(total)
                .finish(),
        }
    }
}
