use std::str::FromStr;

use sqlx::{
    SqlitePool, migrate,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use crate::domain::{config_model::DatabaseConfig, error::AppError};

pub async fn connect(config: &DatabaseConfig) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::from_str(&config.url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    if let Some(parent) = options
        .get_filename()
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections.filter(|n| *n > 0).unwrap_or(2))
        .connect_with(options)
        .await?;

    migrate!("src/db/migrations").run(&pool).await?;

    Ok(pool)
}
