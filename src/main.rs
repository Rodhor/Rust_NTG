use crate::domain::error::AppError;
use crate::sftp::pool::SessionPool;
use crate::state::AppState;
use std::sync::Arc;

mod api;
mod config;
mod csv;
mod domain;
mod extract;
mod logging;
mod middleware;
mod router;
mod sftp;
mod shutdown;
mod state;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv()?;
    let config = config::load_config()?;

    let _guard = logging::init(&config.log)?;

    let app = router::build_router(AppState {
        config: config.clone(),
        sftp: Arc::new(SessionPool::new(config.sftp.clone())),
    });
    let addr = &config.app.bind_addr;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(app = %config.app.name, addr = %addr, "server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await?;

    tracing::info!("server stopped");
    Ok(())
}
