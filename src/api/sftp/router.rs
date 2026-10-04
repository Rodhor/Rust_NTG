use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    api::sftp::{dir_handler, file_handler},
    state::AppState,
};
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .nest("/dir", dir())
        .nest("/file", file())
}

pub fn dir() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/list", get(dir_handler::list))
        .route("/create", post(dir_handler::create_dir))
        .route("/delete", delete(dir_handler::delete_dir))
        .route("/list_archive", get(dir_handler::list_archive))
}

pub fn file() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/upload", post(file_handler::upload))
        .route("/download", get(file_handler::download))
        .route("/delete", delete(file_handler::delete))
        .route("/move", post(file_handler::move_remote))
        .route("/archive", post(file_handler::archive))
}
