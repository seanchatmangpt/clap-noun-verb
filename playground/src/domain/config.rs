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
    pub output_dir: String,
    pub default_family: String,
    pub latex_engine: String,
    pub ontology_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: "output".to_string(),
            default_family: "IMRaD".to_string(),
            latex_engine: "pdflatex".to_string(),
            ontology_path: "../thesis-ontology.ttl".to_string(),
        }
    }
}

impl Config {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<&str> {
        match key {
            "output_dir" => Some(self.output_dir.as_str()),
            "default_family" => Some(self.default_family.as_str()),
            "latex_engine" => Some(self.latex_engine.as_str()),
            "ontology_path" => Some(self.ontology_path.as_str()),
            _ => None,
        }
    }

    /// Get a configuration value with default
    /// FUTURE: Used for fallback configuration
    #[allow(dead_code)]
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or(default).to_string()
    }

    /// Set a configuration value (returns new Config - immutable)
    /// FUTURE: Used for config set persistence
    #[allow(dead_code)]
    pub fn with_value(&self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut new_config = self.clone();
        let key_str = key.into();
        let value_str = value.into();
        match key_str.as_str() {
            "output_dir" => new_config.output_dir = value_str,
            "default_family" => new_config.default_family = value_str,
            "latex_engine" => new_config.latex_engine = value_str,
            "ontology_path" => new_config.ontology_path = value_str,
            _ => {}
        }
        new_config
    }

    /// Get all configuration entries
    pub fn all_entries(&self) -> Vec<(&str, &str)> {
        vec![
            ("output_dir", self.output_dir.as_str()),
            ("default_family", self.default_family.as_str()),
            ("latex_engine", self.latex_engine.as_str()),
            ("ontology_path", self.ontology_path.as_str()),
        ]
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
