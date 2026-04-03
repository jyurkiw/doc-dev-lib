use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::error::DbError;

pub struct Db {
    pub(crate) pool: SqlitePool,
}

impl Db {
    pub async fn open(path: &str) -> Result<Self, DbError> {
        let opts = SqliteConnectOptions::from_str(path)?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;

        Ok(Self { pool })
    }

    pub async fn open_in_memory() -> Result<Self, DbError> {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;

        Ok(Self { pool })
    }

    /// Applies the schema to the database. Safe to call on a fresh database.
    pub async fn initialize(&self) -> Result<(), DbError> {
        let schema = include_str!("../schema.sql");
        sqlx::raw_sql(schema).execute(&self.pool).await?;
        Ok(())
    }
}
