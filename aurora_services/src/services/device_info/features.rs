use crate::backends::{Bus, DBusBackend, MessageItem};
use crate::error::Result;

const DEVICE_INFO_SERVICE: &str = "ru.omp.deviceinfo";
const DEVICE_INFO_PATH_FEATURES: &str = "/ru/omp/deviceinfo/Features";
const DEVICE_INFO_INTERFACE_FEATURES: &str = "ru.omp.deviceinfo.Features";

#[derive(Debug, Clone, Default)]
pub struct DeviceFeatures {
    pub is_os_certified: bool,
    // pub is_os_emulated: bool,
    pub os_type: String,
    pub battery_percentage: u32,
    pub cpu_model: String,
    pub device_model: String,
    pub font_family_names: Vec<String>,
    pub font_names: Vec<String>,
    pub frontal_camera_resolution: f64,
    pub locale: String,
    pub locales: Vec<String>,
    pub main_camera_resolution: f64,
    pub max_cpu_clock_speed: u32,
    pub max_cpu_cores_clock_speed: Vec<u32>,
    pub number_cpu_cores: u32,
    pub os_version: String,
    pub ram_free_size: u64,
    pub ram_total_size: u64,
    pub screen_resolution: String,
    pub serial_number: String,
    pub time_zone: String,
    pub has_bluetooth: bool,
    pub has_gnss: bool,
    pub has_nfc: bool,
    pub has_wlan: bool,
}

pub struct DeviceInfoService {
    dbus: DBusBackend,
}

impl Default for DeviceInfoService {
    fn default() -> Self {
        Self::new().expect("Failed to create DeviceInfoService")
    }
}

impl DeviceInfoService {
    pub fn new() -> Result<Self> {
        Ok(Self::with_backend(DBusBackend::new()?))
    }

    pub fn with_backend(dbus: DBusBackend) -> Self {
        Self { dbus }
    }

    #[allow(clippy::field_reassign_with_default)]
    pub fn get_features(&self) -> Result<DeviceFeatures> {
        let mut features = DeviceFeatures::default();

        features.is_os_certified = self.get_is_os_certified()?;
        // features.is_os_emulated = self.get_is_os_emulated()?;
        features.os_type = self.get_os_type()?;
        features.battery_percentage = self.get_battery_charge_percentage()?;
        features.cpu_model = self.get_cpu_model()?;
        features.device_model = self.get_device_model()?;
        features.font_family_names = self.get_font_family_names()?;
        features.font_names = self.get_font_names()?;
        features.frontal_camera_resolution = self.get_frontal_camera_resolution()?;
        features.locale = self.get_locale()?;
        features.locales = self.get_locales()?;
        features.main_camera_resolution = self.get_main_camera_resolution()?;
        features.max_cpu_clock_speed = self.get_max_cpu_clock_speed()?;
        features.max_cpu_cores_clock_speed = self.get_max_cpu_cores_clock_speed()?;
        features.number_cpu_cores = self.get_number_cpu_cores()?;
        features.os_version = self.get_os_version()?;
        features.ram_free_size = self.get_ram_free_size()?;
        features.ram_total_size = self.get_ram_total_size()?;
        features.screen_resolution = self.get_screen_resolution()?;
        features.serial_number = self.get_serial_number()?;
        features.time_zone = self.get_time_zone()?;
        features.has_bluetooth = self.has_bluetooth()?;
        features.has_gnss = self.has_gnss()?;
        features.has_nfc = self.has_nfc()?;
        features.has_wlan = self.has_wlan()?;

        Ok(features)
    }

    fn get_is_os_certified(&self) -> Result<bool> {
        let item = self.dbus.get_property(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "isOsCertified",
        )?;
        parse_bool(&item)
    }

    // fn get_is_os_emulated(&self) -> Result<bool> {
    //     let item = self.dbus.get_property(
    //         Bus::System,
    //         DEVICE_INFO_SERVICE,
    //         DEVICE_INFO_PATH_FEATURES,
    //         DEVICE_INFO_INTERFACE_FEATURES,
    //         "isOsEmulated",
    //     )?;
    //     parse_bool(&item)
    // }

    fn get_os_type(&self) -> Result<String> {
        let item = self.dbus.get_property(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "osType",
        )?;
        parse_string(&item)
    }

