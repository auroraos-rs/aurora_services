mod backends;
mod error;
mod services;

pub use backends::{Bus, DBusBackend, DConfBackend, DConfValue};
pub use error::{AuroraError, Result};
pub use services::{
    DeviceFeatures, DeviceInfoService, FontSettings, Notification, NotificationBuilder,
    NotificationService, ServerInfo, SettingsService, VariantValue,
};

pub fn notifications() -> Result<NotificationService> {
    NotificationService::new()
}

pub fn device_info() -> Result<DeviceInfoService> {
    DeviceInfoService::new()
}

pub fn settings() -> Result<SettingsService> {
    SettingsService::new()
}
