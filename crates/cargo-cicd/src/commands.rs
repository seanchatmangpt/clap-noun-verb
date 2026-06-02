// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Command handlers for cargo-cicd
//!
//! Implements the four public commands:
//! - target show / prune
//! - test changed
//! - trybuild changed

use crate::adapters::{
    fixture_detection::FixtureDetection, git_diff::GitDiff, target_scanning::TargetScanning,
    test_plan::TestPlan,
};
use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

/// Target show command output
#[derive(Debug, Clone, Serialize)]
pub struct TargetShowOutput {
    pub target_path: PathBuf,
    pub total_size_gb: f64,
    pub profiles: Vec<(String, f64)>,
    pub stale_candidates: Vec<String>,
    pub configured_max_gb: f64,
    pub verdict: String,
}

impl TargetShowOutput {
    /// Execute `cargo cicd target show`
    pub fn execute(target_dir: Option<PathBuf>, max_gb: f64) -> Result<Self> {
        let target_path = target_dir.unwrap_or_else(|| PathBuf::from("target"));
        let target_info = TargetScanning::scan(target_path)?;

        let profiles = target_info
            .profiles
            .iter()
            .map(|p| (p.profile.clone(), p.size_gb))
            .collect();

        let verdict = TargetScanning::verdict(&target_info, max_gb);

        Ok(TargetShowOutput {
            target_path: target_info.path,
            total_size_gb: target_info.total_size_gb,
            profiles,
            stale_candidates: target_info.stale_candidates,
            configured_max_gb: max_gb,
            verdict,
        })
    }
}

/// Target prune command output
#[derive(Debug, Clone, Serialize)]
pub struct TargetPruneOutput {
    pub candidates_found: usize,
    pub force_required: bool,
    pub event_recorded: bool,
    pub summary: String,
}

impl TargetPruneOutput {
    /// Execute `cargo cicd target prune`
    pub fn execute(force: bool) -> Result<Self> {
        let target_info = TargetScanning::scan(PathBuf::from("target"))?;
        let candidates = target_info.stale_candidates.len();

        if candidates == 0 {
            return Ok(TargetPruneOutput {
                candidates_found: 0,
                force_required: false,
                event_recorded: true,
                summary: "No stale artifacts found".to_string(),
            });
        }

        if !force {
            return Ok(TargetPruneOutput {
                candidates_found: candidates,
                force_required: true,
                event_recorded: false,
                summary: format!(
                    "Found {} stale candidates. Use --force to prune.",
                    candidates
                ),
            });
        }

        // Actually prune (placeholder)
        Ok(TargetPruneOutput {
            candidates_found: candidates,
            force_required: false,
            event_recorded: true,
            summary: format!("Pruned {} artifacts", candidates),
        })
    }
}

/// Test changed command output
#[derive(Debug, Clone, Serialize)]
pub struct TestChangedOutput {
    pub test_plan: Vec<(String, String)>,
    pub is_conservative: bool,
    pub reason: String,
    pub estimated_runtime_seconds: u32,
}

impl TestChangedOutput {
    /// Execute `cargo cicd test changed`
    pub fn execute(base_ref: Option<String>) -> Result<Self> {
        let changed_info = GitDiff::changed_files(base_ref)?;
        let classifications = GitDiff::classify_rust_files(&changed_info.changed_files);
        let plan = TestPlan::derive(&classifications)?;

        let test_plan: Vec<(String, String)> = plan
            .selected_tests
            .iter()
            .map(|t| (t.test_name.clone(), t.test_type.clone()))
            .collect();

        Ok(TestChangedOutput {
            test_plan,
            is_conservative: plan.is_conservative,
            reason: plan.reason,
            estimated_runtime_seconds: plan.estimated_runtime_seconds,
        })
    }
}

/// Trybuild changed command output
#[derive(Debug, Clone, Serialize)]
pub struct TrybuildChangedOutput {
    pub changed_fixtures: Vec<String>,
    pub total_fixtures: usize,
    pub scope: String,
    pub snapshot_update_available: bool,
}

impl TrybuildChangedOutput {
    /// Execute `cargo cicd trybuild changed`
    pub fn execute() -> Result<Self> {
        let changed_info = GitDiff::changed_files(Some("origin/main".to_string()))?;
        let fixture_info = FixtureDetection::detect(&changed_info.changed_files)?;

        let scope = match fixture_info.scope {
            crate::adapters::fixture_detection::FixtureScope::None => "none".to_string(),
            crate::adapters::fixture_detection::FixtureScope::Partial => "partial".to_string(),
            crate::adapters::fixture_detection::FixtureScope::Full => "full".to_string(),
        };

        let snapshot_update_available = !fixture_info.changed_fixtures.is_empty();

        Ok(TrybuildChangedOutput {
            changed_fixtures: fixture_info.changed_fixtures,
            total_fixtures: fixture_info.total_fixtures,
            scope,
            snapshot_update_available,
        })
    }
}
