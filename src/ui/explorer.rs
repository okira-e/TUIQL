use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame
};

use crate::{
    actions::{Action, AppAction, ExplorerAction},
    theme::Theme,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    pub name: String,
    pub kind: String,
    pub index: usize,
}

pub struct ExplorerView {
    /// The item under the cursor.
    pub focused_item: Option<ExplorerItem>,
    /// The item the user chose to select.
    pub selected_item: Option<ExplorerItem>,
    /// All items in the tree.
    pub items: Vec<ExplorerItem>,
}

impl ExplorerView {
    pub fn new() -> Self {
        return Self {
            focused_item: None,
            selected_item: None,
            items: vec![],
        };
    }

    pub fn set_items(&mut self, tables: Vec<String>, views: Vec<String>) {
        let tables: Vec<ExplorerItem> = tables.iter().enumerate().map(|(i, name)| ExplorerItem {
            name: name.clone(),
            kind: "table".to_string(),
            index: i,
        }).collect();

        let views: Vec<ExplorerItem> = views.iter().enumerate().map(|(i, name)| ExplorerItem {
            name: name.clone(),
            kind: "view".to_string(),
            index: i,
        }).collect();
        
        self.items = vec![];
        self.items.extend(tables);
        self.items.extend(views);
        
        if self.items.len() > 0 {
            self.focused_item = Some(self.items[0].clone());
        }
    }

