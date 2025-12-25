use crate::actions::JsonViewAction;
use crate::app::View;
use crate::app::App;
use crate::models::explorer::get_items_by_type;
use crate::models::explorer::get_next_item_type;
use crate::models::explorer::get_prev_item_type;
use crate::models::statusline::MsgLifetime;
use crate::models::statusline::MsgKind;
use crate::models::statusline::StatusLineMsg;

use arboard::Clipboard;
use color_eyre::Result;
use tracing::debug;

use crate::actions::{Action, AppAction, DbAction, ExplorerAction, ResultsTableAction};


impl App {
    pub async fn update(&mut self, action: Action) -> Result<()> {
        debug!("Received action: {:?}", action);

        match action {
            Action::App(action) => self.update_app(action).await?,
            Action::Db(action) => self.update_db(action).await?,
            Action::Explorer(action) => self.update_explorer(action),
            Action::ResultsTable(action) => self.update_results_table(action),
            Action::JsonView(action) => self.update_json_view(action)?,
            Action::None => {},
        };
        
        return Ok(());
    }
    
    async fn update_app(&mut self, action: AppAction) -> Result<()> {
        match action {
            AppAction::Quit => {
                self.quit();
            }
            AppAction::Tick => {
                // Increment tick count for spinner animation
                self.statusline_model.spinner_animation_tick_count = self.statusline_model.spinner_animation_tick_count.wrapping_add(1);
                
                // Check if message has expired
                if self.statusline_model.msg.created_at.elapsed() 
                    > self.statusline_model.msg.lifetime.to_duration() 
                {
                    self.statusline_model.msg = StatusLineMsg::default();
                }
            }
            AppAction::CyclePane => {
                if self.focused_view == View::Explorer {
                    self.focused_view = View::ResultsTable;
                } else {
                    self.focused_view = View::Explorer;
                }
            }
            AppAction::SelectTable(name) => {
                self.db_driver.lock().await.reset_query_state();
                self.table_model.reset(Some(0));
                _ = self.action_tx.send(Action::Db(DbAction::QueryTable(name)));
            }
            AppAction::Resize(w, h) => {
                self.area.width = w;
                self.area.height = h;
                self.calculate_widgets_chunks();
                self.table_model.reset(Some(0));
            }
            AppAction::ViewSelectedRowAsJson => {
                self.json_view_model.data = self.table_model.get_selected_row_data();
                self.focused_view = View::JsonView;
            }
            AppAction::ClosePopup => {
                match self.focused_view {
                    View::JsonView => {
                        self.json_view_model.data = None;
                        self.focused_view = View::ResultsTable;
                    }
                    _ => {}
                }
            }
        }
        
        return Ok(());
    }

    fn update_explorer(&mut self, action: ExplorerAction) {
        match action {
            ExplorerAction::MoveUp => {
                let current_index = self.explorer_model.focused_item.clone().unwrap().index;

                if current_index > 0 {
                    self.explorer_model.focused_item = Some(
                        get_items_by_type(
                            &self.explorer_model.items,
                            &self.explorer_model.focused_item.clone().unwrap().kind,
                        )[current_index - 1]
                            .clone(),
                    );
                } else {
                    let prev_item_type = get_prev_item_type(&self.explorer_model.focused_item.clone().unwrap().kind);
                    let prev_items = get_items_by_type(
                        &self.explorer_model.items,
                        &prev_item_type,
                    );

                    if prev_items.len() > 0 {
                        self.explorer_model.focused_item = Some(prev_items[prev_items.len() - 1].clone());
                    }
                }
            }
            ExplorerAction::MoveDown => {
                let current_index = self.explorer_model.focused_item.clone().unwrap().index;

                if current_index + 1
                    < get_items_by_type(
                        &self.explorer_model.items,
                        &self.explorer_model.focused_item.clone().unwrap().kind,
                    )
                    .len()
                {
                    self.explorer_model.focused_item = Some(
                        get_items_by_type(
                            &self.explorer_model.items,
                            &self.explorer_model.focused_item.clone().unwrap().kind,
                        )[current_index + 1]
                            .clone(),
                    );
                } else {
                    let next_item_type = get_next_item_type(
                        &self.explorer_model.focused_item.clone().unwrap().kind,
                    );

                    if get_items_by_type(&self.explorer_model.items, &next_item_type).len() > 0 {
                        self.explorer_model.focused_item = Some(
                            get_items_by_type(&self.explorer_model.items, &next_item_type)[0].clone(),
                        );
                    }
                }
            }
            ExplorerAction::ExpandNextItemType => {
                let current_type = &self.explorer_model.focused_item.clone().unwrap().kind;
                let next_type = get_next_item_type(current_type);

                if get_items_by_type(&self.explorer_model.items, &next_type).len() > 0 {
                    self.explorer_model.focused_item = Some(get_items_by_type(&self.explorer_model.items, &next_type)[0].clone());
                }
            }
        }
    }

