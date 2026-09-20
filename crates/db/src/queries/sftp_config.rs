use crate::SqlitePool;
use crate::models::sftp_configuration;
use crate::models::sftp_configuration::SftpConfigRow;
use ntg_core::models::sftp_configuration::SftpConfiguration;

pub async fn create_sftp_configuration(
    pool: &SqlitePool,
    config: &SftpConfiguration,
) -> Result<i64, String> {
    let result =
        sqlx::query("INSERT INTO sftp_config (host, port, username, password) VALUES (?,?,?,?)")
            .bind(&config.host)
            .bind(config.port as i64)
            .bind(&config.username)
            .bind(&config.password)
            .execute(pool)
            .await
            .map_err(|err| format!("failed to insert sftp_config: {}", err))?;

    Ok(result.last_insert_rowid())
}

pub async fn fetch_sftp_configuration(
    pool: &SqlitePool,
    config_id: i64,
) -> Result<Option<sftp_configuration::SftpConfigRow>, String> {
    sqlx::query_as::<_, SftpConfigRow>("SELECT * FROM sftp_config WHERE config_id = ?")
        .bind(config_id)
        .fetch_optional(pool)
        .await
        .map_err(|err| format!("failed to fetch sftp_config with ID {}: {}", config_id, err))
}
