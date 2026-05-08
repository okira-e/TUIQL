pub mod kinds;
pub mod mysql;
pub mod postgres;
pub mod sqlite;
pub mod turso;

use crate::drivers::kinds::DbKind;
use crate::drivers::mysql::MySqlDriver;
use crate::drivers::postgres::PostgresDriver;
use crate::drivers::sqlite::SqliteDriver;
use crate::drivers::turso::TursoDriver;
use color_eyre::Result;
use color_eyre::eyre::eyre;

pub enum DbDriver {
    Postgres(PostgresDriver),
    MySql(MySqlDriver),
    SQLite(SqliteDriver),
    Turso(TursoDriver),
}

impl DbDriver {
    pub async fn get_tables(&self) -> Result<Vec<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_tables().await,
            DbDriver::MySql(d) => d.get_tables().await,
            DbDriver::SQLite(d) => d.get_tables().await,
            DbDriver::Turso(d) => d.get_tables().await,
        }
    }

    pub async fn get_views(&self) -> Result<Vec<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_views().await,
            DbDriver::MySql(d) => d.get_views().await,
            DbDriver::SQLite(d) => d.get_views().await,
            DbDriver::Turso(d) => d.get_views().await,
        }
    }

    pub async fn get_materialized_views(&self) -> Result<Vec<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_materialized_views().await,
            DbDriver::MySql(d) => d.get_materialized_views().await,
            DbDriver::SQLite(d) => d.get_materialized_views().await,
            DbDriver::Turso(d) => d.get_materialized_views().await,
        }
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
        match self {
            DbDriver::Postgres(d) => {
                d.query(
                    table_name,
                    order_by_clause,
                    where_clause,
                    sort,
                    offset,
                    limit,
                )
                .await
            }
            DbDriver::MySql(d) => {
                d.query(
                    table_name,
                    order_by_clause,
                    where_clause,
                    sort,
                    offset,
                    limit,
                )
                .await
            }
            DbDriver::SQLite(d) => {
                d.query(
                    table_name,
                    order_by_clause,
                    where_clause,
                    sort,
                    offset,
                    limit,
                )
                .await
            }
            DbDriver::Turso(d) => {
                d.query(
                    table_name,
                    order_by_clause,
                    where_clause,
                    sort,
                    offset,
                    limit,
                )
                .await
            }
        }
    }

    pub async fn query_count(&mut self, table_name: &str, where_clause: Option<String>) -> Result<usize> {
        match self {
            DbDriver::Postgres(d) => d.query_count(table_name, where_clause).await,
            DbDriver::MySql(d) => d.query_count(table_name, where_clause).await,
            DbDriver::SQLite(d) => d.query_count(table_name, where_clause).await,
            DbDriver::Turso(d) => d.query_count(table_name, where_clause).await,
        }
    }
}

pub async fn new_connection(kind: DbKind, url: &str, auth_token: Option<&str>) -> Result<DbDriver> {
    return match kind {
        DbKind::MySQL | DbKind::Mariadb => Ok(DbDriver::MySql(MySqlDriver::new_pool(url).await?)),
        DbKind::Postgres => Ok(DbDriver::Postgres(PostgresDriver::new_pool(url).await?)),
        DbKind::SQLite => Ok(DbDriver::SQLite(SqliteDriver::new_pool(url).await?)),
        DbKind::Turso => {
            let token = auth_token.ok_or_else(|| eyre!("Missing auth token for Turso connection"))?;

            Ok(DbDriver::Turso(TursoDriver::new_pool(url, token).await?))
        }
    };
}

pub async fn ping_connection(
    kind: DbKind,
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    db_name: &str,
) -> Result<()> {
    return match kind {
        DbKind::MySQL | DbKind::Mariadb => MySqlDriver::ping(host, port, user, password, db_name).await,
        DbKind::Postgres => PostgresDriver::ping(host, port, user, password, db_name).await,
        DbKind::SQLite => unreachable!("use ping_sqlite_connection for SQLite"),
        DbKind::Turso => unreachable!("use ping_turso_connection for Turso"),
    };
}

pub async fn ping_sqlite_connection(path: &str) -> Result<()> {
    return SqliteDriver::ping(path).await;
}

pub async fn ping_turso_connection(url: &str, auth_token: &str) -> Result<()> {
    return TursoDriver::ping(url, auth_token).await;
}

#[derive(Debug, Default, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
}
