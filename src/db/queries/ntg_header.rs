use crate::domain::error::AppError;
use crate::{db::DbPool, domain::ntg_column::NtgColumnType};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NtgHeader {
    pub id: i64,
    pub ntg_header: String,
    pub original_header: Option<String>,
    pub content_type: NtgColumnType,
    pub content_length: Option<i32>,
    pub required: bool,
    pub deprecated: bool,

    // Internal fields
    #[serde(skip_serializing)]
    pub column_index: Option<i32>,
    #[serde(skip_serializing)]
    pub deleted: bool,
    #[serde(skip_serializing)]
    pub created_at: OffsetDateTime,
    #[serde(skip_serializing)]
    pub updated_at: OffsetDateTime,
    #[serde(skip_serializing)]
    pub deleted_at: Option<OffsetDateTime>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewNtgHeader {
    pub ntg_header: String,
    pub original_header: Option<String>,
    pub content_type: NtgColumnType,
    pub content_length: Option<i32>,
    pub required: bool,
    pub deprecated: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNtgHeader {
    pub ntg_header: Option<String>,
    pub original_header: Option<String>,
    pub content_type: Option<NtgColumnType>,
    pub content_length: Option<i32>,
    pub required: Option<bool>,
    pub deprecated: Option<bool>,
}

pub async fn create(
    pool: &DbPool,
    new: NewNtgHeader,
    column_index: i32,
) -> sqlx::Result<NtgHeader, AppError> {
    let row = sqlx::query_as!(
        NtgHeader,
        r#"
    INSERT INTO ntg_headers(
        ntg_header,
        original_header,
        content_type,
        content_length,
        column_index,
        required,
        deprecated
    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING
        id,
        ntg_header,
        original_header,
        content_type AS "content_type!: NtgColumnType",
        content_length,
        column_index,
        required,
        deprecated,
        deleted,
        created_at,
        updated_at,
        deleted_at
    "#,
        new.ntg_header,
        new.original_header,
        new.content_type.as_str(),
        new.content_length,
        column_index,
        new.required,
        new.deprecated,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn upsert(
    pool: &DbPool,
    new: NewNtgHeader,
    column_index: i32,
) -> Result<NtgHeader, AppError> {
    let row = sqlx::query_as!(
        NtgHeader,
        r#"
         INSERT INTO ntg_headers (
             ntg_header, original_header, content_type,
             content_length, column_index, required
         )
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (ntg_header) DO UPDATE SET
             original_header = EXCLUDED.original_header,
             content_type    = EXCLUDED.content_type,
             content_length  = EXCLUDED.content_length,
             column_index    = EXCLUDED.column_index,
             required        = EXCLUDED.required,
             updated_at      = now()
         RETURNING
             id, ntg_header, original_header,
             content_type AS "content_type!: NtgColumnType",
             content_length, column_index, required, deprecated,
             deleted, created_at, updated_at, deleted_at
         "#,
        new.ntg_header,
        new.original_header,
        new.content_type.as_str(),
        new.content_length,
        column_index,
        new.required,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn delete(pool: &DbPool, id: i64) -> sqlx::Result<(), AppError> {
    sqlx::query!("DELETE FROM ntg_headers WHERE id = $1", id,)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn deprecate(pool: &DbPool, id: i64) -> sqlx::Result<(), AppError> {
    sqlx::query!("UPDATE ntg_headers SET deprecated = true WHERE id = $1", id,)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn undeprecate(pool: &DbPool, id: i64) -> sqlx::Result<(), AppError> {
    sqlx::query!(
        "UPDATE ntg_headers SET deprecated = false WHERE id = $1",
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_by_id(pool: &DbPool, id: i64) -> sqlx::Result<NtgHeader, AppError> {
    let row = sqlx::query_as!(
        NtgHeader,
        r#"SELECT
            id,
            ntg_header,
            original_header,
            content_type AS "content_type!: NtgColumnType",
            content_length,
            column_index,
            required,
            deprecated,
            deleted,
            created_at,
            updated_at,
            deleted_at
        FROM
            ntg_headers
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list(pool: &DbPool) -> sqlx::Result<Vec<NtgHeader>, AppError> {
    let rows = sqlx::query_as!(
        NtgHeader,
        r#"SELECT
            id,
            ntg_header,
            original_header,
            content_type AS "content_type!: NtgColumnType",
            content_length,
            column_index,
            required,
            deprecated,
            deleted,
            created_at,
            updated_at,
            deleted_at
        FROM
            ntg_headers
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_by_ntg_header(
    pool: &DbPool,
    ntg_header: &str,
) -> sqlx::Result<NtgHeader, AppError> {
    let row = sqlx::query_as!(
        NtgHeader,
        r#"SELECT
            id,
            ntg_header,
            original_header,
            content_type AS "content_type!: NtgColumnType",
            content_length,
            column_index,
            required,
            deprecated,
            deleted,
            created_at,
            updated_at,
            deleted_at
        FROM
            ntg_headers
        WHERE ntg_header = $1
        "#,
        ntg_header,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}
