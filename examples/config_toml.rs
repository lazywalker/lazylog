//! Example of loading logging configuration from a TOML file.
//!
//! This example demonstrates how to load logging configuration from
//! a TOML file and initialize the logging system.
//!
//! Run with:
//! ```bash
//! cargo run --example config_toml
//! ```

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    log: lazylog::LogConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the TOML configuration file
    let config_path = "examples/config.toml";
    let config_content = fs::read_to_string(config_path)
        .unwrap_or_else(|_| panic!("Failed to read config file: {}", config_path));

    // Parse the TOML configuration
    let root: Config = toml::from_str(&config_content)?;
    let config = root.log;

    // Initialize logging with the loaded configuration
    lazylog::init_logging(&config)?;

    // Log some messages
    tracing::trace!("This is a trace message (usually not visible)");
    tracing::debug!("This is a debug message (visible because level is debug)");
    tracing::info!("This is an info message");
    tracing::warn!("This is a warning message");
    tracing::error!("This is an error message");

    // Log with structured data (will be formatted as JSON)
    tracing::info!(
        user = "bob",
        action = "logout",
        duration_ms = 1234,
        "User session ended"
    );

    tracing::error!(
        error_code = 500,
        error_type = "database",
        message = "Connection failed",
        "Database error occurred"
    );

    Ok(())
}
