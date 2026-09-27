use crate::{domain::error::AppError, sftp::helpers};
use ssh2::{RenameFlags, Session};
use std::{default, fs, io::copy, path::Path};

pub fn upload_file(
    connection: &Session,
    root: &Path,
    requested: &str,
    local_path: &Path,
) -> Result<String, AppError> {
    let sftp = connection.sftp()?;

    let remote_path = helpers::resolve_path(root, requested)?;
    let staging = helpers::sibling_with_suffix(&remote_path, None);

    let mut src = fs::File::open(local_path)?;
    let mut dst = sftp.create(&staging)?;

    if let Err(e) = copy(&mut src, &mut dst) {
        drop(dst);
        sftp.unlink(&staging)?;
        return Err(e.into());
    }

    drop(dst);
    sftp.rename(&staging, &remote_path, Some(RenameFlags::empty()))?;

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
    let staging = helpers::sibling_with_suffix(local_path, Some(".part"));

    let mut src = sftp.open(&remote_path)?;
    let mut dst = fs::File::create(&staging)?;

    if let Err(e) = copy(&mut src, &mut dst) {
        drop(dst);
        fs::remove_file(&staging)?;
        return Err(e.into());
    }

    drop(dst);

    fs::rename(&staging, &local_path)?;

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
    root: &Path,
    from_requested: &str,
    to_requested: &str,
    overwrite: bool,
) -> Result<String, AppError> {
    let sftp = connection.sftp()?;

    let src = helpers::resolve_path(root, from_requested)?;
    let dst = helpers::resolve_path(root, to_requested)?;

    match overwrite {
        true => {
            sftp.rename(&src, &dst, Some(RenameFlags::OVERWRITE))?;
        }
        false => {
            sftp.rename(&src, &dst, Some(RenameFlags::empty()))?;
        }
    }

    Ok(dst.to_string_lossy().to_string())
}
