use crate::models::table_model::TableModel;
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use serde_json::Value;

pub fn render_table(model: &mut TableModel, theme: &Theme, frame: &mut Frame, area: Rect, focused: bool) {
    let container_border_style = if focused {
        Style::default().fg(theme.pane_focus)
    } else {
        Style::default()
    };

    let page = if model.query_result.rows.is_empty() {
        0
    } else {
        model.current_page + 1
    };

    let page_info = if let Some(total_count) = model.total_count {
        let total_pages = (total_count as f64 / model.query_state.limit as f64).ceil();
        format!("Page {}/{} - Rows {}", page, total_pages, model.results_row_count)
    } else {
        format!("Page {} - Rows {}", page, model.results_row_count)
    };

    let title = model.table_name.clone().unwrap_or("Query Result".to_string());

    let mut container_block = Block::default()
        .title(title)
        .title(
            Line::from(page_info)
                .style(theme.fg)
                .alignment(Alignment::Center),
        )
        .border_style(container_border_style)
        .borders(Borders::ALL);

    if let Some(total_count) = model.total_count {
        container_block = container_block.title(
            Line::from(format!("Total Count: {}", total_count))
                .style(theme.fg)
                .alignment(Alignment::Right),
        );
    }

    //
    // Set the total count/pages of the table if it has been fetched.
    //

    // Build left-aligned bottom title with active where/order-by clauses
    let max_clause_len = (area.width as usize).saturating_sub(2) / 2;
    let mut clauses: Vec<Span> = vec![];

    if let Some(ref where_clause) = model.query_state.where_clause {
        let label = "WHERE: ";
        let max_val = max_clause_len.saturating_sub(label.len());
        let value = if where_clause.len() > max_val {
            format!("{}…", &where_clause[..max_val.saturating_sub(1)])
        } else {
            where_clause.clone()
        };
        clauses.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
        clauses.push(Span::styled(value, theme.fg));
    }

    if let Some(ref order_by) = model.query_state.order_by {
        if !clauses.is_empty() {
            clauses.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        }
        let label = "ORDER: ";
        let max_val = max_clause_len.saturating_sub(label.len());
        let value = if order_by.len() > max_val {
            format!("{}…", &order_by[..max_val.saturating_sub(1)])
        } else {
            order_by.clone()
        };
        clauses.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
        clauses.push(Span::styled(value, theme.fg));
    }

    if !clauses.is_empty() {
        container_block = container_block.title_bottom(
            Line::from(clauses).alignment(Alignment::Left),
        );
    }


    frame.render_widget(&container_block, area);

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(container_block.inner(area));

    let table_area = inner_layout[0];
    let horizontal_scroll_bar_area = inner_layout[1];

    if model.query_result.rows.is_empty() {
        let no_results_message = Paragraph::new("No results to display")
            .style(theme.fg)
            .block(container_block.clone());

        frame.render_widget(no_results_message, area);
        return;
    }

    let (visible_cols, widths) = model.get_visible_cols(area.width);

    //
    // Populate rows
    //

    let visible_rows = model.query_result.rows.clone();
    let mut table_rows: Vec<Row> = vec![];
    for (row_index, row) in visible_rows.iter().enumerate() {
        let row_map = row.as_object().unwrap();

        let mut values = vec![];
        let row_style = if row_index % 2 == 0 {
            Style::default()
        } else {
            Style::default().bg(theme.highlight)
        };
        for key in visible_cols.iter() {
            let cell = match row_map[&key.name].clone() {
                Value::Null => Cell::from(" null").style(row_style.fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                Value::Bool(b) => Cell::from(format!(" {}", b.to_string())).style(row_style.fg(Color::Yellow)),
                Value::Number(number) => {
                    Cell::from(format!(" {}", number.to_string())).style(row_style.fg(Color::Cyan))
                }
                Value::String(s) => Cell::from(format!(" {}", s.clone())).style(row_style.patch(theme.fg)),
                Value::Array(values) => {
                    Cell::from(format!("[{} items]", values.len())).style(row_style.fg(Color::Magenta))
                }
                Value::Object(map) => Cell::from(format!("{{{} keys}}", map.len())).style(row_style.fg(Color::Blue)),
            };

            values.push(cell);
        }

        table_rows.push(Row::new(values));
    }

    let constraints = vec![Constraint::Max(40); visible_cols.len()];
    let table = Table::new(table_rows, constraints)
        .header(
            Row::new(visible_cols.iter().map(|col| col.name.clone()).collect::<Vec<String>>())
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1), // Space between header and rows
        )
        .row_highlight_style(if focused {
            Style::default().fg(theme.selection).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .widths(widths)
        .highlight_symbol("> ")
        .column_spacing(0);

    frame.render_stateful_widget(table, table_area, &mut model.ratatui_table_state);

    //
    // Render the horizontal scrollbar
    //

    if model.should_draw_scrollbar(area.width) {
        let scrollbar_width = (1.0 / (model.query_result.columns.len() as f32 / 1 as f32)) * area.width as f32;

        let offset_width = (1.0 / model.query_result.columns.len() as f32) * area.width as f32;
        let scrollbar_offset = String::from(" ").repeat(offset_width as usize * model.horizontal_scroll_offset);

        let scrollbar = String::from("▃").repeat(scrollbar_width.round() as usize);

        frame.render_widget(
            Span::from(scrollbar_offset + &scrollbar).style(theme.fg),
            horizontal_scroll_bar_area,
        );
    }
}
