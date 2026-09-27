use crate::{domain::error::AppError, sftp::helpers};
use ssh2::{RenameFlags, Session};
use std::{io::copy, path::Path};

pub fn upload_file(
    connection: &Session,
    root: &Path,
    requested: &str,
    local_path: &Path,
) -> Result<String, AppError> {
    let sftp = connection.sftp()?;

    let remote_path = helpers::resolve_path(root, requested)?;
    let mut src = std::fs::File::open(&local_path)?;

    let mut dst = sftp.create(&remote_path.join(".tmp"))?;
    match copy(&mut src, &mut dst) {
        Ok(_) => {
            sftp.rename(
                &remote_path.join(".tmp"),
                &remote_path,
                Some(RenameFlags::empty()),
            )?;
        }
        Err(e) => {
            sftp.unlink(&remote_path.join(".tmp"))?;
            return Err(e.into());
        }
    }

    Ok(remote_path.to_string_lossy().to_string())
}

pub fn download_file(
    connection: &Session,
    root: &Path,
    requested: &str,
    local_path: &Path,
) -> Result<String, AppError> {
    let sftp = connection.sftp()?;

    let remote_path = helpers::resolve_path(root, requested)?;
    let mut src = sftp.open(&remote_path)?;

    let mut dst = std::fs::File::create(&local_path.join("tmp"))?;
    match copy(&mut src, &mut dst) {
        Ok(_) => {
            sftp.rename(
                &local_path.join("tmp"),
                &local_path,
                Some(RenameFlags::empty()),
            )?;
        }
        Err(e) => {
            sftp.unlink(&local_path.join(".tmp"))?;
            return Err(e.into());
        }
    }

    Ok(remote_path.to_string_lossy().to_string())
}

pub fn delete_remote_file(
    connection: &Session,
    root: &Path,
    remote_path: &str,
) -> Result<(), AppError> {
    let sftp = connection.sftp()?;
    let remote_path = helpers::resolve_path(root, remote_path)?;
    sftp.unlink(&remote_path)?;
    Ok(())
}

pub fn move_remote_file(
    connection: &Session,
    from_root: &Path,
    from: &str,
    to_root: &Path,
    to: &str,
) -> Result<String, AppError> {
    let sftp = connection.sftp()?;

    let src_path = helpers::resolve_path(from_root, from)?;
    let mut src = std::fs::File::open(&src_path)?;

    let dst_path = helpers::resolve_path(to_root, to)?;
    let mut dst = std::fs::File::create(&dst_path.join("tmp"))?;

    match copy(&mut src, &mut dst) {
        Ok(_) => {
            sftp.rename(&dst_path.join("tmp"), &dst_path, Some(RenameFlags::empty()))?;
        }
        Err(e) => {
            sftp.unlink(&dst_path.join("tmp"))?;
            return Err(e.into());
        }
    }

    Ok(dst_path.to_string_lossy().to_string())
}
