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
    if model.is_loading {
        // Construct throbber state from its state in the model.
        let mut throbber_state = throbber_widgets_tui::ThrobberState::default();
        for _ in 0..model.spinner_animation_tick_count {
            throbber_state.calc_next();
        }

        let throbber = throbber_widgets_tui::Throbber::default()
            .label("Loading...")
            .style(Style::default().fg(theme.fg))
            .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT)
            .use_type(throbber_widgets_tui::WhichUse::Spin);

        frame.render_stateful_widget(throbber, area, &mut throbber_state);
        return;
    }

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