    fn update_results_table(&mut self, action: ResultsTableAction) {
        let total_rows = self.table_model.query_result.rows.len();
        if total_rows == 0 {
            return;
        }
    
        let current = self.table_model.selected_row.unwrap_or(0);
        
        // Calculate how many rows fit in the viewport
        let table_header_and_footer_height = 5;
        let visible_rows = (self.widgets_chunks.table_chunk.height - table_header_and_footer_height) as usize;
        
        match action {
            ResultsTableAction::MoveUp => {
                let new_index = if current == 0 { 0 } else { current - 1 };
                self.table_model.selected_row = Some(new_index);

                // Only scroll up if cursor would go ABOVE the viewport
                if new_index < self.table_model.vertical_scroll_offset {
                    self.table_model.vertical_scroll_offset = new_index;
                }
            }
            ResultsTableAction::MoveDown => {
                let new_index = if current + 1 >= total_rows {
                    total_rows - 1
                } else {
                    current + 1
                };
                self.table_model.selected_row = Some(new_index);
                
                // Only scroll down if cursor would go BELOW the viewport
                let viewport_bottom = self.table_model.vertical_scroll_offset + visible_rows;
                if new_index >= viewport_bottom {
                    self.table_model.vertical_scroll_offset = new_index.saturating_sub(visible_rows - 1);
                }
            }
            ResultsTableAction::ScrollLeft => {
                if self.table_model.horizontal_scroll_offset > 0 {
                    self.table_model.horizontal_scroll_offset -= 1;
                }
            }
            ResultsTableAction::ScrollRight => {
                let horizontal_scroll_offset = self.table_model.horizontal_scroll_offset;
                    
                if self.table_model.should_draw_scrollbar(self.widgets_chunks.table_chunk.width)
                    && horizontal_scroll_offset < self.table_model.query_result.columns.len() - 1
                {
                    self.table_model.horizontal_scroll_offset += 1;
                }
            }
            ResultsTableAction::JumpUp => {
                let jump = 10;
                let new_index = current.saturating_sub(jump);
                self.table_model.selected_row = Some(new_index);
                // Only scroll up if cursor would go ABOVE the viewport
                if new_index < self.table_model.vertical_scroll_offset {
                    self.table_model.vertical_scroll_offset = new_index;
                }
            }
            ResultsTableAction::JumpDown => {
                let jump = 10;
                let mut new_index = current + jump;
                if new_index >= total_rows {
                    new_index = total_rows - 1;
                }
                self.table_model.selected_row = Some(new_index);
                
                // Only scroll down if cursor would go BELOW the viewport
                let viewport_bottom = self.table_model.vertical_scroll_offset + visible_rows;
                if new_index >= viewport_bottom {
                    self.table_model.vertical_scroll_offset = new_index.saturating_sub(visible_rows - 1);
                }
            }
            ResultsTableAction::GoToFirst => {
                self.table_model.selected_row = Some(0);
                self.table_model.vertical_scroll_offset = 0;
            }
            ResultsTableAction::GoToLast => {
                self.table_model.selected_row = Some(total_rows - 1);
                self.table_model.vertical_scroll_offset = total_rows.saturating_sub(visible_rows);
            }
            ResultsTableAction::YankSelection => {
                if let Some(row) = self.table_model.get_selected_row_data() {
                    let mut clipboard = Clipboard::new().unwrap();
                    clipboard.set_text(serde_json::to_string_pretty(&row).unwrap()).unwrap();
                    self.report_message(
                        "Saved current row to clipboard.",
                        MsgKind::Success,
                        MsgLifetime::Short
                    );
                }
            }
        }
    }

