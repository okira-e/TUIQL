use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};
use serde::Serialize;
use serde_json::{Serializer, Value, ser::PrettyFormatter};

use crate::{models::json_view::JsonViewModel, theme::Theme};

pub fn render_json_view(model: &JsonViewModel, theme: &Theme, frame: &mut Frame, area: Rect) {
    match &model.data {
        None => return,
        Some(json_data) => {
            let pretty = to_4tabs_json_pretty(json_data);

            let lines: Vec<Line> = pretty.lines().map(Line::from).collect();

            frame.render_widget(Clear, area);

            let block = Block::default()
                .title("JSON VIEW")
                .title_alignment(Alignment::Center)
                .border_style(theme.pane_focus)
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
