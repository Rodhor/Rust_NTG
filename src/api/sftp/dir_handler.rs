use axum::extract::State;
use serde::Deserialize;

use crate::{
    domain::{error::AppError, file_item::FileItem, response::ApiResponse},
    extract::ApiQuery,
    sftp::directory_operations::{create_remote_dir, delete_remote_dir, list_dir},
    state::AppState,
};
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub path: String,
}

#[tracing::instrument(skip_all, fields(remote_path = %q.path))]
pub async fn list(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<ListQuery>,
) -> Result<ApiResponse<Vec<FileItem>>, AppError> {
    let root = state.config.sftp.root_dir.clone();

    let items = state
        .sftp
        .with_session(move |session| list_dir(session, &root, &q.path, false))
        .await?;

    Ok(ApiResponse::new(items))
}

#[derive(Debug, Deserialize)]
pub struct CreateDirQuery {
    #[serde(default)]
    pub path: String,
}

#[tracing::instrument(skip_all, fields(remote_path = %q.path))]
pub async fn create_dir(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<CreateDirQuery>,
) -> Result<ApiResponse<()>, AppError> {
    let root = state.config.sftp.root_dir.clone();
    state
        .sftp
        .with_session(move |session| create_remote_dir(session, &root, &q.path))
        .await?;

    Ok(ApiResponse::new(()))
}

#[derive(Debug, Deserialize)]
pub struct DeleteDirQuery {
    #[serde(default)]
    pub path: String,
}

#[tracing::instrument(skip_all, fields(remote_path = %q.path))]
pub async fn delete_dir(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<DeleteDirQuery>,
) -> Result<ApiResponse<()>, AppError> {
    let root = state.config.sftp.root_dir.clone();
    state
        .sftp
        .with_session(move |session| delete_remote_dir(session, &root, &q.path))
        .await?;

    Ok(ApiResponse::new(()))
}
