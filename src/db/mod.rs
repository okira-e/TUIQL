pub mod kinds;
// pub mod mysql;
pub mod postgres;
// pub mod sqlite;


use async_trait::async_trait;
use color_eyre::Result;
use std::sync::Arc;

use crate::{
    db::{
        kinds::DbKinds, postgres::PostgresConnection,
    },
    query_state::Query,
};


#[async_trait]
pub trait DbConnection: Send + Sync {
    async fn get_tables(&self) -> Result<Vec<String>>;
    async fn get_views(&self) -> Result<Vec<String>>;
    async fn query(&self, table_name: &str, query: &mut Query) -> Result<QueryResult>;
    async fn query_count(&self, table_name: &str, query: &Query) -> Result<usize>;
    async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>>;
    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>>;
    async fn decide_pagination_strategy(&self, table_name: &str) -> Result<PaginationStrategy>;
}

pub async fn new_connection(kind: &DbKinds, url: &str) -> Result<Arc<dyn DbConnection>> {
    return match kind {
        DbKinds::MySQL | DbKinds::Mariadb => {
            // Ok(Arc::new(MySqlConnection::new_pool(url).await?))
            todo!()
        }
        DbKinds::Postgres => {
            Ok(Arc::new(PostgresConnection::new_pool(url).await?))
        }
        DbKinds::SQLite => {
            // Ok(Arc::new(SqliteConnection::new_pool(url).await?))
            todo!()
        }
    };
}


pub type ColumnsT = Vec<ColumnMetadata>;
pub type RowsT = Vec<serde_json::Value>;

#[derive(Debug, Default, Clone)]
pub struct QueryResult {
    pub columns: ColumnsT,
    pub rows: RowsT,
}

#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
}

#[derive(Debug, Clone)]
pub enum PaginationStrategy {
    Cursor(String),
    Offset,
}