use crate::{
    domain::{error::AppError, file_item::FileItem, response::ApiResponse},
    sftp::directory_operations::list_dir,
    state::AppState,
};
use axum::extract::{Query, State};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub path: String,
}

#[tracing::instrument(skip_all, fields(remote_path = %q.path))]
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<ApiResponse<Vec<FileItem>>, AppError> {
    let root = state.config.sftp.root_dir.clone();

    let items = state
        .sftp
        .with_session(move |session| list_dir(session, &root, &q.path, false))
        .await?;

    Ok(ApiResponse::new(items))
}
