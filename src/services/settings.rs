use crate::backends::DConfBackend;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

/// Default pixel ratio used on Aurora OS devices when DConf is unavailable.
pub const DEFAULT_PIXEL_RATIO: f64 = 1.5;

/// Default status bar height in pixels when DConf is unavailable.
pub const DEFAULT_STATUSBAR_HEIGHT: i32 = 41;

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
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new().expect("Failed to create SettingsService")
    }
}

impl SettingsService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            dconf: RefCell::new(DConfBackend::new()),
        })
    }

    pub fn get_pixel_ratio(&self) -> Result<f64> {
        self.dconf
            .borrow_mut()
            .get("/desktop/sailfish/silica", "theme_pixel_ratio")?
            .as_double()
    }

    /// Returns the system pixel ratio, falling back to [`DEFAULT_PIXEL_RATIO`]
    /// if DConf is unavailable.
    pub fn pixel_ratio(&self) -> f64 {
        self.get_pixel_ratio().unwrap_or(DEFAULT_PIXEL_RATIO)
    }

    pub fn get_statusbar_height(&self) -> Result<i32> {
        let mut dconf = self.dconf.borrow_mut();

        let statusbar_conf_height = dconf
            .get("/desktop/sailfish/silica/statusbar", "height")?
            .as_int()
            .unwrap_or(0);

        if statusbar_conf_height > 0 {
            return Ok(statusbar_conf_height);
        }

        let icon_size_small = dconf
            .get("/desktop/sailfish/silica", "icon_size_small")?
            .as_int()
            .unwrap_or(32);

        let padding_small = dconf
            .get("/desktop/sailfish/silica", "padding_small")?
            .as_int()
            .unwrap_or(6);

        let padding_medium = 2 * padding_small;

        Ok(2 * (padding_medium + padding_small) + icon_size_small)
    }

    /// Returns the system status bar height, falling back to
    /// [`DEFAULT_STATUSBAR_HEIGHT`] if DConf is unavailable.
    pub fn statusbar_height(&self) -> i32 {
        self.get_statusbar_height().unwrap_or(DEFAULT_STATUSBAR_HEIGHT)
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

    /// Returns the system font settings, falling back to [`FontSettings::default()`]
    /// if DConf is unavailable.
    pub fn font_settings(&self) -> FontSettings {
        self.get_font_settings().unwrap_or_default()
    }
}
