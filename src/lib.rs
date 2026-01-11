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
//! ## Example
//!
//! ```rust
//! use lazylog::{init_logging, LogConfig};
//!
//! let config = LogConfig::new().with_console(true);
//! init_logging(&config, None)?;
//!
//! tracing::info!("This is an info message");
//! # Ok::<(), lazylog::Error>(())
//! ```

pub mod config;
pub mod error;
pub mod rotation;
pub mod writer;

#[cfg(feature = "tracing-integration")]
pub mod tracing_init;

pub use config::{FileLogConfig, LogConfig};
pub use error::{Error, Result};
pub use rotation::{RotationPeriod, RotationTrigger};
pub use writer::RotatingWriter;

#[cfg(feature = "tracing-integration")]
pub use tracing_init::init_logging;
