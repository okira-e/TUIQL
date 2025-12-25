use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Padding, Paragraph},
};

use crate::{
    models::statusline::{MsgKind, StatusLineMode, StatusLineModel},
    theme::Theme,
};

pub fn render_statusline(model: &StatusLineModel, theme: &Theme, frame: &mut Frame, area: Rect) {
    let padding = Block::default().padding(Padding::left(1));
    match &model.mode {
        StatusLineMode::Status => {
            let fg = match model.msg.kind {
                MsgKind::Error => theme.error,
                MsgKind::Success => theme.success,
                MsgKind::Neutral => theme.fg,
            };

            let line_widget = Paragraph::new(model.msg.text.clone())
                .style(Style::default().fg(fg))
                .block(padding);

            frame.render_widget(&line_widget, area);
        }
        StatusLineMode::Command(status_line_command) => todo!(),
    }
}
