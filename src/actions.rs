#[derive(Debug, Clone)]
pub enum Action {
    App(AppAction),
    Explorer(ExplorerAction),
    ResultsTable(ResultsTableAction),
    Db(DbAction),
    JsonView(JsonViewAction),
    None,
}

#[derive(Debug, Clone)]
pub enum AppAction {
    Quit,
    Tick,
    CyclePane,
    SelectTable(String),
    Resize(u16, u16),
    ViewSelectedRowAsJson,
    ClosePopup,
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
    YankSelection,
}

#[derive(Debug, Clone)]
pub enum DbAction {
    QueryTable(String),
    NextPage,
    PrevPage,
}

#[derive(Debug, Clone)]
pub enum JsonViewAction {
    MoveUp,
    MoveDown,
    GoToFirst,
}