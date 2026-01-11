use serde::{Deserialize, Serialize};

/// Configuration for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Enable console logging
    #[serde(default)]
    pub console: bool,
    /// Console log level (e.g., "info", "debug")
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log format ("text" or "json")
    #[serde(default = "default_format")]
    pub format: String,
    /// File logging configuration
    pub file: Option<FileLogConfig>,
}

impl LogConfig {
    /// Create a new LogConfig with defaults
    pub fn new() -> Self {
        Self {
            console: false,
            level: default_log_level(),
            format: default_format(),
            file: None,
        }
    }

    /// Enable console logging
    pub fn with_console(mut self, console: bool) -> Self {
        self.console = console;
        self
    }

    /// Set log level
    pub fn with_level(mut self, level: String) -> Self {
        self.level = level;
        self
    }

    /// Set log format
    pub fn with_format(mut self, format: String) -> Self {
        self.format = format;
        self
    }

    /// Set file logging configuration
    pub fn with_file(mut self, file: FileLogConfig) -> Self {
        self.file = Some(file);
        self
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::new()
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_format() -> String {
    "text".to_string()
}

/// Configuration for file logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLogConfig {
    /// Path to the log file
    pub path: std::path::PathBuf,
    /// Log rotation trigger
    #[serde(default)]
    pub rotation: crate::RotationTrigger,
}

impl FileLogConfig {
    /// Create a new FileLogConfig
    pub fn new<P: Into<std::path::PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            rotation: crate::RotationTrigger::Never,
        }
    }

    /// Set rotation trigger
    pub fn with_rotation_trigger(mut self, rotation: crate::RotationTrigger) -> Self {
        self.rotation = rotation;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_log_config_new() {
        let config = LogConfig::new();
        assert!(!config.console);
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "text");
        assert!(config.file.is_none());
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert!(!config.console);
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "text");
        assert!(config.file.is_none());
    }

    #[test]
    fn test_log_config_with_console() {
        let config = LogConfig::new().with_console(true);
        assert!(config.console);
    }

    #[test]
    fn test_log_config_with_level() {
        let config = LogConfig::new().with_level("debug".to_string());
        assert_eq!(config.level, "debug");
    }

    #[test]
    fn test_log_config_with_format() {
        let config = LogConfig::new().with_format("json".to_string());
        assert_eq!(config.format, "json");
    }

    #[test]
    fn test_log_config_with_file() {
        let file_config = FileLogConfig::new("test.log");
        let config = LogConfig::new().with_file(file_config);
        assert!(config.file.is_some());
        assert_eq!(
            config.file.as_ref().unwrap().path,
            PathBuf::from("test.log")
        );
    }

    #[test]
    fn test_file_log_config_new() {
        let config = FileLogConfig::new("test.log");
        assert_eq!(config.path, PathBuf::from("test.log"));
        assert_eq!(config.rotation, crate::RotationTrigger::Never);
    }

    #[test]
    fn test_file_log_config_with_rotation_trigger() {
        let config = FileLogConfig::new("test.log")
            .with_rotation_trigger(crate::RotationTrigger::size(1024, 5));
        assert_eq!(config.path, PathBuf::from("test.log"));
        assert_eq!(config.rotation, crate::RotationTrigger::size(1024, 5));
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(super::default_log_level(), "info");
        assert_eq!(super::default_format(), "text");
    }
}
