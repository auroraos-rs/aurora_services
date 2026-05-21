# aurora_services

Aurora OS platform service wrappers.

Provides Rust-friendly access to DConf settings, DBus notifications, and device info via the Aurora OS system bus. All services gracefully fall back to sensible defaults when DConf or DBus is unavailable (e.g. on desktop Linux during development).

## Modules

- **`backends::dbus`** — Blocking DBus client with pooled system + session connections.
- **`backends::dconf`** — Parser for Sailfish/Aurora DConf `.txt` files. Caches per-path.
- **`services::notifications`** — Freedesktop Notifications service backed by `notify-rust` (dbus 0.9). Provides a builder pattern with Aurora OS-specific hints (preview summary/body, sound presets, etc.).
- **`services::settings`** — Reads display and font settings from DConf (`/desktop/sailfish/silica`).
- **`services::device_info`** — DBus proxy to `ru.omp.deviceinfo.Features`.

## Usage

### Settings

```rust
use aurora_services::SettingsService;

let settings = SettingsService::new()?;

// Display settings
let pixel_ratio = settings.pixel_ratio();          // e.g. 1.5
let statusbar_height = settings.statusbar_height();  // e.g. 41

// Font settings
let fonts = settings.font_settings();
println!("Body font: {} at {}px", fonts.family, fonts.size_medium);
```

### Notifications

```rust
use aurora_services::notifications::{NotificationService, NotificationBuilder, Sound};

let service = NotificationService::new()?;
let id = NotificationBuilder::new()
    .app_name("My App")
    .summary("Hello")
    .body("This is a test notification")
    .icon("icon-s-status-message")
    .urgency(1)                       // 0=Low, 1=Normal, 2=Critical
    .desktop_entry("my.app")          // maps to desktop-entry hint
    .category("x-nemo.call.missed")   // notification category
    .sound(Sound::Message)            // predefined Aurora OS sound
    .preview_summary("Preview")       // x-nemo-preview-summary
    .preview_body("Preview body")     // x-nemo-preview-body
    .build();
let id = service.notify(&id)?;
```

**Aurora OS Quirks:**
- `urgency` is sent as `int32`, not `byte`. Newer `lipstick` rejects byte-typed urgency.
- `x-nemo-preview-summary` and `x-nemo-preview-body` are auto-filled from `summary`/`body` if not explicitly set.
- `expire_timeout` defaults to `-1` (server default). Use `timeout(0)` for never-expire.

**Closing & Querying:**

```rust
service.close(id)?;
let caps = service.get_capabilities()?;
let info = service.get_server_info()?;
```

### Device Info

```rust
use aurora_services::device_info::DeviceInfoService;

let info = DeviceInfoService::new()?;
let features = info.get_features()?;
println!("Screen: {}x{}", features.screen_width, features.screen_height);
```

## Key Types

### `NotificationBuilder`

| Method | Hint | Notes |
|--------|------|-------|
| `app_name()` | — | Defaults to executable name |
| `summary()` / `body()` | — | Primary text content |
| `icon()` | `app_icon` | Icon name or path |
| `urgency(u8)` | `urgency` | Sent as **int32** for Aurora OS compatibility |
| `desktop_entry()` | `desktop-entry` | e.g. `"com.example.app"` |
| `category()` | `category` | e.g. `"x-nemo.call.missed"` |
| `sound(Sound)` | `sound-file` | Predefined Aurora OS sound paths |
| `sound_file(path)` | `sound-file` | Custom sound file path |
| `suppress_sound(bool)` | `suppress-sound` | Mute notification sound |
| `position(x, y)` | `x`, `y` | Screen coordinates |
| `preview_summary()` | `x-nemo-preview-summary` | Auto-filled from `summary` if unset |
| `preview_body()` | `x-nemo-preview-body` | Auto-filled from `body` if unset |
| `hint(key, VariantValue)` | arbitrary | For custom hints |

### `SettingsService`

| Method | Returns | Fallback |
|--------|---------|----------|
| `pixel_ratio()` | `f64` | `1.5` |
| `statusbar_height()` | `i32` | `41` |
| `font_settings()` | `FontSettings` | ALS Hauss Variable defaults |

### `FontSettings`

```rust
pub struct FontSettings {
    pub family: String,
    pub family_heading: String,
    pub size_tiny: i32,
    pub size_extra_small: i32,
    pub size_small: i32,
    pub size_medium: i32,
    pub size_large: i32,
    pub size_extra_large: i32,
    pub size_huge: i32,
}
```

### `DConfBackend`

Direct DConf access when you need keys not covered by `SettingsService`:

```rust
use aurora_services::backends::DConfBackend;

let mut dconf = DConfBackend::new();
let value = dconf.get("/desktop/sailfish/silica", "theme_pixel_ratio")?;
let ratio = value.as_double()?;
```

## License

Apache 2.0
