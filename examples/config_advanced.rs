//! Advanced example showing configuration file selection at runtime.
//!
//! This example demonstrates how to:
//! - Select configuration file format based on environment or command line
//! - Load and validate configuration
//! - Handle configuration errors gracefully
//!
//! Run with:
//! ```bash
//! # Use YAML configuration
//! cargo run --example config_advanced -- yaml
//!
//! # Use TOML configuration  
//! cargo run --example config_advanced -- toml
//!
//! # Use default (YAML)
//! cargo run --example config_advanced
//! ```

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Deserialize)]
struct Config {
    log: lazylog::LogConfig,
}

fn load_yaml_config(path: &str) -> Result<lazylog::LogConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let root: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&content)?;
    let config: lazylog::LogConfig = serde_yaml::from_value(root["log"].clone())?;
    Ok(config)
}

fn load_toml_config(path: &str) -> Result<lazylog::LogConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let root: Config = toml::from_str(&content)?;
    Ok(root.log)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration format from command line arguments or environment
    let args: Vec<String> = env::args().collect();
    let env_format = env::var("LOG_CONFIG_FORMAT").unwrap_or_else(|_| "yaml".to_string());
    let config_format = if args.len() > 1 {
        args[1].as_str()
    } else {
        env_format.as_str()
    };

    // Load configuration based on format
    let config = match config_format {
        "yaml" | "yml" => {
            println!("Loading YAML configuration from examples/config.yaml");
            load_yaml_config("examples/config.yaml")?
        }
        "toml" => {
            println!("Loading TOML configuration from examples/config.toml");
            load_toml_config("examples/config.toml")?
        }
        _ => {
            eprintln!("Unknown format: {}", config_format);
            eprintln!("Supported formats: yaml, toml");
            std::process::exit(1);
        }
    };

    // Display loaded configuration
    println!("Configuration loaded successfully:");
    println!("  Console: {}", config.console);
    println!("  Level: {}", config.level);
    println!("  Format: {}", config.format);
    println!("  Target: {}", config.target);
    println!("  Thread IDs: {}", config.thread_ids);
    println!("  Thread Names: {}", config.thread_names);
    if let Some(ref file) = config.file {
        println!("  File: {:?}", file.path);
        println!("  Rotation: {:?}", file.rotation);
    }
    println!();

    // Initialize logging with the loaded configuration
    lazylog::init_logging(&config)?;

    // Generate log messages
    tracing::info!("=== Logging initialized successfully ===");

    tracing::trace!("Trace level message");
    tracing::debug!("Debug level message");
    tracing::info!("Info level message");
    tracing::warn!("Warning level message");
    tracing::error!("Error level message");

    // Structured logging examples
    tracing::info!(config_format = config_format, "Configuration format used");

    tracing::info!(
        user_id = 12345,
        username = "admin",
        ip_address = "192.168.1.100",
        "User authenticated"
    );

    tracing::warn!(
        retry_count = 3,
        max_retries = 5,
        service = "database",
        "Service connection retry"
    );

    tracing::error!(
        error_code = "ERR_500",
        component = "api_handler",
        request_id = "req-abc-123",
        "Internal server error"
    );

    // Nested spans example
    let span = tracing::info_span!("request_handler", request_id = "req-xyz-789");
    let _enter = span.enter();

    tracing::info!("Processing request");
    tracing::debug!("Validating input parameters");
    tracing::info!("Request processing completed");

    Ok(())
}
