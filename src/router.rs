use axum::{Router, extract::State, middleware, routing::get};

use crate::{
    domain::{error::AppError, health::Health, response::ApiResponse},
    middleware::log_requests,
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::<AppState>::new()
        .route("/api/v1/health", get(health))
        .layer(middleware::from_fn(log_requests))
        .with_state(state)
}

#[tracing::instrument(skip_all)]
async fn health(State(state): State<AppState>) -> Result<ApiResponse<Health>, AppError> {
    Ok(ApiResponse {
        data: Health {
            status: "ok",
            app: state.config.app.name.clone(),
            version: env!("CARGO_PKG_VERSION"),
        },
    })
}
