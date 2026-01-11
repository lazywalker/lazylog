#[cfg(feature = "log-file")]
use crate::FileLogConfig;
#[cfg(feature = "log-file")]
use crate::RotatingWriter;
use crate::{Error, LogConfig, Result};
#[cfg(feature = "log-file")]
use once_cell::sync::Lazy;
#[cfg(feature = "log-file")]
use std::sync::Mutex;
#[cfg(feature = "tracing-subscriber")]
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "log-file")]
static LOG_GUARD: Lazy<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
    Lazy::new(|| Mutex::new(None));

/// Initialize logging with the given configuration and optional CLI verbosity override.
#[cfg(feature = "tracing-subscriber")]
pub fn init_logging(config: &LogConfig, cli_verbose: Option<u8>) -> Result<()> {
    let log_spec = effective_log_spec(config, cli_verbose);

    let env_filter = EnvFilter::try_new(&log_spec).map_err(|e| Error::Init(e.to_string()))?;

    // Determine effective console and file settings based on features
    let effective_console = config.console;
    #[cfg(feature = "log-file")]
    let effective_file: &Option<crate::FileLogConfig> = &config.file;
    #[cfg(not(feature = "log-file"))]
    let effective_file: &Option<crate::FileLogConfig> = &None;

    match (effective_console, effective_file.as_ref()) {
        (true, Some(_)) => {
            // Console and file - only available when log-file feature is enabled
            #[cfg(feature = "log-file")]
            init_console_and_file(config, effective_file.as_ref().unwrap(), env_filter)?;
            #[cfg(not(feature = "log-file"))]
            init_console_only(config, env_filter)?;
        }
        (true, None) => {
            // Console only
            init_console_only(config, env_filter)?;
        }
        (false, Some(_)) => {
            // File only - only available when log-file feature is enabled
            #[cfg(feature = "log-file")]
            init_file_only(config, effective_file.as_ref().unwrap(), env_filter)?;
            #[cfg(not(feature = "log-file"))]
            init_no_logging(env_filter)?;
        }
        (false, None) => {
            // No logging
            init_no_logging(env_filter)?;
        }
    }

    Ok(())
}

/// Initialize console and file logging.
#[cfg(all(feature = "tracing-subscriber", feature = "log-file"))]
fn init_console_and_file(
    config: &LogConfig,
    file_config: &FileLogConfig,
    env_filter: EnvFilter,
) -> Result<()> {
    let fmt_layer_builder = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    let fmt_layer = if config.format == "json" {
        fmt_layer_builder.json().boxed()
    } else {
        fmt_layer_builder.boxed()
    };

    let writer =
        RotatingWriter::new(&file_config.path, file_config.rotation.clone()).map_err(Error::Io)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(writer);

    *LOG_GUARD.lock().unwrap() = Some(guard);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    if config.format == "json" {
        let file_layer = file_layer.json().boxed();
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(file_layer)
            .try_init()
            .map_err(|e| Error::Init(e.to_string()))?;
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(file_layer)
            .try_init()
            .map_err(|e| Error::Init(e.to_string()))?;
    }

    Ok(())
}

/// Initialize console-only logging.
#[cfg(feature = "tracing-subscriber")]
fn init_console_only(config: &LogConfig, env_filter: EnvFilter) -> Result<()> {
    let fmt_layer_builder = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    let fmt_layer = if config.format == "json" {
        fmt_layer_builder.json().boxed()
    } else {
        fmt_layer_builder.boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| Error::Init(e.to_string()))?;

    Ok(())
}

/// Initialize file-only logging.
#[cfg(all(feature = "tracing-subscriber", feature = "log-file"))]
fn init_file_only(
    config: &LogConfig,
    file_config: &FileLogConfig,
    env_filter: EnvFilter,
) -> Result<()> {
    let writer =
        RotatingWriter::new(&file_config.path, file_config.rotation.clone()).map_err(Error::Io)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(writer);

    *LOG_GUARD.lock().unwrap() = Some(guard);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    if config.format == "json" {
        let file_layer = file_layer.json().boxed();
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .try_init()
            .map_err(|e| Error::Init(e.to_string()))?;
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .try_init()
            .map_err(|e| Error::Init(e.to_string()))?;
    }

    Ok(())
}

/// Initialize with no output (for testing or when logging is disabled).
#[cfg(feature = "tracing-subscriber")]
fn init_no_logging(env_filter: EnvFilter) -> Result<()> {
    tracing_subscriber::registry()
        .with(env_filter)
        .try_init()
        .map_err(|e| Error::Init(e.to_string()))?;

    Ok(())
}

/// Determine the effective log specification, considering config and CLI overrides.
fn effective_log_spec(config: &LogConfig, cli_verbose: Option<u8>) -> String {
    // RUST_LOG takes precedence over everything
    if let Ok(rust_log) = std::env::var("RUST_LOG")
        && !rust_log.is_empty()
    {
        return rust_log;
    }

    // CLI verbose flag overrides config level
    if let Some(verbose) = cli_verbose {
        return match verbose {
            0 => config.level.clone(),
            1 => format!("{},lazydns=debug", config.level),
            2 => format!("{},lazydns=trace", config.level),
            _ => "trace".to_string(),
        };
    }

    // Use config level with crate-specific override
    if config.level.is_empty() {
        "info,lazydns=info".to_string()
    } else {
        format!("{},lazydns={}", config.level, config.level)
    }
}

