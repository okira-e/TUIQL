use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::{Cell, Row, Table, TableState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use serde_json::Value;

use crate::actions::DbAction;
use crate::{
    actions::{Action, ResultsTableAction},
    drivers::QueryResult,
    theme::Theme,
};

pub struct TableView {
    pub title: String,
    columns: Vec<String>,
    query_result: Option<QueryResult>,
    results_row_count: usize,
    total_row_count: usize,
    offset: usize,
    /// Dictates how many columns to skip horizontally
    horizontal_scroll_offset: usize,
    table_state: RefCell<TableState>,
    draw_scrollbar: bool,
}

impl TableView {
    pub fn new(title: &str) -> Self {
        return Self {
            title: String::from(title),
            columns: vec![],
            query_result: None,
            results_row_count: 0,
            total_row_count: 0,
            offset: 0,
            horizontal_scroll_offset: 0,
            table_state: RefCell::new(TableState::default()),
            draw_scrollbar: false,
        };
    }

    pub fn draw(&mut self, theme: &Theme, frame: &mut Frame, area: Rect, focused: bool) {
        let container_border_style = if focused { theme.pane_focus } else { theme.bg };

        let row_status = format!(
            "{}-{}/{}",
            self.offset, self.results_row_count, self.total_row_count,
        );

        let container_block = Block::default()
            .title(self.title.clone())
            .title(Line::from(row_status.clone()).alignment(Alignment::Right))
            .border_style(container_border_style)
            .borders(Borders::ALL);

        frame.render_widget(&container_block, area);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(container_block.inner(area));

        let table_area = inner_layout[0];
        let horizontal_scroll_bar_area = inner_layout[1];

        // Early exit if no result or empty
        let no_results_message = Paragraph::new("No results to display")
            .style(theme.fg)
            .block(container_block.clone());

        let Some(query_result) = &self.query_result else {
            frame.render_widget(no_results_message, area);
            return;
        };

        if query_result.rows.is_empty() {
            frame.render_widget(no_results_message, area);
            return;
        }

        // Populate columns
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

            if width_so_far + w < area.width {
                width_so_far += w;
                widths.push(Constraint::Max(w));
                visible_cols.push(col.clone());
            } else {
                break;
            }
        }

        // Populate rows
        let visible_rows = query_result.rows.clone();
        let mut table_rows: Vec<Row> = vec![];
        for row in visible_rows.iter() {
            let row_map = row.as_object().unwrap();

            let mut values = vec![];
            for key in visible_cols.iter() {
                let cell = match row_map[&key.name].clone() {
                    Value::Null => Cell::from("null").style(
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                    Value::Bool(b) => {
                        Cell::from(b.to_string()).style(Style::default().fg(Color::Yellow))
                    }
                    Value::Number(number) => {
                        Cell::from(number.to_string()).style(Style::default().fg(Color::Cyan))
                    }
                    Value::String(s) => Cell::from(s.clone()).style(theme.fg),
                    Value::Array(values) => Cell::from(format!("[{} items]", values.len()))
                        .style(Style::default().fg(Color::Magenta)),
                    Value::Object(map) => Cell::from(format!("{{{} keys}}", map.len()))
                        .style(Style::default().fg(Color::Blue)),
                };

                values.push(cell);
            }

            table_rows.push(Row::new(values));
        }

        let constraints = vec![Constraint::Max(40); visible_cols.len()];
        let table = Table::new(table_rows, constraints)
            .header(
                Row::new(
                    visible_cols
                        .iter()
                        .map(|col| col.name.clone())
                        .collect::<Vec<String>>(),
                )
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1), // Space between header and rows
            )
            .row_highlight_style(if focused {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
            .widths(widths)
            .highlight_symbol("> ");

        frame.render_stateful_widget(table, table_area, &mut self.table_state.borrow_mut());

        //
        // Render the scrollbar
        //

        self.draw_scrollbar = visible_cols.len() < query_result.columns.len();

        if self.draw_scrollbar {
            let scrollbar_width = (1.0
                / (query_result.columns.len() as f32 / self.columns.len().max(1) as f32))
                * area.width as f32;

            let offset_width = (1.0 / query_result.columns.len() as f32) * area.width as f32;
            let scrollbar_offset =
                String::from(" ").repeat(offset_width as usize * self.horizontal_scroll_offset);

            let scrollbar = String::from("▃").repeat(
                scrollbar_width.round() as usize,
            );

            frame.render_widget(
                Span::from(scrollbar_offset + &scrollbar).style(theme.fg),
                horizontal_scroll_bar_area,
            );
        }
    }

    pub fn update(&mut self, action: ResultsTableAction) {
        match action {
            ResultsTableAction::SetResults(query_result, total_row_count, offset) => {
                self.results_row_count = query_result.rows.len();
                self.total_row_count = total_row_count;
                self.offset = offset;
                self.query_result = Some(query_result);
                self.table_state.borrow_mut().select(Some(0));
            }
        };
    }

    pub fn handle_key_event(&mut self, modifier: KeyModifiers, key: KeyCode) -> Action {
        let Some(query_result) = &self.query_result else {
            return Action::None;
        };

        let mut state = self.table_state.borrow_mut();
        let current = state.selected().unwrap_or(0);
        let total_rows = query_result.rows.len();

        if total_rows == 0 {
            return Action::None;
        }

        match (modifier, key) {
            // Up / k / Ctrl-p
            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                let new_index = if current == 0 { 0 } else { current - 1 };
                state.select(Some(new_index));
                Action::None
            }

            // Down / j / Ctrl-n
            (_, KeyCode::Char('j') | KeyCode::Down)
            | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                let new_index = if current + 1 >= total_rows {
                    total_rows
                } else {
                    current + 1
                };
                state.select(Some(new_index));
                Action::None
            }

            // Left / h
            (_, KeyCode::Char('h') | KeyCode::Left) => {
                if self.horizontal_scroll_offset > 0 {
                    self.horizontal_scroll_offset -= 1;
                }

                // let offset = query_result.columns.len() - visible_cols.len();
                // if offset > 0 {
                //     let col = query_result.columns[offset - 1].clone();
                //     visible_cols.insert(0, col);
                // }

                Action::None
            }

            // Right / l
            (_, KeyCode::Char('l') | KeyCode::Right) => {
                if  self.draw_scrollbar && self.horizontal_scroll_offset < query_result.columns.len() - 1
                {
                    self.horizontal_scroll_offset += 1;
                }

                Action::None
            }

            // Ctrl-u: jump ~10 rows up (clamp at 0)
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                let jump = 10;
                let new_index = current.saturating_sub(jump);
                state.select(Some(new_index));
                Action::None
            }

            // Ctrl-d: jump ~10 rows down (clamp at last)
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                let jump = 10;
                let mut new_index = current + jump;
                if new_index >= total_rows {
                    new_index = total_rows - 1;
                }
                state.select(Some(new_index));
                Action::None
            }

            // g: go to first row
            (_, KeyCode::Char('g')) => {
                state.select(Some(0));
                Action::None
            }

            // G: go to last row
            (_, KeyCode::Char('G')) => {
                state.select(Some(total_rows - 1));
                Action::None
            }

            (_, KeyCode::Char('n')) => {
                Action::Db(DbAction::NextPage)
            }

            (_, KeyCode::Char('p')) => {
                Action::Db(DbAction::PrevPage)
            }

            _ => Action::None,
        }
    }
}
