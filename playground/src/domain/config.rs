//! Domain Logic: Configuration Management
//!
//! Pure functions for configuration handling.
//! NO file I/O - just data structures and validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Known configuration keys
///
/// FUTURE: Used for typed configuration access
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigKey {
    OutputDir,
    DefaultFamily,
    LatexEngine,
    OntologyPath,
}

impl ConfigKey {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "output_dir" => Some(Self::OutputDir),
            "default_family" => Some(Self::DefaultFamily),
            "latex_engine" => Some(Self::LatexEngine),
            "ontology_path" => Some(Self::OntologyPath),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OutputDir => "output_dir",
            Self::DefaultFamily => "default_family",
            Self::LatexEngine => "latex_engine",
            Self::OntologyPath => "ontology_path",
        }
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        vec![
            Self::OutputDir,
            Self::DefaultFamily,
            Self::LatexEngine,
            Self::OntologyPath,
        ]
    }
}

/// Configuration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut values = HashMap::new();
        values.insert("output_dir".to_string(), "output".to_string());
        values.insert("default_family".to_string(), "IMRaD".to_string());
        values.insert("latex_engine".to_string(), "pdflatex".to_string());
        values.insert("ontology_path".to_string(), "../thesis-ontology.ttl".to_string());
        Self { values }
    }
}

impl Config {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Get a configuration value with default
    /// FUTURE: Used for fallback configuration
    #[allow(dead_code)]
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// Set a configuration value (returns new Config - immutable)
    /// FUTURE: Used for config set persistence
    #[allow(dead_code)]
    pub fn with_value(&self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut new_values = self.values.clone();
        new_values.insert(key.into(), value.into());
        Self { values: new_values }
    }

    /// Get all configuration entries
    pub fn all_entries(&self) -> Vec<(&str, &str)> {
        self.values
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    /// Validate a configuration key
    pub fn is_valid_key(key: &str) -> bool {
        ConfigKey::from_str(key).is_some()
    }
}

/// Result of a configuration operation
/// FUTURE: Used for JSON serialization of config operations
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResult {
    pub key: String,
    pub value: String,
    pub operation: String,
}

#[allow(dead_code)]
impl ConfigResult {
    pub fn get(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            operation: "get".to_string(),
        }
    }

    pub fn set(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            operation: "set".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.get("output_dir"), Some("output"));
        assert_eq!(config.get("default_family"), Some("IMRaD"));
    }

    #[test]
    fn test_config_get_unknown() {
        let config = Config::default();
        assert_eq!(config.get("unknown_key"), None);
    }

    #[test]
    fn test_config_with_value_immutable() {
        let config1 = Config::default();
        let config2 = config1.with_value("output_dir", "/tmp/papers");

        // Original unchanged
        assert_eq!(config1.get("output_dir"), Some("output"));
        // New config has new value
        assert_eq!(config2.get("output_dir"), Some("/tmp/papers"));
    }

    #[test]
    fn test_config_key_validation() {
        assert!(Config::is_valid_key("output_dir"));
        assert!(Config::is_valid_key("latex_engine"));
        assert!(!Config::is_valid_key("invalid_key"));
    }

    #[test]
    fn test_config_all_entries() {
        let config = Config::default();
        let entries = config.all_entries();
        assert_eq!(entries.len(), 4);
    }
}
