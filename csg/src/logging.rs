//! `csg::logging`
//! Manage the games logging

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use std::fs::OpenOptions;

use crate::errors::GResult;

/// Initalize tracing
/// 
/// SHOULD ONLY BE CALLED ONCE
/// 
/// IDK what happens if you call this twice so just dont
/// 
/// ```
/// fn main(){
///     init_tracing()
/// 
///     // Run game
/// }
/// ```
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
                .with_ansi(false) // Fix to prevent awful unreadable files
                .with_writer(file)
        )
        .with(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
}

/// logs an error if returned by the lambda
/// 
/// # Example
/// ```
/// log_result(||{
///     // Do some work
///     Ok(7)
/// },"My working function"
/// ); // No error message, returns Ok(7)
/// 
/// 
/// log_result(||{
///     // Do something that fails
///     Err(MyErrorMessage)
/// }, "My non working error mesage"
/// ); // Error message logged, returns Err(MyErrorMessage) afterwards
/// ```
pub fn log_result<F: FnOnce() -> GResult<R>, R>(f: F, name: &str) -> GResult<R>{
    let ret = f();

    if let Err(e) = &ret {
        tracing::error!("{name} exited with error {e}")
    }

    ret
}