pub mod kinds;
pub mod mysql;
// pub mod sqlite;
pub mod postgres;

use crate::drivers::kinds::DbKind;
use crate::drivers::mysql::MySqlDriver;
use crate::drivers::postgres::PostgresDriver;
use color_eyre::Result;

pub enum DbDriver {
    Postgres(PostgresDriver),
    MySql(MySqlDriver),
}

impl DbDriver {
    pub async fn get_tables(&self) -> Result<Vec<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_tables().await,
            DbDriver::MySql(d) => d.get_tables().await,
        }
    }

    pub async fn get_views(&self) -> Result<Vec<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_views().await,
            DbDriver::MySql(d) => d.get_views().await,
        }
    }

    pub async fn get_mateialized_views(&self) -> Result<Vec<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_mateialized_views().await,
            DbDriver::MySql(d) => d.get_mateialized_views().await,
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
        }
    }

    pub async fn query_count(&mut self, table_name: &str, where_clause: Option<String>) -> Result<usize> {
        match self {
            DbDriver::Postgres(d) => d.query_count(table_name, where_clause).await,
            DbDriver::MySql(d) => d.query_count(table_name, where_clause).await,
        }
    }

    pub async fn get_default_order_by(&self, table_name: &str, sort: &str) -> Result<Option<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_default_order_by(table_name, sort).await,
            DbDriver::MySql(d) => d.get_default_order_by(table_name, sort).await,
        }
    }

    pub async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>> {
        match self {
            DbDriver::Postgres(d) => d.get_pk_columns(table_name).await,
            DbDriver::MySql(d) => d.get_pk_columns(table_name).await,
        }
    }

    pub async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>> {
        match self {
            DbDriver::Postgres(d) => d.get_columns(table_name).await,
            DbDriver::MySql(d) => d.get_columns(table_name).await,
        }
    }
}

pub async fn new_connection(kind: DbKind, url: &str) -> Result<DbDriver> {
    return match kind {
        DbKind::MySQL | DbKind::Mariadb => Ok(DbDriver::MySql(MySqlDriver::new_pool(url).await?)),
        DbKind::Postgres => Ok(DbDriver::Postgres(PostgresDriver::new_pool(url).await?)),
        DbKind::SQLite => todo!(),
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
    match kind {
        DbKind::MySQL | DbKind::Mariadb => MySqlDriver::ping(host, port, user, password, db_name).await,
        DbKind::Postgres => PostgresDriver::ping(host, port, user, password, db_name).await,
        DbKind::SQLite => todo!(),
    }
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
