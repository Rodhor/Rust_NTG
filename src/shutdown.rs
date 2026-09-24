pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c signal");
    tracing::info!("shutdown signal received, draining connections");
}