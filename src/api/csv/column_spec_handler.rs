use axum::extract::State;
use serde::Deserialize;

use crate::{
    db::queries::ntg_header::{self, NtgHeader},
    domain::{error::AppError, response::ApiResponse},
    extract::ApiQuery,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct DeprecateQuery {
    pub id: i64,
}

#[tracing::instrument(skip_all, fields(id=?query))]
pub async fn deprecate_header(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<DeprecateQuery>,
) -> Result<(), AppError> {
    let id = query.id;

    ntg_header::deprecate(&state.db_pool, id).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct UndeprecateQuery {
    pub id: i64,
}
#[tracing::instrument(skip_all, fields(id=?query))]
pub async fn undeprecate_header(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<UndeprecateQuery>,
) -> Result<(), AppError> {
    let id = query.id;

    ntg_header::undeprecate(&state.db_pool, id).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub id: i64,
}
#[tracing::instrument(skip_all, fields(id=?query))]
pub async fn delete_header(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<DeleteQuery>,
) -> Result<(), AppError> {
    let id = query.id;

    ntg_header::delete(&state.db_pool, id).await?;

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
) -> Result<ApiResponse<NtgHeader>, AppError> {
    let id = query.id;

    let result = ntg_header::get_by_id(&state.db_pool, id).await?;

    Ok(ApiResponse { data: result })
}

#[derive(Debug, Deserialize)]
pub struct ByNtgHeaderQuery {
    pub ntg_header: String,
}

#[tracing::instrument(skip_all, fields(ntg_header=?query))]
pub async fn get_ntg_header(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<ByNtgHeaderQuery>,
) -> Result<ApiResponse<NtgHeader>, AppError> {
    let ntg_header = query.ntg_header.as_str();

    let result = ntg_header::get_by_ntg_header(&state.db_pool, ntg_header).await?;

    Ok(ApiResponse { data: result })
}

#[tracing::instrument(skip_all)]
pub async fn list(State(state): State<AppState>) -> Result<ApiResponse<Vec<NtgHeader>>, AppError> {
    let result = ntg_header::list(&state.db_pool).await?;

    Ok(ApiResponse { data: result })
}
