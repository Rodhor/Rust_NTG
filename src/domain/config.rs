// Overall Struct
pub struct Config {
    pub log: LogConfig,
    pub sftp: SftpConfig,
    pub database: DatabaseConfig,
}

// Config Structs
pub struct LogConfig {}
pub struct SftpConfig {}
pub struct DatabaseConfig {}

// Traits
trait FromEnv {}

// Enums
pub enum ConfigError {}
pub enum LogLevel {}
pub enum LogFormat {
    Json,
    Pretty,
}
pub enum LogTarget {
    Stdout,
    Stderr,
    File(String, LogFormat),
}
