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
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Sftp(_) => StatusCode::BAD_GATEWAY,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &str {
        match self {
            AppError::Config(_) => "CONFIG_ERROR",
            AppError::Sftp(_) => "SFTP_ERROR",
            AppError::Internal => "INTERNAL_ERROR",
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
