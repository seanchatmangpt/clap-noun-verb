//! Domain Logic: Output Format Handling
//!
//! Pure functions for serializing data to different output formats.
//! Supports JSON, YAML, Table, and Plain text.

use serde::{Deserialize, Serialize};

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    JsonPretty,
    Yaml,
    Table,
    Plain,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "json-pretty" | "jsonpretty" => Some(Self::JsonPretty),
            "yaml" | "yml" => Some(Self::Yaml),
            "table" => Some(Self::Table),
            "plain" | "text" => Some(Self::Plain),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Json, Self::JsonPretty, Self::Yaml, Self::Table, Self::Plain]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::JsonPretty => "json-pretty",
            Self::Yaml => "yaml",
            Self::Table => "table",
            Self::Plain => "plain",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Json => "Compact JSON (machine-readable)",
            Self::JsonPretty => "Pretty-printed JSON (human-readable)",
            Self::Yaml => "YAML format",
            Self::Table => "ASCII table format",
            Self::Plain => "Plain text output",
        }
    }
}

/// Format data to string based on output format
pub fn format_output<T: Serialize>(data: &T, format: OutputFormat) -> Result<String, String> {
    match format {
        OutputFormat::Json => {
            serde_json::to_string(data)
                .map_err(|e| format!("JSON serialization error: {}", e))
        }
        OutputFormat::JsonPretty => {
            serde_json::to_string_pretty(data)
                .map_err(|e| format!("JSON serialization error: {}", e))
        }
        OutputFormat::Yaml => {
            // Simple YAML-like output (avoid adding serde_yaml dependency)
            let json = serde_json::to_value(data)
                .map_err(|e| format!("Serialization error: {}", e))?;
            Ok(json_to_yaml_string(&json, 0))
        }
        OutputFormat::Table => {
            // Simple table format
            let json = serde_json::to_value(data)
                .map_err(|e| format!("Serialization error: {}", e))?;
            Ok(json_to_table_string(&json))
        }
        OutputFormat::Plain => {
            let json = serde_json::to_value(data)
                .map_err(|e| format!("Serialization error: {}", e))?;
            Ok(json_to_plain_string(&json))
        }
    }
}

fn json_to_yaml_string(value: &serde_json::Value, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                arr.iter()
                    .map(|v| format!("{}- {}", prefix, json_to_yaml_string(v, indent + 1)))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        serde_json::Value::Object(map) => {
            map.iter()
                .map(|(k, v)| {
                    if v.is_object() || v.is_array() {
                        format!("{}{}:\n{}", prefix, k, json_to_yaml_string(v, indent + 1))
                    } else {
                        format!("{}{}: {}", prefix, k, json_to_yaml_string(v, indent))
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

fn json_to_table_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(arr) if !arr.is_empty() => {
            // Try to format as table if array of objects
            if let Some(serde_json::Value::Object(first)) = arr.first() {
                let headers: Vec<&str> = first.keys().map(|s| s.as_str()).collect();
                let mut lines = vec![];

                // Header
                lines.push(headers.join(" | "));
                lines.push(headers.iter().map(|h| "-".repeat(h.len())).collect::<Vec<_>>().join("-+-"));

                // Rows
                for item in arr {
                    if let serde_json::Value::Object(obj) = item {
                        let row: Vec<String> = headers.iter()
                            .map(|h| obj.get(*h).map(|v| v.to_string()).unwrap_or_default())
                            .collect();
                        lines.push(row.join(" | "));
                    }
                }
                return lines.join("\n");
            }
        }
        serde_json::Value::Object(map) => {
            return map.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
        }
        _ => {}
    }
    value.to_string()
}

fn json_to_plain_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            arr.iter()
                .map(json_to_plain_string)
                .collect::<Vec<_>>()
                .join("\n")
        }
        serde_json::Value::Object(map) => {
            map.iter()
                .map(|(k, v)| format!("{}: {}", k, json_to_plain_string(v)))
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        count: u32,
    }

    #[test]
    fn test_output_format_parsing() {
        assert_eq!(OutputFormat::from_str("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str("YAML"), Some(OutputFormat::Yaml));
        assert_eq!(OutputFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_format_json() {
        let data = TestData { name: "test".to_string(), count: 42 };
        let result = format_output(&data, OutputFormat::Json).unwrap();
        assert!(result.contains("test"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_all_types() {
        let data = TestData { name: "test".to_string(), count: 42 };
        for format in OutputFormat::all() {
            let result = format_output(&data, format);
            assert!(result.is_ok(), "Failed for format {:?}", format);
        }
    }
}
