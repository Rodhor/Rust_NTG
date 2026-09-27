use ssh2::Session;
use std::path::Path;

use crate::{
    domain::{error::AppError, file_item::FileItem},
    sftp::helpers,
};

pub fn list_dir(
    connection: &Session,
    root: &Path,
    requested: &str,
    show_hidden: bool,
) -> Result<Vec<FileItem>, AppError> {
    let remote = helpers::resolve_path(root, requested)?;
    let sftp = connection.sftp()?;
    let mut files = Vec::new();

    for (file_path, stats) in sftp.readdir(&remote)? {
        let name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        files.push(FileItem {
            name,
            path: file_path,
            is_dir: stats.is_dir(),
            size: stats.size,
            last_modified: stats.mtime,
        })
    }

    files.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(files)
}
pub fn create_remote_dir(connection: &Session, root: &Path, path: &str) -> Result<(), AppError> {
    let new_dir = helpers::resolve_path(root, path)?;
    let sftp = connection.sftp()?;
    sftp.mkdir(&new_dir, 0o755)?;
    Ok(())
}

pub fn delete_remote_dir(connection: &Session, root: &Path, path: &str) -> Result<(), AppError> {
    let remote_dir = helpers::resolve_path(root, path)?;
    let sftp = connection.sftp()?;
    sftp.rmdir(&remote_dir)?;
    Ok(())
}
