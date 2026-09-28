use crate::domain::error::AppError;
use crate::sftp::pool::SessionPool;
use crate::state::AppState;
use std::sync::Arc;

mod api;
mod config;
mod csv;
mod db;
mod domain;
mod extract;
mod logging;
mod middleware;
mod path_helpers;
mod router;
mod sftp;
mod shutdown;
mod state;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv()?;
    let config = config::load_config()?;
    std::fs::create_dir_all(&config.sftp.local_root_dir)?;

    let _guard = logging::init(&config.log)?;

    let sftp = Arc::new(SessionPool::new(config.sftp.clone()));
    let db_pool = db::connection::connect(&config.database).await?;
    let app = router::build_router(AppState {
        config: config.clone(),
        sftp: sftp.clone(),
        db_pool: db_pool.clone(),
    });
    let addr = &config.app.bind_addr;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(app = %config.app.name, addr = %addr, "server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await?;

    // Disconnect all SFTP sessions
    sftp.disconnect_all().await;

    // Close the database connection
    db_pool.close().await;
    tracing::info!("server stopped");
    Ok(())
}
