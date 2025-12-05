use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Block,
};

use crate::{app::{App, View}, ui::{explorer_view::ExplorerView, table_view::TableView}};

impl App {
    pub fn draw(&self, frame: &mut Frame) {
        let root = frame.area();

        // Set app-wide background
        let bg = Block::default().style(self.theme.bg);
        frame.render_widget(bg, root);

        let vertical_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Min(1)
            ])
            .split(root);

        let (left, right) = (vertical_layout[0], vertical_layout[1]);

        ExplorerView::draw(
            &self.explorer_state,
            &self.theme,
            frame,
            left,
            self.focused_view == View::Explorer,
        );
        
        TableView::draw(
            &self.results_table_state,
            &self.theme,
            frame,
            right,
            self.focused_view == View::ResultsTable,
            &self.query_result,
        );
    }
}