# AGENTS.md - Aurora Services

## Project Overview

Rust workspace for Aurora OS system services. Provides D-Bus and DConf backends for notifications, settings, and system configuration.

**Workspace Structure:**
- `aurora_services/` - Main library crate
- `aurora_services_demo/` - Demo GUI application using eframe/egui

**Target Platform:** Aurora OS (Sailfish-based Linux)

---

## Build Commands

```bash
# Build entire workspace
cargo build

# Build in release mode
cargo build --release

# Build specific crate
cargo build -p aurora_services
cargo build -p aurora_services_demo

# Cross-compile and build RPM packages (requires PSDK_DIR env)
./arm_build.sh      # armv7hl RPM
./aarch64_build.sh  # aarch64 RPM
```

---

## Test Commands

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p aurora_services

# Run single test
cargo test test_parse_string

# Run single test with exact name
cargo test --exact test_parse_int

# Run tests and show output
cargo test -- --nocapture

# Run tests in specific module
cargo test backends::dconf::tests
```

---

## Lint/Check Commands

```bash
# Check compilation without building
cargo check

# Run clippy linter
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all

# Format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check
```

---

## Code Style Guidelines

### Imports

Group imports in this order, separated by blank lines:
1. External crates (std, third-party)
2. Internal modules (crate::)

```rust
use crate::backends::{Bus, DBusBackend, MessageItem};
use crate::error::{AuroraError, Result};
use dbus::arg::messageitem::MessageItemArray;
use std::collections::HashMap;
```

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Crates | snake_case | `aurora_services` |
| Modules | snake_case | `mod notifications;` |
| Types (struct, enum) | PascalCase | `NotificationService`, `AuroraError` |
| Traits | PascalCase | `Into<DConfValue>` |
| Functions | snake_case | `get_sound_profile()` |
| Variables | snake_case | `active_ambience` |
| Constants | SCREAMING_SNAKE_CASE | `NOTIFICATIONS_SERVICE` |
| Static | SCREAMING_SNAKE_CASE | `DCONF_DB_PATH` |

### Constants

Define D-Bus service constants at module level:

```rust
const NOTIFICATIONS_SERVICE: &str = "org.freedesktop.Notifications";
const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";
```

### Error Handling

Use `thiserror` for error types. Return `Result<T, AuroraError>` from all public functions.

```rust
#[derive(Error, Debug)]
pub enum AuroraError {
    #[error("DBus error: {0}")]
    DBus(String),
    
    #[error("Invalid value type: expected {expected}, got {actual}")]
    InvalidType { expected: String, actual: String },
}

pub type Result<T> = std::result::Result<T, AuroraError>;
```

Use `.map_err()` to convert external errors:

```rust
Connection::new_system().map_err(|e| AuroraError::ConnectionFailed(e.to_string()))?;
```

### Default Implementations

Provide `Default` for types with sensible defaults:

```rust
impl Default for NotificationService {
    fn default() -> Self {
        Self::new().expect("Failed to create NotificationService")
    }
}
```

### Builder Pattern

Use builder pattern for complex configuration:

```rust
pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    pub fn new() -> Self { ... }
    pub fn app_name(mut self, name: &str) -> Self { ... }
    pub fn build(self) -> Notification { ... }
}
```

### Testing

Place tests in same file under `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_string() {
        let result = parse_dconf_value("'test'").unwrap();
        assert_eq!(result.as_string().unwrap(), "test");
    }
}
```

---

## Architecture

```
aurora_services/src/
├── lib.rs              # Public API, re-exports
├── error.rs            # Error types with thiserror
├── backends/
│   ├── mod.rs
│   ├── dbus.rs         # DBus client (system/session bus)
│   └── dconf.rs        # DConf settings reader
└── services/
    ├── mod.rs
    ├── notifications.rs # NotificationService
    └── settings.rs      # SettingsService (theme, display, sound)
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `dbus` | D-Bus IPC |
| `serde` | Serialization (derive) |
| `thiserror` | Error derive |
| `eframe` | GUI framework (demo only) |

---

## D-Bus Services Used

| Service | Purpose |
|---------|---------|
| `org.freedesktop.Notifications` | Desktop notifications |
| `com.nokia.profiled` | Sound profiles |

---

## DConf Paths

| Path | Keys |
|------|------|
| `/desktop/sailfish/silica` | Font settings, pixel ratio |
| `/desktop/jolla/theme` | Theme configuration |
| `/desktop/jolla/theme/color` | Color scheme |
| `/lipstick` | Orientation lock |
| `/jolla/sound` | Sound theme |

---

## Notes

- DBus connections are wrapped in `Mutex` for thread safety
- DConf backend caches values in memory
- Demo uses egui for UI
- Cross-compilation requires `PSDK_DIR` environment variable pointing to Aurora PSDK installation
- Cross-compilation requires `libdbus-1-dev` on target architecture
