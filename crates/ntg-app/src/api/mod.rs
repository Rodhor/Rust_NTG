use crate::state::AppState;
use axum::Router;

mod database;
mod sftp;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(sftp::sftp_routes())
        .merge(database::db_routes())
}
