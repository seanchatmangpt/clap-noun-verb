// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Git status inspection adapter
//!
//! Retrieves repository state including dirty/staged files, branch info, and upstream status.

use anyhow::{Context, Result};
use serde::Serialize;
use std::process::Command;

/// Git status inspection
pub struct GitStatus;

#[derive(Debug, Clone, Serialize)]
pub struct GitStatusInfo {
    pub branch_name: String,
    pub dirty_count: usize,
    pub dirty_sample: Vec<String>,
    pub staged_count: usize,
    pub untracked_count: usize,
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
}

impl GitStatus {
    /// Get git repository status
    pub fn status() -> Result<GitStatusInfo> {
        let branch_name = Self::get_branch_name()?;
        let dirty_info = Self::get_dirty_files()?;
        let staged_count = Self::get_staged_count()?;
        let untracked_count = Self::get_untracked_count()?;
        let (ahead, behind, has_upstream) = Self::get_upstream_status()?;

        Ok(GitStatusInfo {
            branch_name,
            dirty_count: dirty_info.0,
            dirty_sample: dirty_info.1,
            staged_count,
            untracked_count,
            ahead,
            behind,
            has_upstream,
        })
    }

    fn get_branch_name() -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get branch name")?;

        if !output.status.success() {
            anyhow::bail!("git rev-parse failed");
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string())
    }

    fn get_dirty_files() -> Result<(usize, Vec<String>)> {
        let output = Command::new("git")
            .args(&["diff", "--name-only"])
            .output()
            .context("Failed to get dirty files")?;

        if !output.status.success() {
            anyhow::bail!("git diff failed");
        }

        let dirty: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect();

        let count = dirty.len();
        let sample: Vec<String> = dirty.iter().take(5).cloned().collect();

        Ok((count, sample))
    }

    fn get_staged_count() -> Result<usize> {
        let output = Command::new("git")
            .args(&["diff", "--cached", "--name-only"])
            .output()
            .context("Failed to get staged files")?;

        if !output.status.success() {
            anyhow::bail!("git diff --cached failed");
        }

        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count();

        Ok(count)
    }

    fn get_untracked_count() -> Result<usize> {
        let output = Command::new("git")
            .args(&["ls-files", "--others", "--exclude-standard"])
            .output()
            .context("Failed to get untracked files")?;

        if !output.status.success() {
            anyhow::bail!("git ls-files failed");
        }

        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count();

        Ok(count)
    }

    fn get_upstream_status() -> Result<(usize, usize, bool)> {
        let output = Command::new("git")
            .args(&["rev-list", "--left-right", "--count", "HEAD...@{u}"])
            .output()
            .context("Failed to get upstream status")?;

        if !output.status.success() {
            // No upstream tracking configured
            return Ok((0, 0, false));
        }

        let counts = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = counts.trim().split_whitespace().collect();

        if parts.len() == 2 {
            let ahead = parts[0].parse().unwrap_or(0);
            let behind = parts[1].parse().unwrap_or(0);
            Ok((ahead, behind, true))
        } else {
            Ok((0, 0, false))
        }
    }
}
