use crate::actions::Action;
use crate::actions::AppAction;
use crate::actions::AppCmd;
use crate::actions::CmdLineAction;
use crate::actions::DbAction;
use crate::actions::ExplorerAction;
use crate::actions::HelpViewAction;
use crate::actions::JsonViewAction;
use crate::actions::ResultsTableAction;
use crate::app::App;
use crate::app::Pane;
use crate::app::RightView;
use crate::app::View;
use crate::app::calculate_widgets_chunks;
use crate::app::evaluate_app_action_from_cmd;
use crate::app::focus_pane;
use crate::app::get_focused_view;
use crate::app::quit;
use crate::app::refresh_suggestions;
use crate::app::report_message;
use crate::app::select_table;
use crate::commander::GotoCmd;
use crate::config;
use crate::models::explorer_model::ExplorerItem;
use crate::models::explorer_model::ExplorerItemKind;
use crate::models::statusline_model::MsgKind;
use crate::models::statusline_model::MsgLifetime;
use crate::models::statusline_model::StatusLineMode;
use crate::models::statusline_model::StatusLineModel;
use crate::settings::load_preset;
use crate::settings::remove_preset;
use crate::settings::save_preset;
use crate::settings::update_settings;
use crate::views;
use arboard::Clipboard;
use color_eyre::Result;
use color_eyre::eyre;
use tracing::debug;

pub fn update(app: &mut App, action: Action) {
    if !matches!(action, Action::App(AppAction::Tick)) {
        debug!("Received action: {:?}", action);
    }

    match action {
        Action::App(action) => update_app(app, action),
        Action::Db(action) => update_db(app, action),
        Action::Explorer(action) => update_explorer(app, action),
        Action::ResultsTable(action) => update_results_table(app, action),
        Action::JsonView(action) => update_json_view(app, action),
        Action::HelpView(action) => update_help_view(app, action),
        Action::CmdLine(action) => update_cmdline(app, action),
        Action::Cmd(action) => update_cmd(app, action),
    };
}

fn update_app(app: &mut App, action: AppAction) {
    let focused_view = get_focused_view(app);
    match action {
        AppAction::Quit => {
            quit(app);
        }
        AppAction::Tick => {
            // Increment tick count for spinner animation
            app.statusline_model.spinner_animation_tick_count =
                app.statusline_model.spinner_animation_tick_count.wrapping_add(1);

            // Check if message has expired
            if app.statusline_model.mode == StatusLineMode::Status {
                if app.statusline_model.msg.created_at.elapsed() > app.statusline_model.msg.lifetime.to_duration() {
                    app.statusline_model.reset();
                }
            }
        }
        AppAction::CyclePane => match focused_view {
            View::Explorer => {
                focus_pane(app, Pane::Right);
            }
            _ => {
                focus_pane(app, Pane::Left);
            }
        },
        AppAction::SelectTable(name) => {
            select_table(app, name);
        }
        AppAction::Resize(w, h) => {
            app.area.width = w;
            app.area.height = h;
            calculate_widgets_chunks(app);
            app.table_model.reset_ui(Some(0));
        }
        AppAction::ViewSelectedRowAsJson => {
            let data = app.table_model.get_selected_row_data();
            if data.is_none() {
                return;
            }

            app.json_view_model.data = data;

            app.json_view_model.scroll_y = 0;
            app.right_view = RightView::JsonView;
        }
        AppAction::SetCommandMode => {
            app.statusline_model.mode = StatusLineMode::Command;
            focus_pane(app, Pane::StatusLine);
            refresh_suggestions(app);
        }
        AppAction::CloseJsonView => {
            app.right_view = RightView::ResultsTable;
        }
        AppAction::ReportError(err_report) => {
            let msg = format!("{}", err_report);
            report_message(app, &msg, MsgKind::Error, MsgLifetime::Long);
            let pane = app.prev_focused_pane;
            focus_pane(app, pane);
        }
        AppAction::StartLoading => app.is_loading = true,
        AppAction::StopLoading => app.is_loading = false,
        AppAction::ReportMessage(msg, msg_kind, msg_lifetime) => {
            report_message(app, &msg, msg_kind, msg_lifetime);
        }
        AppAction::UpdateQueryState(where_clause, order_by) => {
            app.table_model.query_state.where_clause = where_clause.clone();
            app.table_model.query_state.order_by_clause = order_by.clone();
        }
        AppAction::OpenHelp => {
            app.right_view = RightView::Help;
            focus_pane(app, Pane::Right);
        }
        AppAction::CloseHelp => {
            app.right_view = RightView::ResultsTable;

            // Prevent sending focus to the statusline if we opened the help from the command
            let pane_to_focus = if app.prev_focused_pane == Pane::StatusLine {
                Pane::Left
            } else {
                app.prev_focused_pane
            };

            focus_pane(app, pane_to_focus);
        }
    }
}

