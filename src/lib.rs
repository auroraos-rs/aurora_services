mod backends;
mod error;
mod services;

pub use backends::{Bus, DBusBackend, DConfBackend, DConfValue};
pub use error::{AuroraError, Result};
pub use services::{
    DEFAULT_PIXEL_RATIO, DEFAULT_STATUSBAR_HEIGHT, DeviceFeatures, DeviceInfoService, FontSettings,
    Notification, NotificationBuilder, NotificationService, ServerInfo, SettingsService, Sound,
    VariantValue,
};

pub mod package_info {
    pub use super::services::package_info::{app_id, app_instance_id, package_name, runtime_dir};
}

pub mod open_uri {
    pub use super::services::open_uri::open_uri;
}

pub fn notifications() -> Result<NotificationService> {
    NotificationService::new()
}

pub fn device_info() -> Result<DeviceInfoService> {
    DeviceInfoService::new()
}

pub fn settings() -> Result<SettingsService> {
    SettingsService::new()
}