    async fn update_db(&mut self, action: DbAction) -> Result<()> {
        match action {
            DbAction::QueryTable(table_name) => {
                self.statusline_model.is_loading = true;
                
                // Spawn background task to query
                let driver = self.db_driver.clone();
                let tx = self.action_tx.clone();
                let table_name_clone = table_name.clone();
                
                tokio::spawn(async move {
                    let mut driver = driver.lock().await;
                    if let Ok(results) = driver.query(&table_name_clone).await {
                        if let Ok(current_page) = driver.get_current_page(&table_name_clone).await {
                            let _ = tx.send(Action::Db(DbAction::QueryTableComplete(
                                table_name_clone,
                                results,
                                current_page,
                            )));
                        }
                    }
                });
            },
            DbAction::QueryTableComplete(table_name, results, current_page) => {
                self.selected_table = Some(table_name.clone());

                let rows_fetched = results.rows.len();
                self.table_model.query_result = results;

                self.focused_view = View::ResultsTable;
                self.table_model.table_name = table_name.clone();
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.current_pos = current_page;
                self.table_model.selected_row = Some(0);
                
                let msg = format!("Fetched {} rows", rows_fetched);
                self.report_message(msg, MsgKind::Neutral, MsgLifetime::Short);
                
                self.statusline_model.is_loading = false;
            },
            DbAction::NextPage => {
                if let Some(selected_table) = &self.selected_table {
                    self.statusline_model.is_loading = true;
                    
                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let table_name = selected_table.clone();
                    
                    tokio::spawn(async move {
                        let mut driver = driver.lock().await;
                        if driver.next_page(&table_name).await.is_ok() {
                            if let Ok(results) = driver.query(&table_name).await {
                                if results.rows.len() > 0 {
                                    if let Ok(current_page) = driver.get_current_page(&table_name).await {
                                        let _ = tx.send(Action::Db(DbAction::NextPageComplete(
                                            table_name,
                                            results,
                                            current_page,
                                        )));
                                    }
                                }
                            }
                        }
                    });
                }
            }
            DbAction::NextPageComplete(table_name, results, current_page) => {
                self.table_model.query_result = results;
                self.table_model.table_name = table_name;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.current_pos = current_page;
                self.table_model.reset(Some(0));
                
                self.statusline_model.is_loading = false;
            }
            DbAction::PrevPage => {
                if let Some(selected_table) = &self.selected_table {
                    self.statusline_model.is_loading = true;
                    
                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let table_name = selected_table.clone();
                    
                    tokio::spawn(async move {
                        let mut driver = driver.lock().await;
                        if driver.prev_page(&table_name).await.is_ok() {
                            if let Ok(results) = driver.query(&table_name).await {
                                if let Ok(current_page) = driver.get_current_page(&table_name).await {
                                    let _ = tx.send(Action::Db(DbAction::PrevPageComplete(
                                        table_name,
                                        results,
                                        current_page,
                                    )));
                                }
                            }
                        }
                    });
                }
            }
            DbAction::PrevPageComplete(table_name, results, current_page) => {
                self.table_model.query_result = results;
                self.table_model.table_name = table_name;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.current_pos = current_page;
                self.table_model.reset(Some(0));
                
                self.statusline_model.is_loading = false;
            }
        };

        return Ok(());
    }
    
    fn update_json_view(&mut self, action: JsonViewAction) -> Result<()> {
        match action {
            JsonViewAction::MoveUp => {
                self.json_view_model.scroll_y = self.json_view_model.scroll_y.saturating_sub(1);
            },
            JsonViewAction::MoveDown => {
                if self.json_view_model.data.is_some() {
                    self.json_view_model.scroll_y = self.json_view_model.scroll_y + 1;
                }
            },
            JsonViewAction::GoToFirst => {
                self.json_view_model.scroll_y = 0;
            }
        }
        
        return Ok(());
    }
}
