use crate::models::sftp_configuration::SftpConfigRow;
use crate::queries::helpers::timestamp_now_naive;
use ntg_core::models::sftp_configuration::SftpConfiguration;
use sqlx::SqlitePool;

pub async fn create_sftp_configuration(
    pool: &SqlitePool,
    config: &SftpConfiguration,
) -> Result<i64, String> {
    let timestamp = timestamp_now_naive();
    let result = sqlx::query!(
        "INSERT INTO sftp_config (name, host, port, username, password_ciphertext, created_at, updated_at) VALUES (?, ?, ?, ?,?,?,?)",
        &config.name,
        &config.host,
        config.port as i64,
        &config.username,
        &config.password,
        &timestamp,
        &timestamp
    )
    .execute(pool)
    .await
    .map_err(|err| format!("failed to insert sftp_config: {}", err))?;

    Ok(result.last_insert_rowid())
}

pub async fn fetch_sftp_configuration(
    pool: &SqlitePool,
    config_id: i64,
) -> Result<Option<SftpConfigRow>, String> {
    sqlx::query_as!(
        SftpConfigRow,
        "SELECT config_id, name, host, port, username, password_ciphertext, created_at, updated_at, deleted, deleted_at FROM sftp_config WHERE config_id = ? AND deleted = FALSE",
        config_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|err| format!("failed to fetch sftp_config: {}", err))
}
