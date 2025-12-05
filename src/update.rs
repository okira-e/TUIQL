use crate::app::{App, View};
use crate::statusline::StatusLineMsgKind;

use color_eyre::Result;
use tracing::debug;

use crate::actions::{Action, AppAction, DbAction, ExplorerAction, ResultsTableAction};
use crate::ui::explorer_view::ExplorerView;


impl App {
    pub async fn update(&mut self, action: Action) -> Result<()> {
        debug!("Received action: {:?}", action);

        match action {
            Action::App(action) => self.update_app(action).await?,
            Action::Db(action) => self.update_db(action).await?,
            Action::Explorer(action) => self.update_explorer(action),
            Action::ResultsTable(action) => self.update_results_table(action),
            Action::None => {},
        };
        
        return Ok(());
    }
    
    async fn update_app(&mut self, action: AppAction) -> Result<()> {
        match action {
            AppAction::Quit => {
                self.quit();
            }
            AppAction::CyclePane => {
                if self.focused_view == View::Explorer {
                    self.focused_view = View::ResultsTable;
                } else {
                    self.focused_view = View::Explorer;
                }
            },
            AppAction::SelectTable(name) => {
                self.db_driver.reset_query_state();
                self.update_db(DbAction::QueryTable(name)).await?;
                self.statusline_state.report_message(
                    format!("Fetched {} rows", self.query_result.rows.len()), 
                    StatusLineMsgKind::Success
                );
            },
            AppAction::Resize(w, h) => {
                self.area.width = w;
                self.area.height = h;
            },
        }
        
        return Ok(());
    }

    fn update_explorer(&mut self, action: ExplorerAction) {
        match action {
            ExplorerAction::MoveUp => {
                let current_index = self.explorer_state.focused_item.clone().unwrap().index;

                if current_index > 0 {
                    self.explorer_state.focused_item = Some(
                        ExplorerView::get_items_by_type(
                            &self.explorer_state.items,
                            &self.explorer_state.focused_item.clone().unwrap().kind,
                        )[current_index - 1]
                            .clone(),
                    );
                } else {
                    let prev_item_type = ExplorerView::get_prev_item_type(&self.explorer_state.focused_item.clone().unwrap().kind);
                    let prev_items = ExplorerView::get_items_by_type(
                        &self.explorer_state.items,
                        &prev_item_type,
                    );

                    if prev_items.len() > 0 {
                        self.explorer_state.focused_item = Some(prev_items[prev_items.len() - 1].clone());
                    }
                }
            }
            ExplorerAction::MoveDown => {
                let current_index = self.explorer_state.focused_item.clone().unwrap().index;

                if current_index + 1
                    < ExplorerView::get_items_by_type(
                        &self.explorer_state.items,
                        &self.explorer_state.focused_item.clone().unwrap().kind,
                    )
                    .len()
                {
                    self.explorer_state.focused_item = Some(
                        ExplorerView::get_items_by_type(
                            &self.explorer_state.items,
                            &self.explorer_state.focused_item.clone().unwrap().kind,
                        )[current_index + 1]
                            .clone(),
                    );
                } else {
                    let next_item_type = ExplorerView::get_next_item_type(
                        &self.explorer_state.focused_item.clone().unwrap().kind,
                    );

                    if ExplorerView::get_items_by_type(&self.explorer_state.items, &next_item_type).len() > 0 {
                        self.explorer_state.focused_item = Some(
                            ExplorerView::get_items_by_type(&self.explorer_state.items, &next_item_type)[0].clone(),
                        );
                    }
                }
            }
            ExplorerAction::ExpandNextItemType => {
                let current_type = &self.explorer_state.focused_item.clone().unwrap().kind;
                let next_type = ExplorerView::get_next_item_type(current_type);

                if ExplorerView::get_items_by_type(&self.explorer_state.items, &next_type).len() > 0 {
                    self.explorer_state.focused_item = Some(ExplorerView::get_items_by_type(&self.explorer_state.items, &next_type)[0].clone());
                }
            }
        }
    }

