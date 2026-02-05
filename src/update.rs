use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::CmdAction;
use crate::actions::CmdLineAction;
use crate::actions::DbAction;
use crate::actions::ExplorerAction;
use crate::actions::JsonViewAction;
use crate::actions::ResultsTableAction;
use crate::app::App;
use crate::app::Pane;
use crate::app::RightView;
use crate::app::View;
use crate::models::explorer::ExplorerItemKind;
use crate::models::statusline::MsgKind;
use crate::models::statusline::MsgLifetime;
use crate::models::statusline::StatusLineMode;
use crate::models::statusline::StatusLineModel;
use arboard::Clipboard;
use color_eyre::Result;
use color_eyre::eyre;
use tracing::debug;

impl App {
    pub fn update(&mut self, action: Action) {
        if !matches!(action, Action::App(AppAction::Tick)) {
            debug!("Received action: {:?}", action);
        }

        match action {
            Action::App(action) => self.update_app(action),
            Action::Db(action) => self.update_db(action),
            Action::Explorer(action) => self.update_explorer(action),
            Action::ResultsTable(action) => self.update_results_table(action),
            Action::JsonView(action) => self.update_json_view(action),
            Action::CmdLine(action) => self.update_cmdline(action),
            Action::Cmd(action) => self.update_cmd(action),
            Action::None => {}
        };
    }

    fn update_app(&mut self, action: AppAction) {
        let focused_view = self.get_focused_view();
        match action {
            AppAction::Quit => {
                self.quit();
            }
            AppAction::Tick => {
                // Increment tick count for spinner animation
                self.statusline_model.spinner_animation_tick_count =
                    self.statusline_model.spinner_animation_tick_count.wrapping_add(1);

                // Check if message has expired
                if self.statusline_model.mode == StatusLineMode::Status {
                    if self.statusline_model.msg.created_at.elapsed() > self.statusline_model.msg.lifetime.to_duration()
                    {
                        self.statusline_model.reset();
                    }
                }
            }
            AppAction::CyclePane => {
                if focused_view == View::Explorer {
                    self.focused_pane = Pane::Right;
                } else {
                    self.focused_pane = Pane::Left;
                }
            }
            AppAction::SelectTable(name) => {
                self.table_model.reset_ui(Some(0));
                self.table_model.total_count = None;
                self.focused_pane = Pane::Right;

                let driver = self.db_driver.clone();
                let tx = self.action_tx.clone();

                tokio::spawn(async move {
                    driver.lock().await.reset_query_state();
                    let _ = tx.send(Action::Db(DbAction::QueryTable(name)));
                });
            }
            AppAction::Resize(w, h) => {
                self.area.width = w;
                self.area.height = h;
                self.calculate_widgets_chunks();
                self.table_model.reset_ui(Some(0));
            }
            AppAction::ViewSelectedRowAsJson => {
                let data = self.table_model.get_selected_row_data();
                if data.is_none() {
                    return;
                }

                self.json_view_model.data = data;

                self.json_view_model.scroll_y = 0;
                self.right_view = RightView::JsonView;
            }
            AppAction::SetCommandMode => {
                self.statusline_model.mode = StatusLineMode::Command;
                self.focused_pane = Pane::StatusLine;
            }
            AppAction::CloseJsonView => {
                self.right_view = RightView::ResultsTable;
            }
            AppAction::ReportError(err_report) => {
                let msg = format!("{}", err_report);
                self.report_message(msg, MsgKind::Error, MsgLifetime::Long);
            }
            AppAction::StartLoading => self.is_loading = true,
            AppAction::StopLoading => self.is_loading = false,
            AppAction::ReportMessage(msg, msg_kind, msg_lifetime) => {
                self.report_message(msg, msg_kind, msg_lifetime);
            }
        }
    }

