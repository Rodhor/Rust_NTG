use crate::db::DbPool;
use crate::domain::config_model::Config;
use crate::sftp::pool::SessionPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub sftp: Arc<SessionPool>,
    pub db_pool: DbPool,
}
