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
}

pub fn file() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/list", get(file_handler::list))
        .route("/create", post(file_handler::create_dir))
        .route("/delete", delete(file_handler::delete_dir))
}
