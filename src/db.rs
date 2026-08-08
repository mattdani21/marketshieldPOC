use std::{path::Path, str::FromStr};

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::error::AppError;

pub async fn connect(database_url: &str) -> Result<SqlitePool, AppError> {
    if let Some(path) = database_url.strip_prefix("sqlite://")
        && let Some(parent) = Path::new(path).parent()
    {
        tokio::fs::create_dir_all(parent).await.map_err(|error| {
            AppError::Internal(format!("failed to create database directory: {error}"))
        })?;
    }

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|error| AppError::Configuration(format!("invalid database URL: {error}")))?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await?;
    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(&pool)
        .await?;
    sqlx::raw_sql(include_str!(
        "../migrations/0002_competitive_intelligence.sql"
    ))
    .execute(&pool)
    .await?;

    // SQLite has no ADD COLUMN IF NOT EXISTS, so columns added to tables that
    // shipped in 0001 are applied here against the live table definition.
    add_column_if_missing(&pool, "market_cases", "product_line_id", "TEXT").await?;
    add_column_if_missing(
        &pool,
        "market_cases",
        "annual_quoted_premium_m",
        "REAL NOT NULL DEFAULT 0",
    )
    .await?;
    add_column_if_missing(&pool, "signals", "product_line_id", "TEXT").await?;

    Ok(pool)
}

/// Identifiers are `&'static str` so that only compile-time literals from this
/// module can reach the interpolated DDL below.
async fn add_column_if_missing(
    pool: &SqlitePool,
    table: &'static str,
    column: &'static str,
    definition: &'static str,
) -> Result<(), AppError> {
    let existing: Vec<String> = sqlx::query_scalar("SELECT name FROM pragma_table_info(?)")
        .bind(table)
        .fetch_all(pool)
        .await?;

    if existing.iter().any(|name| name == column) {
        return Ok(());
    }

    sqlx::query(sqlx::AssertSqlSafe(format!(
        "ALTER TABLE {table} ADD COLUMN {column} {definition}"
    )))
    .execute(pool)
    .await?;

    Ok(())
}
