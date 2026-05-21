use crate::backends::{Bus, DBusBackend, MessageItem};
use crate::error::{AuroraError, Result};
use std::collections::HashMap;

const NOTIFICATIONS_SERVICE: &str = "org.freedesktop.Notifications";
const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub hints: HashMap<String, VariantValue>,
    pub expire_timeout: i32,
    pub preview_summary: Option<String>,
    pub preview_body: Option<String>,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            app_name: super::package_info::package_name().unwrap_or_default(),
            replaces_id: 0,
            app_icon: String::new(),
            summary: String::new(),
            body: String::new(),
            actions: Vec::new(),
            hints: HashMap::new(),
            expire_timeout: -1,
            preview_summary: None,
            preview_body: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VariantValue {
    Int(i32),
    UInt(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    String(String),
    Bool(bool),
    Byte(u8),
    ByteArray(Vec<u8>),
}

/// Predefined notification sounds available on Aurora OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sound {
    Bell,
    Complete,
    DialogError,
    DialogInformation,
    DialogWarning,
    MessageNewInstant,
    Message,
}

impl Sound {
    pub fn path(&self) -> &'static str {
        match self {
            Sound::Bell => "/usr/share/sounds/freedesktop/stereo/bell.oga",
            Sound::Complete => "/usr/share/sounds/freedesktop/stereo/complete.oga",
            Sound::DialogError => "/usr/share/sounds/freedesktop/stereo/dialog-warning.oga",
            Sound::DialogInformation => {
                "/usr/share/sounds/freedesktop/stereo/dialog-information.oga"
            }
            Sound::DialogWarning => "/usr/share/sounds/freedesktop/stereo/dialog-warning.oga",
            Sound::MessageNewInstant => {
                "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga"
            }
            Sound::Message => "/usr/share/sounds/freedesktop/stereo/message.oga",
        }
    }
}

pub struct NotificationService {
    dbus: DBusBackend,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new().expect("Failed to create NotificationService")
    }
}

impl NotificationService {
    pub fn new() -> Result<Self> {
        Ok(Self::with_backend(DBusBackend::new()?))
    }

    pub fn with_backend(dbus: DBusBackend) -> Self {
        Self { dbus }
    }

    pub fn notify(&self, notification: &Notification) -> Result<u32> {
        let mut n = notify_rust::Notification::new();
        n.appname(&notification.app_name)
            .summary(&notification.summary)
            .body(&notification.body)
            .icon(&notification.app_icon);

        if notification.replaces_id != 0 {
            n.id(notification.replaces_id);
        }

        n.timeout(match notification.expire_timeout {
            -1 => notify_rust::Timeout::Default,
            0 => notify_rust::Timeout::Never,
            ms => notify_rust::Timeout::Milliseconds(ms.max(0) as u32),
        });

        // Actions are stored as a flat list of pairs (identifier, label).
        // Push them directly into notify-rust's actions vec.
        for action in &notification.actions {
            n.actions.push(action.clone());
        }

        // Map hints to notify-rust's Hint enum.
        for (key, value) in &notification.hints {
            match (key.as_str(), value) {
                ("desktop-entry", VariantValue::String(v)) => {
                    n.hint(notify_rust::Hint::DesktopEntry(v.clone()));
                }
                ("category", VariantValue::String(v)) => {
                    n.hint(notify_rust::Hint::Category(v.clone()));
                }
                ("suppress-sound", VariantValue::Bool(v)) => {
                    n.hint(notify_rust::Hint::SuppressSound(*v));
                }
                ("sound-file", VariantValue::String(v)) => {
                    n.hint(notify_rust::Hint::SoundFile(v.clone()));
                }
                ("x", VariantValue::Int(v)) => {
                    n.hint(notify_rust::Hint::X(*v));
                }
                ("y", VariantValue::Int(v)) => {
                    n.hint(notify_rust::Hint::Y(*v));
                }
                ("urgency", VariantValue::Int(v)) => {
                    // Workaround: notify-rust sends urgency as a D-Bus byte,
                    // but Aurora OS lipstick requires int32. Use CustomInt
                    // to force the correct wire type.
                    n.hint(notify_rust::Hint::CustomInt("urgency".to_string(), *v));
                }
                (_, VariantValue::String(v)) => {
                    n.hint(notify_rust::Hint::Custom(key.clone(), v.clone()));
                }
                (_, VariantValue::Int(v)) => {
                    n.hint(notify_rust::Hint::CustomInt(key.clone(), *v));
                }
                (_, VariantValue::UInt(v)) => {
                    n.hint(notify_rust::Hint::CustomInt(key.clone(), *v as i32));
                }
                (_, VariantValue::Byte(v)) => {
                    n.hint(notify_rust::Hint::CustomInt(key.clone(), *v as i32));
                }
                (_, VariantValue::Bool(v)) => {
                    n.hint(notify_rust::Hint::CustomInt(
                        key.clone(),
                        if *v { 1 } else { 0 },
                    ));
                }
                // Int64, UInt64, Double, and ByteArray cannot be represented
                // via notify-rust's custom hint API and are silently skipped.
                _ => {}
            }
        }

        // Auto-fill Aurora OS preview hints.
        if let Some(preview) = notification
            .preview_summary
            .as_ref()
            .cloned()
            .or_else(|| Some(notification.summary.clone()))
        {
            if !preview.is_empty() {
                n.hint(notify_rust::Hint::Custom(
                    "x-nemo-preview-summary".to_string(),
                    preview,
                ));
            }
        }
        if let Some(preview) = notification
            .preview_body
            .as_ref()
            .cloned()
            .or_else(|| Some(notification.body.clone()))
        {
            if !preview.is_empty() {
                n.hint(notify_rust::Hint::Custom(
                    "x-nemo-preview-body".to_string(),
                    preview,
                ));
            }
        }

        let handle = n.show().map_err(|e| AuroraError::DBus(e.to_string()))?;
        Ok(handle.id())
    }

