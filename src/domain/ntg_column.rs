use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NtgColumnType {
    An,
    N,
}
impl NtgColumnType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::An => "an",
            Self::N => "n",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "an" => Some(Self::An),
            "n" => Some(Self::N),
            _ => None,
        }
    }
}
