#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    pub name: String,
    pub kind: ExplorerItemKind,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerItemKind {
    Table,
    View,
}

impl ExplorerItemKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Table => "Tables",
            Self::View => "Views",
        }
    }

    pub fn arrow(&self, focused: bool) -> &'static str {
        if focused { "▼" } else { "▶" }
    }
}

#[derive(Debug, Default)]
pub struct ExplorerModel {
    /// The item under the cursor.
    pub focused_item: Option<ExplorerItem>,
    /// All items in the tree.
    pub items: Vec<ExplorerItem>,
}

pub fn get_items_by_kind(
    items: &Vec<ExplorerItem>,
    item_type: &ExplorerItemKind,
) -> Vec<ExplorerItem> {
    return items
        .iter()
        .filter(|e| e.kind == *item_type)
        .cloned()
        .collect();
}

pub fn get_next_item_kind(current_type: &ExplorerItemKind) -> ExplorerItemKind {
    return match current_type {
        ExplorerItemKind::Table => ExplorerItemKind::View,
        ExplorerItemKind::View => ExplorerItemKind::Table,
    };
}

pub fn get_prev_item_kind(current_type: &ExplorerItemKind) -> ExplorerItemKind {
    return match current_type {
        ExplorerItemKind::Table => ExplorerItemKind::View,
        ExplorerItemKind::View => ExplorerItemKind::Table,
    };
}
