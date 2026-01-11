# Lazylog

A flexible logging library with file rotation and structured output for Rust applications.

## Features

- **Console Logging**: Output logs to stdout/stderr with customizable formatting
- **File Logging**: Write logs to files with automatic rotation
- **Structured Logging**: Support for JSON output format
- **Log Rotation**: Rotate logs based on size, time, or both
- **Tracing Integration**: Built on top of the `tracing` ecosystem
- **Async Support**: Non-blocking file I/O using `tracing-appender`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lazylog = "0.1"
```

For tracing integration (recommended):

```toml
[dependencies]
lazylog = { version = "0.1", features = ["tracing-integration"] }
```

Optional features:

- `log-file`: Enable file logging support
- `log-ansi`: Enable ANSI color codes in console output
- `time`: Enable time-based log rotation (requires local timezone support)

## Quick Start

```rust
use lazylog::{init_logging, LogConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure console logging
    let config = LogConfig::new().with_console(true);

    // Initialize logging
    init_logging(&config)?;

    // Log some messages
    tracing::info!("Application started");
    tracing::warn!("This is a warning");
    tracing::error!("This is an error");

    Ok(())
}
```

## Configuration

### Basic Configuration

```rust
use lazylog::{LogConfig, FileLogConfig, RotationTrigger};

let config = LogConfig::new()
    .with_console(true)
    .with_level("debug".to_string())
    .with_format("json".to_string())
    .with_file(
        FileLogConfig::new("/var/log/myapp.log")
            .with_rotation_trigger(RotationTrigger::size(10 * 1024 * 1024)) // 10MB
    );
```

### YAML Configuration

```yaml
log:
  console: true
  level: info
  format: text
  file:
    path: /var/log/app.log
    rotation:
      size: 10485760  # 10MB
```

### Log Rotation

Configure automatic log rotation:

```rust
use lazylog::{RotationTrigger, RotationPeriod};

// Rotate every 10MB
let size_trigger = RotationTrigger::size(10 * 1024 * 1024);

// Rotate daily
let time_trigger = RotationTrigger::time(RotationPeriod::Day);

// Rotate when either condition is met
let hybrid_trigger = RotationTrigger::both(10 * 1024 * 1024, 86400); // 10MB or 24 hours
```

## Examples

### Basic Console Logging

```rust
use lazylog::{init_logging, LogConfig};

let config = LogConfig::new().with_console(true);
init_logging(&config)?;

tracing::info!("Hello, world!");
```

### File Logging with Rotation

```rust
use lazylog::{init_logging, LogConfig, FileLogConfig, RotationTrigger};

let file_config = FileLogConfig::new("app.log")
    .with_rotation_trigger(RotationTrigger::size(1024 * 1024)); // 1MB

let config = LogConfig::new()
    .with_console(true)
    .with_file(file_config);

init_logging(&config)?;
```

### JSON Structured Logging

```rust
use lazylog::{init_logging, LogConfig};

let config = LogConfig::new()
    .with_console(true)
    .with_format("json".to_string());

init_logging(&config)?;

tracing::info!(user_id = 123, action = "login", "User logged in successfully");
```

## API Reference

### LogConfig

Main configuration struct for logging.

- `new()` - Create a new configuration with defaults
- `with_console(bool)` - Enable/disable console logging
- `with_level(String)` - Set log level (trace, debug, info, warn, error)
- `with_format(String)` - Set output format (text or json)
- `with_file(FileLogConfig)` - Configure file logging

### FileLogConfig

Configuration for file logging.

- `new(path)` - Create with a file path
- `with_rotation_trigger(RotationTrigger)` - Set rotation trigger

### RotationTrigger

Defines when to rotate log files.

- `Never` - No rotation
- `Size { max_size, max_files }` - Rotate when file exceeds size, keep max_files
- `Time { period }` - Rotate based on time intervals (requires `time` feature)
- `Both { period, max_size, max_files }` - Rotate on either condition (requires `time` feature)

Constructors:
- `size(max_size: u64, max_files: usize)` - Create size-based trigger
- `time(period: RotationPeriod)` - Create time-based trigger (requires `time` feature)
- `both(period: RotationPeriod, max_size: u64, max_files: usize)` - Create hybrid trigger (requires `time` feature)

### RotationPeriod

Time-based rotation periods (requires `time` feature).

- `Never` - No time-based rotation
- `Hourly` - Rotate every hour
- `Daily` - Rotate every day
- `Weekly` - Rotate every week
- `Monthly` - Rotate every month

at your option.

## Testing

Run the test suite:

```bash
make test
```

Run specific test file:

```bash
cargo test --features log-file --test logging_file_tests
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.