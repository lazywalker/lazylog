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

- `file`: Enable file logging support
- `ansi`: Enable ANSI color codes in console output
- `time`: Enable time-based log rotation (requires local timezone support)

## Quick Start

### Using the Builder API (Recommended)

The simplest way to initialize logging:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple console logging
    lazylog::builder()
        .with_console(true)
        .with_level("info")
        .init()?;

    tracing::info!("Application started");
    tracing::warn!("This is a warning");
    tracing::error!("This is an error");

    Ok(())
}
```

### Using Configuration Objects

For more complex setups or when loading from config files:

```rust
use lazylog::LogConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LogConfig::new().with_console(true);
    
    // Option 1: Use the builder with config
    lazylog::from_config(config).init()?;
    
    // Option 2: Use the traditional init_logging function
    lazylog::init_logging(&config, None)?;

    tracing::info!("Application started");

    Ok(())
}
```

## Configuration

### Builder API Examples

The builder API provides a fluent interface for configuration:

```rust
use lazylog::{RotationTrigger, RotationPeriod};

// Console logging with custom level
lazylog::builder()
    .with_console(true)
    .with_level("debug")
    .init()?;

// JSON format logging
lazylog::builder()
    .with_console(true)
    .with_format("json")
    .with_level("info")
    .init()?;

// File logging with rotation
lazylog::builder()
    .with_console(true)
    .with_file("/var/log/myapp.log")
    .with_rotation(RotationTrigger::size(10 * 1024 * 1024, 5)) // 10MB, keep 5 files
    .init()?;

// Time-based rotation
lazylog::builder()
    .with_file("/var/log/myapp.log")
    .with_rotation(RotationTrigger::time(RotationPeriod::Daily))
    .init()?;

// Hybrid rotation (size or time)
lazylog::builder()
    .with_file("/var/log/myapp.log")
    .with_rotation(RotationTrigger::both(
        RotationPeriod::Daily,
        10 * 1024 * 1024, // 10MB
        5                  // keep 5 files
    ))
    .init()?;
```

### Configuration Object API

For advanced use cases or loading from config files:

```rust
use lazylog::{LogConfig, FileLogConfig, RotationTrigger};

let config = LogConfig::new()
    .with_console(true)
    .with_level("debug".to_string())
    .with_format("json".to_string())
    .with_file(
        FileLogConfig::new("/var/log/myapp.log")
            .with_rotation_trigger(RotationTrigger::size(10 * 1024 * 1024, 5))
    );

// Initialize with the config
lazylog::from_config(config).init()?;
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

Configure automatic log rotation using the builder or config objects:

```rust
use lazylog::{RotationTrigger, RotationPeriod};

// Size-based: Rotate when file reaches 10MB, keep 5 files
lazylog::builder()
    .with_file("app.log")
    .with_rotation(RotationTrigger::size(10 * 1024 * 1024, 5))
    .init()?;

// Time-based: Rotate daily (requires `time` feature)
lazylog::builder()
    .with_file("app.log")
    .with_rotation(RotationTrigger::time(RotationPeriod::Daily))
    .init()?;

// Hybrid: Rotate when either condition is met (requires `time` feature)
lazylog::builder()
    .with_file("app.log")
    .with_rotation(RotationTrigger::both(
        RotationPeriod::Daily,    // Rotate daily
        10 * 1024 * 1024,         // OR when file reaches 10MB
        5                          // Keep 5 old files
    ))
    .init()?;
```

## Examples

### Basic Console Logging

```rust
lazylog::builder()
    .with_console(true)
    .with_level("info")
    .init()?;

tracing::info!("Hello, world!");
```

### File Logging with Rotation

```rust
use lazylog::RotationTrigger;

lazylog::builder()
    .with_console(true)
    .with_file("app.log")
    .with_rotation(RotationTrigger::size(1024 * 1024, 3)) // 1MB, keep 3 files
    .init()?;

tracing::info!("Logged to file with rotation");
```

### JSON Structured Logging

```rust
lazylog::builder()
    .with_console(true)
    .with_format("json")
    .init()?;

tracing::info!(user_id = 123, action = "login", "User logged in successfully");
```

### Advanced: Custom File Configuration

```rust
use lazylog::{FileLogConfig, RotationTrigger, RotationPeriod};

let file_config = FileLogConfig::new("app.log")
    .with_rotation_trigger(RotationTrigger::both(
        RotationPeriod::Hourly,
        5 * 1024 * 1024,
        10
    ));

lazylog::builder()
    .with_console(true)
    .with_file_config(file_config)
    .init()?;
```

## API Reference

### Builder API

#### `lazylog::builder()`

Create a new logging configuration builder.

**Methods:**
- `with_console(bool)` - Enable/disable console logging
- `with_level(impl Into<String>)` - Set log level ("trace", "debug", "info", "warn", "error")
- `with_format(impl Into<String>)` - Set output format ("text" or "json")
- `with_file(impl Into<PathBuf>)` - Configure file logging with a path
- `with_file_config(FileLogConfig)` - Configure file logging with a custom config
- `with_rotation(RotationTrigger)` - Set rotation trigger for file logging
- `with_cli_verbose(u8)` - Set CLI verbosity level override
- `init()` - Initialize logging (consumes the builder)
- `build()` - Get the configuration without initializing

**Example:**
```rust
lazylog::builder()
    .with_console(true)
    .with_level("debug")
    .with_file("/var/log/app.log")
    .with_rotation(RotationTrigger::size(10 * 1024 * 1024, 5))
    .init()?;
```

#### `lazylog::from_config(config)`

Create a builder from an existing configuration.

**Example:**
```rust
let config = LogConfig::new().with_console(true);
lazylog::from_config(config)
    .with_level("debug")
    .init()?;
```

### LogConfig

Main configuration struct for logging.

**Constructors:**
- `new()` - Create a new configuration with defaults
- `default()` - Same as `new()`

**Methods:**
- `with_console(bool)` - Enable/disable console logging
- `with_level(String)` - Set log level (trace, debug, info, warn, error)
- `with_format(String)` - Set output format (text or json)
- `with_file(FileLogConfig)` - Configure file logging

### FileLogConfig

Configuration for file logging.

**Constructors:**
- `new(path)` - Create with a file path

**Methods:**
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
cargo test --features file --test logging_file_tests
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.