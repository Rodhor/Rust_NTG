use crate::{domain::error::AppError, helpers::file_helpers, sftp::helpers};
use ssh2::{OpenFlags, OpenType, RenameFlags, Session};
use std::{fs, io::copy, path::Path};

pub fn upload_file(
    connection: &Session,
    root: &Path,
    requested: &str,
    local_path: &Path,
) -> Result<String, AppError> {
    let sftp = connection.sftp()?;

    let remote_path = helpers::resolve_path(root, requested)?;
    let staging = file_helpers::sibling_with_suffix(&remote_path, None);

    // clear any leftover staging files in case of a previous upload failure
    let _ = sftp.unlink(&staging);

    let mut src = fs::File::open(local_path)?;
    let mut dst = sftp.open_mode(
        &staging,
        OpenFlags::WRITE | OpenFlags::EXCLUSIVE,
        0o644,
        OpenType::File,
    )?;

    if let Err(e) = copy(&mut src, &mut dst) {
        drop(dst);
        let _ = sftp.unlink(&staging);
        return Err(e.into());
    }

    drop(dst);
    if let Err(e) = sftp.rename(&staging, &remote_path, Some(RenameFlags::empty())) {
        let _ = sftp.unlink(&staging);
        return Err(e.into());
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
    let staging = file_helpers::sibling_with_suffix(local_path, Some(".part"));
    let mut src = sftp.open(&remote_path)?;

    // clear any leftover staging files in case of a previous upload failure
    let _ = fs::remove_file(&staging);
    let mut dst = fs::File::options()
        .write(true)
        .create_new(true)
        .open(&staging)?;

    if let Err(e) = copy(&mut src, &mut dst) {
        drop(dst);
        let _ = fs::remove_file(&staging);
        return Err(e.into());
    }

    drop(dst);
    if let Err(e) = fs::rename(&staging, &local_path) {
        let _ = sftp.unlink(&staging);
        return Err(e.into());
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
    root: &Path,
    from_requested: &str,
    to_requested: &str,
    overwrite: bool,
) -> Result<String, AppError> {
    let sftp = connection.sftp()?;

    let src = helpers::resolve_path(root, from_requested)?;
    let dst = helpers::resolve_path(root, to_requested)?;

    let flags = if overwrite {
        RenameFlags::OVERWRITE
    } else {
        RenameFlags::empty()
    };
    sftp.rename(&src, &dst, Some(flags))?;

    Ok(dst.to_string_lossy().to_string())
}
