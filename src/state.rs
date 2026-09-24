use crate::domain::config_model::Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
}
