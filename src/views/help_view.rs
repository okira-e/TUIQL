use crate::models::help_view_model::HelpViewModel;
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;

struct HelpSection {
    title: &'static str,
    items: Vec<HelpItem>,
}

struct HelpItem {
    key: &'static str,
    text: &'static str,
}

#[derive(Debug)]
/// A row in the flat list: either a section header or a selectable item.
pub enum HelpRow {
    SectionTitle(&'static str),
    Item { key: &'static str, text: &'static str },
    Gap,
}

/// Builds the flat list of rows and returns it along with the indices of selectable items.
pub fn get_help_rows() -> (Vec<HelpRow>, Vec<usize>) {
    let sections = vec![
        HelpSection {
            title: "Navigation",
            items: vec![
                HelpItem { key: "j / Down / Ctrl-n", text: "Move down" },
                HelpItem { key: "k / Up / Ctrl-p", text: "Move up" },
                HelpItem { key: "Ctrl-d", text: "Scroll 10 rows down" },
                HelpItem { key: "Ctrl-u", text: "Scroll 10 rows up" },
                HelpItem { key: "g", text: "Go to top" },
                HelpItem { key: "G", text: "Go to bottom" },
                HelpItem { key: "n", text: "Next page" },
                HelpItem { key: "p", text: "Previous page" },
                HelpItem { key: "Tab", text: "Switch between panes" },
            ],
        },
        HelpSection {
            title: "Table",
            items: vec![
                HelpItem { key: "w", text: "Add a WHERE clause" },
                HelpItem { key: "o", text: "Add an ORDER BY clause" },
                HelpItem { key: "r", text: "Refresh query result" },
                HelpItem { key: "y", text: "Copy highlighted row to clipboard" },
            ],
        },
        HelpSection {
            title: "Commands (press : to enter)",
            items: vec![
                HelpItem { key: ":help / :h", text: "Show this help view" },
                HelpItem { key: ":quit / :q", text: "Quit the application" },
                HelpItem { key: ":count / :c", text: "Count fetched rows" },
                HelpItem { key: ":total-count / :tc", text: "Count total rows in table" },
                HelpItem {
                    key: ":goto page N / :g p N",
                    text: "Jump to a specific page",
                },
                HelpItem {
                    key: ":goto table_name / :g",
                    text: "Jump to a table by name",
                },
                HelpItem { key: ":limit N / :l N", text: "Set rows per page" },
                HelpItem {
                    key: ":refresh / :r",
                    text: "Re-fetch data with current filters",
                },
                HelpItem {
                    key: ":order-by col dir / :ob",
                    text: "Add ORDER BY (no args to reset)",
                },
                HelpItem {
                    key: ":where expr / :w",
                    text: "Add WHERE clause (no args to reset)",
                },
                HelpItem {
                    key: ":save-preset name",
                    text: "Save filters for the selected table",
                },
                HelpItem {
                    key: ":load-preset name",
                    text: "Load filters for the selected table",
                },
                HelpItem {
                    key: ":remove-preset name",
                    text: "Remove a preset from the selected table",
                },
                HelpItem { key: ":set key value", text: "Change a setting at runtime" },
            ],
        },
        HelpSection {
            title: "Settings",
            items: vec![
                HelpItem {
                    key: "transparent_background",
                    text: "Use terminal bg instead of theme",
                },
                HelpItem { key: "default_limit", text: "Default query limit per table" },
                HelpItem {
                    key: "default_sort",
                    text: "Default sorting direction (asc/desc)",
                },
            ],
        },
    ];

    let mut rows = Vec::new();
    let mut selectable = Vec::new();

    for section in sections.iter() {
        rows.push(HelpRow::SectionTitle(section.title));
        for item in &section.items {
            selectable.push(rows.len());
            rows.push(HelpRow::Item { key: item.key, text: item.text });
        }
        rows.push(HelpRow::Gap);
    }

    return (rows, selectable);
}

fn compute_scroll_offset(selected_row_idx: Option<usize>, visible_height: usize) -> u16 {
    if visible_height == 0 {
        return 0;
    }

    return if let Some(row_idx) = selected_row_idx {
        row_idx.saturating_sub(visible_height.saturating_sub(1)) as u16
    } else {
        0
    };
}

pub fn render_help_view(model: &HelpViewModel, theme: &Theme, frame: &mut Frame, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(theme.pane_focus)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title("Help")
        .title_alignment(Alignment::Center)
        .border_style(border_style)
        .border_type(BorderType::Double)
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.fg));

    frame.render_widget(&block, area);
    let inner = block.inner(area);

    let (rows, selectable) = get_help_rows();

    // Set the default cursor to be the first selectable element
    let cursor = if selectable.is_empty() {
        0
    } else {
        model.cursor.min(selectable.len() - 1)
    };

    let selected_row_idx = selectable.get(cursor).copied();

    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = focused && selected_row_idx == Some(i);
            match row {
                HelpRow::SectionTitle(title) => Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        *title,
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ),
                ]),
                HelpRow::Item { key, text } => {
                    let key_style = if is_selected {
                        Style::default()
                            .fg(theme.bg)
                            .bg(theme.selection)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.selection).add_modifier(Modifier::BOLD)
                    };

                    let text_style = if is_selected {
                        Style::default().fg(theme.bg).bg(theme.selection)
                    } else {
                        Style::default().fg(theme.fg)
                    };

                    let trailing = if is_selected {
                        // Fill the rest of the line with the selection background
                        let used = 3 + 30 + text.len();
                        let remaining = (inner.width as usize).saturating_sub(used);
                        " ".repeat(remaining)
                    } else {
                        String::new()
                    };

                    Line::from(vec![
                        Span::styled(if is_selected { " > " } else { "   " }, key_style),
                        Span::styled(format!("{:<30}", key), key_style),
                        Span::styled(*text, text_style),
                        Span::styled(trailing, text_style),
                    ])
                }
                HelpRow::Gap => Line::from(""),
            }
        })
        .collect();

    let scroll_offset = compute_scroll_offset(selected_row_idx, inner.height as usize);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().padding(Padding::ZERO))
        .scroll((scroll_offset, 0));

    frame.render_widget(paragraph, inner);
}
