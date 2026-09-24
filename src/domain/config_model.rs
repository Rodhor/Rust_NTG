use std::path::PathBuf;

use serde::{Deserialize, de::DeserializeOwned};

// =========== TRAITS ===============
trait FromEnv: DeserializeOwned {
    fn prefix() -> &'static str;

    fn from_env() -> Result<Self, ConfigError> {
        envy::prefixed(Self::prefix())
            .from_env()
            .map_err(ConfigError::from)
    }
}

// =========== CONFIG ===============
impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            log: LogConfig::from_env()?,
            sftp: SftpConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub log: LogConfig,
    pub sftp: SftpConfig,
    pub database: DatabaseConfig,
}

// =========== LOG CONFIG ===============
impl FromEnv for LogConfig {
    fn prefix() -> &'static str {
        "LOG_"
    }
}

fn default_log_file_path() -> Option<String> {
    Some("logs/app.log".to_string())
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default)]
    pub target: LogTarget,
    #[serde(default)]
    pub format: LogFormat,
    #[serde(default = "default_log_file_path")]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    #[default]
    Pretty,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogTarget {
    Stdout,
    Stderr,
    #[default]
    File,
}
// =========== DATABASE CONFIG ===============
impl FromEnv for DatabaseConfig {
    fn prefix() -> &'static str {
        "DATABASE_"
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub pool_size: Option<usize>,
    #[serde(default)]
    pub max_connections: Option<usize>,
}

// =========== SFTP CONFIG ===============
impl FromEnv for SftpConfig {
    fn prefix() -> &'static str {
        "SFTP_"
    }
}

fn default_sftp_port() -> Option<u16> {
    Some(22)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SftpConfig {
    pub host: String,
    #[serde(default = "default_sftp_port")]
    pub port: Option<u16>,
    pub username: String,
    pub password: Option<String>,
    #[serde(default)]
    pub default_in_directory: Option<PathBuf>,
    #[serde(default)]
    pub default_out_directory: Option<PathBuf>,
    #[serde(default)]
    pub default_archive_directory: Option<PathBuf>,
}

// =========== CONFIG ERROR ===============
#[derive(Debug)]
pub enum ConfigError {
    Env(envy::Error),
}

impl From<envy::Error> for ConfigError {
    fn from(error: envy::Error) -> Self {
        ConfigError::Env(error)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Env(error) => write!(f, "config error: {}", error),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Env(error) => Some(error),
        }
    }
}
