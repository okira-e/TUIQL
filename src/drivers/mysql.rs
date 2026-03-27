use crate::drivers::ColumnMetadata;
use crate::drivers::QueryResult;
use color_eyre::Result;
use dashmap::DashMap;
use sqlx::Row;
use sqlx::mysql::MySqlRow;

pub struct MySqlDriver {
    pool: sqlx::mysql::MySqlPool,
    pk_columns_cache: DashMap<String, Vec<String>>,
    table_columns_cache: DashMap<String, Vec<ColumnMetadata>>,
}

impl MySqlDriver {
    pub async fn new_pool(dsn: &str) -> Result<Self> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
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
        let temp_pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&format!(
                "mysql://{}:{}@{}:{}/{}",
                user, password, host, port, db_name
            ))
            .await?;

        sqlx::query("SELECT 1").fetch_one(&temp_pool).await?;

        return Ok(());
    }

    pub async fn get_tables(&self) -> Result<Vec<String>> {
        let rows: Vec<MySqlRow> = sqlx::query(
            "SELECT CAST(TABLE_NAME AS CHAR) AS TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'BASE TABLE'",
        )
        .fetch_all(&self.pool)
        .await?;

        let tables = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("TABLE_NAME").unwrap())
            .collect();

        return Ok(tables);
    }

    pub async fn get_views(&self) -> Result<Vec<String>> {
        let rows: Vec<MySqlRow> =
            sqlx::query("SELECT CAST(TABLE_NAME AS CHAR) AS TABLE_NAME FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = DATABASE()")
                .fetch_all(&self.pool)
                .await?;

        let views = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("TABLE_NAME").unwrap())
            .collect();

        return Ok(views);
    }

    pub async fn get_mateialized_views(&self) -> Result<Vec<String>> {
        // MySQL does not support materialized views.
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
        let ident = quote_ident(table_name);

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
                format!(
                    "'{}', t.{}",
                    c.name.replace('\'', "''"),
                    quote_ident(&c.name)
                )
            })
            .collect();
        let json_object_expr = format!("JSON_OBJECT({})", json_args.join(", "));

        let sql = format!(
            r#"
            SELECT
                {json_object} AS `row`
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

        let rows: Vec<MySqlRow> = sqlx::query(&sql)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

        let out_rows: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| row.get::<serde_json::Value, _>("row"))
            .collect();

        return Ok(QueryResult { columns, rows: out_rows });
    }

    pub async fn query_count(&mut self, table_name: &str, where_clause: Option<String>) -> Result<usize> {
        let ident = quote_ident(table_name);

        let where_sql = match where_clause {
            None => String::new(),
            Some(clause) => format!(" WHERE {}", clause),
        };

        let sql = format!("SELECT COUNT(*) AS `count` FROM {}{}", ident, where_sql);

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
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(&format!(" {}, ", sort));
            return Ok(Some(format!("{} {}", clause, sort)));
        }
    }

    pub async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>> {
        if let Some(cols) = self.pk_columns_cache.get(table_name) {
            return Ok(cols.clone());
        }

        let sql = r#"
            SELECT CAST(COLUMN_NAME AS CHAR) AS COLUMN_NAME
            FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE TABLE_SCHEMA = DATABASE()
              AND TABLE_NAME = ?
              AND CONSTRAINT_NAME = 'PRIMARY'
            ORDER BY ORDINAL_POSITION
        "#;

        let mut pk_cols: Vec<String> = sqlx::query(sql)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.get::<String, _>("COLUMN_NAME"))
            .collect();

        // Sort integer-like PK columns first, same as postgres driver.
        let all_columns = self.get_columns(table_name).await?;
        pk_cols.sort_by_key(|col| {
            let t = all_columns
                .iter()
                .find(|c| c.name == *col)
                .map(|c| c.data_type.as_str())
                .unwrap_or("");

            match t {
                "int" | "bigint" | "smallint" | "mediumint" | "tinyint" => 0,
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

        let sql = r#"
            SELECT
                CAST(COLUMN_NAME AS CHAR) AS COLUMN_NAME,
                CAST(DATA_TYPE AS CHAR) AS DATA_TYPE
            FROM INFORMATION_SCHEMA.COLUMNS
            WHERE TABLE_SCHEMA = DATABASE()
              AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION
        "#;

        let columns: Vec<ColumnMetadata> = sqlx::query(sql)
            .bind(table_name)
            .map(|row: MySqlRow| ColumnMetadata {
                name: row.get("COLUMN_NAME"),
                data_type: row.get("DATA_TYPE"),
            })
            .fetch_all(&self.pool)
            .await?;

        self.table_columns_cache.insert(table_name.to_string(), columns.clone());

        return Ok(columns);
    }
}

fn quote_ident(name: &str) -> String {
    let mut s = String::with_capacity(name.len() + 2);
    s.push('`');
    for ch in name.chars() {
        if ch == '`' {
            s.push('`');
        }
        s.push(ch);
    }
    s.push('`');

    return s;
}
