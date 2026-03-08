use crate::drivers::ColumnMetadata;
use crate::drivers::OrderBy;
use crate::drivers::OrderByDirection;
use crate::drivers::QueryResult;
use crate::utils;
use color_eyre::Result;
use dashmap::DashMap;
use futures::TryStreamExt;
use sqlx::Row;
use sqlx::postgres::PgRow;

pub struct PostgresDriver {
    pool: sqlx::postgres::PgPool,
    pk_columns_cache: DashMap<String, Vec<String>>,
    table_columns_cache: DashMap<String, Vec<ColumnMetadata>>,
}

impl PostgresDriver {
    pub async fn new_pool(dsn: &str) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
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

    pub async fn get_tables(&self) -> Result<Vec<String>> {
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

    pub async fn get_views(&self) -> Result<Vec<String>> {
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

    pub async fn get_mateialized_views(&self) -> Result<Vec<String>> {
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

    pub async fn query(
        &mut self,
        table_name: &str,
        order_by: Option<OrderBy>,
        offset: usize,
        limit: usize,
    ) -> Result<QueryResult> {
        let order_by = order_by.or(self.get_default_order_by(table_name).await?);

        let columns = self.get_columns(table_name).await?;
        let ident = utils::quote_ident(table_name);

        let order_sql = match order_by {
            None => String::new(),
            Some(ob) => format!(
                " ORDER BY {} {}",
                ob.columns.iter().map(|c| utils::quote_ident(c)).collect::<Vec<_>>().join(", "),
                ob.order.to_string()
            ),
        };

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
            order_sql = order_sql,
        );

        let mut stream = sqlx::query_scalar::<_, serde_json::Value>(&sql)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch(&self.pool);

        let mut out_rows: Vec<serde_json::Value> = Vec::with_capacity(limit as usize);
        while let Some(j) = stream.try_next().await? {
            out_rows.push(j);
        }

        return Ok(QueryResult { columns: columns, rows: out_rows });
    }

    pub async fn query_count(&mut self, table_name: &str) -> Result<usize> {
        let ident = utils::quote_ident(table_name);

        let sql = format!(
            "SELECT COUNT(*) AS count FROM {} {}",
            ident, "" /* self.query_state.where_clause */
        );

        let count: i64 = sqlx::query(&sql).fetch_one(&self.pool).await?.get("count");

        let count_usize = count as usize;

        return Ok(count_usize);
    }

    pub async fn get_default_order_by(&self, table_name: &str) -> Result<Option<OrderBy>> {
        let pk_cols = self.get_pk_columns(&table_name).await?;
        if pk_cols.is_empty() {
            return Ok(None);
        } else {
            return Ok(Some(OrderBy {
                columns: pk_cols,
                order: OrderByDirection::Asc,
            }));
        }
    }

    pub async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>> {
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

    pub async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>> {
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
}
