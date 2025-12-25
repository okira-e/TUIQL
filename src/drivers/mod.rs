pub mod kinds;
// pub mod mysql;
pub mod postgres;
// pub mod sqlite;

use async_trait::async_trait;
use color_eyre::Result;

use crate::drivers::{kinds::DbKinds, postgres::PostgresDriver};

#[async_trait]
pub trait DbDriver: Send + Sync {
    async fn get_tables(&self) -> Result<Vec<String>>;
    async fn get_views(&self) -> Result<Vec<String>>;
    async fn query(&mut self, table_name: &str) -> Result<QueryResult>;
    async fn query_count(&mut self, table_name: &str) -> Result<usize>;
    async fn get_order_by_clause(&mut self, table_name: &str) -> Result<String>;
    async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>>;
    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>>;
    async fn get_pagination_strategy(&self, table_name: &str) -> Result<PaginationStrategy>;
    async fn next_page(&mut self, table_name: &str, total_row_count: usize) -> Result<()>;
    async fn prev_page(&mut self, table_name: &str) -> Result<()>;
    fn reset_query_state(&mut self);
    async fn get_current_page(&self, table_name: &str) -> Result<usize>;
}

pub async fn new_connection(kind: &DbKinds, url: &str) -> Result<Box<dyn DbDriver>> {
    return match kind {
        DbKinds::MySQL | DbKinds::Mariadb => {
            // Ok(Arc::new(MySqlDriver::new_pool(url).await?))
            todo!()
        }
        DbKinds::Postgres => Ok(Box::new(PostgresDriver::new_pool(url).await?)),
        DbKinds::SQLite => {
            // Ok(Arc::new(SqliteDriver::new_pool(url).await?))
            todo!()
        }
    };
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
    pub is_nullable: bool,
}

#[derive(Debug, Clone)]
pub enum PaginationStrategy {
    /// Holds the cursor based column
    Cursor(String),
    Offset,
}
