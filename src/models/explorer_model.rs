#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    pub name: String,
    pub kind: String,
    pub index: usize,
}


#[derive(Debug, Default)]
pub struct ExplorerModel {
    /// The item under the cursor.
    pub focused_item: Option<ExplorerItem>,
    /// The item the user chose to select.
    pub selected_item: Option<ExplorerItem>,
    /// All items in the tree.
    pub items: Vec<ExplorerItem>,
}