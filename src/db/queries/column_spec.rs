use crate::db::DbConnection;
use crate::domain::error::AppError;
use crate::{db::DbPool, domain::ntg_column::NtgColumnType};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use sqlx::{FromRow, query};
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NtgColumnSpec {
    pub id: i64,
    pub ntg_header: String,
    pub original_header: Option<String>,
    pub content_type: NtgColumnType,
    pub content_length: Option<i32>,
    pub required: bool,
    pub deprecated: bool,

    // Internal fields
    #[serde(skip_serializing)]
    pub column_index: i32,
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
pub struct NewNtgColumnSpec {
    pub ntg_header: String,
    pub original_header: Option<String>,
    pub content_type: NtgColumnType,
    pub content_length: Option<i32>,
    pub required: bool,
    pub column_index: i32,
}

pub async fn upsert(
    conn: &mut DbConnection,
    new: NewNtgColumnSpec,
) -> Result<NtgColumnSpec, AppError> {
    let row = query_as!(
        NtgColumnSpec,
        r#"
         INSERT INTO ntg_column_specs (
             ntg_header, original_header, content_type,
             content_length, column_index, required
         )
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (ntg_header) DO UPDATE SET
             original_header  = EXCLUDED.original_header,
             content_type   = EXCLUDED.content_type,
             content_length = EXCLUDED.content_length,
             column_index   = EXCLUDED.column_index,
             required       = EXCLUDED.required,
             updated_at     = now()
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
        new.column_index,
        new.required,
    )
    .fetch_one(conn)
    .await?;

    Ok(row)
}

pub async fn upsert_bulk(
    pool: &DbPool,
    rows: Vec<NewNtgColumnSpec>,
) -> Result<Vec<NtgColumnSpec>, AppError> {
    let mut tx = pool.begin().await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(upsert(&mut *tx, row).await?);
    }

    tx.commit().await?;

    Ok(out)
}

pub async fn delete(pool: &DbPool, id: i64) -> Result<(), AppError> {
    query!("DELETE FROM ntg_column_specs WHERE id = $1", id,)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn deprecate(pool: &DbPool, id: i64) -> Result<(), AppError> {
    query!(
        "UPDATE ntg_column_specs SET deprecated = true WHERE id = $1",
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn undeprecate(pool: &DbPool, id: i64) -> Result<(), AppError> {
    query!(
        "UPDATE ntg_column_specs SET deprecated = false WHERE id = $1",
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<NtgColumnSpec, AppError> {
    let row = query_as!(
        NtgColumnSpec,
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
            ntg_column_specs
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list(pool: &DbPool) -> Result<Vec<NtgColumnSpec>, AppError> {
    let rows = query_as!(
        NtgColumnSpec,
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
            ntg_column_specs
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_by_ntg_header(pool: &DbPool, ntg_header: &str) -> Result<NtgColumnSpec, AppError> {
    let row = query_as!(
        NtgColumnSpec,
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
            ntg_column_specs
        WHERE ntg_header = $1
        "#,
        ntg_header,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}
