use thiserror::Error as ThisError;

/// Errors that can occur in the logging library
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Initialization error: {0}")]
    Init(String),
    #[cfg(feature = "time")]
    #[error("Time error: {0}")]
    Time(#[from] time::error::Error),
    #[error("System time error: {0}")]
    SystemTime(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
