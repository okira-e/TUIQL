use color_eyre::eyre::bail;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum DbKind {
    #[serde(rename = "mysql")]
    MySQL,
    #[serde(rename = "mariadb")]
    Mariadb,
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "sqlite")]
    SQLite,
}
impl FromStr for DbKind {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mysql" => Ok(DbKind::MySQL),
            "mariadb" => Ok(DbKind::Mariadb),
            "postgres" => Ok(DbKind::Postgres),
            "sqlite" => Ok(DbKind::SQLite),
            _ => bail!("Unsupported database type"),
        }
    }
}

impl std::fmt::Display for DbKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbKind::MySQL => write!(f, "mysql"),
            DbKind::Mariadb => write!(f, "mariadb"),
            DbKind::Postgres => write!(f, "postgres"),
            DbKind::SQLite => write!(f, "sqlite"),
        }
    }
}
