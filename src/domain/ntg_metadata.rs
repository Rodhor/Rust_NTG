use serde::{Deserialize, Serialize};

use crate::domain::ntg_column::NtgColumnType;

#[derive(Debug, Serialize, Deserialize)]
pub struct NtgMetadata {
    pub ntg_header: String,
    pub original_header: String,
    pub content_type: NtgColumnType,
    pub content_length: Option<i32>,
    pub required: bool,
    pub column_index: Option<i32>,
}
