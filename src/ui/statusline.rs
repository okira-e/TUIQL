use ratatui::Frame;
use ratatui::layout::Position;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;

use crate::models::statusline::MsgKind;
use crate::models::statusline::StatusLineMode;
use crate::models::statusline::StatusLineModel;
use crate::theme::Theme;

pub fn render_statusline(
    model: &StatusLineModel,
    theme: &Theme,
    frame: &mut Frame,
    area: Rect,
    focused: bool,
    is_app_loading: bool,
) {
    if is_app_loading {
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
        StatusLineMode::Command => {
            let text = format!(": {}", model.cmd.text);
            let line_widget = Paragraph::new(text);

            frame.render_widget(&line_widget, area);
            if focused {
                frame.set_cursor_position(Position { x: area.x + model.cmd.cursor as u16 + 2, y: area.y + 1 });
            }
        }
    }
}
