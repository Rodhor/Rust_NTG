use axum::{Router, extract::State, routing::get};

use crate::{
    domain::{error::AppError, health::Health, response::ApiResponse},
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::<AppState>::new()
        .route("/api/v1/health", get(health))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Result<ApiResponse<Health>, AppError> {
    Ok(ApiResponse {
        data: Health {
            status: "ok",
            app: state.config.app.name.clone(),
            version: env!("CARGO_PKG_VERSION"),
        },
    })
}
