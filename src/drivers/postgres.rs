use std::collections::HashMap;

use async_trait::async_trait;
use color_eyre::{Result, eyre::bail};
use dashmap::DashMap;
use futures::TryStreamExt;
use sqlx::{postgres::PgRow, Postgres, Row};

use crate::{ drivers::{self, ColumnMetadata, PaginationStrategy, QueryResult}, utils};


#[derive(Debug, Default, Clone)]
pub struct QueryState {
    pub table_name: Option<String>, // cache only the current table.
    pub limit: usize,
    pub offset: usize,
    pub where_clause: String,
    pub group_by: String,
    pub order_by: String,
    pub cursor_history: HashMap<String, Vec<usize>>,
    pub row_count: Option<usize>,
}

impl QueryState {
    pub fn new() -> Self {
        return Self {
            table_name: None,
            limit: 200,
            offset: 0,
            where_clause: String::new(),
            group_by: String::new(),
            order_by: String::new(),
            cursor_history: HashMap::new(),
            row_count: None,
        };
    }
}

pub struct PostgresDriver {
    pool: sqlx::postgres::PgPool,
    pagination_strategy_cache: DashMap<String, PaginationStrategy>,
    query_state: QueryState,
}

impl PostgresDriver {
   pub async fn new_pool(dsn: &str) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(dsn)
        .await?;

        return Ok(Self {
            pool,
            pagination_strategy_cache: DashMap::new(),
            query_state: QueryState::new(),
        });
    } 
}

#[async_trait]
impl drivers::DbDriver for PostgresDriver {
    async fn get_tables(&self) -> Result<Vec<String>> {
        let rows: Vec<PgRow> = sqlx::query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE' AND table_name NOT IN (SELECT inhrelid::regclass::text FROM pg_inherits)",
        )
        .fetch_all(&self.pool)
        .await?;

