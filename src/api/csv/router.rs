use axum::{
    Router,
    routing::{get, post},
};

use crate::{api::csv::column_spec_handler, state::AppState};
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .nest("/column-specs", column_specs())
        .nest("/article", article())
}

pub fn column_specs() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/get_by_id", get(column_spec_handler::get))
        .route("/list", get(column_spec_handler::list))
        .route(
            "/get_by_ntg_header",
            get(column_spec_handler::get_by_ntg_header),
        )
        .route("/deprecate", post(column_spec_handler::deprecate_column))
        .route(
            "/undeprecate",
            post(column_spec_handler::undeprecate_column),
        )
        .route("/delete", post(column_spec_handler::delete_column))
}

pub fn article() -> Router<AppState> {
    Router::<AppState>::new()
}
