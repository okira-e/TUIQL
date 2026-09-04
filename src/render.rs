use crate::app;
use crate::app::App;
use crate::app::RightView;
use crate::app::View;
use crate::models::statusline_model::StatusLineMode;
use crate::views::explorer_view::render_explorer;
use crate::views::help_view::render_help_view;
use crate::views::json_view::render_json_view;
use crate::views::statusline_view::render_statusline;
use crate::views::suggestions_popup_view::render_suggestions_popup;
use crate::views::table_view::render_table;
use ratatui::Frame;
use ratatui::style::Style;
use ratatui::widgets::Block;

pub fn render(app: &mut App, frame: &mut Frame) {
    // This is here to make sure that layout is as fresh as possible before drawing since we take the
    // definitive frame in this function. Otherwise, a race condition may exist in app.run.
    app::calculate_widgets_chunks(app);

    // Set app-wide background
    let bg = if app.settings.transparent_background {
        Block::default()
    } else {
        Block::default().style(Style::default().bg(app.theme.bg))
    };

    frame.render_widget(bg, frame.area());
    let focused_view = app::get_focused_view(app);

    // Render left pane
    render_explorer(
        &mut app.explorer_model,
        &app.theme,
        frame,
        app.widgets_chunks.explorer_chunk,
        focused_view == View::Explorer,
    );

    // Render right pane
    match app.right_view {
        RightView::JsonView => {
            render_json_view(
                &app.json_view_model,
                &app.theme,
                frame,
                app.widgets_chunks.json_view_chunk,
                focused_view == View::JsonView,
            );
        }
        RightView::ResultsTable => {
            render_table(
                &mut app.table_model,
                &app.theme,
                frame,
                app.widgets_chunks.table_chunk,
                focused_view == View::ResultsTable,
                &app.settings.default_sort,
            );
        }
        RightView::Help => {
            render_help_view(
                &app.help_view_model,
                &app.theme,
                frame,
                app.widgets_chunks.help_view_chunk,
                focused_view == View::Help,
            );
        }
    }

    render_statusline(
        &app.statusline_model,
        &app.theme,
        frame,
        app.widgets_chunks.statusline_chunk,
        focused_view == View::StatusLine,
        app.is_loading,
    );

    if app.statusline_model.mode == StatusLineMode::Command && !app.statusline_model.completion.candidates.is_empty() {
        render_suggestions_popup(
            &app.statusline_model.completion,
            &app.theme,
            frame,
            app.widgets_chunks.statusline_chunk,
        );
    }
}