    fn update_explorer(&mut self, action: ExplorerAction) {
        let model = &mut self.explorer_model;

        match action {
            ExplorerAction::MoveUp => {
                let visible_items = match model.focused_tab {
                    ExplorerItemKind::Table => model.get_items_by_kind(ExplorerItemKind::Table),
                    ExplorerItemKind::View => model.get_items_by_kind(ExplorerItemKind::View),
                    ExplorerItemKind::MaterializedView => model.get_items_by_kind(ExplorerItemKind::MaterializedView),
                };

                if visible_items.is_empty() {
                    return;
                }

                model.table_state.select_previous();

                if let Some(selected) = model.table_state.selected() {
                    model.focused_item = Some(visible_items[selected].clone());
                }
            }
            ExplorerAction::MoveDown => {
                let visible_items = match model.focused_tab {
                    ExplorerItemKind::Table => model.get_items_by_kind(ExplorerItemKind::Table),
                    ExplorerItemKind::View => model.get_items_by_kind(ExplorerItemKind::View),
                    ExplorerItemKind::MaterializedView => model.get_items_by_kind(ExplorerItemKind::MaterializedView),
                };

                if visible_items.is_empty() {
                    return;
                }

                let total_rows = visible_items.len();
                let current = model.table_state.selected().unwrap_or(0);
                let new_index = if model.table_state.selected().unwrap_or(0) + 1 >= total_rows {
                    total_rows - 1
                } else {
                    current + 1
                };
                model.table_state.select(Some(new_index));

                if let Some(selected) = model.table_state.selected() {
                    model.focused_item = Some(visible_items[selected].clone());
                }
            }
            ExplorerAction::NextTab => {
                model.focused_tab = match model.focused_tab {
                    ExplorerItemKind::Table => ExplorerItemKind::View,
                    ExplorerItemKind::View => ExplorerItemKind::MaterializedView,
                    ExplorerItemKind::MaterializedView => ExplorerItemKind::Table,
                };

                // Reset table state
                let tab_items = model.get_items_by_kind(model.focused_tab);
                if tab_items.is_empty() {
                    model.table_state.select(None);
                    model.focused_item = None;
                } else {
                    model.table_state.select(Some(0));
                    model.focused_item = Some(tab_items[0].clone());
                }
            }
            ExplorerAction::PrevTab => {
                model.focused_tab = match model.focused_tab {
                    ExplorerItemKind::Table => ExplorerItemKind::MaterializedView,
                    ExplorerItemKind::View => ExplorerItemKind::Table,
                    ExplorerItemKind::MaterializedView => ExplorerItemKind::View,
                };

                // Reset table state
                let tab_items = model.get_items_by_kind(model.focused_tab);
                if tab_items.is_empty() {
                    model.table_state.select(None);
                    model.focused_item = None;
                } else {
                    model.table_state.select(Some(0));
                    model.focused_item = Some(tab_items[0].clone());
                }
            }
            ExplorerAction::GoToFirst => {
                let visible_items = match model.focused_tab {
                    ExplorerItemKind::Table => model.get_items_by_kind(ExplorerItemKind::Table),
                    ExplorerItemKind::View => model.get_items_by_kind(ExplorerItemKind::View),
                    ExplorerItemKind::MaterializedView => model.get_items_by_kind(ExplorerItemKind::MaterializedView),
                };

                if visible_items.is_empty() {
                    return;
                }

                model.table_state.select(Some(0));

                if let Some(selected) = model.table_state.selected() {
                    model.focused_item = Some(visible_items[selected].clone());
                }
            }
            ExplorerAction::GoToLast => {
                let visible_items = match model.focused_tab {
                    ExplorerItemKind::Table => model.get_items_by_kind(ExplorerItemKind::Table),
                    ExplorerItemKind::View => model.get_items_by_kind(ExplorerItemKind::View),
                    ExplorerItemKind::MaterializedView => model.get_items_by_kind(ExplorerItemKind::MaterializedView),
                };

                if visible_items.is_empty() {
                    return;
                }

                let total_rows = visible_items.len();
                if total_rows == 0 {
                    return;
                }
                model.table_state.select(Some(total_rows - 1));

                if let Some(selected) = model.table_state.selected() {
                    model.focused_item = Some(visible_items[selected].clone());
                }
            }
        };
    }

