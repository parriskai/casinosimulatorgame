use std::fs::OpenOptions;
use tracing::Level;
use tracing_subscriber::{
    EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn init_tracing() {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("casino.log")
        .expect("failed to open log file");

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_ansi(true)
                .with_writer(std::io::stdout)
        )
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(file)
        )
        .with(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into())
        )
        .init();
}