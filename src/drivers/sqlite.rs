use crate::drivers::ColumnMetadata;
use crate::drivers::QueryResult;
use crate::utils;
use color_eyre::Result;
use dashmap::DashMap;
use libsql::Builder;
use libsql::Connection;
use libsql::params;

pub struct SqliteDriver {
    conn: Connection,
    pk_columns_cache: DashMap<String, Vec<String>>,
    table_columns_cache: DashMap<String, Vec<ColumnMetadata>>,
}

impl SqliteDriver {
    pub async fn new_pool(dsn: &str) -> Result<Self> {
        let path = dsn.strip_prefix("sqlite:").unwrap_or(dsn);
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;

        return Ok(Self {
            conn,
            pk_columns_cache: DashMap::new(),
            table_columns_cache: DashMap::new(),
        });
    }

    pub async fn ping(path: &str) -> Result<()> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        let mut rows = conn.query("SELECT 1", ()).await?;
        let _ = rows.next().await?;

        return Ok(());
    }

    pub async fn get_tables(&self) -> Result<Vec<String>> {
        let mut rows = self
            .conn
            .query(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                (),
            )
            .await?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            out.push(row.get::<String>(0)?);
        }

        return Ok(out);
    }

    pub async fn get_views(&self) -> Result<Vec<String>> {
        let mut rows = self
            .conn
            .query(
                "SELECT name FROM sqlite_master WHERE type = 'view' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                (),
            )
            .await?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            out.push(row.get::<String>(0)?);
        }

        return Ok(out);
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

        let mut rows = self
            .conn
            .query(&sql, params![limit as i64, offset as i64])
            .await?;

        let mut out_rows: Vec<serde_json::Value> = Vec::new();
        while let Some(row) = rows.next().await? {
            let raw: String = row.get(0)?;
            out_rows.push(serde_json::from_str(&raw)?);
        }

        return Ok(QueryResult { columns, rows: out_rows });
    }

    pub async fn query_count(&mut self, table_name: &str, where_clause: Option<String>) -> Result<usize> {
        let ident = utils::quote_ident(table_name);

        let where_sql = match where_clause {
            None => String::new(),
            Some(clause) => format!(" WHERE {}", clause),
        };

        let sql = format!("SELECT COUNT(*) AS \"count\" FROM {}{}", ident, where_sql);

        let mut rows = self.conn.query(&sql, ()).await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| color_eyre::eyre::eyre!("COUNT(*) returned no rows"))?;
        let count: i64 = row.get(0)?;

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

        let mut rows = self.conn.query(&sql, ()).await?;
        let mut pk_cols: Vec<String> = Vec::new();
        while let Some(row) = rows.next().await? {
            // table_info columns: cid, name, type, notnull, dflt_value, pk
            let pk: i64 = row.get(5)?;
            if pk > 0 {
                pk_cols.push(row.get::<String>(1)?);
            }
        }

        // Sort integer-like PK columns first
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

        let mut rows = self.conn.query(&sql, ()).await?;
        let mut columns: Vec<ColumnMetadata> = Vec::new();
        while let Some(row) = rows.next().await? {
            // table_info columns: cid, name, type, notnull, dflt_value, pk
            let name: String = row.get(1)?;
            let data_type: String = row.get(2)?;
            columns.push(ColumnMetadata { name, data_type });
        }

        self.table_columns_cache.insert(table_name.to_string(), columns.clone());

        return Ok(columns);
    }
}
