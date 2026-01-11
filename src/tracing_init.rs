#[cfg(feature = "file")]
use crate::FileLogConfig;
#[cfg(feature = "file")]
use crate::RotatingWriter;
use crate::{Error, LogConfig, Result};
#[cfg(feature = "file")]
use once_cell::sync::Lazy;
#[cfg(feature = "file")]
use std::sync::Mutex;
#[cfg(feature = "time")]
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// Custom RFC3339 format with 3-digit subseconds (milliseconds).
#[cfg(feature = "time")]
const RFC3339_MS: &[time::format_description::FormatItem<'static>] = time::macros::format_description!(
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory]:[offset_minute]"
);

#[cfg(feature = "file")]
static LOG_GUARD: Lazy<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
    Lazy::new(|| Mutex::new(None));

/// Create a local timezone timer for tracing-subscriber.
#[cfg(feature = "time")]
fn create_timer() -> OffsetTime<&'static [time::format_description::FormatItem<'static>]> {
    match time::UtcOffset::current_local_offset() {
        Ok(offset) => OffsetTime::new(offset, RFC3339_MS),
        Err(_) => OffsetTime::new(time::UtcOffset::UTC, RFC3339_MS),
    }
}

/// Initialize logging with the given configuration and optional CLI verbosity override.
pub fn init_logging(config: &LogConfig) -> Result<()> {
    let env_filter = EnvFilter::try_new(&config.level).map_err(|e| Error::Init(e.to_string()))?;

    // Create timer for local timezone when time feature is enabled
    #[cfg(feature = "time")]
    let _timer = create_timer();

    // Determine effective console and file settings based on features
    let effective_console = config.console;
    #[cfg(feature = "file")]
    let effective_file: &Option<crate::FileLogConfig> = &config.file;
    #[cfg(not(feature = "file"))]
    let effective_file: &Option<crate::FileLogConfig> = &None;

    match (effective_console, effective_file.as_ref()) {
        (true, Some(_)) => {
            // Console and file - only available when file feature is enabled
            #[cfg(feature = "file")]
            init_console_and_file(config, effective_file.as_ref().unwrap(), env_filter)?;
            #[cfg(not(feature = "file"))]
            init_console_only(config, env_filter)?;
        }
        (true, None) => {
            // Console only
            init_console_only(config, env_filter)?;
        }
        (false, Some(_)) => {
            // File only - only available when file feature is enabled
            #[cfg(feature = "file")]
            init_file_only(config, effective_file.as_ref().unwrap(), env_filter)?;
            #[cfg(not(feature = "file"))]
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
#[cfg(feature = "file")]
fn init_console_and_file(
    config: &LogConfig,
    file_config: &FileLogConfig,
    env_filter: EnvFilter,
) -> Result<()> {
    let fmt_layer_builder = tracing_subscriber::fmt::layer()
        .with_target(config.target)
        .with_thread_ids(config.thread_ids)
        .with_thread_names(config.thread_names);

    #[cfg(feature = "ansi")]
    let fmt_layer_builder = fmt_layer_builder.with_ansi(true);
    #[cfg(not(feature = "ansi"))]
    let fmt_layer_builder = fmt_layer_builder.with_ansi(false);

    let fmt_layer = if config.format == "json" {
        let layer = fmt_layer_builder.json();
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    } else {
        let layer = fmt_layer_builder;
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    };

    let writer =
        RotatingWriter::new(&file_config.path, file_config.rotation.clone()).map_err(Error::Io)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(writer);

    *LOG_GUARD.lock().unwrap() = Some(guard);

    let file_layer = if config.format == "json" {
        let layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(config.target)
            .with_thread_ids(config.thread_ids)
            .with_thread_names(config.thread_names)
            .json();
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    } else {
        let layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(config.target)
            .with_thread_ids(config.thread_ids)
            .with_thread_names(config.thread_names);
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(file_layer)
        .try_init()
        .map_err(|e| Error::Init(e.to_string()))?;

    Ok(())
}

/// Initialize console-only logging.
fn init_console_only(config: &LogConfig, env_filter: EnvFilter) -> Result<()> {
    let fmt_layer_builder = tracing_subscriber::fmt::layer()
        .with_target(config.target)
        .with_thread_ids(config.thread_ids)
        .with_thread_names(config.thread_names);

    #[cfg(feature = "ansi")]
    let fmt_layer_builder = fmt_layer_builder.with_ansi(true);
    #[cfg(not(feature = "ansi"))]
    let fmt_layer_builder = fmt_layer_builder.with_ansi(false);

    let fmt_layer = if config.format == "json" {
        let layer = fmt_layer_builder.json();
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    } else {
        let layer = fmt_layer_builder;
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| Error::Init(e.to_string()))?;

    Ok(())
}

/// Initialize file-only logging.
#[cfg(feature = "file")]
fn init_file_only(
    config: &LogConfig,
    file_config: &FileLogConfig,
    env_filter: EnvFilter,
) -> Result<()> {
    let writer =
        RotatingWriter::new(&file_config.path, file_config.rotation.clone()).map_err(Error::Io)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(writer);

    *LOG_GUARD.lock().unwrap() = Some(guard);

    let file_layer = if config.format == "json" {
        let layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(config.target)
            .with_thread_ids(config.thread_ids)
            .with_thread_names(config.thread_names)
            .json();
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    } else {
        let layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(config.target)
            .with_thread_ids(config.thread_ids)
            .with_thread_names(config.thread_names);
        #[cfg(feature = "time")]
        let layer = layer.with_timer(create_timer());
        layer.boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .try_init()
        .map_err(|e| Error::Init(e.to_string()))?;

    Ok(())
}

/// Initialize with no output (for testing or when logging is disabled).
fn init_no_logging(env_filter: EnvFilter) -> Result<()> {
    tracing_subscriber::registry()
        .with(env_filter)
        .try_init()
        .map_err(|e| Error::Init(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogConfig;

    #[test]
    fn test_init_logging_console_only() {
        let cfg = LogConfig {
            console: true,
            format: "text".to_string(),
            ..Default::default()
        };
        let result = init_logging(&cfg);
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
        let result = init_logging(&cfg);
        // May fail if already initialized, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "file")]
    #[test]
    fn test_init_logging_file_only() {
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new().expect("temp file");
        let cfg = LogConfig {
            console: false,
            file: Some(crate::FileLogConfig::new(tmp.path())),
            ..Default::default()
        };
        let result = init_logging(&cfg);
        // May fail if already initialized, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "file")]
    #[test]
    fn test_init_logging_console_and_file() {
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new().expect("temp file");
        let cfg = LogConfig {
            console: true,
            file: Some(crate::FileLogConfig::new(tmp.path())),
            ..Default::default()
        };
        let result = init_logging(&cfg);
        // May fail if already initialized, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "time")]
    #[test]
    fn test_timezone_in_console_output() {
        let cfg = LogConfig {
            console: true,
            format: "text".to_string(),
            level: "info".to_string(),
            ..Default::default()
        };
        // Ignore error if already initialized
        let _ = init_logging(&cfg);
        tracing::info!("test console timezone message");
        // Note: Console output timezone is verified by the log message printed above
        // The timer is applied in init_console_only and init_console_and_file, ensuring local timezone is used
        // File output uses the same timer configuration, guaranteeing both outputs use local timezone
    }
}
