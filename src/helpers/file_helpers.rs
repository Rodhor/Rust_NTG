use std::path::{Path, PathBuf};

pub fn sibling_with_suffix(path: &Path, suffix: Option<&str>) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let suffix = suffix.unwrap_or(".tmp");

    path.with_file_name(format!("{name}{suffix}"))
}
