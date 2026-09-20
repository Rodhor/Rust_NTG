#[derive(sqlx::FromRow)]
pub struct SftpConfigRow {
    pub config_id: i64,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password_ciphertext: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted: bool,
    pub deleted_at: Option<String>,
}
