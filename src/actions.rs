use crate::db::QueryResult;

#[derive(Debug, Clone)]
pub enum Action {
    App(AppAction),
    Explorer(ExplorerAction),
    ResultsTable(ResultsTableAction),
    Db(DbAction),
    None,
}

#[derive(Debug, Clone)]
pub enum AppAction {
    Quit,
    CyclePane,
    SelectTable(String),
}

#[derive(Debug, Clone)]
pub enum QueryAction {
    Entity(String),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ExplorerAction {
    MoveUp,
    MoveDown,
    MoveToTop,
    MoveToBottom,
    MoveHalfPageUp,
    MoveHalfPageDown,
    MoveToNextMatch,
    MoveToPrevMatch,
    ClearSearch,
    ExpandNextItemType,
}

#[derive(Debug, Clone)]
pub enum ResultsTableAction {
  SetResults(QueryResult, usize, usize),
}

#[derive(Debug, Clone)]
pub enum DbAction {
    QueryTable(String),
    QueryStatement(String),
    NextPage,
    PrevPage,
}
