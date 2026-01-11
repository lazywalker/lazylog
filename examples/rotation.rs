//! Log rotation example.
//!
//! This example demonstrates file logging with automatic rotation
//! based on file size and time period.

use lazylog::{FileLogConfig, RotationTrigger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = std::path::PathBuf::from("/tmp/test_rotation.log");

    // Configure file logging with rotation
    let file_config =
        FileLogConfig::new(log_path.clone()).with_rotation_trigger(RotationTrigger::size(1024, 5)); // 1KB, keep 5 files

    // Initialize logging with builder API
    lazylog::builder()
        .with_console(true)
        .with_level("info")
        .with_file_config(file_config)
        .init()?;

    for i in 0..100 {
        tracing::info!("Log message number {}", i);
    }

    // Check if log file was created immediately after logging
    if log_path.exists() {
        println!("Log file created successfully at: {:?}", log_path);
        // Also check file size
        if let Ok(metadata) = std::fs::metadata(&log_path) {
            println!("Log file size: {} bytes", metadata.len());
        }
    } else {
        println!("Log file was not created");
        println!("Log path: {:?}", log_path);
    }

    Ok(())
}
