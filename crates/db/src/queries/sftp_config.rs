use crate::models::sftp_configuration::SftpConfigRow;
use ntg_core::models::sftp_configuration::SftpConfiguration;
use sqlx::SqlitePool;

pub async fn create_sftp_configuration(
    pool: &SqlitePool,
    config: &SftpConfiguration,
) -> Result<i64, String> {
    let result = sqlx::query!(
        "INSERT INTO sftp_config (host, port, username, password) VALUES (?, ?, ?, ?)",
        &config.host,
        config.port as i64,
        &config.username,
        &config.password,
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
        "SELECT config_id, host, port, username, password FROM sftp_config WHERE config_id = ?",
        config_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|err| format!("failed to fetch sftp_config: {}", err))
}
