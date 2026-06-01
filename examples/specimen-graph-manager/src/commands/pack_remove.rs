// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pack Remove Command - Remove capability package from registry

use crate::output_models::RemovalStatus;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Check if a package exists in the registry (simulated)
fn package_exists(id: &str) -> bool {
    id.starts_with("pkg-")
}

/// Domain logic: Validate and remove package
fn unregister_package(id: &str) -> Result<String> {
    if id.trim().is_empty() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Package ID cannot be empty".to_string(),
        )
        .into());
    }

    if !id.starts_with("pkg-") {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Invalid package ID format. Must start with 'pkg-'".to_string(),
        )
        .into());
    }

    if !package_exists(id) {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(format!(
            "Package not found in registry: {}",
            id
        ))
        .into());
    }

    Ok(id.to_string())
}

/// Remove a capability package from the registry
///
/// Unregisters a capability package from the registry.
/// The package is removed along with all its associated metadata.
///
/// # Arguments
/// * `id` - Package ID (e.g., "pkg-graphutils")
///
/// # Example
/// ```text
/// specimen-graph-manager pack remove pkg-graphutils
/// ```
#[verb("remove", "pack")]
fn remove_package(id: String) -> Result<RemovalStatus> {
    let removed_id = unregister_package(&id)?;
    Ok(RemovalStatus::new(removed_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_exists() {
        assert!(package_exists("pkg-valid"));
        assert!(!package_exists("invalid"));
    }

    #[test]
    fn test_unregister_package_success() {
        let result = unregister_package("pkg-graphutils");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unregister_package_empty_id() {
        let result = unregister_package("");
        assert!(result.is_err());
    }
}
