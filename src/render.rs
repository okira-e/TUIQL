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

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        // This is here to make sure that layout is as fresh as possible before drawing since we take the
        // definitive frame in this function. Otherwise, a race condition may exist in app.run.
        self.calculate_widgets_chunks(frame.area().width, frame.area().height);

        // Set app-wide background
        let bg = if self.settings.transparent_background {
            Block::default()
        } else {
            Block::default().style(Style::default().bg(self.theme.bg))
        };

        frame.render_widget(bg, frame.area());
        let focused_view = self.get_focused_view();

        // Render left pane
        render_explorer(
            &mut self.explorer_model,
            &self.theme,
            frame,
            self.widgets_chunks.explorer_chunk,
            focused_view == View::Explorer,
        );

        // Render right pane
        match self.right_view {
            RightView::JsonView => {
                render_json_view(
                    &self.json_view_model,
                    &self.theme,
                    frame,
                    self.widgets_chunks.json_view_chunk,
                    focused_view == View::JsonView,
                );
            }
            RightView::ResultsTable => {
                render_table(
                    &mut self.table_model,
                    &self.theme,
                    frame,
                    self.widgets_chunks.table_chunk,
                    focused_view == View::ResultsTable,
                    &self.settings.default_sort,
                );
            }
            RightView::Help => {
                render_help_view(
                    &self.help_view_model,
                    &self.theme,
                    frame,
                    self.widgets_chunks.help_view_chunk,
                    focused_view == View::Help,
                );
            }
        }

        render_statusline(
            &self.statusline_model,
            &self.theme,
            frame,
            self.widgets_chunks.statusline_chunk,
            focused_view == View::StatusLine,
            self.is_loading,
        );

        if self.statusline_model.mode == StatusLineMode::Command
            && !self.statusline_model.completion.candidates.is_empty()
        {
            render_suggestions_popup(
                &self.statusline_model.completion,
                &self.theme,
                frame,
                self.widgets_chunks.statusline_chunk,
            );
        }
    }
}
