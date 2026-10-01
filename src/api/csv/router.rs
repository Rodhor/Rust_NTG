use axum::{
    Router,
    routing::{get, post},
};

use crate::{api::csv::header_handler, state::AppState};
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .nest("/header", header())
        .nest("/article", article())
}

pub fn header() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/get_by_id", get(header_handler::get))
        .route("/list", get(header_handler::list))
        .route("/get_by_ntg", get(header_handler::get_ntg_header))
        .route("/deprecate", post(header_handler::deprecate_header))
        .route("/undeprecate", post(header_handler::undeprecate_header))
        .route("/delete", post(header_handler::delete_header))
}

pub fn article() -> Router<AppState> {
    Router::<AppState>::new()
}
