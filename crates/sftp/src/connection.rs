use ntg_core::models::sftp_configuration::SftpConfiguration;
use ssh2::Session;
use std::net::TcpStream;

pub async fn connect_blocking(config: SftpConfiguration) -> Result<Session, String> {
    tokio::task::spawn_blocking(move || connect(config))
        .await
        .map_err(|err| format!("Connection task panicked: {}", err))?
}

pub async fn disconnect_blocking(session: Session) -> Result<(), String> {
    tokio::task::spawn_blocking(move || disconnect(session))
        .await
        .map_err(|err| format!("Disconnect task panicked: {}", err))?
}

fn connect(config: SftpConfiguration) -> Result<Session, String> {
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

fn disconnect(session: Session) -> Result<(), String> {
    let _ = session.disconnect(None, "User disconnected", None);
    Ok(())
}
