use ratatui::{Frame, layout::Rect, widgets::{Block, Padding, Paragraph}};

use crate::{ models::statusline_model::{StatusLineMode, StatusLineMsgKind, StatusLineModel}, theme::Theme};

pub struct StatusLineView {}

impl StatusLineView {
    pub fn draw(state: &StatusLineModel, theme: &Theme, frame: &mut Frame, area: Rect) {
        let padding = Block::default().padding(Padding::left(1));
        match &state.mode {
            StatusLineMode::Status(msg) => {
                if let Some(msg) = msg {
                    let style = match msg.kind {
                        StatusLineMsgKind::Error => theme.error,
                        StatusLineMsgKind::Success => theme.success,
                        StatusLineMsgKind::Neutral => theme.fg,
                    };

                    let line_widget = Paragraph::new(msg.text.clone())
                        .style(style)
                        .block(padding);

                    frame.render_widget(&line_widget, area);
                }
            },
            StatusLineMode::Command(status_line_command) => todo!(),
        }
    }
}