    fn update_results_table(&mut self, action: ResultsTableAction) {
        let total_rows = self.table_model.query_result.rows.len();
        if total_rows == 0 {
            return;
        }

        let current = self.table_model.table_state.selected().unwrap_or(0);

        // Calculate how many rows fit in the viewport
        let table_header_and_footer_height = 5;
        let visible_rows = (self.widgets_chunks.table_chunk.height - table_header_and_footer_height) as usize;

        match action {
            ResultsTableAction::MoveUp => {
                let new_index = if current == 0 { 0 } else { current - 1 };
                self.table_model.table_state.select(Some(new_index));

                // Only scroll up if cursor would go ABOVE the viewport
                if new_index < *self.table_model.table_state.offset_mut() {
                    *self.table_model.table_state.offset_mut() = new_index;
                }
            }
            ResultsTableAction::MoveDown => {
                let new_index = if current + 1 >= total_rows {
                    total_rows - 1
                } else {
                    current + 1
                };
                self.table_model.table_state.select(Some(new_index));

                // Only scroll down if cursor would go BELOW the viewport
                let viewport_bottom = *self.table_model.table_state.offset_mut() + visible_rows;
                if new_index >= viewport_bottom {
                    *self.table_model.table_state.offset_mut() = new_index.saturating_sub(visible_rows - 1);
                }
            }
            ResultsTableAction::ScrollLeft => {
                if self.table_model.horizontal_scroll_offset > 0 {
                    self.table_model.horizontal_scroll_offset -= 1;
                }
            }
            ResultsTableAction::ScrollRight => {
                let horizontal_scroll_offset = self.table_model.horizontal_scroll_offset;

                if self
                    .table_model
                    .should_draw_scrollbar(self.widgets_chunks.table_chunk.width)
                    && horizontal_scroll_offset < self.table_model.query_result.columns.len() - 1
                {
                    self.table_model.horizontal_scroll_offset += 1;
                }
            }
            ResultsTableAction::JumpUp => {
                let jump = 10;
                let new_index = current.saturating_sub(jump);
                self.table_model.table_state.select(Some(new_index));
                // Only scroll up if cursor would go ABOVE the viewport
                if new_index < *self.table_model.table_state.offset_mut() {
                    *self.table_model.table_state.offset_mut() = new_index;
                }
            }
            ResultsTableAction::JumpDown => {
                let jump = 10;
                let mut new_index = current + jump;
                if new_index >= total_rows {
                    new_index = total_rows - 1;
                }
                self.table_model.table_state.select(Some(new_index));

                // Only scroll down if cursor would go BELOW the viewport
                let viewport_bottom = *self.table_model.table_state.offset_mut() + visible_rows;
                if new_index >= viewport_bottom {
                    *self.table_model.table_state.offset_mut() = new_index.saturating_sub(visible_rows - 1);
                }
            }
            ResultsTableAction::GoToFirstVertically => {
                self.table_model.table_state.select(Some(0));
                *self.table_model.table_state.offset_mut() = 0;
            }
            ResultsTableAction::GoToLastVertically => {
                self.table_model.table_state.select(Some(total_rows - 1));
                *self.table_model.table_state.offset_mut() = total_rows.saturating_sub(visible_rows);
            }
            ResultsTableAction::YankSelection => {
                if let Some(row) = self.table_model.get_selected_row_data() {
                    let mut clipboard = Clipboard::new().unwrap();
                    clipboard.set_text(serde_json::to_string_pretty(&row).unwrap()).unwrap();
                    self.report_message(
                        "Saved current row to clipboard.",
                        MsgKind::Success,
                        MsgLifetime::Short,
                    );
                }
            }
            ResultsTableAction::GoToFirstHorizontally => {
                self.table_model.horizontal_scroll_offset = 0;
            }
            ResultsTableAction::GoToLastHorizontally => {
                self.table_model.horizontal_scroll_offset = self.table_model.query_result.columns.len() - 1;
            }
        }
    }

