use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod api;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(api::router())
        .fallback_service(ServeDir::new("static"));
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}