use thiserror::Error as ThisError;

/// Errors that can occur in the logging library
#[derive(ThisError, Debug)]
pub enum Error {
    /// I/O operation failed.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Configuration is invalid.
    #[error("Configuration error: {0}")]
    Config(String),
    /// Initialization failed.
    #[error("Initialization error: {0}")]
    Init(String),
    #[cfg(feature = "time")]
    #[error("Time error: {0}")]
    Time(#[from] time::error::Error),
    /// System time operation failed.
    #[error("System time error: {0}")]
    SystemTime(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