    fn update_db(&mut self, action: DbAction) {
        match action {
            DbAction::QueryTable(table_name) => {
                self.update(Action::App(AppAction::StartLoading));

                // Spawn background task to query
                let driver = self.db_driver.clone();
                let tx = self.action_tx.clone();

                tokio::spawn(async move {
                    let res: Result<()> = async {
                        let mut driver = driver.lock().await;
                        let results = driver.query(&table_name).await?;
                        let current_page = driver.get_current_pos(&table_name).await?;
                        let _ = tx.send(Action::Db(DbAction::QueryTableComplete(
                            table_name,
                            results,
                            current_page,
                        )));

                        eyre::Ok(())
                    }
                    .await;

                    let _ = tx.send(Action::App(AppAction::StopLoading));

                    if let Err(err) = res {
                        let _ = tx.send(Action::App(AppAction::ReportError(err)));
                    }
                });
            }
            DbAction::QueryCount => match self.selected_table.clone() {
                Some(table) => {
                    self.update(Action::App(AppAction::StartLoading));

                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();

                    tokio::spawn(async move {
                        let res: Result<usize> = async {
                            let mut driver = driver.lock().await;
                            let count = driver.query_count(&table).await?;

                            eyre::Ok(count)
                        }
                        .await;

                        let _ = tx.send(Action::App(AppAction::StopLoading));
                        let _ = tx.send(Action::Db(DbAction::QueryCountComplete(res)));
                    });
                    self.focused_pane = Pane::Right;
                }
                None => {
                    self.report_message(
                        "No table is current selected",
                        MsgKind::Error,
                        MsgLifetime::Short,
                    );
                    self.focused_pane = Pane::Left;
                }
            },
            DbAction::QueryCountComplete(res) => match res {
                Ok(count) => {
                    let msg = format!("Total rows: {}", count);
                    let _ = self.update(Action::App(AppAction::ReportMessage(
                        msg,
                        MsgKind::Neutral,
                        MsgLifetime::Long,
                    )));
                    self.table_model.total_count = Some(count);
                }
                Err(err) => self.update(Action::App(AppAction::ReportError(err))),
            },
            DbAction::QueryTableComplete(table_name, results, current_pos) => {
                self.selected_table = Some(table_name.clone());

                let rows_fetched = results.rows.len();
                self.table_model.query_result = results;

                self.table_model.table_name = table_name.clone();
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.current_pos = current_pos;
                self.table_model.table_state.select(Some(0));
                self.table_model.current_page = 0;

                let msg = format!("Fetched {} rows", rows_fetched);
                self.report_message(msg, MsgKind::Neutral, MsgLifetime::Short);
            }
            DbAction::NextPage => {
                if let Some(selected_table) = self.selected_table.clone() {
                    self.update(Action::App(AppAction::StartLoading));

                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let table_name = selected_table.clone();

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            let mut driver = driver.lock().await;
                            if let Some(results) = driver.next_page(&table_name).await? {
                                let current_pos = driver.get_current_pos(&table_name).await?;
                                let current_page = driver.get_current_page();
                                let _ = tx.send(Action::Db(DbAction::NextPageComplete(
                                    table_name,
                                    results,
                                    current_pos,
                                    current_page,
                                )));
                            }

                            eyre::Ok(())
                        }
                        .await;

                        let _ = tx.send(Action::App(AppAction::StopLoading));

                        if let Err(err) = res {
                            let _ = tx.send(Action::App(AppAction::ReportError(err)));
                        }
                    });
                }
            }
            DbAction::NextPageComplete(table_name, results, current_pos, current_page) => {
                self.table_model.query_result = results;
                self.table_model.table_name = table_name;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.current_pos = current_pos;
                self.table_model.current_page = current_page;
                self.table_model.reset_ui(Some(0));

                self.update(Action::App(AppAction::StopLoading));
            }
            DbAction::PrevPage => {
                if let Some(selected_table) = self.selected_table.clone() {
                    self.update(Action::App(AppAction::StartLoading));

                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let table_name = selected_table.clone();

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            let mut driver = driver.lock().await;
                            driver.prev_page(&table_name).await?;
                            let results = driver.query(&table_name).await?;
                            let current_pos = driver.get_current_pos(&table_name).await?;
                            let current_page = driver.get_current_page();
                            let _ = tx.send(Action::Db(DbAction::PrevPageComplete(
                                table_name,
                                results,
                                current_pos,
                                current_page,
                            )));

                            eyre::Ok(())
                        }
                        .await;

                        let _ = tx.send(Action::App(AppAction::StopLoading));

                        if let Err(err) = res {
                            let _ = tx.send(Action::App(AppAction::ReportError(err)));
                        }
                    });
                }
            }
            DbAction::PrevPageComplete(table_name, results, current_pos, current_page) => {
                self.table_model.query_result = results;
                self.table_model.table_name = table_name;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.current_pos = current_pos;
                self.table_model.current_page = current_page;
                self.table_model.reset_ui(Some(0));

                self.update(Action::App(AppAction::StopLoading));
            }
        };
    }

    fn update_cmdline(&mut self, action: CmdLineAction) {
        match action {
            CmdLineAction::Execute => {
                let cmd = std::mem::take(&mut self.statusline_model.cmd.text);
                self.statusline_model.cmd.cursor = 0;

                let action = self.evaluate_cmd(&cmd);
                self.update(action);
            }
            CmdLineAction::AddChar(character) => {
                if character == ' ' && self.statusline_model.cmd.text.is_empty() {
                    return;
                }

                self.statusline_model
                    .cmd
                    .text
                    .insert(self.statusline_model.cmd.cursor, character);
                self.statusline_model.cmd.cursor += 1;
            }
            CmdLineAction::PopChar => {
                if self.statusline_model.cmd.cursor > 0 {
                    self.statusline_model.cmd.cursor -= 1;
                    self.statusline_model.cmd.text.remove(self.statusline_model.cmd.cursor);
                }
            }
            CmdLineAction::MoveLeft => {
                self.statusline_model.cmd.cursor = self.statusline_model.cmd.cursor.saturating_sub(1);
            }
            CmdLineAction::MoveRight => {
                let new_pos = self.statusline_model.cmd.cursor + 1;
                self.statusline_model.cmd.cursor = new_pos.min(self.statusline_model.cmd.text.len());
            }
            CmdLineAction::Exit => {
                self.focused_pane = Pane::Left;
                self.statusline_model = StatusLineModel::default();
            }
        }
    }

    fn update_cmd(&mut self, action: CmdAction) {
        match action {
            CmdAction::Count => self.update(Action::Db(DbAction::QueryCount)),
        };
    }

    fn update_json_view(&mut self, action: JsonViewAction) {
        match action {
            JsonViewAction::MoveUp => {
                self.json_view_model.scroll_y = self.json_view_model.scroll_y.saturating_sub(1);
            }
            JsonViewAction::MoveDown => {
                if self.json_view_model.data.is_some() {
                    self.json_view_model.scroll_y = self.json_view_model.scroll_y + 1;
                }
            }
            JsonViewAction::GoToFirst => {
                self.json_view_model.scroll_y = 0;
            }
        }
    }
}
