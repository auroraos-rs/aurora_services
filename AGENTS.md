# Agent Notes

## Project Overview

`aurora_services` is a Rust library providing Aurora OS platform service wrappers.

Provides Rust-friendly access to DConf settings, DBus notifications, and device info via the Aurora OS system bus. All services gracefully fall back to sensible defaults when DConf or DBus is unavailable (e.g. on desktop Linux during development).

**Target Platform:** Aurora OS (Sailfish-based Linux)

---

## Build Commands

```bash
# Build library
cargo build

# Build in release mode
cargo build --release

# Cross-compile for Aurora OS devices
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target armv7-unknown-linux-gnueabihf
```

Requires `libdbus-1-dev` on the target architecture (configured in `Cross.toml` for the consuming GUI workspace).

---

## Test Commands

```bash
# Run all tests
cargo test

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
| Modules | snake_case | `mod notifications;` |
| Types (struct, enum) | PascalCase | `NotificationService`, `AuroraError` |
| Traits | PascalCase | `Into<DConfValue>` |
| Functions | snake_case | `get_sound_profile()` |
| Variables | snake_case | `active_ambience` |
| Constants | SCREAMING_SNAKE_CASE | `NOTIFICATIONS_SERVICE` |

### Constants

Define D-Bus service constants at module level:

```rust
const NOTIFICATIONS_SERVICE: &str = "org.freedesktop.Notifications";
const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";
```

### Error Handling

Use `thiserror` for error types. Return `Result<T>` from all public functions.

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
src/
├── lib.rs              # Public API, re-exports
├── error.rs            # Error types with thiserror
├── backends/
│   ├── mod.rs
│   ├── dbus.rs         # DBus client (system/session bus)
│   └── dconf.rs        # DConf settings reader
└── services/
    ├── mod.rs
    ├── device_info/    # DeviceInfoService (feature detection)
    ├── notifications.rs # NotificationService (notify-rust backed)
    ├── package_info.rs  # Package/runtime dir helpers
    └── settings.rs      # SettingsService (theme, display, fonts)
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `dbus` | D-Bus IPC |
| `notify-rust` | Freedesktop Notifications wire protocol (dbus 0.9 backend) |
| `serde` | Serialization (derive) |
| `thiserror` | Error derive |

---

## D-Bus Services Used

| Service | Purpose |
|---------|---------|
| `org.freedesktop.Notifications` | Desktop notifications |
| `ru.omp.deviceinfo.Features` | Device feature detection |

---

## DConf Paths

| Path | Keys |
|------|------|
| `/desktop/sailfish/silica` | Font settings, pixel ratio |

---

## Notes

- DBus connections are wrapped in `Mutex` for thread safety
- DConf backend caches values in memory
- `notify-rust` sends `urgency` as `byte` by default; we use `Hint::CustomInt("urgency", level)` for Aurora OS compatibility
- Cross-compilation requires `libdbus-1-dev` on target architecture
