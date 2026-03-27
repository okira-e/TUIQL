use crate::drivers::ColumnMetadata;
use crate::drivers::QueryResult;
use crate::settings::Settings;
use crate::settings::default_limit;
use ratatui::layout::Constraint;
use ratatui::widgets::TableState;
use serde_json::Value;

#[derive(Debug)]
pub struct TableModel {
    pub table_name: Option<String>,
    pub query_state: QueryState,
    pub query_result: QueryResult,
    /// Current view's row count. Rows that got fetched
    pub results_row_count: usize,
    /// Total count of data (including filteration)
    pub total_count: Option<usize>,
    /// Dictates how many columns to skip horizontally
    pub horizontal_scroll_offset: usize,
    pub ratatui_table_state: TableState,
    pub current_page: usize,
}

impl Default for TableModel {
    fn default() -> Self {
        Self {
            table_name: Default::default(),
            query_state: Default::default(),
            query_result: QueryResult::default(),
            results_row_count: Default::default(),
            total_count: Default::default(),
            horizontal_scroll_offset: Default::default(),
            ratatui_table_state: Default::default(),
            current_page: Default::default(),
        }
    }
}

impl TableModel {
    pub fn new(settings: &Settings) -> Self {
        Self {
            table_name: Default::default(),
            query_state: QueryState::new(settings),
            query_result: QueryResult::default(),
            results_row_count: Default::default(),
            total_count: Default::default(),
            horizontal_scroll_offset: Default::default(),
            ratatui_table_state: Default::default(),
            current_page: Default::default(),
        }
    }
}

impl TableModel {
    pub fn reset_ui(&mut self, selected_row: Option<usize>) {
        self.horizontal_scroll_offset = 0;
        *self.ratatui_table_state.offset_mut() = 0;
        self.ratatui_table_state.select(selected_row);
    }

    pub fn get_visible_cols(&self, width: u16) -> (Vec<ColumnMetadata>, Vec<Constraint>) {
        let mut visible_cols = vec![];
        let mut widths = vec![];
        let mut width_so_far = 0;
        for col in self.query_result.columns.iter().skip(self.horizontal_scroll_offset) {
            let w: u16 = if col.data_type == "integer" {
                20
            } else if col.data_type == "text" {
                35
            } else {
                30
            };

            if width_so_far + w < width {
                width_so_far += w;
                widths.push(Constraint::Max(w));
                visible_cols.push(col.clone());
            } else {
                break;
            }
        }

        return (visible_cols, widths);
    }

    pub fn should_draw_scrollbar(&self, width: u16) -> bool {
        let (visible_cols, _) = self.get_visible_cols(width);
        return visible_cols.len() < self.query_result.columns.len();
    }

