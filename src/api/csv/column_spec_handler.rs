use axum::extract::State;
use serde::Deserialize;

use crate::{
    db::queries::column_spec::{self, NtgColumnSpec},
    domain::{error::AppError, response::ApiResponse},
    extract::ApiQuery,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct DeprecateQuery {
    pub id: i64,
}

#[tracing::instrument(skip_all, fields(id=?query))]
pub async fn deprecate_column(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<DeprecateQuery>,
) -> Result<(), AppError> {
    let id = query.id;

    column_spec::deprecate(&state.db_pool, id).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct UndeprecateQuery {
    pub id: i64,
}
#[tracing::instrument(skip_all, fields(id=?query))]
pub async fn undeprecate_column(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<UndeprecateQuery>,
) -> Result<(), AppError> {
    let id = query.id;

    column_spec::undeprecate(&state.db_pool, id).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub id: i64,
}
#[tracing::instrument(skip_all, fields(id=?query))]
pub async fn delete_column(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<DeleteQuery>,
) -> Result<(), AppError> {
    let id = query.id;

    column_spec::delete(&state.db_pool, id).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ByIdQuery {
    pub id: i64,
}
#[tracing::instrument(skip_all, fields(id=?query))]
pub async fn get(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<ByIdQuery>,
) -> Result<ApiResponse<NtgColumnSpec>, AppError> {
    let id = query.id;

    let result = column_spec::get_by_id(&state.db_pool, id).await?;

    Ok(ApiResponse { data: result })
}

#[derive(Debug, Deserialize)]
pub struct ByNtgHeaderQuery {
    pub ntg_header: String,
}

#[tracing::instrument(skip_all, fields(ntg_header=?query.ntg_header))]
pub async fn get_by_ntg_header(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<ByNtgHeaderQuery>,
) -> Result<ApiResponse<NtgColumnSpec>, AppError> {
    let ntg_header = query.ntg_header.as_str();

    let result = column_spec::get_by_ntg_header(&state.db_pool, ntg_header).await?;

    Ok(ApiResponse { data: result })
}

#[tracing::instrument(skip_all)]
pub async fn list(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<NtgColumnSpec>>, AppError> {
    let result = column_spec::list(&state.db_pool).await?;

    Ok(ApiResponse { data: result })
}
