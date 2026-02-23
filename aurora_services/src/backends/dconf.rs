use crate::error::{AuroraError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const DCONF_DB_PATH: &str = "/etc/dconf/db";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DConfValue {
    String(String),
    Int(i32),
    Int64(i64),
    Double(f64),
    Bool(bool),
    Array(Vec<DConfValue>),
    Dict(HashMap<String, DConfValue>),
    Null,
}

impl DConfValue {
    pub fn as_string(&self) -> Result<String> {
        match self {
            DConfValue::String(s) => Ok(s.clone()),
            _ => Err(AuroraError::InvalidType {
                expected: "String".to_string(),
                actual: self.type_name(),
            }),
        }
    }

    pub fn as_int(&self) -> Result<i32> {
        match self {
            DConfValue::Int(i) => Ok(*i),
            DConfValue::Int64(i) => Ok(*i as i32),
            _ => Err(AuroraError::InvalidType {
                expected: "Int".to_string(),
                actual: self.type_name(),
            }),
        }
    }

    pub fn as_int64(&self) -> Result<i64> {
        match self {
            DConfValue::Int64(i) => Ok(*i),
            DConfValue::Int(i) => Ok(*i as i64),
            _ => Err(AuroraError::InvalidType {
                expected: "Int64".to_string(),
                actual: self.type_name(),
            }),
        }
    }

    pub fn as_double(&self) -> Result<f64> {
        match self {
            DConfValue::Double(d) => Ok(*d),
            DConfValue::Int(i) => Ok(*i as f64),
            DConfValue::Int64(i) => Ok(*i as f64),
            _ => Err(AuroraError::InvalidType {
                expected: "Double".to_string(),
                actual: self.type_name(),
            }),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            DConfValue::Bool(b) => Ok(*b),
            _ => Err(AuroraError::InvalidType {
                expected: "Bool".to_string(),
                actual: self.type_name(),
            }),
        }
    }

    pub fn as_array(&self) -> Result<&Vec<DConfValue>> {
        match self {
            DConfValue::Array(arr) => Ok(arr),
            _ => Err(AuroraError::InvalidType {
                expected: "Array".to_string(),
                actual: self.type_name(),
            }),
        }
    }

    pub fn as_dict(&self) -> Result<&HashMap<String, DConfValue>> {
        match self {
            DConfValue::Dict(dict) => Ok(dict),
            _ => Err(AuroraError::InvalidType {
                expected: "Dict".to_string(),
                actual: self.type_name(),
            }),
        }
    }

    fn type_name(&self) -> String {
        match self {
            DConfValue::String(_) => "String".to_string(),
            DConfValue::Int(_) => "Int".to_string(),
            DConfValue::Int64(_) => "Int64".to_string(),
            DConfValue::Double(_) => "Double".to_string(),
            DConfValue::Bool(_) => "Bool".to_string(),
            DConfValue::Array(_) => "Array".to_string(),
            DConfValue::Dict(_) => "Dict".to_string(),
            DConfValue::Null => "Null".to_string(),
        }
    }
}

pub struct DConfBackend {
    cache: HashMap<String, HashMap<String, DConfValue>>,
}

impl Default for DConfBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl DConfBackend {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, path: &str, key: &str) -> Result<DConfValue> {
        let normalized_path = normalize_path(path);

        if !self.cache.contains_key(&normalized_path) {
            let values = self.read_path_from_files(&normalized_path)?;
            self.cache.insert(normalized_path.clone(), values);
        }

        Ok(self
            .cache
            .get(&normalized_path)
            .and_then(|section| section.get(key).cloned())
            .unwrap_or(DConfValue::Null))
    }

    pub fn get_all(&mut self, path: &str) -> Result<Vec<(String, DConfValue)>> {
        let normalized_path = normalize_path(path);

        if !self.cache.contains_key(&normalized_path) {
            let values = self.read_path_from_files(&normalized_path)?;
            self.cache.insert(normalized_path.clone(), values);
        }

        Ok(self
            .cache
            .get(&normalized_path)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default())
    }

    fn read_path_from_files(&self, target_path: &str) -> Result<HashMap<String, DConfValue>> {
        let mut result = HashMap::new();

        let db_path = Path::new(DCONF_DB_PATH);

        for db_name in &["vendor", "vendor-variant", "nemo"] {
            let dir = db_path.join(format!("{}.d", db_name));
            if dir.exists() {
                self.read_dconf_dir(&dir, target_path, &mut result)?;
            }
        }

        Ok(result)
    }

    fn read_dconf_dir(
        &self,
        dir: &Path,
        target_path: &str,
        result: &mut HashMap<String, DConfValue>,
    ) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            AuroraError::DConf(format!("Failed to read dir {}: {}", dir.display(), e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "txt").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    self.parse_dconf_file(&content, target_path, result);
                }
            }
        }

        Ok(())
    }

    fn parse_dconf_file(
        &self,
        content: &str,
        target_path: &str,
        result: &mut HashMap<String, DConfValue>,
    ) {
        let mut current_section: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                let section = &line[1..line.len() - 1];
                current_section = Some(normalize_path(section));
                continue;
            }

            if let Some(ref section) = &current_section {
                if section == target_path {
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim().to_string();
                        let value = value.trim();

                        if let Ok(parsed_value) = parse_dconf_value(value) {
                            result.insert(key, parsed_value);
                        }
                    }
                }
            }
        }
    }
}

