//! Configuration loading integration layer
//!
//! Handles loading, merging, and validating ggen.toml configuration files.
//! Supports default -> user -> project config layering with workspace detection.

use crate::integration::workspace::WorkspaceDetector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Default ggen configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgenConfig {
    pub version: Option<u32>,
    pub sync_profile: Option<String>,
    pub registry_url: Option<String>,
    pub packs_dir: Option<String>,
    pub receipts_dir: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for GgenConfig {
    fn default() -> Self {
        Self {
            version: Some(2),
            sync_profile: Some("default".to_string()),
            registry_url: Some("https://registry.ggen.dev".to_string()),
            packs_dir: None,
            receipts_dir: None,
            extra: HashMap::new(),
        }
    }
}

/// Configuration loader with workspace detection and layering
pub struct ConfigLoader {
    workspace_root: PathBuf,
}

/// Config filename
pub const CONFIG_FILENAME: &str = "ggen.toml";

impl ConfigLoader {
    /// Create by detecting workspace root
    pub fn detect() -> Result<Self, String> {
        let workspace_root = WorkspaceDetector::find_workspace_root()?;
        Ok(Self::new(workspace_root))
    }

    /// Create for specific workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Get config file path
    pub fn config_path(&self) -> PathBuf {
        self.workspace_root.join(CONFIG_FILENAME)
    }

    /// Check if config file exists
    pub fn config_exists(&self) -> bool {
        self.config_path().exists()
    }

    /// Load config from workspace (returns defaults if not found)
    pub fn load(&self) -> Result<GgenConfig, String> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(GgenConfig::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: GgenConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    /// Save config to workspace
    pub fn save(&self, config: &GgenConfig) -> Result<(), String> {
        let path = self.config_path();
        let content = toml::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    /// Get workspace root
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::detect().unwrap_or_else(|_| Self {
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        })
    }
}
