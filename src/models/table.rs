use crate::drivers::ColumnMetadata;
use crate::drivers::QueryResult;
use ratatui::layout::Constraint;
use ratatui::widgets::TableState;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct TableModel {
    pub table_name: String,
    pub query_result: QueryResult,
    pub results_row_count: usize,
    pub current_pos: usize,
    /// Dictates how many columns to skip horizontally
    pub horizontal_scroll_offset: usize,
    pub table_state: TableState,
    pub current_page: usize,
}

impl TableModel {
    pub fn reset(&mut self, selected_row: Option<usize>) {
        self.horizontal_scroll_offset = 0;
        *self.table_state.offset_mut() = 0;
        self.table_state.select(selected_row);
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

        match self.table_state.selected() {
            None => return None,
            Some(pos) => {
                return Some(self.query_result.rows[pos].clone());
            }
        }
    }
}
