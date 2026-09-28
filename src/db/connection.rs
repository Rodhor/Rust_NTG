use sqlx::{PgPool, migrate, postgres::PgPoolOptions};

use crate::domain::{config_model::DatabaseConfig, error::AppError};

pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections.filter(|n| *n > 0).unwrap_or(5))
        .connect(&config.url)
        .await?;

    migrate!("src/db/migrations").run(&pool).await?;

    Ok(pool)
}
