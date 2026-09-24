use crate::domain::config_model::{Config, ConfigError};

pub fn load_config() -> Result<Config, ConfigError> {
    Config::load()
}