    pub fn get_selected_row_data(&self) -> Option<Value> {
        if self.query_result.rows.is_empty() {
            return None;
        }

        match self.ratatui_table_state.selected() {
            None => return None,
            Some(pos) => {
                return Some(self.query_result.rows[pos].clone());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryState {
    pub offset: usize,
    pub limit: usize,
    pub order_by_clause: Option<String>,
    pub where_clause: Option<String>,
}

impl Default for QueryState {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: default_limit() as usize,
            order_by_clause: Default::default(),
            where_clause: None,
        }
    }
}

impl QueryState {
    pub fn new(settings: &Settings) -> Self {
        Self {
            offset: 0,
            limit: settings.default_limit as usize,
            order_by_clause: Default::default(),
            where_clause: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Calculate total number of pages given a total row count
    fn calculate_total_pages(query_state: &QueryState, total_count: usize) -> usize {
        if query_state.limit == 0 {
            return 0;
        }
        (total_count as f64 / query_state.limit as f64).ceil() as usize
    }

    /// Calculate the offset for the previous page
    fn prev_page_offset(query_state: &QueryState) -> usize {
        query_state.offset.saturating_sub(query_state.limit)
    }

    // Helper to create a test model with columns
    fn create_test_model() -> TableModel {
        let mut model = TableModel::default();
        model.query_result.columns = vec![
            ColumnMetadata { name: "id".into(), data_type: "integer".into() },
            ColumnMetadata { name: "name".into(), data_type: "text".into() },
            ColumnMetadata { name: "email".into(), data_type: "text".into() },
            ColumnMetadata { name: "created_at".into(), data_type: "timestamp".into() },
        ];

        return model;
    }

    // TableModel tests

    #[test]
    fn test_get_visible_cols_narrow_width() {
        let model = create_test_model();

        // With narrow width (25), should only show first column (integer=20)
        let (visible, widths) = model.get_visible_cols(25);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "id");
        assert_eq!(widths.len(), 1);
    }

    #[test]
    fn test_get_visible_cols_medium_width() {
        let model = create_test_model();

        // With width 60, should show id(20) + name(35) = 55 total
        let (visible, _) = model.get_visible_cols(60);
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].name, "id");
        assert_eq!(visible[1].name, "name");
    }

    #[test]
    fn test_get_visible_cols_wide_width() {
        let model = create_test_model();

        // With width 200, should show all columns
        let (visible, _) = model.get_visible_cols(200);
        assert_eq!(visible.len(), 4);
    }

    #[test]
    fn test_get_visible_cols_with_horizontal_scroll() {
        let mut model = create_test_model();
        model.horizontal_scroll_offset = 2; // Skip first 2 columns

        let (visible, _) = model.get_visible_cols(100);
        // Should start from "email" column
        assert_eq!(visible[0].name, "email");
    }

    #[test]
    fn test_get_selected_row_data_returns_correct_row() {
        let mut model = create_test_model();
        model.query_result.rows = vec![
            serde_json::json!({"id": 1, "name": "Alice"}),
            serde_json::json!({"id": 2, "name": "Bob"}),
            serde_json::json!({"id": 3, "name": "Charlie"}),
        ];
        model.ratatui_table_state.select(Some(1));

        let data = model.get_selected_row_data().unwrap();
        assert_eq!(data["id"], 2);
        assert_eq!(data["name"], "Bob");
    }

    // QueryState tests

    #[test]
    fn test_calculate_total_pages() {
        let qs = QueryState {
            offset: 0,
            limit: 50,
            order_by_clause: None,
            where_clause: None,
        };

        // Exact pages
        assert_eq!(calculate_total_pages(&qs, 200), 4);
        assert_eq!(calculate_total_pages(&qs, 50), 1);

        // Partial pages (requires ceil)
        assert_eq!(calculate_total_pages(&qs, 225), 5);
        assert_eq!(calculate_total_pages(&qs, 51), 2);
    }

    #[test]
    fn test_calculate_total_pages_edge_cases() {
        let qs = QueryState {
            offset: 0,
            limit: 50,
            order_by_clause: None,
            where_clause: None,
        };

        assert_eq!(calculate_total_pages(&qs, 0), 0);
        assert_eq!(calculate_total_pages(&qs, 1), 1);

        // Zero limit edge case
        let qs_zero = QueryState {
            offset: 0,
            limit: 0,
            order_by_clause: None,
            where_clause: None,
        };
        assert_eq!(calculate_total_pages(&qs_zero, 100), 0);
    }

    #[test]
    fn test_prev_page_offset_saturating() {
        // Test saturating_sub edge cases
        let qs = QueryState {
            offset: 0,
            limit: 50,
            order_by_clause: None,
            where_clause: None,
        };
        assert_eq!(prev_page_offset(&qs,), 0); // Should saturate at 0

        let qs = QueryState {
            offset: 25,
            limit: 50,
            order_by_clause: None,
            where_clause: None,
        };
        assert_eq!(prev_page_offset(&qs,), 0); // 25 - 50 would be negative, saturates to 0
    }
}
