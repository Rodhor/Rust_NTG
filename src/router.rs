use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};

use crate::{
    api::{csv, operations, sftp},
    domain::{health::Health, response::ApiResponse},
    middleware::log_requests,
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::<AppState>::new()
        .nest("/api/v1", v1_router())
        .layer(middleware::from_fn(log_requests))
        .with_state(state)
}

pub fn v1_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/health", get(health))
        .nest("/sftp", sftp::router::router())
        .nest("/csv", csv::router::router())
        .nest("/operations", operations::router::router())
}

#[tracing::instrument(skip_all)]
async fn health(State(state): State<AppState>) -> Response {
    let (database, code) = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(_) => ("reachable".to_string(), StatusCode::OK),
        Err(e) => {
            tracing::warn!(error = %e, "database health probe failed");
            ("unreachable".to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        }
    };
    let body = ApiResponse {
        data: Health {
            status: if code == StatusCode::OK {
                "ok"
            } else {
                "degraded"
            },
            app: state.config.app.name.clone(),
            version: env!("CARGO_PKG_VERSION"),
            database: Some(database),
            url: state.config.app.url.clone(),
        },
    };
    (code, Json(body)).into_response()
}
