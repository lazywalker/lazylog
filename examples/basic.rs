//! Basic console logging example.
//!
//! This example demonstrates the simplest way to initialize logging
//! with lazylog using the builder API.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with the builder API
    lazylog::builder()
        .with_console(true)
        .with_level("info")
        .init()?;

    tracing::info!("This is an info message");
    tracing::warn!("This is a warning message");
    tracing::error!("This is an error message");

    Ok(())
}