    pub fn draw(&self, theme: &Theme, frame: &mut Frame, area: Rect, focused: bool) {
        let container_border_style = if focused {
            theme.pane_focus
        } else {
            theme.bg
        };


        let container_block = Block::default()
            .title("Database Schema")
            .border_style(container_border_style)
            .borders(Borders::ALL);

        if self.items.len() == 0 {
            frame.render_widget(container_block.clone(), area);
            frame.render_widget(
                Paragraph::new("\n\nDatabase is empty").centered(),
                container_block.inner(area)
            );
            return;
        }

        let mut layout_constraints_arr = vec![];

        let tables_count = self.get_items_by_type("table").len();
        let views_count = self.get_items_by_type("view").len();

        layout_constraints_arr.push(Constraint::Length(1));
        if tables_count > 0 && self.focused_item.clone().unwrap().kind == "table" {
            layout_constraints_arr.push(Constraint::Length(tables_count as u16));
        }

        layout_constraints_arr.push(Constraint::Length(1));
        if views_count > 0 && self.focused_item.clone().unwrap().kind == "view" {
            layout_constraints_arr.push(Constraint::Length(views_count as u16));
        }


        // Render outer block and get the inner rect
        frame.render_widget(&container_block, area);

        let inner_area = container_block.inner(area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(layout_constraints_arr)
            .split(inner_area);

        let focused_element_style = theme.selection;

        let mut next_buff_idx = 0;

        //
        // Render tables
        //

        let currently_focused_item_type = &self.focused_item.clone().unwrap().kind;

        let title = Line::from(
            if currently_focused_item_type == "table" {
                "▼ Tables"
            } else {
                "▶ Tables"
            }
        )
            .style(theme.fg)
            .bold();

        frame.render_widget(
            title,
            layout[next_buff_idx]
        );
        next_buff_idx += 1;

        if self.focused_item.clone().unwrap().kind == "table".to_string() {
            let mut table_lines: Vec<Line> = vec![];
            let tables = self.get_items_by_type("table");
            for table in tables {
                let line = if self.focused_item.clone().unwrap().name.eq(&table.name) 
                && self.focused_item.clone().unwrap().kind == "table".to_string() 
                {
                    let indentation_span = Span::raw("  > ");
                    indentation_span.clone() + Span::from(table.name.clone()).style(if focused { focused_element_style } else { Style::default() })
                } else {
                    let indentation_span = Span::raw("    ");
                    indentation_span.clone() + Span::from(table.name.clone()).style(theme.fg)
                };

                table_lines.push(line);
            }

            frame.render_widget(
                Paragraph::new(table_lines),
                layout[next_buff_idx]
            );
            next_buff_idx += 1;
        }


        //
        // Render views
        //

        let title = Line::from(
            if currently_focused_item_type == "view" {
                "▼ Views"
            } else {
                "▶ Views"
            }
        )
            .style(theme.fg)
            .bold();

        frame.render_widget(
            title,
            layout[next_buff_idx]
        );
        next_buff_idx += 1;

        if &self.focused_item.clone().unwrap().kind == "view" {
            let mut view_lines: Vec<Line> = vec![];
            let views = self.get_items_by_type("view");
            for view in views {
                let line = if self.focused_item.clone().unwrap().name.eq(&view.name) 
                && self.focused_item.clone().unwrap().kind == "view".to_string() 
                {
                    let indentation_span = Span::raw("  > ").style(theme.fg);
                    indentation_span.clone() + Span::from(view.name.clone()).style(focused_element_style)
                } else {
                    let indentation_span = Span::raw("    ");
                    indentation_span.clone() + Span::from(view.name.clone()).style(theme.fg)
                };

                view_lines.push(line);
            }

            frame.render_widget(
                Paragraph::new(view_lines),
                layout[next_buff_idx]
            );
        }       
    }

    pub fn update(&mut self, action: ExplorerAction) {
        if self.items.is_empty() {
            return;
        }

        match action {
            ExplorerAction::MoveUp => {
                let current_index = self.focused_item.clone().unwrap().index;

                if current_index > 0 {
                    self.focused_item = Some(self.get_items_by_type(&self.focused_item.clone().unwrap().kind)[current_index - 1].clone());
                } else {
                    let prev_item_type = self.get_prev_item_type(&self.focused_item.clone().unwrap().kind);
                    let prev_items = self.get_items_by_type(&prev_item_type);

                    if prev_items.len() > 0 {
                        self.focused_item = Some(prev_items[prev_items.len() - 1].clone());
                    }
                }
            },
            ExplorerAction::MoveDown => {
                let current_index = self.focused_item.clone().unwrap().index;

                if current_index + 1 < self.get_items_by_type(&self.focused_item.clone().unwrap().kind).len() {
                    self.focused_item = Some(self.get_items_by_type(&self.focused_item.clone().unwrap().kind)[current_index + 1].clone());
                } else {
                    let next_item_type = self.get_next_item_type(&self.focused_item.clone().unwrap().kind);

                    if self.get_items_by_type(&next_item_type).len() > 0 {
                        self.focused_item = Some(self.get_items_by_type(&next_item_type)[0].clone());
                    }
                }
            },
            ExplorerAction::MoveToTop => {},
            ExplorerAction::MoveToBottom => {},
            ExplorerAction::MoveHalfPageUp => {},
            ExplorerAction::MoveHalfPageDown => {},
            ExplorerAction::MoveToNextMatch => {},
            ExplorerAction::MoveToPrevMatch => {},
            ExplorerAction::ClearSearch => {},
            ExplorerAction::ExpandNextItemType => {
                let current_type = &self.focused_item.clone().unwrap().kind;
                let next_type = self.get_next_item_type(current_type);

                if self.get_items_by_type(&next_type).len() > 0 {
                    self.focused_item = Some(self.get_items_by_type(&next_type)[0].clone());
                } 
            }
        }
    }

    pub fn handle_key_event(&self, modifier: KeyModifiers, key: KeyCode) -> Action {
        return match (modifier, key) {
            (_, KeyCode::Char('k') | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                Action::Explorer(ExplorerAction::MoveUp)
            }

            (_, KeyCode::Char('j') | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                Action::Explorer(ExplorerAction::MoveDown)
            }

            (_, KeyCode::Char('h') | KeyCode::Left) => {
                Action::Explorer(ExplorerAction::ExpandNextItemType)
            }

            (_, KeyCode::Enter) => {
                if let Some(item) = &self.focused_item {
                    Action::App(AppAction::SelectTable(item.name.clone()))
                } else {
                    Action::None
                }
            }

            _ => Action::None,
        };
    }

    fn get_items_by_type(&self, item_type: &str) -> Vec<ExplorerItem> {
        return self.items
            .iter()
            .filter(|e| e.kind == item_type)
            .cloned()
            .collect();
    }

    fn get_next_item_type(&self, current_type: &str) -> String {
        match current_type {
            "table" => "view".to_string(),
            "view" => "procedure".to_string(),
            "procedure" => "function".to_string(),
            "function" => "table".to_string(),
            _ => "table".to_string(),
        }
    }

    fn get_prev_item_type(&self, current_type: &str) -> String {
        match current_type {
            "table" => "function".to_string(),
            "view" => "table".to_string(),
            "procedure" => "view".to_string(),
            "function" => "procedure".to_string(),
            _ => "table".to_string(),
        }
    }
}

