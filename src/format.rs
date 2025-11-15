//! Output formatting system
//!
//! This module provides pluggable output formatters for different output formats.
//! The default is JSON, but YAML, TOML, and table formats are also supported.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::format::{OutputFormat, Formatter};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Output {
//!     name: String,
//!     value: u32,
//! }
//!
//! let output = Output { name: "example".to_string(), value: 42 };
//! let formatted = OutputFormat::Yaml.format(&output)?;
//! println!("{}", formatted);
//! ```

use serde::Serialize;
use std::fmt;

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format (default)
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// Pretty printed table format
    Table,
    /// Tab-separated values
    Tsv,
}

impl OutputFormat {
    /// Format a serializable value
    pub fn format<S: Serialize>(self, value: &S) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            OutputFormat::Json => format_json(value),
            OutputFormat::Yaml => format_yaml(value),
            OutputFormat::Toml => format_toml(value),
            OutputFormat::Table => format_table(value),
            OutputFormat::Tsv => format_tsv(value),
        }
    }

    /// Get all available format names
    pub fn available_formats() -> &'static [&'static str] {
        &["json", "yaml", "toml", "table", "tsv"]
    }

    /// Check if a format is available
    pub fn is_available(&self) -> bool {
        true
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Toml => write!(f, "toml"),
            Self::Table => write!(f, "table"),
            Self::Tsv => write!(f, "tsv"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" | "yml" => Ok(OutputFormat::Yaml),
            "toml" => Ok(OutputFormat::Toml),
            "table" | "ascii" => Ok(OutputFormat::Table),
            "tsv" | "tab" => Ok(OutputFormat::Tsv),
            _ => Err(format!(
                "Unknown format '{}'. Available: {:?}",
                s,
                OutputFormat::available_formats()
            )),
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Json
    }
}

/// JSON formatter (default)
fn format_json<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(serde_json::to_string_pretty(&json)?)
}

/// YAML formatter
fn format_yaml<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(serde_yaml::to_string(&json)?)
}

/// TOML formatter
fn format_toml<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(toml::to_string_pretty(&json)?)
}

/// Table formatter - converts JSON to ASCII table
fn format_table<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    json_to_table(&json)
}

/// TSV formatter - converts JSON to tab-separated values
fn format_tsv<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    json_to_tsv(&json)
}

/// Helper: Convert JSON value to ASCII table
fn json_to_table(value: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return Ok("(empty)".to_string());
            }

            // Extract headers from first object if present
            let first = &arr[0];
            if let serde_json::Value::Object(obj) = first {
                let mut output = String::new();
                let keys: Vec<&String> = obj.keys().collect();

                // Header row - avoid Vec allocation, build directly
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 { output.push('\t'); }
                    output.push_str(k.as_str());
                }
                output.push('\n');

                // Data rows
                for item in arr {
                    if let serde_json::Value::Object(item_obj) = item {
                        for (i, k) in keys.iter().enumerate() {
                            if i > 0 { output.push('\t'); }
                            let value_str = item_obj
                                .get(k.as_str())
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "-".to_string());
                            output.push_str(&value_str);
                        }
                        output.push('\n');
                    }
                }
                Ok(output)
            } else {
                Ok(format_list_table(arr))
            }
        }
        serde_json::Value::Object(obj) => {
            let mut output = String::new();
            for (k, v) in obj {
                output.push_str(&format!("{}\t{}\n", k, v));
            }
            Ok(output)
        }
        other => Ok(other.to_string()),
    }
}

/// Helper: Format list as table
fn format_list_table(arr: &[serde_json::Value]) -> String {
    let mut output = String::from("item\n");
    for item in arr.iter() {
        match item {
            serde_json::Value::String(s) => output.push_str(&format!("{}\n", s)),
            _ => output.push_str(&format!("{}\n", item)),
        }
    }
    output
}

/// Helper: Convert JSON to TSV
fn json_to_tsv(value: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return Ok(String::new());
            }

            let first = &arr[0];
            if let serde_json::Value::Object(obj) = first {
                let mut output = String::new();
                let keys: Vec<&String> = obj.keys().collect();

                // Header - avoid Vec allocation, build directly
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 { output.push('\t'); }
                    output.push_str(k.as_str());
                }
                output.push('\n');

                // Rows
                for item in arr {
                    if let serde_json::Value::Object(item_obj) = item {
                        for (i, k) in keys.iter().enumerate() {
                            if i > 0 { output.push('\t'); }
                            let escaped = item_obj
                                .get(k.as_str())
                                .map(|v| escape_tsv(&v.to_string()))
                                .unwrap_or_default();
                            output.push_str(&escaped);
                        }
                        output.push('\n');
                    }
                }
                Ok(output)
            } else {
                // Avoid Vec allocation for simple string join
                let mut result = String::new();
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { result.push('\n'); }
                    result.push_str(&item.to_string());
                }
                Ok(result)
            }
        }
        serde_json::Value::Object(obj) => {
            let mut output = String::new();
            for (k, v) in obj {
                output.push_str(&format!("{}\t{}\n", k, escape_tsv(&v.to_string())));
            }
            Ok(output)
        }
        other => Ok(other.to_string()),
    }
}

/// Helper: Escape TSV special characters
fn escape_tsv(s: &str) -> String {
    if s.contains('\t') || s.contains('\n') || s.contains('\\') {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Yaml.to_string(), "yaml");
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("yaml".parse::<OutputFormat>().unwrap(), OutputFormat::Yaml);
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_available_formats() {
        let formats = OutputFormat::available_formats();
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"yaml"));
        assert!(formats.contains(&"toml"));
    }
}
