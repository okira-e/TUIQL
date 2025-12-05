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
    Resize(u16, u16),
}

#[derive(Debug, Clone)]
pub enum ExplorerAction {
    MoveUp,
    MoveDown,
    ExpandNextItemType,
}

#[derive(Debug, Clone)]
pub enum ResultsTableAction {
    MoveUp,
    MoveDown,
    ScrollLeft,
    ScrollRight,
    JumpUp,
    JumpDown,
    GoToFirst,
    GoToLast,
}

#[derive(Debug, Clone)]
pub enum DbAction {
    QueryTable(String),
    NextPage,
    PrevPage,
}
