use crate::models::json_view_model::JsonViewModel;
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use serde::Serialize;
use serde_json::Serializer;
use serde_json::Value;
use serde_json::ser::PrettyFormatter;

pub fn render_json_view(model: &JsonViewModel, theme: &Theme, frame: &mut Frame, area: Rect, focused: bool) {
    match &model.data {
        None => return,
        Some(json_data) => {
            let pretty = to_4tabs_json_pretty(json_data);

            let lines: Vec<Line> = pretty.lines().map(Line::from).collect();

            frame.render_widget(Clear, area);

            let block = Block::default()
                .title("JSON VIEW")
                .title_alignment(Alignment::Center)
                .border_style(if focused { theme.pane_focus } else { Color::default() })
                .border_type(BorderType::Double)
                .borders(Borders::ALL)
                .style(Style::default().bg(theme.bg).fg(theme.fg));

            let text = Text::from(lines);

            let paragraph = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((model.scroll_y, 0));

            frame.render_widget(paragraph, area);
        }
    };
}

fn to_4tabs_json_pretty(v: &Value) -> String {
    let mut out = Vec::new();
    let formatter = PrettyFormatter::with_indent(b"    "); // 4 spaces; tabs are a bad idea
    let mut ser = Serializer::with_formatter(&mut out, formatter);
    v.serialize(&mut ser).expect("serialize failed");
    String::from_utf8(out).unwrap()
}
