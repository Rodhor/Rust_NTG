use ntg_core::models::sftp_configuration::SftpConfiguration;
use ssh2::Session;
use std::net::TcpStream;

pub async fn connect(config: SftpConfiguration) -> Result<Session, String> {
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    let mut session = Session::new().map_err(|e| format!("Failed to initialize session: {}", e))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("Failed to handshake: {}", e))?;

    session
        .userauth_password(&config.username, &config.password)
        .map_err(|e| format!("Failed to authenticate: {}", e))?;

    if !session.authenticated() {
        return Err("Authentication failed".to_string());
    }

    Ok(session)
}

pub async fn disconnect(session: Session) -> Result<(), String> {
    let _ = session.disconnect(None, "User disconnected", None);
    Ok(())
}
