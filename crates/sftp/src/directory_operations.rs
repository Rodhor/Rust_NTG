use crate::helpers::sftp;
use crate::types::FileItem;
use ssh2::Session;
use std::path::Path;

pub fn list_dir(connection: Session, path: String) -> Result<Vec<FileItem>, String> {
    let sftp = sftp(&connection)?;

    let mut files = Vec::new();
    let dir_path = Path::new(&path);
    let entries = sftp
        .readdir(dir_path)
        .map_err(|err| format!("Failed to read directory: {}", err))?;

    for (file_path, stats) in entries {
        let name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if name.starts_with(".") && name != ".." {
            continue; // Skips hidden files
        }

        files.push(FileItem {
            name,
            path: file_path.to_string_lossy().replace("\\", "/"),
            is_dir: stats.is_dir(),
            size: stats.size.unwrap_or(0),
            last_modified: stats.mtime.unwrap_or(0),
        })
    }

    // Sort directories before files
    files.sort_by(|a, b| {
        if a.name == ".." {
            return std::cmp::Ordering::Less;
        }
        if b.name == ".." {
            return std::cmp::Ordering::Greater;
        }
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(files)
}

pub fn create_remote_dir(connection: Session, path: String) -> Result<(), String> {
    let sftp = sftp(&connection)?;

    sftp.mkdir(Path::new(&path), 0o1777)
        .map_err(|err| format!("Failed to create directory: {}", err))?;

    Ok(())
}

pub fn delete_remote_dir(connection: Session, path: String) -> Result<(), String> {
    let sftp = sftp(&connection)?;
    sftp.rmdir(Path::new(&path))
        .map_err(|err| format!("Failed to remove directory: {}", err))?;

    Ok(())
}
