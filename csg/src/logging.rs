use std::fs::OpenOptions;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initalize tracing, SHOULD ONLY BE CALLED ONCE
/// IDK what happens if you call this twice so just dont
pub fn init_tracing() {
    // Log file
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
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
}