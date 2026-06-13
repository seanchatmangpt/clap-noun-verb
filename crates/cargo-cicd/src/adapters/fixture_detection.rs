// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Trybuild fixture detection and selection
//!
//! Identifies changed UI test snapshots and compile-fail artifacts,
//! avoiding full fixture suite runs when only specific fixtures changed.

use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// Fixture detection for trybuild
pub struct FixtureDetection;

#[derive(Debug, Clone, Serialize)]
pub struct FixtureInfo {
    pub changed_fixtures: Vec<String>,
    pub total_fixtures: usize,
    pub scope: FixtureScope,
}

#[derive(Debug, Clone, Serialize)]
pub enum FixtureScope {
    None,
    Partial,
    Full,
}

impl FixtureDetection {
    /// Detect changed fixtures from git changes
    pub fn detect(changed_files: &[String]) -> Result<FixtureInfo> {
        let changed_fixtures: Vec<String> =
            changed_files.iter().filter(|f| f.contains("trybuild/")).map(|f| f.clone()).collect();

        // Count total fixtures in the project
        let total_fixtures = Self::count_total_fixtures()?;

        let scope = if changed_fixtures.is_empty() {
            FixtureScope::None
        } else if changed_fixtures.len() < total_fixtures / 2 {
            FixtureScope::Partial
        } else {
            FixtureScope::Full
        };

        Ok(FixtureInfo { changed_fixtures, total_fixtures, scope })
    }

    /// List all fixtures that need updating
    pub fn list_fixtures_for_update(fixture_root: PathBuf) -> Result<Vec<String>> {
        let mut fixtures = vec![];

        if !fixture_root.exists() {
            return Ok(fixtures);
        }

        Self::walk_fixtures(&fixture_root, &mut fixtures)?;
        Ok(fixtures)
    }

    fn count_total_fixtures() -> Result<usize> {
        let trybuild_path = PathBuf::from("tests/ui");
        if !trybuild_path.exists() {
            return Ok(0);
        }

        let count = fs::read_dir(&trybuild_path)
            .context("Failed to read trybuild directory")?
            .flatten()
            .filter(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    metadata.is_file() && entry.file_name().to_string_lossy().ends_with(".rs")
                } else {
                    false
                }
            })
            .count();

        Ok(count)
    }

    fn walk_fixtures(path: &PathBuf, fixtures: &mut Vec<String>) -> Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".rs") {
                            fixtures.push(name.to_string());
                        }
                    }
                } else if metadata.is_dir() {
                    Self::walk_fixtures(&entry.path(), fixtures)?;
                }
            }
        }
        Ok(())
    }
}
