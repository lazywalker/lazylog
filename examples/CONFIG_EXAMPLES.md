# Configuration File Examples

This directory contains example configuration files and programs demonstrating how to configure lazylog using YAML and TOML files.

## Configuration Files

### config.yaml
YAML format configuration file with text output and size-based rotation:
- Console logging enabled
- Log level: `info`
- Format: `text`
- File rotation: 10MB max size, keep 5 backups

### config.toml
TOML format configuration file with JSON output and daily rotation:
- Console logging enabled
- Log level: `debug`
- Format: `json` (structured output)
- File rotation: daily, keep 7 backups

## Example Programs

### config_yaml.rs
Basic example showing how to load and use a YAML configuration file.

**Run:**
```bash
cargo run --example config_yaml
```

### config_toml.rs
Basic example showing how to load and use a TOML configuration file.

**Run:**
```bash
cargo run --example config_toml
```

### config_advanced.rs
Advanced example with runtime configuration selection.

**Run:**
```bash
# Use YAML configuration
cargo run --example config_advanced -- yaml

# Use TOML configuration
cargo run --example config_advanced -- toml

# Use environment variable
LOG_CONFIG_FORMAT=toml cargo run --example config_advanced
```

## Configuration Options

All configuration files support the following options under the `log` section:

### Basic Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `console` | boolean | `false` | Enable console output |
| `level` | string | `"info"` | Log level: trace, debug, info, warn, error |
| `format` | string | `"text"` | Output format: text or json |
| `target` | boolean | `false` | Show module/target in logs |
| `thread_ids` | boolean | `false` | Show thread IDs |
| `thread_names` | boolean | `false` | Show thread names |

### File Logging

| Option | Type | Description |
|--------|------|-------------|
| `file.path` | string | Path to log file |
| `file.rotation` | object | Rotation configuration |

### Rotation Triggers

**Never (no rotation):**
```yaml
log:
  file:
    path: ./log/app.log
    rotation: never
```

**Size-based rotation:**
```yaml
log:
  file:
    path: ./log/app.log
    rotation:
      size:
        max_size: "10M"  # Supports: 1234 (bytes), "10K", "10M", "1G"
        max_backups: 5
```

**Daily rotation:**
```yaml
log:
  file:
    path: ./log/app.log
    rotation:
      daily:
        max_backups: 7
```

**Hourly rotation:**
```yaml
log:
  file:
    path: ./log/app.log
    rotation:
      hourly:
        max_backups: 24
```

## Usage in Your Application

### YAML Configuration

```rust
use std::fs;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config_content = fs::read_to_string("config.yaml")?;
    let root: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&config_content)?;
    let config: lazylog::LogConfig = serde_yaml::from_value(root["log"].clone())?;
    
    // Initialize logging
    lazylog::init_logging(&config)?;
    
    // Use logging
    tracing::info!("Application started");
    
    Ok(())
}
```

### TOML Configuration

```rust
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    log: lazylog::LogConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config_content = fs::read_to_string("config.toml")?;
    let root: Config = toml::from_str(&config_content)?;
    
    // Initialize logging
    lazylog::init_logging(&root.log)?;
    
    // Use logging
    tracing::info!("Application started");
    
    Ok(())
}
```

## Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
lazylog = "0.1"
tracing = "0.1"

# For YAML support
serde_yaml = "0.9"

# For TOML support
toml = "0.8"
```

## Output Examples

### Text Format (config.yaml)
```
2026-01-14T07:00:37.034892Z  INFO main config_advanced: === Logging initialized successfully ===
2026-01-14T07:00:37.034956Z  INFO main config_advanced: Info level message
2026-01-14T07:00:37.034970Z  WARN main config_advanced: Warning level message
```

### JSON Format (config.toml)
```json
{"timestamp":"2026-01-14T07:00:44.018679Z","level":"INFO","fields":{"message":"=== Logging initialized successfully ==="},"target":"config_advanced","threadName":"main","threadId":"ThreadId(1)"}
{"timestamp":"2026-01-14T07:00:44.018792Z","level":"DEBUG","fields":{"message":"Debug level message"},"target":"config_advanced","threadName":"main","threadId":"ThreadId(1)"}
```