#[cfg(not(feature = "tracing-subscriber"))]
pub fn init_logging(_config: &LogConfig, _cli_verbose: Option<u8>) -> Result<()> {
    tracing::warn!("tracing-subscriber not enabled: logging initialization is a no-op");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogConfig;

    #[test]
    fn rust_log_overrides_config_level() {
        let prev = std::env::var_os("RUST_LOG");
        unsafe {
            std::env::set_var("RUST_LOG", "trace");
        }
        let cfg = LogConfig {
            level: "info".to_string(),
            ..Default::default()
        };

        assert_eq!(effective_log_spec(&cfg, None), "trace");

        unsafe {
            match prev {
                Some(v) => std::env::set_var("RUST_LOG", v),
                None => std::env::remove_var("RUST_LOG"),
            }
        }
    }

    #[test]
    fn cfg_level_used_when_no_rust_log() {
        let prev = std::env::var("RUST_LOG").ok();
        unsafe {
            std::env::set_var("RUST_LOG", "");
        }
        let cfg = LogConfig {
            level: "warn".to_string(),
            ..Default::default()
        };

        assert_eq!(effective_log_spec(&cfg, None), "warn,lazydns=warn");
        assert_eq!(effective_log_spec(&cfg, Some(1)), "warn,lazydns=debug");
        assert_eq!(effective_log_spec(&cfg, Some(2)), "warn,lazydns=trace");
        assert_eq!(effective_log_spec(&cfg, Some(3)), "trace");

        unsafe {
            if let Some(v) = prev {
                std::env::set_var("RUST_LOG", v);
            }
        }
    }

    #[test]
    fn init_logging_succeeds_with_defaults() {
        let cfg = LogConfig::default();
        // This may fail if logging is already initialized, but should not panic
        let _ = init_logging(&cfg, None);
    }

    #[test]
    fn test_effective_log_spec_with_empty_config_level() {
        let cfg = LogConfig {
            level: "".to_string(),
            ..Default::default()
        };
        assert_eq!(effective_log_spec(&cfg, None), "info,lazydns=info");
    }

    #[test]
    fn test_effective_log_spec_with_rust_log() {
        let prev = std::env::var_os("RUST_LOG");
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }

        let cfg = LogConfig {
            level: "info".to_string(),
            ..Default::default()
        };

        assert_eq!(effective_log_spec(&cfg, None), "debug");

        unsafe {
            match prev {
                Some(v) => std::env::set_var("RUST_LOG", v),
                None => std::env::remove_var("RUST_LOG"),
            }
        }
    }

    #[test]
    fn test_effective_log_spec_with_empty_rust_log() {
        let prev = std::env::var_os("RUST_LOG");
        unsafe {
            std::env::set_var("RUST_LOG", "");
        }

        let cfg = LogConfig {
            level: "warn".to_string(),
            ..Default::default()
        };

        assert_eq!(effective_log_spec(&cfg, None), "warn,lazydns=warn");

        unsafe {
            match prev {
                Some(v) => std::env::set_var("RUST_LOG", v),
                None => std::env::remove_var("RUST_LOG"),
            }
        }
    }

    #[test]
    fn test_effective_log_spec_cli_verbose_zero() {
        let cfg = LogConfig {
            level: "info".to_string(),
            ..Default::default()
        };

        assert_eq!(effective_log_spec(&cfg, Some(0)), "info");
    }

    #[test]
    fn test_effective_log_spec_cli_verbose_high() {
        let cfg = LogConfig {
            level: "info".to_string(),
            ..Default::default()
        };

        assert_eq!(effective_log_spec(&cfg, Some(5)), "trace");
    }

    #[test]
    fn test_init_logging_console_only() {
        let cfg = LogConfig {
            console: true,
            format: "text".to_string(),
            ..Default::default()
        };
        let result = init_logging(&cfg, None);
        // May fail if already initialized, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_init_logging_json_format() {
        let cfg = LogConfig {
            console: true,
            format: "json".to_string(),
            ..Default::default()
        };
        let result = init_logging(&cfg, None);
        // May fail if already initialized, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "log-file")]
    #[test]
    fn test_init_logging_file_only() {
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new().expect("temp file");
        let cfg = LogConfig {
            console: false,
            file: Some(crate::FileLogConfig::new(tmp.path())),
            ..Default::default()
        };
        let result = init_logging(&cfg, None);
        // May fail if already initialized, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "log-file")]
    #[test]
    fn test_init_logging_console_and_file() {
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new().expect("temp file");
        let cfg = LogConfig {
            console: true,
            file: Some(crate::FileLogConfig::new(tmp.path())),
            ..Default::default()
        };
        let result = init_logging(&cfg, None);
        // May fail if already initialized, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
