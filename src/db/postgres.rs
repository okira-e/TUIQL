use async_trait::async_trait;
use color_eyre::{Result, eyre::bail};
use futures::TryStreamExt;
use sqlx::{postgres::PgRow, Row};

use crate::{
    db::{self, shared, ColumnMetadata, QueryResult},
    query_state::Query,
};


pub struct PostgresConnection {
    pool: sqlx::postgres::PgPool,
}

impl PostgresConnection {
   pub async fn new_pool(dsn: &str) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(dsn)
        .await?;

        return Ok(Self {
            pool,
        });
    } 
}

#[async_trait]
impl db::DbConnection for PostgresConnection {
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
    
    async fn query_table(&self, name: &str, query: &Query) -> Result<QueryResult> {
        if name.trim().is_empty() {
            bail!("Table/view name cannot be empty");
        }
        let limit: usize = if query.limit > 0 { query.limit } else { 100 };

        let columns = self.get_columns(name).await?;

        let ident = shared::quote_ident(name);
        
        // Use cursor-based fetching
        let sql = format!(r#"
            SELECT to_jsonb(t) AS row
            FROM (
                SELECT * FROM {table} {order_sql} LIMIT $1
            ) AS t
        "#, table = ident, order_sql = query.order_by);

        let mut stream = sqlx::query_scalar::<_, serde_json::Value>(&sql)
            .bind(limit as i64)
            .fetch(&self.pool);

        let mut out_rows: Vec<serde_json::Value> = Vec::with_capacity(limit);
        while let Some(j) = stream.try_next().await? {
            out_rows.push(j);
        }
        
        return Ok(QueryResult {
          columns: columns,
          rows: out_rows
        });
    }

    async fn get_pk_columns(&self, name: &str) -> Result<Vec<String>> {
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
            .bind(name)
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
            .map(|row: sqlx::postgres::PgRow| ColumnMetadata {
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
}
