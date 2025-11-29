pub mod explorer;
pub mod table;

use std::collections::HashMap;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Block,
};

use crate::{
    actions::Action,
    theme::Theme,
    ui::{explorer::ExplorerView, table::TableView},
};


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Pane {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ViewId {
    Explorer,
    ResultsTable,
}

pub struct UI {
    /// Index to the views
    pub focused_pane: Pane,
    pub layout: HashMap<ViewId, Pane>,
    pub explorer: ExplorerView,
    pub results_table: TableView,
}

impl UI {
    pub fn new() -> UI {
        let mut layout = HashMap::new();
        layout.insert(ViewId::Explorer, Pane::Left);
        layout.insert(ViewId::ResultsTable, Pane::Right);

        let explorer = ExplorerView::new();
        
        return Self {
            focused_pane: Pane::Left,
            layout,
            explorer,
            results_table: TableView::new("Query Result"),
        };
    }
    
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Explorer(a) => self.explorer.update(a),
            Action::ResultsTable(a) => self.results_table.update(a),
            _ => {},
        };
    }
    
    pub fn draw(&mut self, theme: &Theme, frame: &mut Frame) {
        let root = frame.area();

        // Set app-wide background
        let bg = Block::default().style(theme.bg);
        frame.render_widget(bg, root);

        let vertical_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Min(1)
            ])
            .split(root);
        let (left, right) = (vertical_layout[0], vertical_layout[1]);

        self.explorer.draw(
            theme,
            frame,
            left,
            self.layout.get(&ViewId::Explorer) == Some(&self.focused_pane)
        );
        
        self.results_table.draw(
            theme,
            frame,
            right,
            self.layout.get(&ViewId::ResultsTable) == Some(&self.focused_pane)
        );
    }

    pub fn get_view_by_pane(&self, pane: Pane) -> ViewId {
        for (comp_id, comp_pane) in &self.layout {
            if *comp_pane == pane {
                return *comp_id;
            }
        }
        panic!("No view found for the given pane");
    }
}

