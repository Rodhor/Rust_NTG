use std::path::Path;

use axum::extract::{Query, State};
use serde::Deserialize;

use crate::{
    domain::{error::AppError, response::ApiResponse},
    sftp::file_operations::{delete_remote_file, download_file, move_remote_file, upload_file},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    #[serde(default)]
    pub local_path: String,
    #[serde(default)]
    pub remote_path: String,
}

#[tracing::instrument(skip_all, fields(remote_path = %q.remote_path, local_path = %q.local_path))]
pub async fn upload(
    State(state): State<AppState>,
    Query(q): Query<UploadQuery>,
) -> Result<ApiResponse<String>, AppError> {
    let root = state.config.sftp.root_dir.clone();

    let final_path = state
        .sftp
        .with_session(move |session| {
            upload_file(session, &root, &q.local_path, Path::new(&q.remote_path))
        })
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
    Query(q): Query<DownloadQuery>,
) -> Result<ApiResponse<String>, AppError> {
    let root = state.config.sftp.root_dir.clone();
    let final_path = state
        .sftp
        .with_session(move |session| {
            download_file(session, &root, &q.remote_path, Path::new(&q.local_path))
        })
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
    Query(q): Query<DeleteFileQuery>,
) -> Result<ApiResponse<()>, AppError> {
    let root = state.config.sftp.root_dir.clone();
    state
        .sftp
        .with_session(move |session| delete_remote_file(session, &root, &q.path))
        .await?;

    Ok(ApiResponse::new(()))
}

#[derive(Debug, Deserialize)]
pub struct MoveFileQuery {
    #[serde(default)]
    pub from_root: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to_root: String,
    #[serde(default)]
    pub to: String,
}

#[tracing::instrument(skip_all, fields(from_root = %q.from_root, from = %q.from, to_root = %q.to_root, to = %q.to))]
pub async fn move_remote(
    State(state): State<AppState>,
    Query(q): Query<MoveFileQuery>,
) -> Result<ApiResponse<String>, AppError> {
    let final_path = state
        .sftp
        .with_session(move |session| {
            move_remote_file(
                session,
                Path::new(&q.from_root),
                &q.from,
                Path::new(&q.to_root),
                &q.to,
            )
        })
        .await?;

    Ok(ApiResponse::new(final_path))
}
