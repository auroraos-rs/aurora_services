mod backends;
mod error;
mod services;

pub use backends::{Bus, DBusBackend, DConfBackend, DConfValue};
pub use error::{AuroraError, Result};
pub use services::{
    DisplaySettings, FontSettings, Notification, NotificationBuilder, NotificationService,
    Orientation, ServerInfo, SettingsService, SoundSettings, ThemeSettings, VariantValue,
};

pub fn notifications() -> Result<NotificationService> {
    NotificationService::new()
}

pub fn settings() -> Result<SettingsService> {
    SettingsService::new()
}
