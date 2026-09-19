use crate::response::ApiResponse;
use crate::state::AppState;
use axum::Router;
use axum::extract::State;
use axum::routing::post;
pub fn db_routes() -> Router<AppState> {
    Router::new().nest(
        "/api/v1/db",
        Router::new()
            .route("/connect", post(connect_db))
            .route("/disconnect", post(disconnect_db))
            .route("/query", post(run_query)),
    )
}
async fn connect_db(State(state): State<AppState>) -> ApiResponse<String> {
    match db::connection::init_db_connection().await {
        Ok(pool) => {
            let mut guard = state.pool.lock().await;
            *guard = Some(pool);
            ApiResponse::ok("Connection successful".into())
        }
        Err(err) => ApiResponse::err(format!("Connection failed: {}", err)),
    }
}

async fn disconnect_db(State(state): State<AppState>) -> ApiResponse<String> {
    let pool = state.pool.lock().await.take();
    if let Some(pool) = pool {
        match db::connection::disconnect_db(&pool).await {
            Ok(_) => ApiResponse::ok("Disconnected successfully".into()),
            Err(err) => ApiResponse::err(format!("Disconnect failed: {}", err)),
        }
    } else {
        ApiResponse::err("No db connection to disconnect")
    }
}
async fn run_query() -> ApiResponse<String> {
    ApiResponse::ok("query endpoint ready".into())
}