        let tables = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("table_name").unwrap())
            .collect();

        return Ok(tables);
    }

    async fn get_views(&self) -> Result<Vec<String>> {
        let rows: Vec<PgRow> = sqlx::query(
            "SELECT table_name FROM information_schema.views WHERE table_schema = 'public'",
        )
        .fetch_all(&self.pool)
        .await?;

        let views = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("table_name").unwrap())
            .collect();

        return Ok(views);
    }
    
    async fn query(&mut self, table_name: &str) -> Result<QueryResult> {
        if table_name.trim().is_empty() {
            bail!("Table/view name cannot be empty");
        }

        // Sort by the primary key(s) by default. If no order by was specified
        // by the user.
        if self.query_state.order_by.is_empty() {
            let pk_cols = self.get_pk_columns(&table_name).await?;
            if pk_cols.is_empty() {
                self.query_state.order_by = String::new()
            } else {
                self.query_state.order_by = format!(" ORDER BY {}", pk_cols.join(", "));
            }
        }
        
        let limit = if self.query_state.limit > 0 { self.query_state.limit } else { 100 };

        let columns = self.get_columns(table_name).await?;
        let ident = utils::quote_ident(table_name);
        
        let pagination_strategy = self.get_pagination_strategy(table_name).await?;
        
        return match pagination_strategy {
            PaginationStrategy::Cursor(column_name) => {
                let order_direction = if self.query_state.order_by.to_uppercase().contains("DESC") {
                    "DESC"
                } else {
                    "ASC"
                };

                let comparison_op = if order_direction == "DESC" { "<" } else { ">" };

                let where_clause = format!("WHERE {} {} $1", column_name, comparison_op);

                let sql = format!(
                    r#"
                        SELECT 
                            to_jsonb(t) AS row
                        FROM (
                            SELECT *
                            FROM {table}
                            {where_clause}
                            {order_sql}
                            LIMIT $2
                        ) AS t;
                    "#,
                    table = ident,
                    where_clause = where_clause,
                    order_sql = self.query_state.order_by
                );

                let cursor_pos: usize = match self.query_state.cursor_history.get(table_name) {
                    None => 0,
                    Some(cursor_history) => {
                        match cursor_history.last() {
                            Some(val) => *val,
                            None => 0,
                        }
                    },
                };

                let query_builder = sqlx::query_scalar::<Postgres, serde_json::Value>(&sql)
                    .bind(cursor_pos as i64)
                    .bind(limit as i64);

                let mut stream = query_builder.fetch(&self.pool);

                let mut out_rows: Vec<serde_json::Value> = Vec::with_capacity(limit);
                while let Some(j) = stream.try_next().await? {
                    out_rows.push(j);
                }
                
                return if let Some(last) = out_rows.last() {
                    let cursor_position = last
                        .get(column_name)
                        .expect("missing column")
                        .as_u64()
                        .expect("not a number") as usize;
                    
                    self.query_state.cursor_history
                        .entry(table_name.to_string())
                        .or_insert_with(Vec::new)
                        .push(cursor_position);
                    
                    Ok(QueryResult {
                        columns: columns,
                        rows: out_rows,
                    })
                } else {
                    Ok(QueryResult {
                        columns: vec![],
                        rows: vec![],
                    })
                }
            },
            PaginationStrategy::Offset => {
                let sql = format!(r#"
                    SELECT 
                        to_jsonb(t) AS row
                    FROM (
                        SELECT *
                        FROM {table}
                        {order_sql}
                        LIMIT $1
                        OFFSET $2
                    ) AS t;
                "#, table = ident, order_sql = self.query_state.order_by);
                
                let mut stream = sqlx::query_scalar::<_, serde_json::Value>(&sql)
                    .bind(limit as i64)
                    .bind(self.query_state.offset as i64)
                    .fetch(&self.pool);
                
                let mut out_rows: Vec<serde_json::Value> = Vec::with_capacity(limit);
                while let Some(j) = stream.try_next().await? {
                    out_rows.push(j);
                }
                
                Ok(QueryResult {
                    columns: columns,
                    rows: out_rows,
                })
            },
        };
    }
    
    async fn query_count(&mut self, table_name: &str) -> Result<usize> {
        let cache_hit = match &self.query_state.table_name {
            Some(name) if name == table_name => self.query_state.row_count,
            _ => None,
        };
        
        if let Some(c) = cache_hit {
            return Ok(c);
        }
        
        let sql = format!(
            "SELECT COUNT(*) AS count FROM {} {}",
            table_name,
            self.query_state.where_clause
        );
        
        let count: i64 = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await?
            .get("count");
        
        let count_usize = count as usize;
        
        // update cache
        self.query_state.table_name = Some(table_name.to_string());
        self.query_state.row_count = Some(count_usize);
        
        Ok(count_usize)
    }

    async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>> {
        let sql = r#"
          SELECT a.attname AS column_name
          FROM pg_index i
                   JOIN pg_attribute a ON a.attrelid = i.indrelid
              AND a.attnum = ANY(i.indkey)
          WHERE i.indrelid = $1::regclass
            AND i.indisprimary
          ORDER BY array_position(i.indkey, a.attnum);
        "#;
        
        let columns = sqlx::query(sql)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.get::<String, _>("column_name"))
            .collect::<Vec<String>>();
        
        return Ok(columns);
    }

    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>> {
        let sql = r#"
            SELECT
                column_name,
                data_type,
                is_nullable
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = $1
            ORDER BY ordinal_position;
        "#;
    
        let columns = sqlx::query(sql)
            .bind(table_name)
            .map(|row: PgRow| ColumnMetadata {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: match row.get::<String, _>("is_nullable").as_str() {
                    "YES" => true,
                    _ => false,
                },
            })
            .fetch_all(&self.pool)
            .await?;
    
        Ok(columns)
    }

    /// Tries to find a column that can be used for cursor based pagination.
    /// If none are found, it decides the fallback.
    ///
    /// This function uses caches result.
    /// @Todo: Add a command to control this.
    async fn get_pagination_strategy(&self, table_name: &str) -> Result<PaginationStrategy> {
        // Check cache
        if let Some(strat) = self.pagination_strategy_cache.get(table_name) {
            return Ok(strat.clone());
        }
        
        let sql = r#"
            SELECT
                c.column_name AS column_name
            FROM information_schema.columns c
                LEFT JOIN pg_index ix
                    ON c.table_name::regclass = ix.indrelid
                LEFT JOIN pg_attribute a
                    ON a.attrelid = ix.indrelid
                        AND a.attnum = ANY(ix.indkey)
            WHERE c.table_name = $1
              AND c.column_default LIKE 'nextval(%' -- SERIAL type
              AND a.attname = c.column_name
        "#;

        let columns: Vec<String> = sqlx::query(sql)
            .bind(table_name)
            .map(|row: PgRow| {
                row.get("column_name")
            })
            .fetch_all(&self.pool)
            .await?;
        
        let strat = if !columns.is_empty() {
            PaginationStrategy::Cursor(columns[0].clone())
        } else {
            PaginationStrategy::Offset
        };
        
        self.pagination_strategy_cache.insert(table_name.to_string(), strat.clone());
        
        return Ok(strat);
    }

    /// Modifies the state so the next fetch returns the next page.
    /// Returns if the state is new and fetching will yield newer results.
    async fn next_page(&mut self, table: &str, total: usize) -> Result<()> {
        match self.get_pagination_strategy(table).await? {
            PaginationStrategy::Cursor(_) => {
                // nothing — cursor advanced when fetching
            }
            PaginationStrategy::Offset => {
                let new_offset = self.query_state.offset + self.query_state.limit;
                if new_offset < total {
                    self.query_state.offset = new_offset;
                }
            }
        }
        
        return Ok(());
    }

    async fn prev_page(&mut self, table_name: &str) -> Result<()> {
        match self.get_pagination_strategy(table_name).await? {
            PaginationStrategy::Cursor(_) => {
                if let Some(cursor_history) = self.query_state.cursor_history.get_mut(table_name) {
                    cursor_history.pop();
                    cursor_history.pop();
                }
            }
            PaginationStrategy::Offset => {
                let old_offset = self.query_state.offset;
                let new_offset = self.query_state.offset.saturating_sub(self.query_state.limit);
                
                if old_offset != new_offset {
                    self.query_state.offset = new_offset;
                }
            }
        };
        
        return Ok(());
    }
    
    fn reset_query_state(&mut self) {
        self.query_state = QueryState::new();
    }
    
    async fn get_current_page(&self, table_name: &str) -> Result<usize> {
        return match self.get_pagination_strategy(table_name).await? {
            PaginationStrategy::Cursor(_) => {
                let pos = self.query_state.cursor_history.get(table_name)
                    .and_then(|history| history.last().cloned())
                    .unwrap_or(0);
                Ok(pos)
            }
            PaginationStrategy::Offset => {
                Ok(self.query_state.offset)
            }
        };
    }
}
