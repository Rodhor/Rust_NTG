use std::sync::Arc;

#[derive(Clone, Default)]
pub struct AppState {
    pub session: Arc<tokio::sync::Mutex<Option<sftp::Session>>>,
}
