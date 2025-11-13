use std::collections::BTreeMap;

use color_eyre::Result;
use serde_json::Value;
use sqlx::{Row, mysql::MySqlRow};
use async_trait::async_trait;

use crate::{db, query_state::Query};


pub struct MySqlDriver {
    pool: sqlx::mysql::MySqlPool,
}

impl MySqlDriver {
    pub async fn new_pool(dsn: &str) -> Result<Self> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
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
impl drivers::DbConnection for MySqlDriver {
    async fn get_tables(&self) -> Result<Vec<String>> {
        let rows: Vec<MySqlRow> = sqlx::query("SHOW TABLES")
            .fetch_all(&self.pool)
            .await?;

        let tables = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>(0).unwrap())
            .collect();

        return Ok(tables);
    }

    async fn get_views(&self) -> Result<Vec<String>> {
        let rows: Vec<MySqlRow> = sqlx::query("SHOW FULL TABLES WHERE Table_type = 'VIEW'")
            .fetch_all(&self.pool)
            .await?;
        let views = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>(0).unwrap())
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
