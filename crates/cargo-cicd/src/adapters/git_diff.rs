// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Git diff inspection adapter
//!
//! Retrieves changed files from git history for a given base ref.

use anyhow::{Context, Result};
use std::process::Command;

/// Git diff inspection
pub struct GitDiff;

#[derive(Debug, Clone)]
pub struct ChangedFilesInfo {
    pub changed_files: Vec<String>,
    pub base_ref: String,
    #[allow(dead_code)]
    pub contains_rust_changes: bool,
    #[allow(dead_code)]
    pub contains_test_changes: bool,
    #[allow(dead_code)]
    pub contains_fixture_changes: bool,
}

impl GitDiff {
    /// Get changed files since base ref (default: origin/main)
    pub fn changed_files(base_ref: Option<String>) -> Result<ChangedFilesInfo> {
        let base = base_ref.unwrap_or_else(|| "origin/main".to_string());

        let output = Command::new("git")
            .args(&["diff", "--name-only", &base, "HEAD"])
            .output()
            .context("Failed to execute git diff")?;

        if !output.status.success() {
            anyhow::bail!("git diff failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let changed_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        let contains_rust_changes = changed_files.iter().any(|f| f.ends_with(".rs"));
        let contains_test_changes = changed_files.iter().any(|f| f.contains("tests/"));
        let contains_fixture_changes = changed_files.iter().any(|f| f.contains("trybuild/"));

        Ok(ChangedFilesInfo {
            changed_files,
            base_ref: base,
            contains_rust_changes,
            contains_test_changes,
            contains_fixture_changes,
        })
    }

    /// Classify changed Rust files
    pub fn classify_rust_files(changed_files: &[String]) -> Vec<FileClassification> {
        changed_files
            .iter()
            .filter(|f| f.ends_with(".rs"))
            .map(|f| {
                let is_test = f.contains("tests/") || f.ends_with("_test.rs");
                let is_lib = f.contains("src/lib.rs") || f.contains("src/mod.rs");
                let is_macro = f.contains("macro") || f.contains("derive");
                let module_path = Self::extract_module_path(f);

                FileClassification {
                    path: f.clone(),
                    is_test,
                    is_lib,
                    is_macro,
                    module_path,
                }
            })
            .collect()
    }

    fn extract_module_path(file_path: &str) -> String {
        if let Some(src_idx) = file_path.find("src/") {
            let rest = &file_path[src_idx + 4..];
            rest.replace("/", "::")
                .replace(".rs", "")
                .replace("mod", "")
        } else {
            file_path.to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileClassification {
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub is_test: bool,
    pub is_lib: bool,
    pub is_macro: bool,
    pub module_path: String,
}
