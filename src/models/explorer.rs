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
    /// All items in the tree.
    pub items: Vec<ExplorerItem>,
}

pub fn get_items_by_type(items: &Vec<ExplorerItem>, item_type: &str) -> Vec<ExplorerItem> {
    return items
        .iter()
        .filter(|e| e.kind == item_type)
        .cloned()
        .collect();
}

pub fn get_next_item_type(current_type: &str) -> String {
    match current_type {
        "table" => "view".to_string(),
        "view" => "procedure".to_string(),
        "procedure" => "function".to_string(),
        "function" => "table".to_string(),
        _ => "table".to_string(),
    }
}

pub fn get_prev_item_type(current_type: &str) -> String {
    match current_type {
        "table" => "function".to_string(),
        "view" => "table".to_string(),
        "procedure" => "view".to_string(),
        "function" => "procedure".to_string(),
        _ => "table".to_string(),
    }
}