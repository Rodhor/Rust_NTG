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
        let config = Self {
            app: AppConfig::from_env()?,
            log: LogConfig::from_env()?,
            sftp: SftpConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        let s = &self.sftp;
        let bad = |m: &str| Err(ConfigError::Invalid(m.to_string()));

        if s.host.trim().is_empty() {
            return bad("SFTP_HOST must not be empty");
        }
        if s.username.trim().is_empty() {
            return bad("SFTP_USERNAME must not be empty");
        }
        if s.password.is_none() {
            return bad("SFTP_PASSWORD is required (only password auth is implemented)");
        }
        // 64 hex chars = 32 bytes = SHA256
        if s.host_key_sha256.len() != 64
            || !s.host_key_sha256.chars().all(|c| c.is_ascii_hexdigit())
        {
            return bad("SFTP_HOST_KEY_SHA256 must be 64 hex characters (sha256)");
        }
        if s.pool_size == 0 {
            return bad("SFTP_POOL_SIZE must be greater than 0");
        }
        if s.connect_timeout_secs == 0 {
            return bad("SFTP_CONNECT_TIMEOUT_SECS must be greater than 0");
        }
        if s.root_dir.to_string_lossy().is_empty() {
            return bad("SFTP_ROOT_DIR must not be empty");
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub log: LogConfig,
    pub sftp: SftpConfig,
    pub database: DatabaseConfig,
}

// =========== APP CONFIG ===============
impl FromEnv for AppConfig {
    fn prefix() -> &'static str {
        "APP_"
    }
}

fn default_bind_addr() -> String {
    "127.0.0.1:8080".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    pub url: Option<String>,
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

impl std::fmt::Debug for SftpConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SftpConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("host_key_sha256", &self.host_key_sha256)
            .field("connect_timeout_secs", &self.connect_timeout_secs)
            .field("pool_size", &self.pool_size)
            .field("root_dir", &self.root_dir)
            .field("in_dir", &self.in_dir)
            .field("out_dir", &self.out_dir)
            .field("archive_dir", &self.archive_dir)
            .finish()
    }
}

fn default_sftp_port() -> Option<u16> {
    Some(22)
}

fn default_connect_timeout_secs() -> u64 {
    10
}

fn default_pool_size() -> usize {
    2
}

#[derive(Clone, Deserialize, Default)]
pub struct SftpConfig {
    pub host: String,
    #[serde(default = "default_sftp_port")]
    pub port: Option<u16>,
    pub username: String,
    pub password: Option<String>,
    pub host_key_sha256: String,
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    pub root_dir: PathBuf,
    #[serde(default)]
    pub in_dir: Option<PathBuf>,
    #[serde(default)]
    pub out_dir: Option<PathBuf>,
    #[serde(default)]
    pub archive_dir: Option<PathBuf>,
}

// =========== CONFIG ERROR ===============
#[derive(Debug)]
pub enum ConfigError {
    Env(envy::Error),
    Invalid(String),
}

impl From<envy::Error> for ConfigError {
    fn from(error: envy::Error) -> Self {
        ConfigError::Env(error)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Env(error) => write!(f, "config error: {error}"),
            ConfigError::Invalid(msg) => write!(f, "invalid config: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Env(error) => Some(error),
            ConfigError::Invalid(_) => None,
        }
    }
}
