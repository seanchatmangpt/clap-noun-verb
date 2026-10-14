// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pack commands - Add and remove capability packages

use serde::{Deserialize, Serialize};

/// Result from pack add operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackAddedOutput {
    /// Generated package ID.
    pub id: String,
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Operation status (set to "added").
    pub status: String,
}

impl PackAddedOutput {
    /// Create an "added" result for the given package ID, name, and version.
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            status: "added".to_string(),
        }
    }
}

/// Result from pack remove operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackRemovedOutput {
    /// ID of the removed package.
    pub removed_id: String,
    /// Operation status (set to "removed").
    pub status: String,
    /// Human-readable confirmation message.
    pub message: String,
}

impl PackRemovedOutput {
    /// Create a "removed" result for the given package ID.
    pub fn new(removed_id: impl Into<String>) -> Self {
        Self {
            removed_id: removed_id.into(),
            status: "removed".to_string(),
            message: "Package successfully removed from registry".to_string(),
        }
    }
}

/// Add a capability package to the registry
///
/// Registers a new capability package in the capability registry.
/// Packages represent reusable ontology components and semantic capabilities.
///
/// Uses the generated registry implementation for optimized package management.
///
/// # Arguments
/// * `name` - Package name (must be unique)
/// * `version` - Version string (e.g., "1.0.0")
///
/// # Example
/// ```text
/// myapp pack add GraphUtils 1.2.0
/// ```
pub fn add_package(name: String, version: String) -> crate::Result<PackAddedOutput> {
    let registry = super::impl_generated::GeneratedRegistry::new();
    let (pkg_id, pkg_name, pkg_version) = registry.register_package(&name, &version)?;
    Ok(PackAddedOutput::new(pkg_id, pkg_name, pkg_version))
}

/// Remove a capability package from the registry
///
/// Unregisters a capability package from the registry.
/// The package is removed along with all its associated metadata.
///
/// Uses the generated registry implementation for optimized package management.
///
/// # Arguments
/// * `id` - Package ID (e.g., "pkg-graphutils")
///
/// # Example
/// ```text
/// myapp pack remove pkg-graphutils
/// ```
pub fn remove_package(id: String) -> crate::Result<PackRemovedOutput> {
    let registry = super::impl_generated::GeneratedRegistry::new();
    let removed_id = registry.unregister_package(&id)?;
    Ok(PackRemovedOutput::new(removed_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_added_output() {
        let output = PackAddedOutput::new("pkg-001", "GraphUtils", "2.1.0");
        assert_eq!(output.id, "pkg-001");
        assert_eq!(output.name, "GraphUtils");
        assert_eq!(output.version, "2.1.0");
        assert_eq!(output.status, "added");
    }

    #[test]
    fn test_pack_removed_output() {
        let output = PackRemovedOutput::new("pkg-001");
        assert_eq!(output.removed_id, "pkg-001");
        assert_eq!(output.status, "removed");
    }
}
