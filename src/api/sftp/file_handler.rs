use std::path::PathBuf;

use axum::extract::State;
use serde::Deserialize;

use crate::{
    domain::{error::AppError, response::ApiResponse},
    extract::ApiQuery,
    sftp::file_operations::{delete_remote_file, download_file, move_remote_file, upload_file},
    sftp::helpers,
    state::AppState,
};

fn local_target(state: &AppState, requested: &str) -> Result<PathBuf, AppError> {
    if requested.trim().is_empty() {
        return Err(AppError::InvalidQuery {
            detail: "local_path must name a file, not a directory".to_string(),
        });
    }
    helpers::resolve_path(&state.config.sftp.local_root_dir, requested)
}

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    #[serde(default)]
    pub remote_path: String,
    #[serde(default)]
    pub local_path: String,
}

#[tracing::instrument(skip_all, fields(remote_path = %q.remote_path, local_path = %q.local_path))]
pub async fn upload(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<UploadQuery>,
) -> Result<ApiResponse<String>, AppError> {
    let root = state.config.sftp.remote_root_dir.clone();
    let local_path = local_target(&state, &q.local_path)?;

    let final_path = state
        .sftp
        .with_session(move |session| upload_file(session, &root, &q.remote_path, &local_path))
        .await?;

    Ok(ApiResponse::new(final_path))
}

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    #[serde(default)]
    pub remote_path: String,
    #[serde(default)]
    pub local_path: String,
}
#[tracing::instrument(skip_all, fields(remote_path = %q.remote_path, local_path = %q.local_path))]
pub async fn download(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<DownloadQuery>,
) -> Result<ApiResponse<String>, AppError> {
    let root = state.config.sftp.remote_root_dir.clone();
    let local_path = local_target(&state, &q.local_path)?;

    let final_path = state
        .sftp
        .with_session(move |session| download_file(session, &root, &q.remote_path, &local_path))
        .await?;

    Ok(ApiResponse::new(final_path))
}

#[derive(Debug, Deserialize)]
pub struct DeleteFileQuery {
    #[serde(default)]
    pub path: String,
}

#[tracing::instrument(skip_all, fields(remote_path = %q.path))]
pub async fn delete(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<DeleteFileQuery>,
) -> Result<ApiResponse<()>, AppError> {
    let root = state.config.sftp.remote_root_dir.clone();
    state
        .sftp
        .with_session(move |session| delete_remote_file(session, &root, &q.path))
        .await?;

    Ok(ApiResponse::new(()))
}

#[derive(Debug, Deserialize)]
pub struct MoveFileQuery {
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[tracing::instrument(skip_all, fields(from = %q.from, to = %q.to))]
pub async fn move_remote(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<MoveFileQuery>,
) -> Result<ApiResponse<String>, AppError> {
    let root = state.config.sftp.remote_root_dir.clone();
    let final_path = state
        .sftp
        .with_session(move |session| move_remote_file(session, &root, &q.from, &q.to, q.overwrite))
        .await?;

    Ok(ApiResponse::new(final_path))
}
