pub mod kinds;
// pub mod mysql;
pub mod postgres;
// pub mod sqlite;

use async_trait::async_trait;
use color_eyre::Result;

use crate::drivers::kinds::DbKind;
use crate::drivers::postgres::PostgresDriver;

#[async_trait]
pub trait DbDriver: Send + Sync {
    async fn get_tables(&self) -> Result<Vec<String>>;
    async fn get_views(&self) -> Result<Vec<String>>;
    async fn get_mateialized_views(&self) -> Result<Vec<String>>;
    async fn query(&mut self, table_name: &str, limit: usize) -> Result<QueryResult>;
    async fn query_count(&mut self, table_name: &str) -> Result<usize>;
    async fn get_order_by_clause(&mut self, table_name: &str) -> Result<String>;
    async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>>;
    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>>;
    async fn next_page(&mut self, table_name: &str, limit: usize) -> Result<Option<QueryResult>>;
    async fn prev_page(&mut self, limit: usize) -> Result<()>;
    async fn goto_page(&mut self, page: usize, table: &str, limit: usize) -> Result<Option<QueryResult>>;
    fn reset_query_state(&mut self);
}

pub async fn new_connection(kind: DbKind, url: &str) -> Result<Box<dyn DbDriver>> {
    return match kind {
        DbKind::MySQL | DbKind::Mariadb => todo!(),
        DbKind::Postgres => Ok(Box::new(PostgresDriver::new_pool(url).await?)),
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
        DbKind::MySQL | DbKind::Mariadb => todo!(),
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
