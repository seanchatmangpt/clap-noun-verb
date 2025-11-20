//! Stable capability identifiers for swarm-native CLI
//!
//! In a world with trillions of agents, capability IDs provide stable references
//! that survive cosmetic renames and allow for safe evolution of the command ontology.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A stable, opaque identifier for a noun or verb capability
///
/// Capability IDs are immutable once assigned and survive renames,
/// allowing agents to bind to stable capability contracts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(String);

impl CapabilityId {
    /// Create a new capability ID from a string
    ///
    /// This should be a stable, opaque identifier (e.g., UUID, hash-based ID).
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a deterministic capability ID from a path
    ///
    /// This creates a stable ID based on the fully-qualified path (e.g., "services.restart").
    /// The ID is a SHA-256 hash truncated to 16 characters for readability.
    pub fn from_path(path: impl AsRef<str>) -> Self {
        let hash = Sha256::digest(path.as_ref().as_bytes());
        let truncated = hex::encode(&hash[..8]); // First 8 bytes = 16 hex chars
        Self(format!("cap_{}", truncated))
    }

    /// Generate a capability ID with a version suffix
    ///
    /// This allows tracking capability versions independently of binary versions.
    pub fn from_path_versioned(path: impl AsRef<str>, version: impl AsRef<str>) -> Self {
        let combined = format!("{}@{}", path.as_ref(), version.as_ref());
        let hash = Sha256::digest(combined.as_bytes());
        let truncated = hex::encode(&hash[..8]);
        Self(format!("cap_{}_{}", truncated, version.as_ref().replace('.', "_")))
    }

    /// Get the underlying ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a versioned capability ID
    pub fn is_versioned(&self) -> bool {
        self.0.contains('_') && self.0.split('_').count() > 2
    }

    /// Extract version from a versioned capability ID
    pub fn version(&self) -> Option<String> {
        if !self.is_versioned() {
            return None;
        }

        // Format is: cap_{hash}_{version_with_underscores}
        // Skip first two parts and join the rest with dots
        let parts: Vec<&str> = self.0.split('_').collect();
        if parts.len() > 2 {
            Some(parts[2..].join("."))
        } else {
            None
        }
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CapabilityId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CapabilityId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Capability version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityVersion {
    /// Semantic version (e.g., "1.2.3")
    pub version: String,
    /// Capability ID for this version
    pub capability_id: CapabilityId,
    /// Whether this version is stable
    pub stable: bool,
    /// Deprecation information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<DeprecationInfo>,
}

impl CapabilityVersion {
    /// Create a new capability version
    pub fn new(version: impl Into<String>, capability_id: CapabilityId) -> Self {
        Self { version: version.into(), capability_id, stable: true, deprecated: None }
    }

    /// Mark this version as unstable
    pub fn unstable(mut self) -> Self {
        self.stable = false;
        self
    }

    /// Mark this version as deprecated
    pub fn deprecated(mut self, info: DeprecationInfo) -> Self {
        self.deprecated = Some(info);
        self
    }
}

/// Information about capability deprecation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeprecationInfo {
    /// When the capability was deprecated (ISO 8601 date)
    pub deprecated_on: String,
    /// When the capability will be removed (ISO 8601 date)
    pub removed_on: Option<String>,
    /// Replacement capability ID
    pub replacement: Option<CapabilityId>,
    /// Migration instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration_guide: Option<String>,
}

impl DeprecationInfo {
    /// Create new deprecation info
    pub fn new(deprecated_on: impl Into<String>) -> Self {
        Self {
            deprecated_on: deprecated_on.into(),
            removed_on: None,
            replacement: None,
            migration_guide: None,
        }
    }

    /// Set removal date
    pub fn will_be_removed(mut self, date: impl Into<String>) -> Self {
        self.removed_on = Some(date.into());
        self
    }

    /// Set replacement capability
    pub fn replaced_by(mut self, capability_id: CapabilityId) -> Self {
        self.replacement = Some(capability_id);
        self
    }

