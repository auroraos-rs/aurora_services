use crate::error::{AuroraError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

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

#[derive(Debug, Clone, Default)]
pub struct DConfBackend {}

impl DConfBackend {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, path: &str, key: &str) -> Result<DConfValue> {
        let full_path = format!("{}/{}", path.trim_end_matches('/'), key);
        let output = Command::new("dconf")
            .arg("read")
            .arg(&full_path)
            .output()
            .map_err(|e| AuroraError::CommandFailed(e.to_string()))?;

        let value_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if value_str.is_empty() {
            return Ok(DConfValue::Null);
        }

        parse_dconf_value(&value_str)
    }

    pub fn set(&self, path: &str, key: &str, value: &DConfValue) -> Result<()> {
        let full_path = format!("{}/{}", path.trim_end_matches('/'), key);
        let value_str = value_to_dconf_string(value);

        let status = Command::new("dconf")
            .arg("write")
            .arg(&full_path)
            .arg(&value_str)
            .status()
            .map_err(|e| AuroraError::CommandFailed(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(AuroraError::CommandFailed(format!(
                "dconf write failed with status: {}",
                status
            )))
        }
    }

    pub fn get_all(&self, path: &str) -> Result<Vec<(String, DConfValue)>> {
        let output = Command::new("dconf")
            .arg("dump")
            .arg(path)
            .output()
            .map_err(|e| AuroraError::CommandFailed(e.to_string()))?;

        let content = String::from_utf8_lossy(&output.stdout);
        parse_dconf_dump(&content)
    }

    pub fn reset(&self, path: &str, key: &str) -> Result<()> {
        let full_path = format!("{}/{}", path.trim_end_matches('/'), key);

        let status = Command::new("dconf")
            .arg("reset")
            .arg(&full_path)
            .status()
            .map_err(|e| AuroraError::CommandFailed(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(AuroraError::CommandFailed(format!(
                "dconf reset failed with status: {}",
                status
            )))
        }
    }
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

fn parse_dconf_dump(content: &str) -> Result<Vec<(String, DConfValue)>> {
    let mut result = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = parse_dconf_value(value.trim())?;
            result.push((key, value));
        }
    }

    Ok(result)
}

fn value_to_dconf_string(value: &DConfValue) -> String {
    match value {
        DConfValue::String(s) => format!("'{}'", escape_string(s)),
        DConfValue::Int(i) => i.to_string(),
        DConfValue::Int64(i) => i.to_string(),
        DConfValue::Double(d) => d.to_string(),
        DConfValue::Bool(b) => b.to_string(),
        DConfValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_dconf_string).collect();
            format!("[{}]", items.join(", "))
        }
        DConfValue::Dict(dict) => {
            let items: Vec<String> = dict
                .iter()
                .map(|(k, v)| format!("'{}': {}", escape_string(k), value_to_dconf_string(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        DConfValue::Null => "@mv".to_string(),
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
