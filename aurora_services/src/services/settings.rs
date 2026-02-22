use crate::backends::{Bus, DBusBackend, DConfBackend, DConfValue, MessageItem};
use crate::error::{AuroraError, Result};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

const PROFILED_SERVICE: &str = "com.nokia.profiled";
const PROFILED_PATH: &str = "/com/nokia/profiled";
const PROFILED_INTERFACE: &str = "com.nokia.profiled";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub active_ambience: Option<String>,
    pub color_scheme: Option<u32>,
    pub highlight_color: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    #[default]
    Dynamic,
    Portrait,
    Landscape,
}

impl Orientation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Orientation::Portrait => "portrait",
            Orientation::Landscape => "landscape",
            Orientation::Dynamic => "dynamic",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "portrait" => Some(Orientation::Portrait),
            "landscape" => Some(Orientation::Landscape),
            "dynamic" | "none" => Some(Orientation::Dynamic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub orientation_lock: Option<Orientation>,
    pub brightness: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundSettings {
    pub profile: Option<String>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: "ALS Hauss Variable".to_string(),
            family_heading: "ALS Hauss Variable".to_string(),
            size_tiny: 19,
            size_extra_small: 22,
            size_small: 25,
            size_medium: 29,
            size_large: 32,
            size_extra_large: 40,
            size_huge: 42,
        }
    }
}

pub struct SettingsService {
    dconf: RefCell<DConfBackend>,
    dbus: DBusBackend,
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new().expect("Failed to create SettingsService")
    }
}

impl SettingsService {
    pub fn new() -> Result<Self> {
        Ok(Self::with_backends(
            DBusBackend::new()?,
            DConfBackend::new(),
        ))
    }

    pub fn with_backends(dbus: DBusBackend, dconf: DConfBackend) -> Self {
        Self {
            dconf: RefCell::new(dconf),
            dbus,
        }
    }

    pub fn get_theme(&self) -> Result<ThemeSettings> {
        let mut dconf = self.dconf.borrow_mut();

        let active_ambience = dconf
            .get("/desktop/jolla/theme", "active_ambience")?
            .as_string()
            .ok();

        let color_scheme = dconf
            .get("/desktop/jolla/theme", "color_scheme")?
            .as_int()
            .ok()
            .map(|i| i as u32);

        let highlight_color = dconf
            .get("/desktop/jolla/theme/color", "highlight")?
            .as_string()
            .ok();

        let primary_color = dconf
            .get("/desktop/jolla/theme/color", "primary")?
            .as_string()
            .ok();

        let secondary_color = dconf
            .get("/desktop/jolla/theme/color", "secondary")?
            .as_string()
            .ok();

        Ok(ThemeSettings {
            active_ambience,
            color_scheme,
            highlight_color,
            primary_color,
            secondary_color,
        })
    }

    pub fn get_wallpaper(&self) -> Result<Option<String>> {
        self.dconf
            .borrow_mut()
            .get(
                "/desktop/jolla/background/portrait",
                "home_picture_filename",
            )?
            .as_string()
            .ok()
            .map(|s| Ok(Some(s)))
            .unwrap_or(Ok(None))
    }

    pub fn get_display(&self) -> Result<DisplaySettings> {
        let mut dconf = self.dconf.borrow_mut();

        let orientation_lock = dconf
            .get("/lipstick", "orientationLock")?
            .as_string()
            .ok()
            .and_then(|s| Orientation::parse(&s));

        let brightness = dconf
            .get("/desktop/jolla/display", "brightness")?
            .as_int()
            .ok()
            .map(|i| i as u32);

        Ok(DisplaySettings {
            orientation_lock,
            brightness,
        })
    }

    pub fn get_sound_profile(&self) -> Result<String> {
        let result = self.dbus.call_method(
            Bus::System,
            PROFILED_SERVICE,
            PROFILED_PATH,
            PROFILED_INTERFACE,
            "get_profile",
            &[],
        )?;

        result
            .first()
            .and_then(|item| {
                if let MessageItem::Str(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| AuroraError::DBus("Invalid response from get_profile".to_string()))
    }

    pub fn set_sound_profile(&self, profile: &str) -> Result<()> {
        self.dbus.call_method(
            Bus::System,
            PROFILED_SERVICE,
            PROFILED_PATH,
            PROFILED_INTERFACE,
            "set_profile",
            &[MessageItem::Str(profile.to_string())],
        )?;

        Ok(())
    }

    pub fn get_sound_profiles(&self) -> Result<Vec<String>> {
        let result = self.dbus.call_method(
            Bus::System,
            PROFILED_SERVICE,
            PROFILED_PATH,
            PROFILED_INTERFACE,
            "get_profiles",
            &[],
        )?;

        let profiles = result
            .first()
            .and_then(|item| {
                if let MessageItem::Array(arr) = item {
                    Some(
                        arr.iter()
                            .filter_map(|i| {
                                if let MessageItem::Str(s) = i {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(profiles)
    }

    pub fn get_sound_settings(&self) -> Result<SoundSettings> {
        let profile = self.get_sound_profile().ok();
        let theme = self
            .dconf
            .borrow_mut()
            .get("/jolla/sound", "theme")?
            .as_string()
            .ok();

        Ok(SoundSettings { profile, theme })
    }

    pub fn get_all(&self, path: &str) -> Result<Vec<(String, DConfValue)>> {
        self.dconf.borrow_mut().get_all(path)
    }

    pub fn get_pixel_ratio(&self) -> Result<f64> {
        self.dconf
            .borrow_mut()
            .get("/desktop/sailfish/silica", "theme_pixel_ratio")?
            .as_double()
    }

    pub fn get_statusbar_height(&self) -> Result<i32> {
        self.dconf
            .borrow_mut()
            .get("/desktop/sailfish/silica/statusbar", "height")?
            .as_int()
    }

    pub fn get_font_settings(&self) -> Result<FontSettings> {
        let mut dconf = self.dconf.borrow_mut();
        let path = "/desktop/sailfish/silica";

        let family = dconf
            .get(path, "font_family")?
            .as_string()
            .unwrap_or_else(|_| "ALS Hauss Variable".to_string());

        let family_heading = dconf
            .get(path, "font_family_heading")?
            .as_string()
            .unwrap_or_else(|_| "ALS Hauss Variable".to_string());

        let size_tiny = dconf.get(path, "font_size_tiny")?.as_int().unwrap_or(19);

        let size_extra_small = dconf
            .get(path, "font_size_extra_small")?
            .as_int()
            .unwrap_or(22);

        let size_small = dconf.get(path, "font_size_small")?.as_int().unwrap_or(25);

        let size_medium = dconf.get(path, "font_size_medium")?.as_int().unwrap_or(29);

        let size_large = dconf.get(path, "font_size_large")?.as_int().unwrap_or(32);

        let size_extra_large = dconf
            .get(path, "font_size_extra_large")?
            .as_int()
            .unwrap_or(40);

        let size_huge = dconf.get(path, "font_size_huge")?.as_int().unwrap_or(42);

        Ok(FontSettings {
            family,
            family_heading,
            size_tiny,
            size_extra_small,
            size_small,
            size_medium,
            size_large,
            size_extra_large,
            size_huge,
        })
    }
}
