use lazylog::{LogConfig, init_logging};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LogConfig::new().with_console(true);
    init_logging(&config, None)?;

    tracing::info!("This is an info message");
    tracing::warn!("This is a warning message");
    tracing::error!("This is an error message");

    Ok(())
}
