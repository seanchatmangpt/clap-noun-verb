// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Workspace health assessment adapter
//!
//! Comprehensive workspace diagnostics including metadata, git status, and target directory.

use crate::adapters::{
    cargo_metadata::CargoMetadata, git_status::GitStatus, target_scanning::TargetScanning,
};
use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

/// Workspace health assessment
pub struct WorkspaceDoctor;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceHealth {
    pub verdict: String,
    pub metadata_healthy: bool,
    pub git_healthy: bool,
    pub target_healthy: bool,
    pub target_size_gb: f64,
    pub dirty_files: usize,
    pub untracked_files: usize,
    pub workspace_members: usize,
}

#[derive(Debug, Clone, Serialize)]
pub enum HealthVerdict {
    Healthy,
    Degraded,
    Broken,
}

impl WorkspaceDoctor {
    /// Run comprehensive workspace diagnostic
    pub fn diagnose() -> Result<WorkspaceHealth> {
        let metadata_result = CargoMetadata::load(None);
        let metadata_healthy = metadata_result.is_ok();
        let workspace_members = metadata_result.map(|m| m.members.len()).unwrap_or(0);

        let git_result = GitStatus::status();
        let (dirty_files, untracked_files) = if let Ok(status) = &git_result {
            (status.dirty_count, status.untracked_count)
        } else {
            (0, 0)
        };
        let git_healthy = git_result.is_ok();

        let target_result = TargetScanning::scan(PathBuf::from("target"));
        let target_size_gb = target_result.as_ref().map(|t| t.total_size_gb).unwrap_or(0.0);
        let target_healthy = target_result.is_ok() && target_size_gb < 15.0; // Reasonable threshold

        let verdict = Self::compute_verdict(metadata_healthy, git_healthy, target_healthy);

        Ok(WorkspaceHealth {
            verdict,
            metadata_healthy,
            git_healthy,
            target_healthy,
            target_size_gb,
            dirty_files,
            untracked_files,
            workspace_members,
        })
    }

    fn compute_verdict(metadata: bool, git: bool, target: bool) -> String {
        match (metadata, git, target) {
            (true, true, true) => "healthy".to_string(),
            (false, _, _) | (_, false, _) => "broken".to_string(),
            _ => "degraded".to_string(),
        }
    }
}
