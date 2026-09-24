mod config;
mod csv;
mod domain;
mod error;
mod extract;
mod middleware;
mod router;
mod sftp;
mod shutdown;
mod state;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let config = config::load_config()?;

    Ok(())
}
