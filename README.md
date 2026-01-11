# Lazylog

A flexible logging library with file rotation and structured output for Rust applications.

## Features

- **Console Logging**: Output logs to stdout/stderr with customizable formatting
- **File Logging**: Write logs to files with automatic rotation
- **Structured Logging**: Support for JSON output format
- **Log Rotation**: Rotate logs based on size, time, or both
- **Tracing Integration**: Built on top of the `tracing` ecosystem

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lazylog = "0.1"
```

Optional features:
- `file`: Enable file logging support
- `ansi`: Enable ANSI color codes in console output
- `time`: Enable time-based log rotation

## Quick Start

```rust
use lazylog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    lazylog::builder()
        .with_console(true)
        .with_level("info")
        .init()?;

    tracing::info!("Application started");
    Ok(())
}
```

## Configuration

### Basic Usage

```rust
// Console logging
lazylog::builder()
    .with_console(true)
    .with_level("debug")
    .init()?;

// File logging with size rotation
lazylog::builder()
    .with_file("app.log")
    .with_rotation(lazylog::RotationTrigger::size(1024 * 1024, 5)) // 1MB, keep 5 files
    .init()?;

// JSON format
lazylog::builder()
    .with_console(true)
    .with_format("json")
    .init()?;
```

### Log Rotation

```rust
use lazylog::{RotationTrigger, RotationPeriod};

// Size-based rotation
RotationTrigger::size(10 * 1024 * 1024, 5) // 10MB, keep 5 files

// Time-based rotation (requires `time` feature)
RotationTrigger::time(RotationPeriod::Daily)

// Hybrid rotation
RotationTrigger::both(RotationPeriod::Daily, 10 * 1024 * 1024, 5)
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
      size: 10M
```

## Examples

### Basic Logging

```rust
lazylog::builder()
    .with_console(true)
    .with_level("info")
    .init()?;

tracing::info!("Hello, world!");
tracing::error!("Something went wrong");
```

### File Logging

```rust
lazylog::builder()
    .with_file("app.log")
    .with_rotation(RotationTrigger::size(1024 * 1024, 3))
    .init()?;

tracing::info!("This will be logged to file");
```

### Structured Logging

```rust
lazylog::builder()
    .with_console(true)
    .with_format("json")
    .init()?;

tracing::info!(user_id = 123, action = "login", "User logged in");
```

## API Reference

### Builder API

- `lazylog::builder()` - Create a new builder
- `with_console(bool)` - Enable console logging
- `with_level(&str)` - Set log level
- `with_format(&str)` - Set format ("text" or "json")
- `with_file(path)` - Enable file logging
- `with_rotation(RotationTrigger)` - Set rotation
- `init()` - Initialize logging

### RotationTrigger

- `size(max_size: u64, max_files: usize)` - Size-based rotation
- `time(period: RotationPeriod)` - Time-based rotation
- `both(period, max_size, max_files)` - Hybrid rotation

### RotationPeriod

- `Hourly`, `Daily`, `Weekly`, `Monthly`

## Testing

```bash
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.