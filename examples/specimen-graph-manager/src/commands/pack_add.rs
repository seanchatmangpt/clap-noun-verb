// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pack Add Command - Add capability package to registry

use crate::output_models::PackAddedOutput;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Validate semantic version format
fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Domain logic: Generate package ID from name
fn generate_package_id(name: &str) -> String {
    format!(
        "pkg-{}",
        name.to_lowercase().replace(' ', "-").chars().take(20).collect::<String>()
    )
}

/// Domain logic: Validate and register package
fn register_package(name: &str, version: &str) -> Result<(String, String, String)> {
    if name.trim().is_empty() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Package name cannot be empty".to_string(),
        )
        .into());
    }

    if version.trim().is_empty() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Version cannot be empty".to_string(),
        )
        .into());
    }

    if !is_valid_version(version) {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Invalid version format. Use semantic versioning (e.g., 1.0.0)".to_string(),
        )
        .into());
    }

    let package_id = generate_package_id(name);
    Ok((package_id, name.to_string(), version.to_string()))
}

/// Add a capability package to the registry
///
/// Registers a new capability package in the capability registry.
/// Packages represent reusable ontology components and semantic capabilities.
///
/// # Arguments
/// * `name` - Package name (must be unique)
/// * `version` - Version string (e.g., "1.0.0")
///
/// # Example
/// ```text
/// specimen-graph-manager pack add GraphUtils 1.2.0
/// ```
#[verb("add", "pack")]
fn add_package(name: String, version: String) -> Result<PackAddedOutput> {
    let (pkg_id, pkg_name, pkg_version) = register_package(&name, &version)?;
    Ok(PackAddedOutput::new(pkg_id, pkg_name, pkg_version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_version() {
        assert!(is_valid_version("1.0.0"));
        assert!(is_valid_version("26.6.1"));
        assert!(!is_valid_version("1.0"));
    }

    #[test]
    fn test_generate_package_id() {
        assert_eq!(generate_package_id("TestPkg"), "pkg-testpkg");
        assert_eq!(generate_package_id("Graph Utils"), "pkg-graph-utils");
    }

    #[test]
    fn test_register_package_success() {
        let result = register_package("TestPkg", "1.0.0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_package_empty_name() {
        let result = register_package("", "1.0.0");
        assert!(result.is_err());
    }
}
