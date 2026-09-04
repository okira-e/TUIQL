use crate::models::statusline_model::Completion;
use crate::suggestor::SuggestionKind;
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Clear;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;

/// Most suggestion rows we'll ever show at once.
const MAX_VISIBLE: usize = 10;

/// Renders a full-width box stacked directly on top of the status line, listing the
/// current command suggestions and highlighting the one being cycled to with Tab.
pub fn render_suggestions_popup(completion: &Completion, theme: &Theme, frame: &mut Frame, chunk: Rect) {
    let rows_above = chunk.y as usize;
    let height = completion.candidates.len().min(MAX_VISIBLE).min(rows_above);
    if height == 0 {
        return;
    }

    // Scroll just enough to keep the selected row inside the visible window.
    let offset = match completion.selected {
        Some(i) if i >= height => i - height + 1,
        _ => 0,
    };

    let area = Rect {
        x: chunk.x,
        y: chunk.y - height as u16,
        width: chunk.width,
        height: height as u16,
    };

    let items: Vec<ListItem> = completion
        .candidates
        .iter()
        .enumerate()
        .skip(offset)
        .take(height)
        .map(|(i, suggestion)| {
            let color = match suggestion.kind {
                SuggestionKind::Command => theme.accent,
                SuggestionKind::SubCommand => theme.pane_focus,
                SuggestionKind::Keyword => theme.fg,
                SuggestionKind::Column => theme.selection,
                SuggestionKind::Table => theme.pane_focus,
                SuggestionKind::Preset => theme.pane_focus,
            };

            let style = if completion.selected == Some(i) {
                // The cycled selection reads as a filled, bold row.
                Style::default().fg(theme.bg).bg(theme.selection).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            // Leading space keeps the text off the left edge of the box.
            ListItem::new(format!(" {}", suggestion.display)).style(style)
        })
        .collect();

    // The base style paints the whole box background so every row is filled.
    let list = List::new(items).style(Style::default().bg(theme.highlight));

    frame.render_widget(Clear, area);
    frame.render_widget(list, area);
}
