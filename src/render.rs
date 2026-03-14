use crate::app::App;
use crate::app::RightView;
use crate::app::View;
use crate::views::explorer_view::render_explorer;
use crate::views::json_view::render_json_view;
use crate::views::statusline_view::render_statusline;
use crate::views::table_view::render_table;
use ratatui::Frame;
use ratatui::style::Style;
use ratatui::widgets::Block;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let root = frame.area();
        // Set app-wide background
        let bg = if self.theme.transparent_background {
            Block::default()
        } else {
            Block::default().style(Style::default().bg(self.theme.bg))
        };
        frame.render_widget(bg, root);
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
    }
}
