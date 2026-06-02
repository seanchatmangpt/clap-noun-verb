// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cargo metadata inspection adapter
//!
//! Reads workspace structure, crate information, and build profiles.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Cargo metadata inspection
pub struct CargoMetadata;

impl CargoMetadata {
    /// Load workspace metadata from Cargo.toml
    pub fn load(workspace_root: Option<PathBuf>) -> Result<WorkspaceInfo> {
        let root = workspace_root.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(root.join("Cargo.toml"))
            .exec()
            .context("Failed to load workspace metadata")?;

        let root = metadata.workspace_root.clone().into();
        let target_directory = metadata.target_directory.clone().into();
        let members = metadata
            .workspace_members
            .iter()
            .map(|id| metadata[id].name.clone())
            .collect();

        Ok(WorkspaceInfo {
            root,
            members,
            target_directory,
        })
    }

    /// Get all crates in workspace
    pub fn crates(workspace_root: Option<PathBuf>) -> Result<Vec<CrateInfo>> {
        let root = workspace_root.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(root.join("Cargo.toml"))
            .exec()
            .context("Failed to load crate metadata")?;

        let crates: Vec<_> = metadata
            .workspace_members
            .iter()
            .map(|id| {
                let pkg = &metadata[id];
                CrateInfo {
                    name: pkg.name.clone(),
                    version: pkg.version.to_string(),
                    manifest_path: pkg.manifest_path.clone().into(),
                }
            })
            .collect();

        Ok(crates)
    }
}

/// Workspace information
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub members: Vec<String>,
    pub target_directory: PathBuf,
}

/// Crate information
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub manifest_path: PathBuf,
}
