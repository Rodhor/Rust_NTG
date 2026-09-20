use crate::response::ApiResponse;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Form, Router};
use ntg_core::models::sftp_configuration::SftpConfiguration;

pub fn sftp_routes() -> Router<AppState> {
    Router::new().nest(
        "/api/v1/sftp",
        Router::new()
            .route("/connect", post(connect_sftp))
            .route("/disconnect", get(disconnect_sftp))
            .route("/list", get(connect_sftp))
            .route("/upload", post(upload_file))
            .route("/download", post(download_file)),
    )
}
async fn connect_sftp(
    State(state): State<AppState>,
    Form(config): Form<SftpConfiguration>,
) -> ApiResponse<String> {
    match sftp::connection::connect(config).await {
        Ok(session) => {
            let mut guard = state.session.lock().await;
            *guard = Some(session);
            ApiResponse::ok("Connection successful".into())
        }
        Err(err) => ApiResponse::err(format!("Connection failed: {}", err)),
    }
}
async fn disconnect_sftp(State(state): State<AppState>) -> ApiResponse<String> {
    let session = state.session.lock().await.take();
    if let Some(session) = session {
        match sftp::connection::disconnect(session).await {
            Ok(_) => ApiResponse::ok("Disconnected successfully".into()),
            Err(err) => ApiResponse::err(format!("Disconnect failed: {}", err)),
        }
    } else {
        ApiResponse::err("No sftp connection to disconnect")
    }
}

async fn upload_file() -> ApiResponse<String> {
    ApiResponse::ok("upload endpoint ready".into())
}

async fn download_file() -> ApiResponse<String> {
    ApiResponse::ok("download endpoint ready".into())
}
