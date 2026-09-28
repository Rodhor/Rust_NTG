use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
    pub app: String,
    pub version: &'static str,
    pub database: Option<String>,
    pub url: Option<String>,
}