fn update_explorer(app: &mut App, action: ExplorerAction) {
    let model = &mut app.explorer_model;

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

fn update_results_table(app: &mut App, action: ResultsTableAction) {
    let total_rows = app.table_model.query_result.rows.len();
    if total_rows == 0 {
        return;
    }

    let current = app.table_model.ratatui_table_state.selected().unwrap_or(0);

    // Calculate how many rows fit in the viewport
    let table_header_and_footer_height = 5;
    let visible_rows = (app.widgets_chunks.table_chunk.height - table_header_and_footer_height) as usize;

    match action {
        ResultsTableAction::MoveUp => {
            let new_index = if current == 0 { 0 } else { current - 1 };
            app.table_model.ratatui_table_state.select(Some(new_index));

            // Only scroll up if cursor would go ABOVE the viewport
            if new_index < *app.table_model.ratatui_table_state.offset_mut() {
                *app.table_model.ratatui_table_state.offset_mut() = new_index;
            }
        }
        ResultsTableAction::MoveDown => {
            let new_index = if current + 1 >= total_rows {
                total_rows - 1
            } else {
                current + 1
            };
            app.table_model.ratatui_table_state.select(Some(new_index));

            // Only scroll down if cursor would go BELOW the viewport
            let viewport_bottom = *app.table_model.ratatui_table_state.offset_mut() + visible_rows;
            if new_index >= viewport_bottom {
                *app.table_model.ratatui_table_state.offset_mut() = new_index.saturating_sub(visible_rows - 1);
            }
        }
        ResultsTableAction::ScrollLeft => {
            if app.table_model.horizontal_scroll_offset > 0 {
                app.table_model.horizontal_scroll_offset -= 1;
            }
        }
        ResultsTableAction::ScrollRight => {
            let horizontal_scroll_offset = app.table_model.horizontal_scroll_offset;

            if app
                .table_model
                .should_draw_scrollbar(app.widgets_chunks.table_chunk.width)
                && horizontal_scroll_offset < app.table_model.query_result.columns.len() - 1
            {
                app.table_model.horizontal_scroll_offset += 1;
            }
        }
        ResultsTableAction::JumpUp => {
            let jump = 10;
            let new_index = current.saturating_sub(jump);
            app.table_model.ratatui_table_state.select(Some(new_index));
            // Only scroll up if cursor would go ABOVE the viewport
            if new_index < *app.table_model.ratatui_table_state.offset_mut() {
                *app.table_model.ratatui_table_state.offset_mut() = new_index;
            }
        }
        ResultsTableAction::JumpDown => {
            let jump = 10;
            let mut new_index = current + jump;
            if new_index >= total_rows {
                new_index = total_rows - 1;
            }
            app.table_model.ratatui_table_state.select(Some(new_index));

            // Only scroll down if cursor would go BELOW the viewport
            let viewport_bottom = *app.table_model.ratatui_table_state.offset_mut() + visible_rows;
            if new_index >= viewport_bottom {
                *app.table_model.ratatui_table_state.offset_mut() = new_index.saturating_sub(visible_rows - 1);
            }
        }
        ResultsTableAction::GoToFirstVertically => {
            app.table_model.ratatui_table_state.select(Some(0));
            *app.table_model.ratatui_table_state.offset_mut() = 0;
        }
        ResultsTableAction::GoToLastVertically => {
            app.table_model.ratatui_table_state.select(Some(total_rows - 1));
            *app.table_model.ratatui_table_state.offset_mut() = total_rows.saturating_sub(visible_rows);
        }
        ResultsTableAction::YankSelection => {
            if let Some(row) = app.table_model.get_selected_row_data() {
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(serde_json::to_string_pretty(&row).unwrap()).unwrap();
                report_message(
                    app,
                    "Saved current row to clipboard.",
                    MsgKind::Success,
                    MsgLifetime::Short,
                );
            }
        }
        ResultsTableAction::GoToFirstHorizontally => {
            app.table_model.horizontal_scroll_offset = 0;
        }
        ResultsTableAction::GoToLastHorizontally => {
            app.table_model.horizontal_scroll_offset = app.table_model.query_result.columns.len() - 1;
        }
    }
}

fn update_db(app: &mut App, action: DbAction) {
    match action {
        DbAction::QueryTable => {
            if let Some(table_name) = app.table_model.table_name.clone() {
                update(app, Action::App(AppAction::StartLoading));

                // Spawn background task to query
                let driver = app.db_driver.clone();
                let tx = app.action_tx.clone();
                let order_by = app.table_model.query_state.order_by_clause.clone();
                let where_clause = app.table_model.query_state.where_clause.clone();
                let sort = app.settings.default_sort.clone();
                let offset = app.table_model.query_state.offset;
                let limit = app.table_model.query_state.limit;

                tokio::spawn(async move {
                    let res: Result<()> = async {
                        let mut driver = driver.lock().await;
                        let results = driver
                            .query(
                                &table_name,
                                order_by,
                                where_clause,
                                sort.as_str(),
                                offset,
                                limit,
                            )
                            .await?;
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
        DbAction::QueryCount(ignore_filters) => match app.table_model.table_name.clone() {
            Some(table) => {
                update(app, Action::App(AppAction::StartLoading));

                let driver = app.db_driver.clone();
                let tx = app.action_tx.clone();
                let where_clause = if ignore_filters {
                    None
                } else {
                    app.table_model.query_state.where_clause.clone()
                };

                tokio::spawn(async move {
                    let res: Result<usize> = async {
                        let mut driver = driver.lock().await;
                        let count = driver.query_count(&table, where_clause).await?;

                        eyre::Ok(count)
                    }
                    .await;

                    let _ = tx.send(Action::App(AppAction::StopLoading));
                    let _ = tx.send(Action::Db(DbAction::QueryCountComplete(
                        ignore_filters,
                        res,
                    )));
                });
                focus_pane(app, Pane::Right);
            }
            None => {
                report_message(
                    app,
                    "No table is currently selected",
                    MsgKind::Error,
                    MsgLifetime::Short,
                );
                focus_pane(app, Pane::Left);
            }
        },
        DbAction::QueryCountComplete(ignored_filters, res) => match res {
            Ok(count) => {
                let msg = format!(
                    "{} rows: {}",
                    if ignored_filters { "Total" } else { "Filterd" },
                    count
                );
                update(
                    app,
                    Action::App(AppAction::ReportMessage(
                        msg,
                        MsgKind::Neutral,
                        MsgLifetime::Long,
                    )),
                );
                if !ignored_filters {
                    app.table_model.total_count = Some(count);
                }
            }
            Err(err) => update(app, Action::App(AppAction::ReportError(err))),
        },
        DbAction::QueryTableComplete(results) => {
            let rows_fetched = results.rows.len();
            app.table_model.query_result = results;

            app.table_model.current_page = 0;
            app.table_model.results_row_count = rows_fetched;
            app.table_model.ratatui_table_state.select(Some(0));
            app.table_model.current_page = 0;
            app.table_model.total_count = None;
            focus_pane(app, Pane::Right);

            let msg = format!("Fetched {} rows", rows_fetched);
            report_message(app, &msg, MsgKind::Neutral, MsgLifetime::Short);
        }
        DbAction::NextPage => {
            if let Some(selected_table) = app.table_model.table_name.clone() {
                update(app, Action::App(AppAction::StartLoading));

                let driver = app.db_driver.clone();
                let tx = app.action_tx.clone();
                let table_name = selected_table.clone();
                let order_by = app.table_model.query_state.order_by_clause.clone();
                let where_clause = app.table_model.query_state.where_clause.clone();
                let sort = app.settings.default_sort.clone();
                let mut offset = app.table_model.query_state.offset;
                let limit = app.table_model.query_state.limit;

                tokio::spawn(async move {
                    let res: Result<()> = async {
                        let mut driver = driver.lock().await;
                        offset += limit; // new offset
                        let results = driver
                            .query(
                                &table_name,
                                order_by,
                                where_clause,
                                sort.as_str(),
                                offset,
                                limit,
                            )
                            .await?;
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
            app.table_model.query_result = results;
            app.table_model.current_page += 1;
            app.table_model.query_state.offset = new_offset;
            app.table_model.results_row_count = app.table_model.query_result.rows.len();
            app.table_model.reset_ui(Some(0));

            update(app, Action::App(AppAction::StopLoading));
        }
        DbAction::PrevPage => {
            if let Some(selected_table) = app.table_model.table_name.clone() {
                if app.table_model.current_page == 0 {
                    return;
                }

                update(app, Action::App(AppAction::StartLoading));

                let driver = app.db_driver.clone();
                let tx = app.action_tx.clone();
                let table_name = selected_table.clone();
                let order_by = app.table_model.query_state.order_by_clause.clone();
                let where_clause = app.table_model.query_state.where_clause.clone();
                let sort = app.settings.default_sort.clone();
                let mut offset = app.table_model.query_state.offset;
                let limit = app.table_model.query_state.limit;

                tokio::spawn(async move {
                    let res: Result<()> = async {
                        offset = offset.saturating_sub(limit);

                        let mut driver = driver.lock().await;
                        let results = driver
                            .query(
                                &table_name,
                                order_by,
                                where_clause,
                                sort.as_str(),
                                offset,
                                limit,
                            )
                            .await?;
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
            app.table_model.query_result = results;
            app.table_model.current_page = app.table_model.current_page.saturating_sub(1);
            app.table_model.query_state.offset = new_offset;
            app.table_model.results_row_count = app.table_model.query_result.rows.len();
            app.table_model.reset_ui(Some(0));

            update(app, Action::App(AppAction::StopLoading));
        }
        DbAction::GotoPageComplete(results, page, offset) => {
            app.table_model.query_result = results;
            app.table_model.current_page = page.saturating_sub(1);
            app.table_model.query_state.offset = offset;
            app.table_model.results_row_count = app.table_model.query_result.rows.len();
            app.table_model.reset_ui(Some(0));
            focus_pane(app, Pane::Right);

            update(app, Action::App(AppAction::StopLoading));
        }
    };
}

fn update_cmdline(app: &mut App, action: CmdLineAction) {
    match action {
        CmdLineAction::Execute => {
            let cmd = std::mem::take(&mut app.statusline_model.cmd.text);
            app.statusline_model.cmd.cursor = 0;
            app.statusline_model.history_cursor = 0;

            let action = evaluate_app_action_from_cmd(&cmd);
            match action {
                Ok(action) => {
                    update(app, action);
                    app.statusline_model.mode = StatusLineMode::Status;
                }
                Err(err) => {
                    report_message(app, &err.to_string(), MsgKind::Error, MsgLifetime::Short);
                    focus_pane(app, Pane::Left);
                }
            };

            if let Some(config) = app.config.as_mut() {
                if let Err(err) = config::project::append_history(config, cmd) {
                    report_message(app, &err.to_string(), MsgKind::Error, MsgLifetime::Long);
                }
            }
        }
        CmdLineAction::NextSuggestion => {
            app.statusline_model.cycle_completion(true);
        }
        CmdLineAction::PrevSuggestion => {
            app.statusline_model.cycle_completion(false);
        }
        CmdLineAction::AddChar(character) => {
            if character == ' ' && app.statusline_model.cmd.text.is_empty() {
                return;
            }

            app.statusline_model
                .cmd
                .text
                .insert(app.statusline_model.cmd.cursor, character);
            app.statusline_model.cmd.cursor += 1;
            refresh_suggestions(app);
        }
        CmdLineAction::PopWord => {
            if app.statusline_model.cmd.cursor > 0 {
                let text = &app.statusline_model.cmd.text;
                let mut new_cursor = app.statusline_model.cmd.cursor;

                // skip trailing whitespace
                while new_cursor > 0 && text.chars().nth(new_cursor - 1).map_or(false, |c| c.is_whitespace()) {
                    new_cursor -= 1;
                }
                // skip the word
                while new_cursor > 0 && text.chars().nth(new_cursor - 1).map_or(false, |c| !c.is_whitespace()) {
                    new_cursor -= 1;
                }

                app.statusline_model
                    .cmd
                    .text
                    .drain(new_cursor..app.statusline_model.cmd.cursor);
                app.statusline_model.cmd.cursor = new_cursor;

                refresh_suggestions(app);
            }
        }
        CmdLineAction::PopLine => {
            app.statusline_model.cmd.text.drain(..app.statusline_model.cmd.cursor);
            app.statusline_model.cmd.cursor = 0;
            refresh_suggestions(app);
        }
        CmdLineAction::PopChar => {
            if app.statusline_model.cmd.cursor > 0 {
                app.statusline_model.cmd.cursor -= 1;
                app.statusline_model.cmd.text.remove(app.statusline_model.cmd.cursor);
                refresh_suggestions(app);
            }
        }
        CmdLineAction::MoveLeft => {
            app.statusline_model.cmd.cursor = app.statusline_model.cmd.cursor.saturating_sub(1);
        }
        CmdLineAction::MoveRight => {
            let new_pos = app.statusline_model.cmd.cursor + 1;
            app.statusline_model.cmd.cursor = new_pos.min(app.statusline_model.cmd.text.len());
        }
        CmdLineAction::MoveLeftWord => {
            let text = &app.statusline_model.cmd.text;
            let mut cursor = app.statusline_model.cmd.cursor;

            // skip trailing whitespace
            while cursor > 0 && text.chars().nth(cursor - 1).map_or(false, |c| c.is_whitespace()) {
                cursor -= 1;
            }
            // skip the word
            while cursor > 0 && text.chars().nth(cursor - 1).map_or(false, |c| !c.is_whitespace()) {
                cursor -= 1;
            }

            app.statusline_model.cmd.cursor = cursor;
        }
        CmdLineAction::MoveRightWord => {
            let text = &app.statusline_model.cmd.text;
            let len = text.len();
            let mut cursor = app.statusline_model.cmd.cursor;

            // skip current word
            while cursor < len && text.chars().nth(cursor).map_or(false, |c| !c.is_whitespace()) {
                cursor += 1;
            }
            // skip whitespace
            while cursor < len && text.chars().nth(cursor).map_or(false, |c| c.is_whitespace()) {
                cursor += 1;
            }

            app.statusline_model.cmd.cursor = cursor;
        }
        CmdLineAction::SetText(text) => {
            app.statusline_model.mode = StatusLineMode::Command;
            app.statusline_model.cmd.text = text;
            app.statusline_model.cmd.cursor = app.statusline_model.cmd.text.len();
            focus_pane(app, Pane::StatusLine);
        }
        CmdLineAction::ToggleWhereClause => {
            app.statusline_model.mode = StatusLineMode::Command;
            app.statusline_model.cmd.text = format!(
                "where {}",
                app.table_model
                    .query_state
                    .where_clause
                    .clone()
                    .unwrap_or(String::new())
            );
            app.statusline_model.cmd.cursor = app.statusline_model.cmd.text.len();
            focus_pane(app, Pane::StatusLine);
            refresh_suggestions(app);
        }
        CmdLineAction::ToggleOrderByClause => {
            app.statusline_model.mode = StatusLineMode::Command;
            app.statusline_model.cmd.text = format!(
                "order-by {}",
                app.table_model
                    .query_state
                    .order_by_clause
                    .clone()
                    .unwrap_or(String::new())
            );
            app.statusline_model.cmd.cursor = app.statusline_model.cmd.text.len();
            focus_pane(app, Pane::StatusLine);
            refresh_suggestions(app);
        }
        CmdLineAction::TogglePrevCommand => {
            if let Some(config) = &app.config {
                // while loop for skipping/hopping two identical commands after each other.
                while app.statusline_model.history_cursor < config.commands.history.len() {
                    let text = config
                        .commands
                        .history
                        .get(app.statusline_model.history_cursor) // Starts at 0. Use before increment
                        .unwrap()
                        .clone();

                    if text == app.statusline_model.cmd.text || text.is_empty() {
                        app.statusline_model.history_cursor += 1;
                        continue;
                    }

                    app.statusline_model.cmd.text = text;
                    app.statusline_model.cmd.cursor = app.statusline_model.cmd.text.len();
                    app.statusline_model.history_cursor += 1;
                    break;
                }
            }
            refresh_suggestions(app);
        }
        CmdLineAction::ToggleNextCommand => {
            if let Some(config) = &app.config {
                // while loop for skipping/hopping two identical commands after each other.
                while app.statusline_model.history_cursor != 0 {
                    app.statusline_model.history_cursor -= 1;
                    let text = config
                        .commands
                        .history
                        .get(app.statusline_model.history_cursor.saturating_sub(1))
                        .unwrap()
                        .clone();

                    if text == app.statusline_model.cmd.text || text.is_empty() {
                        continue;
                    }

                    app.statusline_model.cmd.text = text;
                    app.statusline_model.cmd.cursor = app.statusline_model.cmd.text.len();
                    break;
                }

                if app.statusline_model.history_cursor == 0 {
                    app.statusline_model.cmd.text = String::new();
                    app.statusline_model.cmd.cursor = 0;
                }
            }
            refresh_suggestions(app);
        }
        CmdLineAction::Exit => {
            let pane = app.prev_focused_pane;
            focus_pane(app, pane);
            app.statusline_model = StatusLineModel::default();
        }
    }
}

fn update_cmd(app: &mut App, action: AppCmd) {
    match action {
        AppCmd::Count => update(app, Action::Db(DbAction::QueryCount(false))),
        AppCmd::TotalCount => update(app, Action::Db(DbAction::QueryCount(true))),
        AppCmd::Goto(sub_cmd) => match sub_cmd {
            GotoCmd::Page(page) => match app.table_model.table_name.clone() {
                Some(table) => {
                    let db_driver = app.db_driver.clone();
                    let tx = app.action_tx.clone();
                    let order_by = app.table_model.query_state.order_by_clause.clone();
                    let where_clause = app.table_model.query_state.where_clause.clone();
                    let sort = app.settings.default_sort.clone();
                    let mut offset = app.table_model.query_state.offset;
                    let limit = app.table_model.query_state.limit;

                    tokio::spawn(async move {
                        let res: Result<()> = async {
                            let mut driver = db_driver.lock().await;

                            let app_page = page.saturating_sub(1);
                            let new_offset = app_page * limit;
                            let old_offset = offset;
                            if new_offset != old_offset {
                                // Fetch with the new offset and commit the new offset and page if there are results.
                                offset = new_offset;

                                let results = driver
                                    .query(&table, order_by, where_clause, sort.as_str(), offset, limit)
                                    .await?;

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
                    report_message(
                        app,
                        "No table is currently selected",
                        MsgKind::Error,
                        MsgLifetime::Short,
                    );
                    focus_pane(app, Pane::Left);
                }
            },
            GotoCmd::Table(table_name) => {
                let matches: Vec<&ExplorerItem> = app
                    .explorer_model
                    .items
                    .iter()
                    .filter(|it| it.name == table_name)
                    .collect();

                if !matches.is_empty() {
                    let table = matches[0];
                    select_table(app, table.name.clone());
                } else {
                    report_message(
                        app,
                        &format!("Table \"{}\" is not found", table_name),
                        MsgKind::Error,
                        MsgLifetime::Short,
                    );
                    focus_pane(app, Pane::Left);
                }
            }
        },
        AppCmd::OrderBy(clause) => match app.table_model.table_name.clone() {
            Some(table) => {
                update(app, Action::App(AppAction::StartLoading));

                app.table_model.query_state.offset = 0;

                let driver = app.db_driver.clone();
                let tx = app.action_tx.clone();
                let limit = app.table_model.query_state.limit;
                let where_clause = app.table_model.query_state.where_clause.clone();
                let sort = app.settings.default_sort.clone();

                tokio::spawn(async move {
                    let res: Result<()> = async {
                        let mut driver = driver.lock().await;
                        let results = driver
                            .query(
                                &table,
                                clause.clone(),
                                where_clause.clone(),
                                sort.as_str(),
                                0,
                                limit,
                            )
                            .await?;
                        let _ = tx.send(Action::Db(DbAction::QueryTableComplete(results)));
                        let _ = tx.send(Action::App(AppAction::UpdateQueryState(
                            where_clause.clone(),
                            clause.clone(),
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
            None => {
                report_message(
                    app,
                    "No table is currently selected",
                    MsgKind::Error,
                    MsgLifetime::Short,
                );
                focus_pane(app, Pane::Left);
            }
        },
        AppCmd::Where(clause) => match app.table_model.table_name.clone() {
            Some(table) => {
                update(app, Action::App(AppAction::StartLoading));

                app.table_model.query_state.offset = 0;

                let driver = app.db_driver.clone();
                let tx = app.action_tx.clone();
                let limit = app.table_model.query_state.limit;
                let order_by = app.table_model.query_state.order_by_clause.clone();
                let sort = app.settings.default_sort.clone();

                tokio::spawn(async move {
                    let res: Result<()> = async {
                        let mut driver = driver.lock().await;
                        let results = driver
                            .query(
                                &table,
                                order_by.clone(),
                                clause.clone(),
                                sort.as_str(),
                                0,
                                limit,
                            )
                            .await?;
                        let _ = tx.send(Action::Db(DbAction::QueryTableComplete(results)));
                        let _ = tx.send(Action::App(AppAction::UpdateQueryState(
                            clause.clone(),
                            order_by.clone(),
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
            None => {
                report_message(
                    app,
                    "No table is currently selected",
                    MsgKind::Error,
                    MsgLifetime::Short,
                );
                focus_pane(app, Pane::Left);
            }
        },
        AppCmd::Limit(limit) => match app.table_model.table_name.clone() {
            Some(table) => {
                update(app, Action::App(AppAction::StartLoading));

                app.table_model.query_state.limit = limit;
                app.table_model.query_state.offset = 0;
                app.table_model.current_page = 0;

                let driver = app.db_driver.clone();
                let tx = app.action_tx.clone();
                let order_by = app.table_model.query_state.order_by_clause.clone();
                let where_clause = app.table_model.query_state.where_clause.clone();
                let sort = app.settings.default_sort.clone();

                tokio::spawn(async move {
                    let res: Result<()> = async {
                        let mut driver = driver.lock().await;
                        let results = driver
                            .query(&table, order_by, where_clause, sort.as_str(), 0, limit)
                            .await?;
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
                app.table_model.query_state.limit = limit;
                report_message(
                    app,
                    &format!("Limit set to {}", limit),
                    MsgKind::Neutral,
                    MsgLifetime::Short,
                );
            }
        },
        AppCmd::SettingChange(key, value) => {
            if let Err(err) = update_settings(app, key, value) {
                report_message(app, &err.to_string(), MsgKind::Error, MsgLifetime::Long);
            }
            let pane = app.prev_focused_pane;
            focus_pane(app, pane);
        }
        AppCmd::ChangeTheme(theme) => {
            if let Err(err) = update_settings(app, "theme".to_string(), Some(theme.clone())) {
                report_message(app, &err.to_string(), MsgKind::Error, MsgLifetime::Long);
            }
            app.theme = theme.as_str().parse().unwrap();
            let pane = app.prev_focused_pane;
            focus_pane(app, pane);
        }
        AppCmd::SavePreset(name) => {
            let query_state = app.table_model.query_state.clone();
            match save_preset(app, name, query_state) {
                Err(err) => {
                    report_message(app, &err.to_string(), MsgKind::Error, MsgLifetime::Long);
                }
                Ok(_) => {
                    report_message(app, "Preset saved", MsgKind::Success, MsgLifetime::Short);
                }
            }

            let pane = app.prev_focused_pane;
            focus_pane(app, pane);
        }
        AppCmd::LoadPreset(name) => {
            match load_preset(app, name.clone()) {
                Err(err) => report_message(app, &err.to_string(), MsgKind::Error, MsgLifetime::Long),
                Ok(query_state) => {
                    app.table_model.query_state = query_state;
                    app.table_model.current_page = 0;
                    update_db(app, DbAction::QueryTable);
                    report_message(
                        app,
                        &format!("Preset \"{}\" loaded", { name }),
                        MsgKind::Success,
                        MsgLifetime::Short,
                    );
                }
            };

            let pane = app.prev_focused_pane;
            focus_pane(app, pane);
        }
        AppCmd::RemovePreset(name) => {
            match remove_preset(app, name.clone()) {
                Err(err) => {
                    report_message(app, &err.to_string(), MsgKind::Error, MsgLifetime::Long);
                }
                Ok(_) => {
                    report_message(
                        app,
                        &format!("Preset \"{}\" removed", { name }),
                        MsgKind::Success,
                        MsgLifetime::Short,
                    );
                }
            }

            let pane = app.prev_focused_pane;
            focus_pane(app, pane);
        }
    };
}

fn update_json_view(app: &mut App, action: JsonViewAction) {
    match action {
        JsonViewAction::MoveUp => {
            app.json_view_model.scroll_y = app.json_view_model.scroll_y.saturating_sub(1);
        }
        JsonViewAction::MoveDown => {
            if app.json_view_model.data.is_some() {
                app.json_view_model.scroll_y = app.json_view_model.scroll_y + 1;
            }
        }
        JsonViewAction::GoToFirst => {
            app.json_view_model.scroll_y = 0;
        }
    }
}

fn update_help_view(app: &mut App, action: HelpViewAction) {
    let (_, selectable) = views::help_view::get_help_rows();
    let total = selectable.len();
    if total == 0 {
        return;
    }

    match action {
        HelpViewAction::MoveUp => {
            app.help_view_model.cursor = app.help_view_model.cursor.saturating_sub(1);
        }
        HelpViewAction::MoveDown => {
            if app.help_view_model.cursor + 1 < total {
                app.help_view_model.cursor += 1;
            }
        }
        HelpViewAction::GoToFirst => {
            app.help_view_model.cursor = 0;
        }
        HelpViewAction::GoToLast => {
            app.help_view_model.cursor = total - 1;
        }
        HelpViewAction::ActivateAction => {}
    }

    app.help_view_model.cursor = app.help_view_model.cursor.min(total - 1);
}
