use crate::helpers::sftp;
use ssh2::Session;
use std::io::copy;
use std::path::Path;

pub fn upload_file(
    connection: Session,
    local_path: String,
    remote_path: String,
) -> Result<(), String> {
    let sftp = sftp(&connection)?;

    let mut local_file = std::fs::File::open(&local_path)
        .map_err(|err| format!("Failed to open local file: {}", err))?;

    let mut remote_file = sftp
        .create(Path::new(&remote_path))
        .map_err(|err| format!("Failed to create remote file: {}", err))?;

    copy(&mut local_file, &mut remote_file)
        .map_err(|err| format!("Failed to upload file: {}", err))?;

    Ok(())
}

pub fn download_file(
    connection: Session,
    local_path: String,
    remote_path: String,
) -> Result<(), String> {
    let sftp = sftp(&connection)?;

    let mut remote_file = sftp
        .open(&remote_path)
        .map_err(|err| format!("Failed to create remote file: {}", err))?;

    let mut local_file = std::fs::File::create(&local_path)
        .map_err(|err| format!("Failed to open local file: {}", err))?;

    copy(&mut remote_file, &mut local_file)
        .map_err(|err| format!("Failed to download file: {}", err))?;

    Ok(())
}

pub fn delete_remote_file(connection: Session, remote_path: String) -> Result<(), String> {
    let sftp = sftp(&connection)?;

    sftp.unlink(Path::new(&remote_path))
        .map_err(|err| format!("Failed to delete remote file: {}", err))?;

    Ok(())
}

pub fn move_file_remote(
    connection: Session,
    remote_source_path: String,
    remote_destination_path: String,
) -> Result<(), String> {
    let sftp = sftp(&connection)?;
    sftp.rename(
        Path::new(&remote_source_path),
        Path::new(&remote_destination_path),
        None,
    )
    .map_err(|err| format!("Failed to rename file: {}", err))?;

    Ok(())
}
