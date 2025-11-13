use color_eyre::eyre::bail;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub enum DbKinds {
    #[serde(rename = "mysql")]
    MySQL,
    #[serde(rename = "mariadb")]
    Mariadb,
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "sqlite")]
    SQLite,
}
impl FromStr for DbKinds {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mysql" => Ok(DbKinds::MySQL),
            "mariadb" => Ok(DbKinds::Mariadb),
            "postgres" => Ok(DbKinds::Postgres),
            "sqlite" => Ok(DbKinds::SQLite),
            _ => bail!("Unsupported database type"),
        }
    }
}

impl std::fmt::Display for DbKinds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbKinds::MySQL => write!(f, "mysql"),
            DbKinds::Mariadb => write!(f, "mariadb"),
            DbKinds::Postgres => write!(f, "postgres"),
            DbKinds::SQLite => write!(f, "sqlite"),
        }
    }
}
