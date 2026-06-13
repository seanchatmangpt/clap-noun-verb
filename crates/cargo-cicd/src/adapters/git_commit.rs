// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Git commit and phase closing adapter
//!
//! Handles staging files, committing changes, and optional push operations.

use anyhow::{Context, Result};
use serde::Serialize;
use std::process::Command;

/// Git commit operations
pub struct GitCommit;

#[derive(Debug, Clone, Serialize)]
pub struct CommitResult {
    pub success: bool,
    pub commit_hash: Option<String>,
    pub message: String,
    pub event_recorded: bool,
}

impl GitCommit {
    /// Stage and commit configured outputs
    pub fn commit(message: &str, files_to_stage: Option<Vec<String>>) -> Result<CommitResult> {
        // Check for unrelated dirty files
        let dirty_files = Self::get_dirty_files()?;
        let staged_files = Self::get_staged_files()?;

        if let Some(ref to_stage) = files_to_stage {
            // Ensure all dirty files are either being staged or explicitly allowed
            for dirty in &dirty_files {
                if !staged_files.contains(dirty) && !to_stage.contains(dirty) {
                    return Ok(CommitResult {
                        success: false,
                        commit_hash: None,
                        message: format!(
                            "Unrelated dirty files found: {}. Commit discipline requires explicit staging.",
                            dirty
                        ),
                        event_recorded: false,
                    });
                }
            }

            // Stage the specified files
            for file in to_stage {
                Self::stage_file(&file)?;
            }
        }

        // Check if there are staged files
        let final_staged = Self::get_staged_files()?;
        if final_staged.is_empty() {
            return Ok(CommitResult {
                success: false,
                commit_hash: None,
                message: "No files staged for commit".to_string(),
                event_recorded: false,
            });
        }

        // Perform the commit
        let output = Command::new("git")
            .args(&["commit", "-m", message])
            .output()
            .context("Failed to execute git commit")?;

        if output.status.success() {
            let commit_hash = Self::get_current_commit_hash()?;
            Ok(CommitResult {
                success: true,
                commit_hash: Some(commit_hash),
                message: format!("Committed: {}", message),
                event_recorded: true,
            })
        } else {
            Ok(CommitResult {
                success: false,
                commit_hash: None,
                message: format!("Commit failed: {}", String::from_utf8_lossy(&output.stderr)),
                event_recorded: false,
            })
        }
    }

    /// Push to remote if configured
    pub fn push() -> Result<bool> {
        let output =
            Command::new("git").args(&["push"]).output().context("Failed to execute git push")?;

        Ok(output.status.success())
    }

    fn get_dirty_files() -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(&["diff", "--name-only"])
            .output()
            .context("Failed to get dirty files")?;

        Ok(String::from_utf8_lossy(&output.stdout).lines().map(|l| l.to_string()).collect())
    }

    fn get_staged_files() -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(&["diff", "--cached", "--name-only"])
            .output()
            .context("Failed to get staged files")?;

        Ok(String::from_utf8_lossy(&output.stdout).lines().map(|l| l.to_string()).collect())
    }

    fn stage_file(file: &str) -> Result<()> {
        let output =
            Command::new("git").args(&["add", file]).output().context("Failed to stage file")?;

        if !output.status.success() {
            anyhow::bail!("Failed to stage {}: {}", file, String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn get_current_commit_hash() -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .context("Failed to get commit hash")?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
