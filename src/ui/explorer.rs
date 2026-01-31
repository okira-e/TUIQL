use crate::models::explorer::ExplorerItem;
use crate::models::explorer::ExplorerItemKind;
use crate::models::explorer::ExplorerModel;

use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

pub fn render_explorer(
    model: &mut ExplorerModel,
    theme: &Theme,
    frame: &mut Frame,
    area: Rect,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(theme.pane_focus)
    } else {
        Style::default().bg(theme.bg)
    };

    //
    // Draw the container
    //

    let table_tab_color = if model.selected_tab == ExplorerItemKind::Table {
        theme.selection
    } else {
        theme.fg
    };
    let view_tab_color = if model.selected_tab == ExplorerItemKind::View {
        theme.selection
    } else {
        theme.fg
    };
    let mview_tab_color = if model.selected_tab == ExplorerItemKind::MaterializedView {
        theme.selection
    } else {
        theme.fg
    };

    let container = Block::default()
        .title("Database Schema")
        .title(
            Line::from("t")
                .style(table_tab_color)
                .alignment(Alignment::Right),
        )
        .title(
            Line::from("v")
                .style(view_tab_color)
                .alignment(Alignment::Right),
        )
        .title(
            Line::from("m")
                .style(mview_tab_color)
                .alignment(Alignment::Right),
        )
        .border_style(border_style)
        .borders(Borders::ALL);

    frame.render_widget(&container, area);
    let inner_container = container.inner(area);

    if model.items.is_empty() {
        frame.render_widget(
            Paragraph::new("\n\nDatabase is empty").centered(),
            inner_container,
        );
        return;
    }

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(0),    // items list
        ])
        .split(inner_container);

    let title_area = areas[0];
    let items_list_area = areas[1];

    //
    // Draw tab title
    //

    let tab_title = match model.selected_tab {
        ExplorerItemKind::Table => "Tables",
        ExplorerItemKind::View => "Views",
        ExplorerItemKind::MaterializedView => "Materialized Views",
    };

    let p = Paragraph::new(tab_title)
        .style(theme.fg)
        .block(Block::default().padding(Padding::left(1)))
        .add_modifier(Modifier::BOLD);
    frame.render_widget(p, title_area);

    //
    // Draw the items table
    //

    // Get items for current tab (MaterializedView uses View data for now)
    let items: Vec<ExplorerItem> = match model.selected_tab {
        ExplorerItemKind::Table => model.get_items_by_kind(ExplorerItemKind::Table),
        ExplorerItemKind::View => model.get_items_by_kind(ExplorerItemKind::View),
        ExplorerItemKind::MaterializedView => model.get_items_by_kind(ExplorerItemKind::MaterializedView),
    };

    if items.is_empty() {
        let empty_msg = match model.selected_tab {
            ExplorerItemKind::Table => "No tables",
            ExplorerItemKind::View => "No views",
            ExplorerItemKind::MaterializedView => "No materialized views",
        };
        frame.render_widget(Paragraph::new(empty_msg).centered(), items_list_area);
        return;
    }

    // Build rows with horizontal scroll offset applied
    let rows: Vec<Row> = items
        .iter()
        .map(|item| {
            let display_name: String = item
                .name
                .chars()
                .skip(model.horizontal_scroll_offset)
                .collect();
            Row::new(vec![Cell::from(display_name).style(theme.fg)])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(100)])
        .row_highlight_style(if focused {
            Style::default().fg(theme.selection)
        } else {
            Style::default()
        })
        .block(Block::default().padding(Padding::left(1)))
        .highlight_symbol("> ");

    frame.render_stateful_widget(table, items_list_area, &mut model.table_state);
}
