use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

pub async fn log_requests(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;

    // Potentially use as_u16 to get the status code as an unsigned integer
    // current implementation gets the status code plus reason as a string
    // TODO: Decide after looking at the logs
    let status = response.status();
    let duration_ms = start.elapsed().as_millis();
    tracing::info!(%method, %path, %status, duration_ms, "request completed");

    response
}
