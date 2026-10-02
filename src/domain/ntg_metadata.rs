use serde::{Deserialize, Serialize};

use crate::{db::queries::ntg_header::NewNtgHeader, domain::ntg_column::NtgColumnType};

#[derive(Serialize, Deserialize)]
pub struct NtgMetadata {
    pub ntg_header: String,
    pub original_header: String,
    pub content_type: NtgColumnType,
    pub content_length: Option<i32>,
    pub required: bool,
    pub column_index: i32,
}

impl From<NtgMetadata> for NewNtgHeader {
    fn from(m: NtgMetadata) -> Self {
        NewNtgHeader {
            ntg_header: m.ntg_header,
            original_header: Some(m.original_header),
            content_type: m.content_type,
            content_length: m.content_length,
            required: m.required,
            column_index: m.column_index,
        }
    }
}
