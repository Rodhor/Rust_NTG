use std::path::{Component, Path, PathBuf};

use crate::domain::error::AppError;

pub fn resolve_path(root: &Path, requested: &str) -> Result<PathBuf, AppError> {
    let escapes = || AppError::PathEscapesRoot {
        path: requested.to_string(),
        root: root.display().to_string(),
    };

    let requested = requested.trim();
    if requested.is_empty() {
        return Ok(root.to_path_buf());
    }

    let relative = Path::new(requested);
    if relative.is_absolute() {
        return Err(escapes());
    }

    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(escapes());
    }

    Ok(relative
        .components()
        .filter_map(|c| match c {
            Component::Normal(n) => Some(n),
            _ => None,
        })
        .fold(root.to_path_buf(), |mut acc, n| {
            acc.push(n);
            acc
        }))
}
