//! Builder pattern for initializing logging configuration.
//!
//! This module provides a convenient builder API for configuring and initializing
//! logging in a single chain of method calls.
//!
//! # Example
//!
//! ```rust,no_run
//! use lazylog;
//!
//! // Simple console logging
//! lazylog::builder()
//!     .with_console(true)
//!     .with_level("info")
//!     .init()
//!     .expect("Failed to initialize logging");
//!
//! // With file logging
//! lazylog::builder()
//!     .with_console(true)
//!     .with_level("debug")
//!     .with_file("/var/log/app.log")
//!     .init()
//!     .expect("Failed to initialize logging");
//! ```

use crate::init_logging;
use crate::{FileLogConfig, LogConfig, Result, RotationTrigger};
use std::path::PathBuf;

/// A builder for configuring and initializing logging.
///
/// This provides a fluent interface for setting up logging configuration
/// and initializing the logging system in one chain of calls.
#[derive(Debug, Clone)]
pub struct LogBuilder {
    config: LogConfig,
}

impl LogBuilder {
    /// Create a new LogBuilder with default configuration.
    pub fn new() -> Self {
        Self {
            config: LogConfig::new(),
        }
    }

    /// Create a LogBuilder from an existing configuration.
    pub fn from_config(config: LogConfig) -> Self {
        Self { config }
    }

    /// Enable or disable console logging.
    pub fn with_console(mut self, enabled: bool) -> Self {
        self.config = self.config.with_console(enabled);
        self
    }

    /// Set the log level (e.g., "trace", "debug", "info", "warn", "error").
    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.config = self.config.with_level(level.into());
        self
    }

    /// Set the log output format ("text" or "json").
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.config = self.config.with_format(format.into());
        self
    }

    /// Configure file logging with a path.
    ///
    /// This creates a FileLogConfig with the default rotation settings (no rotation).
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        let file_config = FileLogConfig::new(path);
        self.config = self.config.with_file(file_config);
        self
    }

    /// Configure file logging with a custom FileLogConfig.
    pub fn with_file_config(mut self, file_config: FileLogConfig) -> Self {
        self.config = self.config.with_file(file_config);
        self
    }

    /// Set the rotation trigger for file logging.
    ///
    /// This is a convenience method that modifies the file configuration.
    /// If no file is configured, this will create a default file at "app.log".
    pub fn with_rotation(mut self, rotation: RotationTrigger) -> Self {
        if let Some(ref mut file) = self.config.file {
            file.rotation = rotation;
        } else {
            // If no file is configured, create one with default path
            self.config.file = Some(FileLogConfig::new("app.log").with_rotation_trigger(rotation));
        }
        self
    }

    /// Show target/module in logs
    pub fn with_target(mut self, target: bool) -> Self {
        self.config = self.config.with_target(target);
        self
    }

    /// Show thread IDs in logs
    pub fn with_thread_ids(mut self, thread_ids: bool) -> Self {
        self.config = self.config.with_thread_ids(thread_ids);
        self
    }

    /// Show thread names in logs
    pub fn with_thread_names(mut self, thread_names: bool) -> Self {
        self.config = self.config.with_thread_names(thread_names);
        self
    }

    /// Get the current configuration without initializing.
    pub fn build(self) -> LogConfig {
        self.config
    }

    /// Initialize logging with the configured settings.
    ///
    /// This consumes the builder and initializes the global logging system.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The tracing subscriber is already initialized
    /// - File operations fail
    /// - Invalid configuration is provided
    pub fn init(self) -> Result<()> {
        init_logging(&self.config)
    }
}

impl Default for LogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = LogBuilder::new();
        let config = builder.build();
        assert!(!config.console);
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "text");
    }

    #[test]
    fn test_builder_with_console() {
        let builder = LogBuilder::new().with_console(true);
        let config = builder.build();
        assert!(config.console);
    }

    #[test]
    fn test_builder_with_level() {
        let builder = LogBuilder::new().with_level("debug");
        let config = builder.build();
        assert_eq!(config.level, "debug");
    }

    #[test]
    fn test_builder_with_format() {
        let builder = LogBuilder::new().with_format("json");
        let config = builder.build();
        assert_eq!(config.format, "json");
    }

    #[test]
    fn test_builder_with_file() {
        let builder = LogBuilder::new().with_file("test.log");
        let config = builder.build();
        assert!(config.file.is_some());
        assert_eq!(config.file.unwrap().path, PathBuf::from("test.log"));
    }

    #[test]
    fn test_builder_chaining() {
        let builder = LogBuilder::new()
            .with_console(true)
            .with_level("debug")
            .with_format("json")
            .with_file("app.log");

        let config = builder.build();
        assert!(config.console);
        assert_eq!(config.level, "debug");
        assert_eq!(config.format, "json");
        assert!(config.file.is_some());
    }

    #[test]
    fn test_builder_from_config() {
        let original = LogConfig::new()
            .with_console(true)
            .with_level("warn".to_string());
        let builder = LogBuilder::from_config(original.clone());
        let config = builder.build();
        assert_eq!(config.console, original.console);
        assert_eq!(config.level, original.level);
    }

    #[test]
    fn test_builder_with_rotation() {
        use crate::RotationTrigger;

        let builder = LogBuilder::new()
            .with_file("test.log")
            .with_rotation(RotationTrigger::size(1024 * 1024, 5));

        let config = builder.build();
        assert!(config.file.is_some());
        let file_config = config.file.unwrap();
        assert_eq!(file_config.rotation, RotationTrigger::size(1024 * 1024, 5));
    }

    #[test]
    fn test_builder_with_target() {
        let builder = LogBuilder::new().with_target(true);
        let config = builder.build();
        assert!(config.target);
    }

    #[test]
    fn test_builder_with_thread_ids() {
        let builder = LogBuilder::new().with_thread_ids(true);
        let config = builder.build();
        assert!(config.thread_ids);
    }

    #[test]
    fn test_builder_with_thread_names() {
        let builder = LogBuilder::new().with_thread_names(true);
        let config = builder.build();
        assert!(config.thread_names);
    }
}
