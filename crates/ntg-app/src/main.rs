use crate::state::AppState;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod api;
mod response;
mod state;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(api::router())
        .fallback_service(ServeDir::new("static"))
        .with_state(AppState::default());
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
