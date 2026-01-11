use lazylog::{FileLogConfig, LogConfig, RotationPeriod, RotationTrigger, init_logging};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let log_path = temp_dir.path().join("test.log");

    let file_config = FileLogConfig::new(log_path.clone())
        .with_rotation_trigger(RotationTrigger::both(RotationPeriod::Hourly, 1024, 5)); // 1KB or 1 hour

    let config = LogConfig::new().with_console(true).with_file(file_config);

    init_logging(&config, None)?;

    for i in 0..100 {
        tracing::info!("Log message number {}", i);
    }

    // Check if log file was created
    if log_path.exists() {
        println!("Log file created successfully");
    } else {
        println!("Log file was not created");
    }

    Ok(())
}
