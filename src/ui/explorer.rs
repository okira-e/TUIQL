use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use crate::models::explorer::ExplorerItem;
use crate::models::explorer::ExplorerItemKind;
use crate::models::explorer::ExplorerModel;
use crate::models::explorer::get_items_by_kind;
use crate::theme::Theme;

pub fn render_explorer(
    model: &ExplorerModel,
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

    let container = Block::default()
        .title("Database Schema")
        .border_style(border_style)
        .borders(Borders::ALL);

    frame.render_widget(&container, area);
    let inner = container.inner(area);

    if model.items.is_empty() {
        frame.render_widget(Paragraph::new("\n\nDatabase is empty").centered(), inner);
        return;
    }

    let focused_item = model.focused_item.as_ref();

    let tables = get_items_by_kind(&model.items, &ExplorerItemKind::Table);
    let views = get_items_by_kind(&model.items, &ExplorerItemKind::View);

    let mut constraints = Vec::new();

    // Tables section
    constraints.push(Constraint::Length(1)); // Header
    if focused_item.is_some_and(|f| f.kind == ExplorerItemKind::Table) {
        constraints.push(Constraint::Length(tables.len() as u16));
    }

    // Views section
    constraints.push(Constraint::Length(1)); // Header
    if focused_item.is_some_and(|f| f.kind == ExplorerItemKind::View) {
        constraints.push(Constraint::Length(views.len() as u16));
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut idx = 0;

    //
    // Tables
    //
    frame.render_widget(
        section_title(ExplorerItemKind::Table, focused_item, theme),
        layout[idx],
    );
    idx += 1;

    if focused_item.is_some_and(|f| f.kind == ExplorerItemKind::Table) {
        let lines = tables
            .iter()
            .map(|item| explorer_line(item, focused_item, focused, theme));

        frame.render_widget(Paragraph::new(lines.collect::<Vec<_>>()), layout[idx]);
        idx += 1;
    }

    //
    // Views
    //
    frame.render_widget(
        section_title(ExplorerItemKind::View, focused_item, theme),
        layout[idx],
    );
    idx += 1;

    if focused_item.is_some_and(|f| f.kind == ExplorerItemKind::View) {
        let lines = views
            .iter()
            .map(|item| explorer_line(item, focused_item, focused, theme));

        frame.render_widget(Paragraph::new(lines.collect::<Vec<_>>()), layout[idx]);
    }
}

fn section_title(
    kind: ExplorerItemKind,
    focused_item: Option<&ExplorerItem>,
    theme: &Theme,
) -> Line<'static> {
    let focused = focused_item.is_some_and(|f| f.kind == kind);

    Line::from(format!("{} {}", kind.arrow(focused), kind.label()))
        .style(theme.fg)
        .bold()
}

fn explorer_line(
    item: &ExplorerItem,
    focused_item: Option<&ExplorerItem>,
    pane_focused: bool,
    theme: &Theme,
) -> Line<'static> {
    let selected = focused_item.is_some_and(|f| f.name == item.name && f.kind == item.kind);

    let prefix = if selected { "  > " } else { "    " };
    let content = format!("{prefix}{}", item.name);

    let style = if selected && pane_focused {
        theme.selection
    } else {
        theme.fg
    };

    Line::from(content).style(style)
}