fn normalize_path(path: &str) -> String {
    path.trim_matches('/').trim_start_matches('/').to_string()
}

fn parse_dconf_value(s: &str) -> Result<DConfValue> {
    let s = s.trim();

    if s.is_empty() {
        return Ok(DConfValue::Null);
    }

    if s == "true" {
        return Ok(DConfValue::Bool(true));
    }
    if s == "false" {
        return Ok(DConfValue::Bool(false));
    }

    if s.starts_with('\'') && s.ends_with('\'') {
        let inner = &s[1..s.len() - 1];
        return Ok(DConfValue::String(unescape_string(inner)));
    }

    if s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        return Ok(DConfValue::String(unescape_string(inner)));
    }

    if s.starts_with('[') && s.ends_with(']') {
        return parse_dconf_array(s);
    }

    if s.starts_with('{') && s.ends_with('}') {
        return parse_dconf_dict(s);
    }

    if let Ok(i) = s.parse::<i32>() {
        return Ok(DConfValue::Int(i));
    }

    if let Ok(i) = s.parse::<i64>() {
        return Ok(DConfValue::Int64(i));
    }

    if let Ok(d) = s.parse::<f64>() {
        return Ok(DConfValue::Double(d));
    }

    Ok(DConfValue::String(s.to_string()))
}

fn unescape_string(s: &str) -> String {
    s.replace("\\'", "'")
        .replace("\\\"", "\"")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
}

