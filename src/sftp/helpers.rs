use std::path::{Component, Path, PathBuf};

use ssh2::Session;

use crate::domain::error::AppError;

pub fn resolve_path(root: &Path, requested: &str) -> Result<PathBuf, AppError> {
    let escapes = || AppError::PathEscapesRoot {
        path: requested.to_string(),
        root: root.display().to_string(),
    };

    if requested.trim().is_empty() {
        return Err(escapes());
    }

    let relative = Path::new(requested);
    if relative.is_absolute() {
        return Err(escapes());
    }
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(escapes());
    }

    Ok(root.join(relative))
}
