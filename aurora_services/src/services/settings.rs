use crate::backends::{Bus, DBusBackend, DConfBackend, DConfValue, MessageItem};
use crate::error::{AuroraError, Result};
use serde::{Deserialize, Serialize};

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

pub struct SettingsService {
    dconf: DConfBackend,
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
        Self { dconf, dbus }
    }

    pub fn get_theme(&self) -> Result<ThemeSettings> {
        let active_ambience = self
            .dconf
            .get("/desktop/jolla/theme", "active_ambience")?
            .as_string()
            .ok();

        let color_scheme = self
            .dconf
            .get("/desktop/jolla/theme", "color_scheme")?
            .as_int()
            .ok()
            .map(|i| i as u32);

        let highlight_color = self
            .dconf
            .get("/desktop/jolla/theme/color", "highlight")?
            .as_string()
            .ok();

        let primary_color = self
            .dconf
            .get("/desktop/jolla/theme/color", "primary")?
            .as_string()
            .ok();

        let secondary_color = self
            .dconf
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

    pub fn set_theme(&self, theme: &ThemeSettings) -> Result<()> {
        if let Some(ref ambience) = theme.active_ambience {
            self.dconf.set(
                "/desktop/jolla/theme",
                "active_ambience",
                &DConfValue::String(ambience.clone()),
            )?;
        }

        if let Some(scheme) = theme.color_scheme {
            self.dconf.set(
                "/desktop/jolla/theme",
                "color_scheme",
                &DConfValue::Int(scheme as i32),
            )?;
        }

        if let Some(ref color) = theme.highlight_color {
            self.dconf.set(
                "/desktop/jolla/theme/color",
                "highlight",
                &DConfValue::String(color.clone()),
            )?;
        }

        if let Some(ref color) = theme.primary_color {
            self.dconf.set(
                "/desktop/jolla/theme/color",
                "primary",
                &DConfValue::String(color.clone()),
            )?;
        }

        if let Some(ref color) = theme.secondary_color {
            self.dconf.set(
                "/desktop/jolla/theme/color",
                "secondary",
                &DConfValue::String(color.clone()),
            )?;
        }

        Ok(())
    }

    pub fn set_wallpaper(&self, path: &str) -> Result<()> {
        self.dconf.set(
            "/desktop/jolla/background/portrait",
            "home_picture_filename",
            &DConfValue::String(path.to_string()),
        )
    }

    pub fn get_wallpaper(&self) -> Result<Option<String>> {
        self.dconf
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
        let orientation_lock = self
            .dconf
            .get("/lipstick", "orientationLock")?
            .as_string()
            .ok()
            .and_then(|s| Orientation::parse(&s));

        let brightness = self
            .dconf
            .get("/desktop/jolla/display", "brightness")?
            .as_int()
            .ok()
            .map(|i| i as u32);

        Ok(DisplaySettings {
            orientation_lock,
            brightness,
        })
    }

    pub fn set_orientation_lock(&self, orientation: Orientation) -> Result<()> {
        self.dconf.set(
            "/lipstick",
            "orientationLock",
            &DConfValue::String(orientation.as_str().to_string()),
        )
    }

    pub fn set_brightness(&self, brightness: u32) -> Result<()> {
        self.dconf.set(
            "/desktop/jolla/display",
            "brightness",
            &DConfValue::Int(brightness as i32),
        )
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
        let theme = self.dconf.get("/jolla/sound", "theme")?.as_string().ok();

        Ok(SoundSettings { profile, theme })
    }

    pub fn get_all(&self, path: &str) -> Result<Vec<(String, DConfValue)>> {
        self.dconf.get_all(path)
    }

    pub fn reset_key(&self, path: &str, key: &str) -> Result<()> {
        self.dconf.reset(path, key)
    }

    pub fn get_pixel_ratio(&self) -> Result<f64> {
        self.dconf
            .get("/desktop/sailfish/silica", "theme_pixel_ratio")?
            .as_double()
    }

    pub fn get_statusbar_height(&self) -> Result<i32> {
        self.dconf
            .get("/desktop/sailfish/silica/statusbar", "height")?
            .as_int()
    }
}
