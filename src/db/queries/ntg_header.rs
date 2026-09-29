use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use time::OffsetDateTime;
use crate::db::DbPool;
use crate::domain::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NtgColumnType {
    An,
    N,
}
impl NtgColumnType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::An => "an",
            Self::N => "n",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NtgHeader{
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

pub async fn create(
    pool: &DbPool,
    new : NewNtgHeader,
    column_index: i32
) -> sqlx::Result<NtgHeader, AppError> {
    let row = sqlx::query_as!(NtgHeader,
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