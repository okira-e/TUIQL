use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::AppCmd;
use crate::actions::CmdLineAction;
use crate::actions::DbAction;
use crate::actions::ExplorerAction;
use crate::actions::JsonViewAction;
use crate::actions::ResultsTableAction;
use crate::app::App;
use crate::app::Pane;
use crate::app::RightView;
use crate::app::View;
use crate::commander::GotoCmd;
use crate::config::project::append_history;
use crate::drivers::OrderBy;
use crate::models::explorer::ExplorerItem;
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
            AppAction::CyclePane => match focused_view {
                View::Explorer => {
                    self.focus_pane(Pane::Right);
                }
                _ => {
                    self.focus_pane(Pane::Left);
                }
            },
            AppAction::SelectTable(name) => {
                self.select_table(name);
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
                self.focus_pane(Pane::StatusLine);
            }
            AppAction::CloseJsonView => {
                self.right_view = RightView::ResultsTable;
            }
            AppAction::ReportError(err_report) => {
                let msg = format!("{}", err_report);
                self.report_message(&msg, MsgKind::Error, MsgLifetime::Long);
            }
            AppAction::StartLoading => self.is_loading = true,
            AppAction::StopLoading => self.is_loading = false,
            AppAction::ReportMessage(msg, msg_kind, msg_lifetime) => {
                self.report_message(&msg, msg_kind, msg_lifetime);
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

        let current = self.table_model.ratatui_table_state.selected().unwrap_or(0);

        // Calculate how many rows fit in the viewport
        let table_header_and_footer_height = 5;
        let visible_rows = (self.widgets_chunks.table_chunk.height - table_header_and_footer_height) as usize;

        match action {
            ResultsTableAction::MoveUp => {
                let new_index = if current == 0 { 0 } else { current - 1 };
                self.table_model.ratatui_table_state.select(Some(new_index));

                // Only scroll up if cursor would go ABOVE the viewport
                if new_index < *self.table_model.ratatui_table_state.offset_mut() {
                    *self.table_model.ratatui_table_state.offset_mut() = new_index;
                }
            }
            ResultsTableAction::MoveDown => {
                let new_index = if current + 1 >= total_rows {
                    total_rows - 1
                } else {
                    current + 1
                };
                self.table_model.ratatui_table_state.select(Some(new_index));

                // Only scroll down if cursor would go BELOW the viewport
                let viewport_bottom = *self.table_model.ratatui_table_state.offset_mut() + visible_rows;
                if new_index >= viewport_bottom {
                    *self.table_model.ratatui_table_state.offset_mut() = new_index.saturating_sub(visible_rows - 1);
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
                self.table_model.ratatui_table_state.select(Some(new_index));
                // Only scroll up if cursor would go ABOVE the viewport
                if new_index < *self.table_model.ratatui_table_state.offset_mut() {
                    *self.table_model.ratatui_table_state.offset_mut() = new_index;
                }
            }
            ResultsTableAction::JumpDown => {
                let jump = 10;
                let mut new_index = current + jump;
                if new_index >= total_rows {
                    new_index = total_rows - 1;
                }
                self.table_model.ratatui_table_state.select(Some(new_index));

                // Only scroll down if cursor would go BELOW the viewport
                let viewport_bottom = *self.table_model.ratatui_table_state.offset_mut() + visible_rows;
                if new_index >= viewport_bottom {
                    *self.table_model.ratatui_table_state.offset_mut() = new_index.saturating_sub(visible_rows - 1);
                }
            }
            ResultsTableAction::GoToFirstVertically => {
                self.table_model.ratatui_table_state.select(Some(0));
                *self.table_model.ratatui_table_state.offset_mut() = 0;
            }
            ResultsTableAction::GoToLastVertically => {
                self.table_model.ratatui_table_state.select(Some(total_rows - 1));
                *self.table_model.ratatui_table_state.offset_mut() = total_rows.saturating_sub(visible_rows);
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
            DbAction::QueryTable => {
                if let Some(table_name) = self.table_model.table_name.clone() {
                    self.update(Action::App(AppAction::StartLoading));

                    // Spawn background task to query
                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let order_by = self.table_model.query_state.order_by.clone();
                    let offset = self.table_model.query_state.offset;
                    let limit = self.table_model.query_state.limit;

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            let mut driver = driver.lock().await;
                            let results = driver.query(&table_name, order_by, offset, limit).await?;
                            let _ = tx.send(Action::Db(DbAction::QueryTableComplete(results)));

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
            DbAction::QueryCount => match self.table_model.table_name.clone() {
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
                    self.focus_pane(Pane::Right);
                }
                None => {
                    self.report_message(
                        "No table is currently selected",
                        MsgKind::Error,
                        MsgLifetime::Short,
                    );
                    self.focus_pane(Pane::Left);
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
            DbAction::QueryTableComplete(results) => {
                let rows_fetched = results.rows.len();
                self.table_model.query_result = results;

                self.table_model.current_page = 0;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.ratatui_table_state.select(Some(0));
                self.table_model.current_page = 0;
                self.focus_pane(Pane::Right);

                let msg = format!("Fetched {} rows", rows_fetched);
                self.report_message(&msg, MsgKind::Neutral, MsgLifetime::Short);
            }
            DbAction::NextPage => {
                if let Some(selected_table) = self.table_model.table_name.clone() {
                    self.update(Action::App(AppAction::StartLoading));

                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let table_name = selected_table.clone();
                    let order_by = self.table_model.query_state.order_by.clone();
                    let mut offset = self.table_model.query_state.offset;
                    let limit = self.table_model.query_state.limit;

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            let mut driver = driver.lock().await;
                            offset += limit; // new offset
                            let results = driver.query(&table_name, order_by, offset, limit).await?;
                            if !results.rows.is_empty() {
                                let _ = tx.send(Action::Db(DbAction::NextPageComplete(results, offset)));
                            } else {
                                offset -= limit; // There is no next page. Reset the offset and send it back
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
            DbAction::NextPageComplete(results, new_offset) => {
                self.table_model.query_result = results;
                self.table_model.current_page += 1;
                self.table_model.query_state.offset = new_offset;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.reset_ui(Some(0));

                self.update(Action::App(AppAction::StopLoading));
            }
            DbAction::PrevPage => {
                if let Some(selected_table) = self.table_model.table_name.clone() {
                    if self.table_model.current_page == 0 {
                        return;
                    }

                    self.update(Action::App(AppAction::StartLoading));

                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let table_name = selected_table.clone();
                    let order_by = self.table_model.query_state.order_by.clone();
                    let mut offset = self.table_model.query_state.offset;
                    let limit = self.table_model.query_state.limit;

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            offset = offset.saturating_sub(limit);

                            let mut driver = driver.lock().await;
                            let results = driver.query(&table_name, order_by, offset, limit).await?;
                            let _ = tx.send(Action::Db(DbAction::PrevPageComplete(results, offset)));

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
            DbAction::PrevPageComplete(results, new_offset) => {
                self.table_model.query_result = results;
                self.table_model.current_page = self.table_model.current_page.saturating_sub(1);
                self.table_model.query_state.offset = new_offset;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.reset_ui(Some(0));

                self.update(Action::App(AppAction::StopLoading));
            }
            DbAction::GotoPageComplete(results, page, offset) => {
                self.table_model.query_result = results;
                self.table_model.current_page = page.saturating_sub(1);
                self.table_model.query_state.offset = offset;
                self.table_model.results_row_count = self.table_model.query_result.rows.len();
                self.table_model.reset_ui(Some(0));
                self.focus_pane(Pane::Right);

                self.update(Action::App(AppAction::StopLoading));
            }
        };
    }

    fn update_cmdline(&mut self, action: CmdLineAction) {
        match action {
            CmdLineAction::Execute => {
                let cmd = std::mem::take(&mut self.statusline_model.cmd.text);
                self.statusline_model.cmd.cursor = 0;
                self.statusline_model.history_cursor = 0;

                let action = self.evaluate_app_action_from_cmd(&cmd);
                match action {
                    Ok(action) => {
                        self.update(action);
                        self.statusline_model.mode = StatusLineMode::Status;
                    }
                    Err(err) => {
                        self.report_message(&err.to_string(), MsgKind::Error, MsgLifetime::Short);
                        self.focus_pane(Pane::Left);
                    }
                };

                if let Some(config) = self.config.as_mut() {
                    if let Err(err) = append_history(config, cmd) {
                        self.report_message(&err.to_string(), MsgKind::Error, MsgLifetime::Long);
                    }
                }
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
            CmdLineAction::PopWord => {
                if self.statusline_model.cmd.cursor > 0 {
                    let text = &self.statusline_model.cmd.text;
                    let mut new_cursor = self.statusline_model.cmd.cursor;

                    // skip trailing whitespace
                    while new_cursor > 0 && text.chars().nth(new_cursor - 1).map_or(false, |c| c.is_whitespace()) {
                        new_cursor -= 1;
                    }
                    // skip the word
                    while new_cursor > 0 && text.chars().nth(new_cursor - 1).map_or(false, |c| !c.is_whitespace()) {
                        new_cursor -= 1;
                    }

                    self.statusline_model
                        .cmd
                        .text
                        .drain(new_cursor..self.statusline_model.cmd.cursor);
                    self.statusline_model.cmd.cursor = new_cursor;
                }
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
            CmdLineAction::TogglePrevCommand => {
                if let Some(config) = &self.config {
                    // while loop for skipping/hopping two identical commands after each other.
                    while self.statusline_model.history_cursor < config.commands.history.len() {
                        let text = config
                            .commands
                            .history
                            .get(self.statusline_model.history_cursor) // Starts at 0. Use before increment
                            .unwrap()
                            .clone();

                        if text == self.statusline_model.cmd.text {
                            self.statusline_model.history_cursor += 1;
                            continue;
                        }

                        self.statusline_model.cmd.text = text;
                        self.statusline_model.cmd.cursor = self.statusline_model.cmd.text.len();
                        self.statusline_model.history_cursor += 1;
                        break;
                    }
                }
            }
            CmdLineAction::ToggleNextCommand => {
                if let Some(config) = &self.config {
                    // while loop for skipping/hopping two identical commands after each other.
                    while self.statusline_model.history_cursor != 0 {
                        self.statusline_model.history_cursor -= 1;
                        let text = config
                            .commands
                            .history
                            .get(self.statusline_model.history_cursor.saturating_sub(1))
                            .unwrap()
                            .clone();

                        if text == self.statusline_model.cmd.text {
                            continue;
                        }

                        self.statusline_model.cmd.text = text;
                        self.statusline_model.cmd.cursor = self.statusline_model.cmd.text.len();
                        break;
                    }

                    if self.statusline_model.history_cursor == 0 {
                        self.statusline_model.cmd.text = String::new();
                        self.statusline_model.cmd.cursor = 0;
                    }
                }
            }
            CmdLineAction::Exit => {
                self.focus_pane(self.prev_pane);
                self.statusline_model = StatusLineModel::default();
            }
        }
    }

    fn update_cmd(&mut self, action: AppCmd) {
        match action {
            AppCmd::Count => self.update(Action::Db(DbAction::QueryCount)),
            AppCmd::Goto(sub_cmd) => match sub_cmd {
                GotoCmd::Page(page) => match self.table_model.table_name.clone() {
                    Some(table) => {
                        let db_driver = self.db_driver.clone();
                        let tx = self.action_tx.clone();
                        let order_by = self.table_model.query_state.order_by.clone();
                        let mut offset = self.table_model.query_state.offset;
                        let limit = self.table_model.query_state.limit;

                        tokio::spawn(async move {
                            let res: Result<()> = async {
                                let mut driver = db_driver.lock().await;

                                let app_page = page.saturating_sub(1);
                                let new_offset = app_page * limit;
                                let old_offset = offset;
                                if new_offset != old_offset {
                                    // Fetch with the new offset and commit the new offset and page if there are results.
                                    offset = new_offset;

                                    let results = driver.query(&table, order_by, offset, limit).await?;

                                    if !results.rows.is_empty() {
                                        let _ = tx.send(Action::Db(DbAction::GotoPageComplete(
                                            results, page, offset,
                                        )));
                                    } else {
                                        offset = old_offset;
                                    }
                                }

                                eyre::Ok(())
                            }
                            .await;

                            if let Err(err) = res {
                                let _ = tx.send(Action::App(AppAction::ReportError(err)));
                            }
                        });
                    }
                    None => {
                        self.report_message(
                            "No table is currently selected",
                            MsgKind::Error,
                            MsgLifetime::Short,
                        );
                        self.focus_pane(Pane::Left);
                    }
                },
                GotoCmd::Table(table_name) => {
                    let matches: Vec<&ExplorerItem> = self
                        .explorer_model
                        .items
                        .iter()
                        .filter(|it| it.name == table_name)
                        .collect();

                    if !matches.is_empty() {
                        let table = matches[0];
                        self.select_table(table.name.clone());
                    } else {
                        self.report_message(
                            &format!("Table \"{}\" is not found", table_name),
                            MsgKind::Error,
                            MsgLifetime::Short,
                        );
                        self.focus_pane(Pane::Left);
                    }
                }
            },
            AppCmd::Sort(column, direction) => match self.table_model.table_name.clone() {
                Some(table) => {
                    self.update(Action::App(AppAction::StartLoading));

                    let order_by = match column {
                        Some(col) => Some(OrderBy::new(vec![col], direction)),
                        None => None,
                    };
                    self.table_model.query_state.order_by = order_by.clone();
                    self.table_model.query_state.offset = 0;

                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let limit = self.table_model.query_state.limit;

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            let mut driver = driver.lock().await;
                            let results = driver.query(&table, order_by, 0, limit).await?;
                            let _ = tx.send(Action::Db(DbAction::QueryTableComplete(results)));

                            eyre::Ok(())
                        }
                        .await;

                        let _ = tx.send(Action::App(AppAction::StopLoading));

                        if let Err(err) = res {
                            let _ = tx.send(Action::App(AppAction::ReportError(err)));
                        }
                    });
                }
                None => {
                    self.report_message(
                        "No table is currently selected",
                        MsgKind::Error,
                        MsgLifetime::Short,
                    );
                    self.focus_pane(Pane::Left);
                }
            },
            AppCmd::Limit(limit) => match self.table_model.table_name.clone() {
                Some(table) => {
                    self.update(Action::App(AppAction::StartLoading));

                    self.table_model.query_state.limit = limit;
                    self.table_model.query_state.offset = 0;
                    self.table_model.current_page = 0;

                    let driver = self.db_driver.clone();
                    let tx = self.action_tx.clone();
                    let order_by = self.table_model.query_state.order_by.clone();

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            let mut driver = driver.lock().await;
                            let results = driver.query(&table, order_by, 0, limit).await?;
                            let _ = tx.send(Action::Db(DbAction::QueryTableComplete(results)));

                            eyre::Ok(())
                        }
                        .await;

                        let _ = tx.send(Action::App(AppAction::StopLoading));

                        if let Err(err) = res {
                            let _ = tx.send(Action::App(AppAction::ReportError(err)));
                        }
                    });
                }
                None => {
                    self.table_model.query_state.limit = limit;
                    self.report_message(
                        &format!("Limit set to {}", limit),
                        MsgKind::Neutral,
                        MsgLifetime::Short,
                    );
                }
            },
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
