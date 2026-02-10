use crate::drivers::ColumnMetadata;
use crate::drivers::QueryResult;
use crate::drivers::{self};
use crate::utils;
use async_trait::async_trait;
use color_eyre::Result;
use dashmap::DashMap;
use futures::TryStreamExt;
use sqlx::Row;
use sqlx::postgres::PgRow;

#[derive(Debug, Default, Clone)]
pub struct QueryState {
    /// For cache purposes.
    pub offset: usize,
    pub where_clause: String,
    pub order_by: String,
}

impl QueryState {
    pub fn new() -> Self {
        return Self {
            offset: 0,
            where_clause: String::new(),
            order_by: String::new(),
        };
    }
}

pub struct PostgresDriver {
    pool: sqlx::postgres::PgPool,
    pk_columns_cache: DashMap<String, Vec<String>>,
    table_columns_cache: DashMap<String, Vec<ColumnMetadata>>,
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
            pk_columns_cache: DashMap::new(),
            table_columns_cache: DashMap::new(),
            query_state: QueryState::new(),
        });
    }

    pub async fn ping(host: &str, port: u16, user: &str, password: &str, db_name: &str) -> Result<()> {
        let temp_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&format!(
                "postgres://{}:{}@{}:{}/{}",
                user, password, host, port, db_name
            ))
            .await?;

        sqlx::query("SELECT 1").fetch_one(&temp_pool).await?;

        return Ok(());
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
        let rows: Vec<PgRow> =
            sqlx::query("SELECT table_name FROM information_schema.views WHERE table_schema = 'public'")
                .fetch_all(&self.pool)
                .await?;

        let views = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("table_name").unwrap())
            .collect();

        return Ok(views);
    }

    async fn get_mateialized_views(&self) -> Result<Vec<String>> {
        let rows: Vec<PgRow> = sqlx::query(
            "SELECT matviewname AS table_name
            FROM pg_matviews
            WHERE schemaname = 'public'
            ORDER BY matviewname;",
        )
        .fetch_all(&self.pool)
        .await?;

        let views = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("table_name").unwrap())
            .collect();

        return Ok(views);
    }

    async fn query(&mut self, table_name: &str, limit: usize) -> Result<QueryResult> {
        self.query_state.order_by = self.get_order_by_clause(table_name).await?;

        let columns = self.get_columns(table_name).await?;
        let ident = utils::quote_ident(table_name);

        let sql = format!(
            r#"
            SELECT 
                to_jsonb(t) AS row
            FROM (
                SELECT *
                FROM {table}
                {order_sql}
                LIMIT $1
                OFFSET $2
            ) AS t;
        "#,
            table = ident,
            order_sql = self.query_state.order_by
        );

        let mut stream = sqlx::query_scalar::<_, serde_json::Value>(&sql)
            .bind(limit as i64)
            .bind(self.query_state.offset as i64)
            .fetch(&self.pool);

        let mut out_rows: Vec<serde_json::Value> = Vec::with_capacity(limit as usize);
        while let Some(j) = stream.try_next().await? {
            out_rows.push(j);
        }

        return Ok(QueryResult { columns: columns, rows: out_rows });
    }

    async fn query_count(&mut self, table_name: &str) -> Result<usize> {
        let ident = utils::quote_ident(table_name);

        let sql = format!(
            "SELECT COUNT(*) AS count FROM {} {}",
            ident, self.query_state.where_clause
        );

        let count: i64 = sqlx::query(&sql).fetch_one(&self.pool).await?.get("count");

        let count_usize = count as usize;

        return Ok(count_usize);
    }

    async fn get_order_by_clause(&mut self, table_name: &str) -> Result<String> {
        // Sort by the primary key(s) by default. If no order by was specified
        // by the user.
        if self.query_state.order_by.is_empty() {
            let pk_cols = self.get_pk_columns(&table_name).await?;
            if pk_cols.is_empty() {
                return Ok(String::new());
            } else {
                return Ok(format!(" ORDER BY {}", pk_cols.join(", ")));
            }
        }

        return Ok(self.query_state.order_by.clone());
    }

    async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>> {
        // Check cache
        if let Some(cols) = self.pk_columns_cache.get(table_name) {
            return Ok(cols.clone());
        }

        let sql = r#"
            SELECT att.attname AS column_name
            FROM pg_constraint con
            JOIN pg_class rel
                ON rel.oid = con.conrelid
            JOIN pg_attribute att
                ON att.attrelid = rel.oid
               AND att.attnum  = ANY(con.conkey)
            WHERE con.contype = 'p'
              AND rel.relname = $1
            ORDER BY array_position(con.conkey, att.attnum)
        "#;

        let mut pk_cols = sqlx::query(sql)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.get::<String, _>("column_name"))
            .collect::<Vec<String>>();

        //
        // Sort the primary columns so that integer/serial types are first for ordering.
        //

        let all_columns = self.get_columns(table_name).await?;
        pk_cols.sort_by_key(|col| {
            let t = all_columns
                .iter()
                .find(|c| c.name == *col)
                .map(|c| c.data_type.as_str())
                .unwrap_or("");

            // 0 for integer types, 1 for everything else
            match t {
                "integer" | "bigint" | "smallint" | "serial" | "bigserial" => 0,
                _ => 1,
            }
        });

        self.pk_columns_cache.insert(table_name.to_string(), pk_cols.clone());

        return Ok(pk_cols);
    }

    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>> {
        // Check cache
        if let Some(cols) = self.table_columns_cache.get(table_name) {
            return Ok(cols.clone());
        }

        let sql = r#"
            SELECT
                a.attname              AS column_name,
                pg_catalog.format_type(a.atttypid, a.atttypmod) AS data_type,
                NOT a.attnotnull       AS is_nullable
            FROM pg_catalog.pg_attribute a
            JOIN pg_catalog.pg_class c ON c.oid = a.attrelid
            JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
            WHERE
                n.nspname = 'public'
                AND c.relname = $1
                AND a.attnum > 0
                AND NOT a.attisdropped
            ORDER BY a.attnum;
        "#;

        let columns = sqlx::query(sql)
            .bind(table_name)
            .map(|row: PgRow| ColumnMetadata {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
            })
            .fetch_all(&self.pool)
            .await?;

        self.table_columns_cache.insert(table_name.to_string(), columns.clone());

        Ok(columns)
    }

    /// Attempts to fetch the next page. Only commits the new offset if data exists.
    /// Returns `Some(result)` if there's data, `None` if we're at the end.
    async fn next_page(&mut self, table: &str, limit: usize) -> Result<Option<QueryResult>> {
        self.query_state.offset += limit;

        return match self.query(table, limit).await? {
            result if result.rows.is_empty() => {
                self.query_state.offset -= limit;
                Ok(None)
            }
            result => Ok(Some(result)),
        };
    }

    async fn prev_page(&mut self, limit: usize) -> Result<()> {
        let old_offset = self.query_state.offset;
        let new_offset = self.query_state.offset.saturating_sub(limit);

        if old_offset != new_offset {
            self.query_state.offset = new_offset;
        }

        return Ok(());
    }

    async fn goto_page(&mut self, page: usize, table: &str, limit: usize) -> Result<Option<QueryResult>> {
        let app_page = page.saturating_sub(1);
        let new_offset = app_page.saturating_mul(limit);
        let old_offset = self.query_state.offset;
        if new_offset == old_offset {
            return Ok(None);
        }

        // Fetch with the new offset and commit the new offset and page if there are results.
        self.query_state.offset = new_offset;

        let result = self.query(table, limit).await?;

        return if result.rows.is_empty() {
            self.query_state.offset = old_offset;
            Ok(None)
        } else {
            Ok(Some(result))
        };
    }

    fn reset_query_state(&mut self) {
        self.query_state = QueryState::new();
    }
}
