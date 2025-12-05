use std::cell::RefCell;

use ratatui::{layout::Constraint, widgets::TableState};

use crate::drivers::{ColumnMetadata, QueryResult};

#[derive(Debug, Default)]
pub struct ResultsTableState {
    pub table_name: String,
    pub results_row_count: usize,
    pub total_row_count: usize,
    pub current_pos: usize,
    /// Dictates how many columns to skip horizontally
    pub horizontal_scroll_offset: usize,
    pub ratatui_table_state: RefCell<TableState>,
}

impl ResultsTableState {
    pub fn get_visible_cols(&self, query_result: &QueryResult, width: u16) -> (Vec<ColumnMetadata>, Vec<Constraint>) {
        let mut visible_cols = vec![];
        let mut widths = vec![];
        let mut width_so_far = 0;
        for col in query_result
            .columns
            .iter()
            .skip(self.horizontal_scroll_offset)
        {
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
    
    pub fn should_draw_scrollbar(&self, query_result: &QueryResult, width: u16) -> bool {
        let (visible_cols, _) = self.get_visible_cols(query_result, width); 
        return visible_cols.len() < query_result.columns.len();
    }
}
