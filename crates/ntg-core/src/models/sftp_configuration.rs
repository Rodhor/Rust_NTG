use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SftpConfiguration {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}
