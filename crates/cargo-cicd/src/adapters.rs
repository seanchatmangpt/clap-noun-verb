// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Adapters for external systems and tools
//!
//! - cargo_metadata: Parse Cargo.toml and workspace structure
//! - git_diff: Retrieve changed files from git history
//! - target_scanning: Inspect build target directory
//! - test_plan: Derive affected tests from changed files
//! - fixture_detection: Identify trybuild fixtures

pub mod cargo_metadata;
pub mod fixture_detection;
pub mod git_commit;
pub mod git_diff;
pub mod git_status;
pub mod target_scanning;
pub mod test_plan;
pub mod workspace_doctor;

pub use cargo_metadata::CargoMetadata;
pub use fixture_detection::FixtureDetection;
pub use git_commit::GitCommit;
pub use git_diff::GitDiff;
pub use git_status::GitStatus;
pub use target_scanning::TargetScanning;
pub use test_plan::TestPlan;
pub use workspace_doctor::WorkspaceDoctor;
