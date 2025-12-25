use ratatui::{Frame, style::Style, widgets::Block};

use crate::{
    app::{App, View},
    ui::{explorer::render_explorer, json_view::render_json_view, statusline::render_statusline, table::render_table},
};

impl App {
    pub fn render(&self, frame: &mut Frame) {
        let root = frame.area();
        // Set app-wide background
        let bg = Block::default().style(Style::default().bg(self.theme.bg));
        frame.render_widget(bg, root);

        render_explorer(
            &self.explorer_model,
            &self.theme,
            frame,
            self.widgets_chunks.explorer_chunk,
            self.focused_view == View::Explorer,
        );

        render_table(
            &self.table_model,
            &self.theme,
            frame,
            self.widgets_chunks.table_chunk,
            self.focused_view == View::ResultsTable,
        );

        render_statusline(
            &self.statusline_model,
            &self.theme,
            frame,
            self.widgets_chunks.statusline_chunk,
        );

        if self.json_view_model.data.is_some() {
            render_json_view(
                &self.json_view_model,
                &self.theme,
                frame,
                self.widgets_chunks.json_view_chunk,
            );
        }
    }
}
