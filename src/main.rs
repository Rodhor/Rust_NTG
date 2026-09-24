use std::error::Error;

use crate::state::AppState;

mod config;
mod csv;
mod domain;
mod extract;
mod middleware;
mod router;
mod sftp;
mod shutdown;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let config = config::load_config()?;
    let app = router::build_router(AppState {
        config: config.clone(),
    });
    let addr = &config.app.bind_addr;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
