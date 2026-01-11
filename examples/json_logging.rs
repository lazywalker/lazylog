use lazylog::{LogConfig, init_logging};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LogConfig::new()
        .with_console(true)
        .with_format("json".to_string());

    init_logging(&config, None)?;

    tracing::info!(user_id = 123, action = "login", "User logged in");
    tracing::warn!(error_code = 404, "Resource not found");
    tracing::error!(component = "auth", error = "Authentication failed");

    Ok(())
}
