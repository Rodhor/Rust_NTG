use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: u64,
}
