use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    models::explorer::{ExplorerModel, get_items_by_type},
    theme::Theme,
};

pub fn render_explorer(
    model: &ExplorerModel,
    theme: &Theme,
    frame: &mut Frame,
    area: Rect,
    focused: bool,
) {
    let container_border_style = if focused {
        Style::default().fg(theme.pane_focus)
    } else {
        Style::default().bg(theme.bg)
    };

    let container_block = Block::default()
        .title("Database Schema")
        .border_style(container_border_style)
        .borders(Borders::ALL);

    if model.items.len() == 0 {
        frame.render_widget(container_block.clone(), area);
        frame.render_widget(
            Paragraph::new("\n\nDatabase is empty").centered(),
            container_block.inner(area),
        );
        return;
    }

    assert!(model.focused_item.is_some());
    let focused_item = model.focused_item.as_ref().unwrap();

    let mut layout_constraints_arr = vec![];

    let tables_count = get_items_by_type(&model.items, "table").len();
    let views_count = get_items_by_type(&model.items, "view").len();

    layout_constraints_arr.push(Constraint::Length(1));
    if tables_count > 0 && focused_item.clone().kind == "table" {
        layout_constraints_arr.push(Constraint::Length(tables_count as u16));
    }

    layout_constraints_arr.push(Constraint::Length(1));
    if views_count > 0 && focused_item.clone().kind == "view" {
        layout_constraints_arr.push(Constraint::Length(views_count as u16));
    }

    // Render outer block and get the inner rect
    frame.render_widget(&container_block, area);

    let inner_area = container_block.inner(area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(layout_constraints_arr)
        .split(inner_area);

    let focused_element_style = theme.selection;

    let mut next_buff_idx = 0;

    //
    // Render tables
    //

    let title = Line::from(if focused_item.kind == "table" {
        "▼ Tables"
    } else {
        "▶ Tables"
    })
    .style(theme.fg)
    .bold();

    frame.render_widget(title, layout[next_buff_idx]);
    next_buff_idx += 1;

    if focused_item.kind == "table".to_string() {
        let mut table_lines: Vec<Line> = vec![];
        let tables = get_items_by_type(&model.items, "table");
        for table in tables {
            let line =
                if focused_item.name.eq(&table.name) && focused_item.kind == "table".to_string() {
                    let indentation_span = Span::raw("  > ");
                    indentation_span.clone()
                        + Span::from(table.name.clone()).style(if focused {
                            focused_element_style
                        } else {
                            Color::default()
                        })
                } else {
                    let indentation_span = Span::raw("    ");
                    indentation_span.clone() + Span::from(table.name.clone()).style(theme.fg)
                };

            table_lines.push(line);
        }

        frame.render_widget(Paragraph::new(table_lines), layout[next_buff_idx]);
        next_buff_idx += 1;
    }

    //
    // Render views
    //

    let title = Line::from(if focused_item.kind == "view" {
        "▼ Views"
    } else {
        "▶ Views"
    })
    .style(theme.fg)
    .bold();

    frame.render_widget(title, layout[next_buff_idx]);
    next_buff_idx += 1;

    if focused_item.kind == "view" {
        let mut view_lines: Vec<Line> = vec![];
        let views = get_items_by_type(&model.items, "view");
        for view in views {
            let line =
                if focused_item.name.eq(&view.name) && focused_item.kind == "view".to_string() {
                    let indentation_span = Span::raw("  > ").style(theme.fg);
                    indentation_span.clone()
                        + Span::from(view.name.clone()).style(if focused {
                            focused_element_style
                        } else {
                            Color::default()
                        })
                } else {
                    let indentation_span = Span::raw("    ");
                    indentation_span.clone() + Span::from(view.name.clone()).style(theme.fg)
                };

            view_lines.push(line);
        }

        frame.render_widget(Paragraph::new(view_lines), layout[next_buff_idx]);
    }
}
