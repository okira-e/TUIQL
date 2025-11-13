use async_trait::async_trait;
use color_eyre::{Result, eyre::bail};
use futures::TryStreamExt;
use sqlx::{postgres::PgRow, Postgres, Row};

use crate::{
    drivers::{self, ColumnMetadata, PaginationStrategy, QueryResult},
    query_state::Query, utils,
};


pub struct PostgresDriver {
    pool: sqlx::postgres::PgPool,
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
    
    async fn query(&self, table_name: &str, query: &mut Query) -> Result<QueryResult> {
        if table_name.trim().is_empty() {
            bail!("Table/view name cannot be empty");
        }

        let limit = if query.limit > 0 { query.limit } else { 100 };

        let columns = self.get_columns(table_name).await?;
        let ident = utils::quote_ident(table_name);
        
        let pagination_strategy = self.decide_pagination_strategy(table_name).await?;
        
        return match pagination_strategy {
            PaginationStrategy::Cursor(column_name) => {
                let order_direction = if query.order_by.to_uppercase().contains("DESC") {
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
                    order_sql = query.order_by
                );

                let query_builder = sqlx::query_scalar::<Postgres, serde_json::Value>(&sql)
                    .bind(query.current_cursor_value as i64)
                    .bind(limit as i64);

                let mut stream = query_builder.fetch(&self.pool);

                let mut out_rows: Vec<serde_json::Value> = Vec::with_capacity(limit);
                while let Some(j) = stream.try_next().await? {
                    out_rows.push(j);
                }
                
                query.current_cursor_value = out_rows
                    .last()
                    .expect("no rows")
                    .get(column_name)
                    .expect("missing column")
                    .as_u64()
                    .expect("not a number") as usize;
                
                Ok(QueryResult {
                    columns: columns,
                    rows: out_rows,
                })
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
                "#, table = ident, order_sql = query.order_by);
                
                let mut stream = sqlx::query_scalar::<_, serde_json::Value>(&sql)
                    .bind(limit as i64)
                    .bind(query.offset as i64)
                    .fetch(&self.pool);
                
                let mut out_rows: Vec<serde_json::Value> = Vec::with_capacity(limit);
                while let Some(j) = stream.try_next().await? {
                    out_rows.push(j);
                }
                
                Ok(QueryResult {
                    columns: columns,
                    rows: out_rows
                })
            },
        };
    }
    
    async fn query_count(&self, table_name: &str, query: &Query) -> Result<usize> {
        let sql = format!("SELECT COUNT(*) AS count FROM {} {}", table_name, query.where_clause);

        let count = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count");

        Ok(count as usize)
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

    /// Tries to find a column that can be used for cursor based pagination.
    /// If none are found, it decides the fallback.
    async fn decide_pagination_strategy(&self, table_name: &str) -> Result<PaginationStrategy> {
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
            .map(|row: sqlx::postgres::PgRow| {
                row.get("column_name")
            })
            .fetch_all(&self.pool)
            .await?;
        
        return if !columns.is_empty() {
            Ok(PaginationStrategy::Cursor(columns[0].clone()))
        } else {
            Ok(PaginationStrategy::Offset)
        };
    }
}
