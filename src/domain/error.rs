use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(#[from] crate::domain::config_model::ConfigError),

    #[error("sftp: {0}")]
    Sftp(String),

    #[error("internal server error")]
    Internal,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to initialize logging: {0}")]
    Logging(#[from] tracing_subscriber::util::TryInitError),

    #[error("dotenv: {0}")]
    Dotenv(#[from] dotenvy::Error),

    #[error("ssh2: {0}")]
    Ssh2(#[from] ssh2::Error),

    #[error("failed to spawn blocking task: {0}")]
    SpawnBlocking(#[from] tokio::task::JoinError),

    #[error("host key mismatch: expected {expected}, got {actual}")]
    HostKeyMismatch { expected: String, actual: String },

    #[error("path {path:?} escapes root {root:?}")]
    PathEscapesRoot { path: String, root: String },

    #[error("invalid query: {detail:?}")]
    InvalidQuery { detail: String },

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),

    #[error("csv error: {detail:?}")]
    CsvMalformed { detail: String },
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Sftp(_) => StatusCode::BAD_GATEWAY,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Logging(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Dotenv(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Ssh2(_) => StatusCode::BAD_GATEWAY,
            AppError::SpawnBlocking(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HostKeyMismatch { .. } => StatusCode::BAD_GATEWAY,
            AppError::PathEscapesRoot { .. } => StatusCode::BAD_REQUEST,
            AppError::InvalidQuery { .. } => StatusCode::BAD_REQUEST,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Csv(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::CsvMalformed { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    fn code(&self) -> &str {
        match self {
            AppError::Config(_) => "CONFIG_ERROR",
            AppError::Sftp(_) => "SFTP_ERROR",
            AppError::Internal => "INTERNAL_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Logging(_) => "LOG_INIT_ERROR",
            AppError::Dotenv(_) => "DOTENV_ERROR",
            AppError::Ssh2(_) => "SSH2_ERROR",
            AppError::SpawnBlocking(_) => "SPAWN_BLOCKING_ERROR",
            AppError::HostKeyMismatch { .. } => "HOST_KEY_MISMATCH",
            AppError::PathEscapesRoot { .. } => "PATH_ESCAPES_ROOT",
            AppError::InvalidQuery { .. } => "INVALID_QUERY",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Migration(_) => "MIGRATION_ERROR",
            AppError::Csv(_) => "CSV_ERROR",
            AppError::CsvMalformed { .. } => "CSV_MALFORMED",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status();
        let code = self.code();
        let body = ErrorBody {
            error: ErrorDetail {
                code: code.to_string(),
                message: self.to_string(),
                details: None,
            },
        };
        (status, Json(body)).into_response()
    }
}