    fn get_battery_charge_percentage(&self) -> Result<u32> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getBatteryChargePercentage",
            &[],
        )?;

        parse_single_uint(&result)
    }

    fn get_cpu_model(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getCpuModel",
            &[],
        )?;

        parse_single_string(&result)
    }

    fn get_device_model(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getDeviceModel",
            &[],
        )?;

        parse_single_string(&result)
    }

    fn get_font_family_names(&self) -> Result<Vec<String>> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getFontFamilyNames",
            &[],
        )?;

        parse_string_array(&result)
    }

    fn get_font_names(&self) -> Result<Vec<String>> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getFontNames",
            &[],
        )?;

        parse_string_array(&result)
    }

    fn get_frontal_camera_resolution(&self) -> Result<f64> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getFrontalCameraResolution",
            &[],
        )?;

        parse_single_double(&result)
    }

    fn get_locale(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getLocale",
            &[],
        )?;

        parse_single_string(&result)
    }

    fn get_locales(&self) -> Result<Vec<String>> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getLocales",
            &[],
        )?;

        parse_string_array(&result)
    }

    fn get_main_camera_resolution(&self) -> Result<f64> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getMainCameraResolution",
            &[],
        )?;

        parse_single_double(&result)
    }

    fn get_max_cpu_clock_speed(&self) -> Result<u32> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getMaxCpuClockSpeed",
            &[],
        )?;

        parse_single_uint(&result)
    }

    fn get_max_cpu_cores_clock_speed(&self) -> Result<Vec<u32>> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getMaxCpuCoresClockSpeed",
            &[],
        )?;

        parse_uint_array(&result)
    }

    fn get_number_cpu_cores(&self) -> Result<u32> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getNumberCpuCores",
            &[],
        )?;

        parse_single_uint(&result)
    }

    fn get_os_version(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getOsVersion",
            &[],
        )?;

        parse_single_string(&result)
    }

    fn get_ram_free_size(&self) -> Result<u64> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getRamFreeSize",
            &[],
        )?;

        parse_single_uint64(&result)
    }

    fn get_ram_total_size(&self) -> Result<u64> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getRamTotalSize",
            &[],
        )?;

        parse_single_uint64(&result)
    }

    fn get_screen_resolution(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getScreenResolution",
            &[],
        )?;

        parse_single_string(&result)
    }

    fn get_serial_number(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getSerialNumber",
            &[],
        )?;

        parse_single_string(&result)
    }

    fn get_time_zone(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "getTimeZone",
            &[],
        )?;

        parse_single_string(&result)
    }

    fn has_bluetooth(&self) -> Result<bool> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "hasBluetooth",
            &[],
        )?;

        parse_single_bool(&result)
    }

    fn has_gnss(&self) -> Result<bool> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "hasGNSS",
            &[],
        )?;

        parse_single_bool(&result)
    }

    fn has_nfc(&self) -> Result<bool> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "hasNFC",
            &[],
        )?;

        parse_single_bool(&result)
    }

    fn has_wlan(&self) -> Result<bool> {
        let result = self.dbus.call_method(
            Bus::System,
            DEVICE_INFO_SERVICE,
            DEVICE_INFO_PATH_FEATURES,
            DEVICE_INFO_INTERFACE_FEATURES,
            "hasWlan",
            &[],
        )?;

        parse_single_bool(&result)
    }
}

fn parse_string(item: &MessageItem) -> Result<String> {
    if let MessageItem::Str(s) = item {
        Ok(s.clone())
    } else {
        Ok(String::new())
    }
}

fn parse_bool(item: &MessageItem) -> Result<bool> {
    if let MessageItem::Bool(b) = item {
        Ok(*b)
    } else {
        Ok(false)
    }
}

fn parse_single_string(result: &[MessageItem]) -> Result<String> {
    result
        .first()
        .map(|item| {
            if let MessageItem::Str(s) = item {
                Ok(s.clone())
            } else {
                Ok(String::new())
            }
        })
        .unwrap_or(Ok(String::new()))
}

fn parse_single_uint(result: &[MessageItem]) -> Result<u32> {
    result
        .first()
        .map(|item| {
            if let MessageItem::UInt32(v) = item {
                Ok(*v)
            } else {
                Ok(0)
            }
        })
        .unwrap_or(Ok(0))
}

fn parse_single_uint64(result: &[MessageItem]) -> Result<u64> {
    result
        .first()
        .map(|item| {
            if let MessageItem::UInt64(v) = item {
                Ok(*v)
            } else if let MessageItem::UInt32(v) = item {
                Ok(*v as u64)
            } else {
                Ok(0)
            }
        })
        .unwrap_or(Ok(0))
}

fn parse_single_double(result: &[MessageItem]) -> Result<f64> {
    result
        .first()
        .map(|item| {
            if let MessageItem::Double(v) = item {
                Ok(*v)
            } else {
                Ok(0.0)
            }
        })
        .unwrap_or(Ok(0.0))
}

fn parse_single_bool(result: &[MessageItem]) -> Result<bool> {
    result
        .first()
        .map(|item| {
            if let MessageItem::Bool(v) = item {
                Ok(*v)
            } else {
                Ok(false)
            }
        })
        .unwrap_or(Ok(false))
}

fn parse_string_array(result: &[MessageItem]) -> Result<Vec<String>> {
    if let Some(MessageItem::Array(arr)) = result.first() {
        let strings: Vec<String> = arr
            .iter()
            .filter_map(|item| {
                if let MessageItem::Str(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(strings)
    } else {
        Ok(Vec::new())
    }
}

fn parse_uint_array(result: &[MessageItem]) -> Result<Vec<u32>> {
    if let Some(MessageItem::Array(arr)) = result.first() {
        let values: Vec<u32> = arr
            .iter()
            .filter_map(|item| match item {
                MessageItem::UInt32(v) => Some(*v),
                MessageItem::Variant(v) => {
                    if let MessageItem::UInt32(val) = **v {
                        Some(val)
                    } else {
                        eprintln!("Unexpected variant found: {:?}", item);
                        None
                    }
                }
                _ => {
                    eprintln!("Unexpected value found: {:?}", item);
                    None
                }
            })
            .collect();
        Ok(values)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_features() {
        let features = DeviceFeatures::default();
        assert!(!features.is_os_certified);
        // assert!(!features.is_os_emulated);
        assert!(features.os_type.is_empty());
    }
}
