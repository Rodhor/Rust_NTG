use sqlx::FromRow;

#[derive(FromRow)]
pub struct SftpConfigRow {
    pub config_id: i64,
    pub host: String,
    pub port: Option<i64>,
    pub username: String,
    pub password: Option<String>,
}
