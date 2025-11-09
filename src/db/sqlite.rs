use std::collections::BTreeMap;

use color_eyre::{Result};
use serde_json::Value;
use sqlx::{Row, sqlite::SqliteRow};
use async_trait::async_trait;

use crate::{db, query_state::Query};


pub struct SqliteConnection {
    pool: sqlx::sqlite::SqlitePool,
}

impl SqliteConnection {
    pub async fn new_pool(dsn: &str) -> Result<Self> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
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
impl db::DbConnection for SqliteConnection {
    async fn get_tables(&self) -> Result<Vec<String>> {
        let rows: Vec<SqliteRow> = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )
        .fetch_all(&self.pool)
        .await?;

        let tables = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("name").unwrap())
            .collect();

        return Ok(tables);
    }

    async fn get_views(&self) -> Result<Vec<String>> {
        let rows: Vec<SqliteRow> = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type='view' AND name NOT LIKE 'sqlite_%'",
        )
        .fetch_all(&self.pool)
        .await?;

        let views = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("name").unwrap())
            .collect();

        return Ok(views);
    }
    
    async fn query_table(&self, name: &str, query: &Query) -> Result<BTreeMap<String, Value>> {
        todo!()
    }
    
    async fn get_pk_columns(&self, name: &str) -> Result<Vec<String>> {
        todo!()
    }
}
