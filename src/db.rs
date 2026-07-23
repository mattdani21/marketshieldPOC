use std::{path::Path, str::FromStr};

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::error::AppError;

pub async fn connect(database_url: &str) -> Result<SqlitePool, AppError> {
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|error| {
                AppError::Internal(format!("failed to create database directory: {error}"))
            })?;
        }
    }

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|error| AppError::Configuration(format!("invalid database URL: {error}")))?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await?;
    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}
