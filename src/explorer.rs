use crate::ui::explorer_view::ExplorerItem;

#[derive(Debug, Default)]
pub struct ExplorerState {
    /// The item under the cursor.
    pub focused_item: Option<ExplorerItem>,
    /// The item the user chose to select.
    pub selected_item: Option<ExplorerItem>,
    /// All items in the tree.
    pub items: Vec<ExplorerItem>,
}
