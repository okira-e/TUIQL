#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    pub name: String,
    pub kind: ExplorerItemKind,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerItemKind {
    Table,
    View,
    MaterializedView,
}

impl Default for ExplorerItemKind {
    fn default() -> Self {
        return Self::Table;
    }
}

#[derive(Debug, Default)]
pub struct ExplorerModel {
    /// The item under the cursor
    pub focused_item: Option<ExplorerItem>,
    /// All fetched tables/views
    pub items: Vec<ExplorerItem>,
    pub table_state: ratatui::widgets::TableState,
    pub selected_tab: ExplorerItemKind,
    /// Horizontal scroll offset for long item names
    pub horizontal_scroll_offset: usize,
}

impl ExplorerModel {
    pub fn get_items_by_kind(&self, item_type: ExplorerItemKind) -> Vec<ExplorerItem> {
        return self
            .items
            .iter()
            .filter(|e| e.kind == item_type)
            .cloned()
            .collect();
    }
}