    /// Set migration guide
    pub fn with_migration_guide(mut self, guide: impl Into<String>) -> Self {
        self.migration_guide = Some(guide.into());
        self
    }
}

/// Change type for capability evolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// New capability added (safe)
    Addition,
    /// Existing capability extended with optional fields (safe)
    Extension,
    /// Breaking change to capability (requires migration)
    Breaking,
    /// Capability deprecated but still functional
    Deprecation,
    /// Capability removed
    Removal,
}

/// Capability change record for evolution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChange {
    /// The capability ID affected
    pub capability_id: CapabilityId,
    /// Type of change
    pub change_type: ChangeType,
    /// Version where this change occurred
    pub in_version: String,
    /// Human-readable description
    pub description: String,
    /// Machine-readable details (e.g., argument mappings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl CapabilityChange {
    /// Create a new capability change record
    pub fn new(
        capability_id: CapabilityId,
        change_type: ChangeType,
        in_version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            capability_id,
            change_type,
            in_version: in_version.into(),
            description: description.into(),
            details: None,
        }
    }

    /// Add machine-readable details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Changelog for capability evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChangelog {
    /// CLI version
    pub cli_version: String,
    /// List of changes in this version
    pub changes: Vec<CapabilityChange>,
    /// Whether this release has breaking changes
    pub has_breaking_changes: bool,
}

impl CapabilityChangelog {
    /// Create a new changelog for a version
    pub fn new(cli_version: impl Into<String>) -> Self {
        Self { cli_version: cli_version.into(), changes: Vec::new(), has_breaking_changes: false }
    }

    /// Add a change to the changelog
    pub fn add_change(mut self, change: CapabilityChange) -> Self {
        if change.change_type == ChangeType::Breaking {
            self.has_breaking_changes = true;
        }
        self.changes.push(change);
        self
    }

    /// Check if this changelog has any breaking changes
    pub fn is_breaking(&self) -> bool {
        self.has_breaking_changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_id_from_path() {
        let id1 = CapabilityId::from_path("services.restart");
        let id2 = CapabilityId::from_path("services.restart");
        let id3 = CapabilityId::from_path("services.status");

        // Same path should produce same ID
        assert_eq!(id1, id2);

        // Different paths should produce different IDs
        assert_ne!(id1, id3);

        // IDs should have the expected format
        assert!(id1.as_str().starts_with("cap_"));
    }

    #[test]
    fn test_versioned_capability_id() {
        let id = CapabilityId::from_path_versioned("services.restart", "2.0.0");

        assert!(id.is_versioned());
        assert_eq!(id.version(), Some("2.0.0".to_string()));
    }

    #[test]
    fn test_deprecation_info() {
        let _old_cap = CapabilityId::from_path("old.verb");
        let new_cap = CapabilityId::from_path("new.verb");

        let deprecation = DeprecationInfo::new("2025-01-01")
            .will_be_removed("2026-01-01")
            .replaced_by(new_cap.clone())
            .with_migration_guide("Use new.verb instead");

        assert_eq!(deprecation.deprecated_on, "2025-01-01");
        assert_eq!(deprecation.removed_on, Some("2026-01-01".to_string()));
        assert_eq!(deprecation.replacement, Some(new_cap));
    }

    #[test]
    fn test_capability_changelog() {
        let change1 = CapabilityChange::new(
            CapabilityId::from_path("services.status"),
            ChangeType::Addition,
            "3.0.0",
            "Added new status verb",
        );

        let change2 = CapabilityChange::new(
            CapabilityId::from_path("services.restart"),
            ChangeType::Breaking,
            "3.0.0",
            "Changed restart argument format",
        );

        let changelog = CapabilityChangelog::new("3.0.0").add_change(change1).add_change(change2);

        assert!(changelog.is_breaking());
        assert_eq!(changelog.changes.len(), 2);
    }
}
