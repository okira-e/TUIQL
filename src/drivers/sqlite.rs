use crate::drivers::ColumnMetadata;
use crate::drivers::QueryResult;
use crate::utils;
use color_eyre::Result;
use dashmap::DashMap;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

pub struct SqliteDriver {
    pool: sqlx::sqlite::SqlitePool,
    pk_columns_cache: DashMap<String, Vec<String>>,
    table_columns_cache: DashMap<String, Vec<ColumnMetadata>>,
}

impl SqliteDriver {
    pub async fn new_pool(dsn: &str) -> Result<Self> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .idle_timeout(std::time::Duration::from_secs(600))
            .connect(dsn)
            .await?;

        return Ok(Self {
            pool,
            pk_columns_cache: DashMap::new(),
            table_columns_cache: DashMap::new(),
        });
    }

    pub async fn ping(path: &str) -> Result<()> {
        let dsn = format!("sqlite:{}", path);
        let temp_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&dsn)
            .await?;

        sqlx::query("SELECT 1").fetch_one(&temp_pool).await?;

        return Ok(());
    }

    pub async fn get_tables(&self) -> Result<Vec<String>> {
        let rows: Vec<SqliteRow> = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        let tables = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("name").unwrap())
            .collect();

        return Ok(tables);
    }

    pub async fn get_views(&self) -> Result<Vec<String>> {
        let rows: Vec<SqliteRow> = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type = 'view' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        let views = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("name").unwrap())
            .collect();

        return Ok(views);
    }

    pub async fn get_materialized_views(&self) -> Result<Vec<String>> {
        // SQLite does not support materialized views.
        return Ok(Vec::new());
    }

    pub async fn query(
        &mut self,
        table_name: &str,
        order_by_clause: Option<String>,
        where_clause: Option<String>,
        sort: &str,
        offset: usize,
        limit: usize,
    ) -> Result<QueryResult> {
        let order_by = order_by_clause.or(self.get_default_order_by(table_name, sort).await?);

        let columns = self.get_columns(table_name).await?;
        let ident = utils::quote_ident(table_name);

        let where_sql = match where_clause {
            None => String::new(),
            Some(clause) => format!(" WHERE {}", clause),
        };

        let order_sql = match order_by {
            None => String::new(),
            Some(clause) => format!(" ORDER BY {}", clause),
        };

        let json_args: Vec<String> = columns
            .iter()
            .map(|c| {
                let col = utils::quote_ident(&c.name);
                let value_expr = match c.data_type.to_lowercase().as_str() {
                    "blob" => format!("CASE WHEN t.{} IS NULL THEN NULL ELSE '[BINARY]' END", col),
                    _ => format!("t.{}", col),
                };
                format!("'{}', {}", c.name.replace('\'', "''"), value_expr)
            })
            .collect();
        let json_object_expr = format!("json_object({})", json_args.join(", "));

        let sql = format!(
            r#"
            SELECT
                {json_object} AS "row"
            FROM (
                SELECT *
                FROM {table}
                {where_sql}
                {order_sql}
                LIMIT ?
                OFFSET ?
            ) AS t
        "#,
            json_object = json_object_expr,
            table = ident,
            where_sql = where_sql,
            order_sql = order_sql,
        );

        let rows: Vec<SqliteRow> = sqlx::query(&sql)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

        let out_rows: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| {
                let raw: String = row.get("row");
                serde_json::from_str(&raw).unwrap()
            })
            .collect();

        return Ok(QueryResult { columns, rows: out_rows });
    }

    pub async fn query_count(&mut self, table_name: &str, where_clause: Option<String>) -> Result<usize> {
        let ident = utils::quote_ident(table_name);

        let where_sql = match where_clause {
            None => String::new(),
            Some(clause) => format!(" WHERE {}", clause),
        };

        let sql = format!("SELECT COUNT(*) AS \"count\" FROM {}{}", ident, where_sql);

        let count: i64 = sqlx::query(&sql).fetch_one(&self.pool).await?.get("count");

        return Ok(count as usize);
    }

    pub async fn get_default_order_by(&self, table_name: &str, sort: &str) -> Result<Option<String>> {
        let pk_cols = self.get_pk_columns(table_name).await?;
        if pk_cols.is_empty() {
            return Ok(None);
        } else {
            let clause = pk_cols
                .iter()
                .map(|c| utils::quote_ident(c))
                .collect::<Vec<_>>()
                .join(&format!(" {}, ", sort));
            return Ok(Some(format!("{} {}", clause, sort)));
        }
    }

    pub async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>> {
        if let Some(cols) = self.pk_columns_cache.get(table_name) {
            return Ok(cols.clone());
        }

        let sql = format!("PRAGMA table_info({})", utils::quote_ident(table_name));

        let mut pk_cols: Vec<String> = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .filter(|row| row.get::<i32, _>("pk") > 0)
            .map(|row| row.get::<String, _>("name"))
            .collect();

        // Sort integer-like PK columns first, same as other drivers.
        let all_columns = self.get_columns(table_name).await?;
        pk_cols.sort_by_key(|col| {
            let t = all_columns
                .iter()
                .find(|c| c.name == *col)
                .map(|c| c.data_type.to_lowercase())
                .unwrap_or_default();

            match t.as_str() {
                "integer" | "int" | "bigint" | "smallint" | "tinyint" | "mediumint" => 0,
                _ => 1,
            }
        });

        self.pk_columns_cache.insert(table_name.to_string(), pk_cols.clone());

        return Ok(pk_cols);
    }

    pub async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>> {
        if let Some(cols) = self.table_columns_cache.get(table_name) {
            return Ok(cols.clone());
        }

        let sql = format!("PRAGMA table_info({})", utils::quote_ident(table_name));

        let columns: Vec<ColumnMetadata> = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row: SqliteRow| ColumnMetadata { name: row.get("name"), data_type: row.get("type") })
            .collect();

        self.table_columns_cache.insert(table_name.to_string(), columns.clone());

        return Ok(columns);
    }
}
