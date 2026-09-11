use axum::routing::{get, post};
use axum::{Json, Router};

pub fn router() -> Router {
    Router::new()
        .merge(sftp_routes())
        .merge(db_routes())
        .merge(io_routes())
}

fn sftp_routes() -> Router {
    Router::new().nest(
        "/api/v1/sftp",
        Router::new()
            .route("/list", get(list_sftps))
            .route("/upload", post(upload_file))
            .route("/download", post(download_file)),
    )
}

fn db_routes() -> Router {
    Router::new().nest(
        "/api/v1/db",
        Router::new()
            .route("/ping", get(ping_db))
            .route("/query", post(run_query)),
    )
}

fn io_routes() -> Router {
    Router::new().nest(
        "/api/v1/io",
        Router::new()
            .route("/read", post(read_file))
            .route("/write", post(write_file)),
    )
}

async fn list_sftps() -> Json<Vec<String>> {
    Json(sftp::some_function())
}

async fn upload_file() -> Json<String> {
    Json("upload endpoint ready".into())
}

async fn ping_db() -> Json<String> {
    Json(db::some_function())
}

async fn run_query() -> Json<String> {
    Json("query endpoint ready".into())
}

async fn read_file() -> Json<String> {
    Json("read endpoint ready".into())
}

async fn write_file() -> Json<String> {
    Json("write endpoint ready".into())
}
async fn download_file() -> Json<String> {
    Json("download endpoint ready".into())
}
