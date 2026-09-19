use ssh2::Session;

pub fn sftp(connection: &Session) -> Result<ssh2::Sftp, String> {
    connection
        .sftp()
        .map_err(|err| format!("Failed to create sftp channel: {}", err))
}