    pub fn close(&self, id: u32) -> Result<()> {
        self.dbus.call_method(
            Bus::Session,
            NOTIFICATIONS_SERVICE,
            NOTIFICATIONS_PATH,
            NOTIFICATIONS_INTERFACE,
            "CloseNotification",
            &[MessageItem::UInt32(id)],
        )?;

        Ok(())
    }

    pub fn get_capabilities(&self) -> Result<Vec<String>> {
        notify_rust::get_capabilities()
            .map_err(|e| AuroraError::DBus(e.to_string()))
    }

    pub fn get_server_info(&self) -> Result<ServerInfo> {
        let info = notify_rust::get_server_information()
            .map_err(|e| AuroraError::DBus(e.to_string()))?;

        Ok(ServerInfo {
            name: info.name,
            vendor: info.vendor,
            version: info.version,
            spec_version: info.spec_version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub spec_version: String,
}

pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    pub fn new() -> Self {
        Self {
            notification: Notification::default(),
        }
    }

    pub fn app_name(mut self, name: &str) -> Self {
        self.notification.app_name = name.to_string();
        self
    }

    pub fn summary(mut self, summary: &str) -> Self {
        self.notification.summary = summary.to_string();
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.notification.body = body.to_string();
        self
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.notification.app_icon = icon.to_string();
        self
    }

    pub fn timeout(mut self, timeout_ms: i32) -> Self {
        self.notification.expire_timeout = timeout_ms;
        self
    }

    pub fn replaces(mut self, id: u32) -> Self {
        self.notification.replaces_id = id;
        self
    }

    pub fn action(mut self, action: &str) -> Self {
        self.notification.actions.push(action.to_string());
        self
    }

    pub fn hint(mut self, key: &str, value: VariantValue) -> Self {
        self.notification.hints.insert(key.to_string(), value);
        self
    }

    pub fn desktop_entry(mut self, entry: &str) -> Self {
        self.notification.hints.insert(
            "desktop-entry".to_string(),
            VariantValue::String(entry.to_string()),
        );
        self
    }

    pub fn category(mut self, category: &str) -> Self {
        self.notification.hints.insert(
            "category".to_string(),
            VariantValue::String(category.to_string()),
        );
        self
    }

    pub fn suppress_sound(mut self, suppress: bool) -> Self {
        self.notification
            .hints
            .insert("suppress-sound".to_string(), VariantValue::Bool(suppress));
        self
    }

    pub fn sound_file(mut self, path: &str) -> Self {
        self.notification.hints.insert(
            "sound-file".to_string(),
            VariantValue::String(path.to_string()),
        );
        self
    }

    pub fn sound(self, sound: Sound) -> Self {
        self.sound_file(sound.path())
    }

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.notification
            .hints
            .insert("x".to_string(), VariantValue::Int(x));
        self.notification
            .hints
            .insert("y".to_string(), VariantValue::Int(y));
        self
    }

    pub fn urgency(mut self, urgency: u8) -> Self {
        self.notification
            .hints
            .insert("urgency".to_string(), VariantValue::Int(urgency as i32));
        self
    }

    pub fn preview_summary(mut self, summary: &str) -> Self {
        self.notification.preview_summary = Some(summary.to_string());
        self
    }

    pub fn preview_body(mut self, body: &str) -> Self {
        self.notification.preview_body = Some(body.to_string());
        self
    }

    pub fn build(self) -> Notification {
        self.notification
    }
}

impl Default for NotificationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
