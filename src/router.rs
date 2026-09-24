use axum::Router;

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::<AppState>::new().with_state(state)
}
