use std::{
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::domain::{config_model::SftpConfig, error::AppError};
use ssh2::{HashType, Session};

pub async fn connect_blocking(config: SftpConfig) -> Result<Session, AppError> {
    tokio::task::spawn_blocking(|| connect(config)).await?
}

pub async fn disconnect_blocking(session: Session) -> Result<(), AppError> {
    tokio::task::spawn_blocking(|| disconnect(session)).await?
}

fn connect(config: SftpConfig) -> Result<Session, AppError> {
    let port = config.port.unwrap_or(22);
    let timeout = Duration::from_secs(config.connect_timeout_secs);
    let addr = (config.host.as_str(), port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| AppError::Sftp(format!("no address for {}:{port}", config.host)))?;

    let tcp = TcpStream::connect_timeout(&addr, timeout)?;
    let mut session = Session::new()?;
    session.set_blocking(true);
    session.set_timeout(timeout.as_millis() as u32);
    session.set_tcp_stream(tcp);
    session.handshake()?;
    session.set_keepalive(false, 60);

    verify_host_key(&session, &config.host_key_sha256)?;

    let password = config
        .password
        .as_deref()
        .ok_or_else(|| AppError::Sftp("no password".to_string()))?;
    session.userauth_password(&config.username, password)?;

    if !session.authenticated() {
        return Err(AppError::Sftp("authentication rejected".to_string()));
    }

    Ok(session)
}

fn verify_host_key(session: &Session, expected_hash: &str) -> Result<(), AppError> {
    let actual = session
        .host_key_hash(HashType::Sha256)
        .ok_or_else(|| AppError::Sftp("server presented no host key".to_string()))?;

    let actual_hex: String = actual.iter().map(|b| format!("{b:02x}")).collect();
    if !actual_hex.eq_ignore_ascii_case(expected_hash.trim()) {
        return Err(AppError::HostKeyMismatch {
            expected: expected_hash.trim().to_string(),
            actual: actual_hex,
        });
    }
    Ok(())
}

fn disconnect(session: Session) -> Result<(), AppError> {
    session.disconnect(None, "User disconnected", None)?;
    Ok(())
}
