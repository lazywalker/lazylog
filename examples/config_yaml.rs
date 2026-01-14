//! Example of loading logging configuration from a YAML file.
//!
//! This example demonstrates how to load logging configuration from
//! a YAML file and initialize the logging system.
//!
//! Run with:
//! ```bash
//! cargo run --example config_yaml
//! ```

use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the YAML configuration file
    let config_path = "examples/config.yaml";
    let config_content = fs::read_to_string(config_path)
        .unwrap_or_else(|_| panic!("Failed to read config file: {}", config_path));

    // Parse the YAML configuration
    let root: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&config_content)?;
    let config: lazylog::LogConfig = serde_yaml::from_value(root["log"].clone())?;

    // Initialize logging with the loaded configuration
    lazylog::init_logging(&config)?;

    // Log some messages
    tracing::trace!("This is a trace message (usually not visible)");
    tracing::debug!("This is a debug message");
    tracing::info!("This is an info message");
    tracing::warn!("This is a warning message");
    tracing::error!("This is an error message");

    // Log with structured data
    tracing::info!(user = "alice", action = "login", "User performed an action");

    tracing::warn!(error_code = 404, path = "/api/users", "Resource not found");

    Ok(())
}