    fn update_results_table(&mut self, action: ResultsTableAction) {
        let mut state = self.results_table_state.ratatui_table_state.borrow_mut();
        let current = state.selected().unwrap_or(0);
        let total_rows = self.query_result.rows.len();

        if total_rows == 0 {
            return;
        }

        match action {
            ResultsTableAction::MoveUp => {
                let new_index = if current == 0 { 0 } else { current - 1 };
                state.select(Some(new_index));
            }
            ResultsTableAction::MoveDown => {
                let new_index = if current + 1 >= total_rows {
                    total_rows
                } else {
                    current + 1
                };
                state.select(Some(new_index));
            }
            ResultsTableAction::ScrollLeft => {
                if self.results_table_state.horizontal_scroll_offset > 0 {
                    self.results_table_state.horizontal_scroll_offset -= 1;
                }
            }
            ResultsTableAction::ScrollRight => {
                let horizontal_scroll_offset = self.results_table_state.horizontal_scroll_offset;

                if self.results_table_state.should_draw_scrollbar(
                    &self.query_result,
                    self.area.width,
                ) && horizontal_scroll_offset < self.query_result.columns.len() - 1
                {
                    self.results_table_state.horizontal_scroll_offset += 1;
                }
            }
            ResultsTableAction::JumpUp => {
                let jump = 10;
                let new_index = current.saturating_sub(jump);
                state.select(Some(new_index));
            }
            ResultsTableAction::JumpDown => {
                let jump = 10;
                let mut new_index = current + jump;
                if new_index >= total_rows {
                    new_index = total_rows - 1;
                }
                state.select(Some(new_index));
            }
            ResultsTableAction::GoToFirst => {
                state.select(Some(0));
            }
            ResultsTableAction::GoToLast => {
                state.select(Some(total_rows - 1));
            }
        }
    }

    async fn update_db(&mut self, action: DbAction) -> Result<()> {
        match action {
            DbAction::QueryTable(table_name) => {
                self.selected_table = Some(table_name.clone());

                let results = self.db_driver.query(&table_name).await?;
                self.query_result = results;

                let count = self.db_driver.query_count(&table_name).await?;
                
                self.focused_view = View::ResultsTable;
                self.results_table_state.table_name = table_name.clone();
                self.results_table_state.results_row_count = self.query_result.rows.len();
                self.results_table_state.total_row_count = count;
                self.results_table_state.current_pos = self.db_driver.get_current_page(&table_name).await?;
                self.results_table_state.ratatui_table_state.borrow_mut().select(Some(0));
            },
            DbAction::NextPage => {
                if let Some(selected_table) = &self.selected_table {
                    let count = self.db_driver.query_count(selected_table).await?;

                    self.db_driver.next_page(
                        &selected_table,
                        count,
                    ).await?;
                    
                    let results = self.db_driver.query(selected_table).await?;
                    if results.rows.len() == 0 {
                        return Ok(());
                    }
                    
                    self.query_result = results;
                    
                    self.results_table_state.table_name = selected_table.clone();
                    self.results_table_state.results_row_count = self.query_result.rows.len();
                    self.results_table_state.total_row_count = count;
                    self.results_table_state.current_pos = self.db_driver.get_current_page(&selected_table).await?;
                    self.results_table_state.ratatui_table_state.borrow_mut().select(Some(0));
                }
            }
            DbAction::PrevPage => {
                if let Some(selected_table) = &self.selected_table {
                    self.db_driver.prev_page(&selected_table).await?;

                    let results = self.db_driver.query(&selected_table).await?;
                    self.query_result = results;
                    let count = self.db_driver.query_count(selected_table).await?;

                    self.results_table_state.table_name = selected_table.clone();
                    self.results_table_state.results_row_count = self.query_result.rows.len();
                    self.results_table_state.total_row_count = count;
                    self.results_table_state.current_pos = self.db_driver.get_current_page(&selected_table).await?;
                    self.results_table_state.ratatui_table_state.borrow_mut().select(Some(0));
                }
            }
        };

        return Ok(());
    }
}
