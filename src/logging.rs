use std::{fs, io, path::Path};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer, fmt, fmt::writer::BoxMakeWriter, layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::domain::{
    config_model::{LogConfig, LogFormat, LogLevel, LogTarget},
    error::AppError,
};

pub fn init(config: &LogConfig) -> Result<Option<WorkerGuard>, AppError> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level_str(&config.level)));

    let mut file_guard = None;
    let writer: BoxMakeWriter = match config.target {
        LogTarget::Stdout => BoxMakeWriter::new(io::stdout),
        LogTarget::Stderr => BoxMakeWriter::new(io::stderr),
        LogTarget::File => {
            let path = Path::new(config.file_path.as_deref().unwrap_or("logs/app.log"));
            let dir = path.parent().unwrap_or(Path::new("logs"));
            fs::create_dir_all(dir)?;
            let prefix = path.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
            let appender = tracing_appender::rolling::daily(dir, prefix);
            let (writer, guard) = tracing_appender::non_blocking(appender);
            file_guard = Some(guard);
            BoxMakeWriter::new(writer)
        }
    };

    let use_ansi = matches!(config.target, LogTarget::Stdout | LogTarget::Stderr);
    let fmt_layer = match config.format {
        LogFormat::Json => fmt::layer().json().with_writer(writer).boxed(),
        LogFormat::Pretty => fmt::layer()
            .pretty()
            .with_writer(writer)
            .with_ansi(use_ansi)
            .boxed(),
    };

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .try_init()?;

    Ok(file_guard)
}

fn level_str(level: &LogLevel) -> &'static str {
    match level {
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    }
}
