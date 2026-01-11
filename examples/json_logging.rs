//! JSON structured logging example.
//!
//! This example demonstrates how to configure JSON output format
//! for structured logging with additional fields.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with JSON format
    lazylog::builder()
        .with_console(true)
        .with_format("json")
        .with_level("info")
        .init()?;

    tracing::info!(user_id = 123, action = "login", "User logged in");
    tracing::warn!(error_code = 404, "Resource not found");
    tracing::error!(component = "auth", error = "Authentication failed");

    Ok(())
}
