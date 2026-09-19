use crate::response::ApiResponse;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Form, Router};
use sftp::types::SftpConfiguration;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(sftp_routes())
        .merge(db_routes())
        .merge(io_routes())
}

fn sftp_routes() -> Router<AppState> {
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

fn db_routes() -> Router<AppState> {
    Router::new().nest(
        "/api/v1/db",
        Router::new()
            .route("/connection", post(connect_db))
            .route("/query", post(run_query)),
    )
}

fn io_routes() -> Router<AppState> {
    Router::new().nest(
        "/api/v1/io",
        Router::new()
            .route("/read", post(read_file))
            .route("/write", post(write_file)),
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


async fn connect_db(
    State(state): State<AppState>,
) -> ApiResponse<String> {
    match db::connection::init_db_connection().await {
        Ok(pool) => {
            let mut guard = state.pool.lock().await;
            *guard = Some(pool);
            ApiResponse::ok("Connection successful".into())
        }
        Err(err) => ApiResponse::err(format!("Connection failed: {}", err)),
    }
}

async fn disconnect_db(
    State(state): State<AppState>,
) -> ApiResponse<String> {
    let pool = state.pool.lock().await.take();
    if let Some(pool) = pool {
        match db::connection::disconnect_db(&pool).await {
           Ok(_) => ApiResponse::ok("Disconnected successfully".into()),
            Err(err) => ApiResponse::err(format!("Disconnect failed: {}", err)),
        }
    } else {
        ApiResponse::err("No db connection to disconnect")
    }
}
async fn run_query() -> ApiResponse<String> {
    ApiResponse::ok("query endpoint ready".into())
}

async fn read_file() -> ApiResponse<String> {
    ApiResponse::ok("read endpoint ready".into())
}

async fn write_file() -> ApiResponse<String> {
    ApiResponse::ok("write endpoint ready".into())
}
async fn download_file() -> ApiResponse<String> {
    ApiResponse::ok("download endpoint ready".into())
}
