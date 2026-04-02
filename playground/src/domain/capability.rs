//! Capability domain - intent surface
//! Capabilities resolve desired surfaces into pack/install/runtime consequences.

use serde::{Deserialize, Serialize};

/// A capability surface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub surface: String,
    pub projection: Option<String>,
    pub runtime: Option<String>,
    pub profile: String,
}

/// Result from capability resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResolution {
    pub capability: Capability,
    pub packs: Vec<String>,
    pub actions: Vec<String>,
    pub needs_policy_check: bool,
}

/// Capability resolver
pub struct CapabilityResolver;

impl CapabilityResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, capability: &Capability) -> Result<CapabilityResolution, String> {
        // Validate constraints: runtime requires projection, projection requires surface
        if capability.runtime.is_some() && capability.projection.is_none() {
            return Err("runtime requires projection".to_string());
        }
        if capability.projection.is_some() && capability.surface.is_empty() {
            return Err("projection requires surface".to_string());
        }

        // TODO: Implement resolution logic
        Ok(CapabilityResolution {
            capability: capability.clone(),
            packs: vec![],
            actions: vec![],
            needs_policy_check: false,
        })
    }
}

impl Default for CapabilityResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Capability {
    pub fn all_available() -> Vec<CapabilityInfo> {
        vec![
            CapabilityInfo {
                name: "mcp".to_string(),
                description: "MCP server capability".to_string(),
            },
            CapabilityInfo {
                name: "projection".to_string(),
                description: "Code projection capability".to_string(),
            },
            CapabilityInfo {
                name: "runtime".to_string(),
                description: "Runtime capability".to_string(),
            },
        ]
    }

    pub fn find(name: &str) -> Result<CapabilityInfo, String> {
        Self::all_available()
            .into_iter()
            .find(|c| c.name == name)
            .ok_or_else(|| format!("Capability not found: {}", name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub name: String,
    pub description: String,
}
