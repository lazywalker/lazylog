//! # Lazylog
//!
//! A flexible logging library with file rotation and structured output.
//!
//! ## Features
//!
//! - Console and file logging
//! - Automatic log rotation based on size or time
//! - Structured logging with JSON output
//! - Integration with the `tracing` ecosystem
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! // Simple console logging
//! lazylog::builder()
//!     .with_console(true)
//!     .with_level("info")
//!     .init()
//!     .expect("Failed to initialize logging");
//!
//! tracing::info!("This is an info message");
//! ```
//!
//! ## With File Logging
//!
//! ```rust,no_run
//! use lazylog::RotationTrigger;
//!
//! lazylog::builder()
//!     .with_console(true)
//!     .with_level("debug")
//!     .with_file("/var/log/app.log")
//!     .with_rotation(RotationTrigger::size(10 * 1024 * 1024, 5)) // 10MB, keep 5 files
//!     .init()
//!     .expect("Failed to initialize logging");
//! ```
//!
//! ## From Existing Config
//!
//! ```rust,no_run
//! use lazylog::LogConfig;
//!
//! let config = LogConfig::new().with_console(true);
//! lazylog::from_config(config)
//!     .with_level("debug")
//!     .init()
//!     .expect("Failed to initialize logging");
//! ```

pub mod builder;
/// Configuration structures for logging setup.
pub mod config;
/// Error types for the logging library.
pub mod error;
/// Log rotation functionality.
pub mod rotation;
/// Tracing initialization utilities.
pub mod tracing_init;
/// Log writer implementations.
pub mod writer;

pub use builder::LogBuilder;
pub use config::{FileLogConfig, LogConfig};
pub use error::{Error, Result};
pub use rotation::{RotationPeriod, RotationTrigger};
pub use tracing_init::init_logging;
pub use writer::RotatingWriter;

/// Create a new logging configuration builder.
///
/// This is the recommended way to initialize logging with a fluent API.
///
/// # Example
///
/// ```rust,no_run
/// lazylog::builder()
///     .with_console(true)
///     .with_level("info")
///     .init()
///     .expect("Failed to initialize logging");
/// ```
pub fn builder() -> LogBuilder {
    LogBuilder::new()
}

/// Create a logging configuration builder from an existing config.
///
/// # Example
///
/// ```rust,no_run
/// use lazylog::LogConfig;
///
/// let config = LogConfig::new().with_console(true);
/// lazylog::from_config(config)
///     .with_level("debug")
///     .init()
///     .expect("Failed to initialize logging");
/// ```
pub fn from_config(config: LogConfig) -> LogBuilder {
    LogBuilder::from_config(config)
}
