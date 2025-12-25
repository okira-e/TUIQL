use ratatui::layout::Constraint;
use serde_json::Value;

use crate::drivers::{ColumnMetadata, QueryResult};


#[derive(Debug, Default)]
pub struct TableModel {
    pub table_name: String,
    pub query_result: QueryResult,
    pub results_row_count: usize,
    pub total_row_count: usize,
    pub current_pos: usize,
    pub selected_row: Option<usize>,
    /// Dictates how many columns to skip horizontally
    pub horizontal_scroll_offset: usize,
    /// First visible row in viewport
    pub vertical_scroll_offset: usize, // Is needed since ratatui's table state sets the offset incorrectly when we initialize it on every render
}

impl TableModel {
    pub fn reset(&mut self, selected_row: Option<usize>) {
        self.horizontal_scroll_offset = 0;
        self.vertical_scroll_offset = 0;
        self.selected_row = selected_row;
    }
    
    pub fn get_visible_cols(&self, width: u16) -> (Vec<ColumnMetadata>, Vec<Constraint>) {
        let mut visible_cols = vec![];
        let mut widths = vec![];
        let mut width_so_far = 0;
        for col in self.query_result
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
    
    pub fn should_draw_scrollbar(&self, width: u16) -> bool {
        let (visible_cols, _) = self.get_visible_cols(width); 
        return visible_cols.len() < self.query_result.columns.len();
    }
    
    pub fn get_selected_row_data(&self) -> Option<Value> {
        if self.query_result.rows.is_empty() {
            return None;
        }
        
        match self.selected_row {
            None => return None,
            Some(pos) => {
                return Some(self.query_result.rows[pos].clone());
            },
        }

    }
}