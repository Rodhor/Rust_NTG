use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

pub async fn init_db_connection() -> Result<SqlitePool, String> {
    let base_dir = dirs::executable_dir().ok_or("Failed to determine executable directory")?;
    let db_path = base_dir.join("app.db");
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))
        .map_err(|e| format!("Failed to parse db file: {}", e))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .test_before_acquire(true)
        .connect_with(options)
        .await
        .map_err(|e| format!("Failed to create db pool: {}", e))?;

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Failed to run migrations: {}", e))?;

    Ok(pool)
}

pub async fn disconnect_db(pool: &SqlitePool) -> Result<(), String> {
    pool.close().await;

    if pool.is_closed() {
        Ok(())
    } else {
        Err("Failed to disconnect db".into())
    }
}
