pub mod kinds;
// pub mod mysql;
// pub mod sqlite;
pub mod postgres;

use crate::commander::SortCmdDirection;
use crate::drivers::kinds::DbKind;
use crate::drivers::postgres::PostgresDriver;
use async_trait::async_trait;
use color_eyre::Result;
use std::fmt;

#[async_trait]
pub trait DbDriver: Send + Sync {
    async fn get_tables(&self) -> Result<Vec<String>>;
    async fn get_views(&self) -> Result<Vec<String>>;
    async fn get_mateialized_views(&self) -> Result<Vec<String>>;
    async fn query(
        &mut self,
        table_name: &str,
        order_by: Option<OrderBy>,
        offset: usize,
        limit: usize,
    ) -> Result<QueryResult>;
    async fn query_count(&mut self, table_name: &str) -> Result<usize>;
    // async fn get_default_order_by(&mut self, table_name: &str) -> Result<String>;
    async fn get_default_order_by(&self, table_name: &str) -> Result<Option<OrderBy>>;
    async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>>;
    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnMetadata>>;
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OrderBy {
    columns: Vec<String>,
    order: OrderByDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderByDirection {
    Asc,
    Desc,
}

impl From<SortCmdDirection> for OrderByDirection {
    fn from(sort_cmd_direction: SortCmdDirection) -> Self {
        match sort_cmd_direction {
            SortCmdDirection::Asc => Self::Asc,
            SortCmdDirection::Desc => Self::Desc,
        }
    }
}

impl fmt::Display for OrderByDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderByDirection::Asc => write!(f, "ASC"),
            OrderByDirection::Desc => write!(f, "DESC"),
        }
    }
}

impl Default for OrderByDirection {
    fn default() -> Self {
        Self::Asc
    }
}