fn parse_dconf_array(s: &str) -> Result<DConfValue> {
    let inner = &s[1..s.len() - 1];
    if inner.trim().is_empty() {
        return Ok(DConfValue::Array(vec![]));
    }

    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = ' ';
    let mut depth = 0;

    for ch in inner.chars() {
        match ch {
            '\'' | '"' if depth == 0 && !in_string => {
                in_string = true;
                string_char = ch;
                current.push(ch);
            }
            '\'' | '"' if in_string && ch == string_char => {
                in_string = false;
                current.push(ch);
            }
            '[' | '{' if !in_string => {
                depth += 1;
                current.push(ch);
            }
            ']' | '}' if !in_string => {
                depth -= 1;
                current.push(ch);
            }
            ',' if !in_string && depth == 0 => {
                let val = parse_dconf_value(current.trim())?;
                result.push(val);
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.trim().is_empty() {
        let val = parse_dconf_value(current.trim())?;
        result.push(val);
    }

    Ok(DConfValue::Array(result))
}

fn parse_dconf_dict(s: &str) -> Result<DConfValue> {
    let inner = &s[1..s.len() - 1];
    if inner.trim().is_empty() {
        return Ok(DConfValue::Dict(HashMap::new()));
    }

    let mut result = HashMap::new();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_key = true;
    let mut in_string = false;
    let mut string_char = ' ';
    let mut depth = 0;

    for ch in inner.chars() {
        match ch {
            '\'' | '"' if !in_string => {
                in_string = true;
                string_char = ch;
                if in_key {
                    current_key.push(ch);
                } else {
                    current_value.push(ch);
                }
            }
            '\'' | '"' if in_string && ch == string_char => {
                in_string = false;
                if in_key {
                    current_key.push(ch);
                } else {
                    current_value.push(ch);
                }
            }
            '[' | '{' if !in_string => {
                depth += 1;
                current_value.push(ch);
            }
            ']' | '}' if !in_string => {
                depth -= 1;
                current_value.push(ch);
            }
            ':' if !in_string && depth == 0 && in_key => {
                in_key = false;
            }
            ',' if !in_string && depth == 0 => {
                let key = parse_dconf_value(current_key.trim())?
                    .as_string()
                    .unwrap_or_else(|_| current_key.trim().to_string());
                let value = parse_dconf_value(current_value.trim())?;
                result.insert(key, value);
                current_key.clear();
                current_value.clear();
                in_key = true;
            }
            _ => {
                if in_key {
                    current_key.push(ch);
                } else {
                    current_value.push(ch);
                }
            }
        }
    }

    if !current_key.trim().is_empty() && !current_value.trim().is_empty() {
        let key = parse_dconf_value(current_key.trim())?
            .as_string()
            .unwrap_or_else(|_| current_key.trim().to_string());
        let value = parse_dconf_value(current_value.trim())?;
        result.insert(key, value);
    }

    Ok(DConfValue::Dict(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let result = parse_dconf_value("'ALS Hauss Variable'").unwrap();
        assert_eq!(result.as_string().unwrap(), "ALS Hauss Variable");
    }

    #[test]
    fn test_parse_int() {
        let result = parse_dconf_value("29").unwrap();
        assert_eq!(result.as_int().unwrap(), 29);
    }

    #[test]
    fn test_parse_double() {
        let result = parse_dconf_value("1.25").unwrap();
        assert_eq!(result.as_double().unwrap(), 1.25);
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_dconf_value("true").unwrap().as_bool().unwrap());
        assert!(!parse_dconf_value("false").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_parse_empty() {
        let result = parse_dconf_value("").unwrap();
        assert!(matches!(result, DConfValue::Null));
    }

    #[test]
    fn test_parse_array() {
        let result = parse_dconf_value("['a', 'b', 'c']").unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_string().unwrap(), "a");
        assert_eq!(arr[1].as_string().unwrap(), "b");
        assert_eq!(arr[2].as_string().unwrap(), "c");
    }

    #[test]
    fn test_parse_empty_array() {
        let result = parse_dconf_value("[]").unwrap();
        let arr = result.as_array().unwrap();
        assert!(arr.is_empty());
    }

    #[test]
    fn test_parse_dconf_file_silica() {
        let content = r#"
[desktop/sailfish/silica]
theme_pixel_ratio=1.25
theme_icon_subdir='z1.25'
"#;
        let backend = DConfBackend::new();
        let mut cache: HashMap<String, DConfValue> = HashMap::new();
        backend.parse_dconf_file(content, "desktop/sailfish/silica", &mut cache);

        assert_eq!(
            cache.get("theme_pixel_ratio").unwrap().as_double().unwrap(),
            1.25
        );
        assert_eq!(
            cache.get("theme_icon_subdir").unwrap().as_string().unwrap(),
            "z1.25"
        );
    }

    #[test]
    fn test_parse_dconf_file_fonts() {
        let content = r#"
[apps/jolla-settings]
default_signature_translation_id='la-default_signature_text'

[desktop/sailfish/silica]
tab_bar_style='aurora'
font_family='ALS Hauss Variable'
font_family_heading='ALS Hauss Variable'
font_size_tiny=19
font_size_extra_small=22
font_size_small=25
font_size_medium=29
font_size_large=32
font_size_extra_large=40
font_size_huge=42
auto_scale_values=true
"#;
        let backend = DConfBackend::new();
        let mut cache: HashMap<String, DConfValue> = HashMap::new();
        backend.parse_dconf_file(content, "desktop/sailfish/silica", &mut cache);

        assert_eq!(
            cache.get("font_family").unwrap().as_string().unwrap(),
            "ALS Hauss Variable"
        );
        assert_eq!(cache.get("font_size_medium").unwrap().as_int().unwrap(), 29);
        assert!(cache.get("auto_scale_values").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path("/desktop/sailfish/silica"),
            "desktop/sailfish/silica"
        );
        assert_eq!(
            normalize_path("desktop/sailfish/silica"),
            "desktop/sailfish/silica"
        );
        assert_eq!(
            normalize_path("desktop/sailfish/silica/"),
            "desktop/sailfish/silica"
        );
    }

    #[test]
    fn test_unescape_string() {
        assert_eq!(unescape_string("hello\\nworld"), "hello\nworld");
        assert_eq!(unescape_string("hello\\tworld"), "hello\tworld");
        assert_eq!(unescape_string("hello\\'world"), "hello'world");
        assert_eq!(unescape_string("hello\\\\world"), "hello\\world");
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut backend = DConfBackend::new();
        let result = backend.get("/nonexistent/path", "key").unwrap();
        assert!(matches!(result, DConfValue::Null));
    }

    #[test]
    fn test_parse_real_dconf_dump() {
        let content = include_str!("../../../dconf_dump.txt");
        let backend = DConfBackend::new();
        let mut cache: HashMap<String, DConfValue> = HashMap::new();

        backend.parse_dconf_file(content, "desktop/sailfish/silica", &mut cache);

        assert_eq!(
            cache.get("theme_pixel_ratio").unwrap().as_double().unwrap(),
            1.25
        );
        assert_eq!(
            cache.get("font_family").unwrap().as_string().unwrap(),
            "ALS Hauss Variable"
        );
        assert_eq!(
            cache
                .get("font_family_heading")
                .unwrap()
                .as_string()
                .unwrap(),
            "ALS Hauss Variable"
        );
        assert_eq!(cache.get("font_size_medium").unwrap().as_int().unwrap(), 29);
        assert_eq!(cache.get("font_size_tiny").unwrap().as_int().unwrap(), 19);
        assert_eq!(cache.get("font_size_huge").unwrap().as_int().unwrap(), 42);
        assert!(cache.get("auto_scale_values").unwrap().as_bool().unwrap());
        assert_eq!(
            cache.get("theme_icon_subdir").unwrap().as_string().unwrap(),
            "z1.25"
        );
        assert_eq!(
            cache.get("tab_bar_style").unwrap().as_string().unwrap(),
            "aurora"
        );
    }

    #[test]
    fn test_parse_dconf_dump_lipstick() {
        let content = include_str!("../../../dconf_dump.txt");
        let backend = DConfBackend::new();
        let mut cache: HashMap<String, DConfValue> = HashMap::new();

        backend.parse_dconf_file(content, "lipstick", &mut cache);

        assert_eq!(
            cache.get("orientationLock").unwrap().as_string().unwrap(),
            "portrait"
        );
    }

    #[test]
    fn test_parse_dconf_dump_with_arrays() {
        let content = include_str!("../../../dconf_dump.txt");
        let backend = DConfBackend::new();
        let mut cache: HashMap<String, DConfValue> = HashMap::new();

        backend.parse_dconf_file(content, "apps/jolla-camera/primary/image", &mut cache);

        let exposure_values = cache
            .get("exposureCompensationValues")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(exposure_values.len(), 5);
        assert_eq!(exposure_values[0].as_int().unwrap(), 4);
        assert_eq!(exposure_values[4].as_int().unwrap(), -4);

        let viewfinder_values = cache
            .get("viewfinderGridValues")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(viewfinder_values.len(), 3);
        assert_eq!(viewfinder_values[0].as_string().unwrap(), "none");
        assert_eq!(viewfinder_values[1].as_string().unwrap(), "thirds");
        assert_eq!(viewfinder_values[2].as_string().unwrap(), "ambience");
    }
}
