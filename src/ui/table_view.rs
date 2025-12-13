use ratatui::widgets::{Cell, Row, Table};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use serde_json::Value;

use crate::drivers::QueryResult;
use crate::models::results_table_model::ResultsTableModel;
use crate::{theme::Theme};

pub struct TableView {}

impl TableView {
    pub fn draw(
        state: &ResultsTableModel,
        theme: &Theme,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        query_result: &QueryResult,
    ) {
        let container_border_style = if focused {
            theme.pane_focus
        } else {
            theme.bg
        };

        let row_status = format!(
            "{}-{}/{}",
            state.current_pos, state.total_row_count, state.results_row_count,
        );

        let container_block = Block::default()
            .title("Query Result".to_string())
            .title(
                Line::from(
                    state.table_name.clone()
                )
                .style(theme.fg)
                .alignment(Alignment::Right)
            )
            .title_bottom(
                Line::from(
                    row_status.clone()
                )
                .style(theme.fg)
                .alignment(Alignment::Center)
            )
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

        if query_result.rows.is_empty() {
            frame.render_widget(no_results_message, area);
            return;
        }

        let (visible_cols, widths) = state.get_visible_cols(
            query_result,
            area.width,
        );

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

        frame.render_stateful_widget(table, table_area, &mut state.ratatui_table_state.borrow_mut());

        //
        // Render the horizontal scrollbar
        //

        if state.should_draw_scrollbar(query_result, area.width) {
            let scrollbar_width = (
                1.0 / (query_result.columns.len() as f32 / 1 as f32)
            ) * area.width as f32;

            let offset_width = (1.0 / query_result.columns.len() as f32) * area.width as f32;
            let scrollbar_offset =
                String::from(" ").repeat(offset_width as usize * state.horizontal_scroll_offset);

            let scrollbar = String::from("▃").repeat(
                scrollbar_width.round() as usize,
            );

            frame.render_widget(
                Span::from(scrollbar_offset + &scrollbar).style(theme.fg),
                horizontal_scroll_bar_area,
            );
        }
    }
}
