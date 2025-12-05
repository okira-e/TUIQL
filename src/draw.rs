use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Block,
};

use crate::{app::{App, View}, ui::{explorer_view::ExplorerView, statusline_view::StatusLineView, table_view::TableView}};

impl App {
    pub fn draw(&self, frame: &mut Frame) {
        let root = frame.area();

        // Set app-wide background
        let bg = Block::default().style(self.theme.bg);
        frame.render_widget(bg, root);

        let app_statusline_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Max(1)
            ])
            .split(root);
        
        let explorer_table_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Min(1)
            ])
            .split(app_statusline_split[0]);

        let (explorer_split, table_split) = (explorer_table_split[0], explorer_table_split[1]);

        ExplorerView::draw(
            &self.explorer_state,
            &self.theme,
            frame,
            explorer_split,
            self.focused_view == View::Explorer,
        );
        
        TableView::draw(
            &self.results_table_state,
            &self.theme,
            frame,
            table_split,
            self.focused_view == View::ResultsTable,
            &self.query_result,
        );

        StatusLineView::draw(
            &self.statusline_state,
            &self.theme,
            frame,
            app_statusline_split[1],
        );
    }
}