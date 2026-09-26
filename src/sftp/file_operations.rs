use crate::{domain::error::AppError, sftp::helpers};
use ssh2::{RenameFlags, Session};
use std::path::Path;

//TODO: Add tmp and then rename
pub fn upload_file(
    connection: &Session,
    root: &Path,
    requested: &str,
    local_path: &Path,
) -> Result<(), AppError> {
    let sftp = connection.sftp()?;
    let mut local_file = std::fs::File::open(&local_path)?;
    let remote_path = helpers::resolve_path(root, requested)?;
    let mut remote_file = sftp.create(&remote_path)?;
    std::io::copy(&mut local_file, &mut remote_file)?;
    Ok(())
}

//TODO: Add tmp and then rename
pub fn download_file(
    connection: &Session,
    root: &Path,
    requested: &str,
    local_path: &Path,
) -> Result<(), AppError> {
    let sftp = connection.sftp()?;
    let remote_path = helpers::resolve_path(root, requested)?;
    let mut remote_file = sftp.open(&remote_path)?;
    let mut local_file = std::fs::File::create(&local_path)?;
    std::io::copy(&mut remote_file, &mut local_file)?;
    Ok(())
}

pub fn delete_remote_file(connection: &Session, remote_path: &Path) -> Result<(), AppError> {
    let sftp = connection.sftp()?;
    sftp.unlink(&remote_path)?;
    Ok(())
}

//TODO: Add tmp and then rename
pub fn move_remote_file(
    connection: &Session,
    from_root: &Path,
    from: &str,
    to_root: &Path,
    to: &str,
) -> Result<(), AppError> {
    let sftp = connection.sftp()?;
    let src = helpers::resolve_path(from_root, from)?;
    let dst = helpers::resolve_path(to_root, to)?;
    sftp.rename(&src, &dst, Some(RenameFlags::ATOMIC))?; // Error if file already exists
    Ok(())
}
