// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cargo CI/CD toolkit library
//!
//! Local-first CI/CD helpers for Rust workspaces. Provides adapters and commands for:
//!
//! - **Target directory** inspection and pruning (disk usage, stale artifacts)
//! - **Test selection** based on changed files (run only affected tests)
//! - **Trybuild fixture** management (detect stale compiler output snapshots)
//! - **Git state** inspection (clean/dirty, unpushed commits)
//! - **Workspace diagnostics** (health check, build integrity)
//! - **Process event recording** (cicd.toml audit log)
//!
//! All commands output JSON for easy integration with CI systems, shell scripts, and monitoring tools.
//! Everything runs locally without network calls.
//!
//! # Examples
//!
//! ```no_run
//! use cargo_cicd::{TargetScanning, TestPlan};
//! use std::path::PathBuf;
//!
//! // Scan target directory
//! let info = TargetScanning::scan(PathBuf::from("target"))?;
//! println!("Target size: {} GB", info.total_size_gb);
//!
//! // Determine which tests to run
//! let plan = TestPlan::discover()?;
//! println!("Test suites affected: {:?}", plan);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![warn(missing_docs, missing_debug_implementations, unused_results, clippy::all)]
#![allow(clippy::module_name_repetitions)]

pub mod adapters;
pub mod commands;

pub use adapters::{
    CargoMetadata, FixtureDetection, GitCommit, GitDiff, GitStatus, TargetScanning, TestPlan,
    WorkspaceDoctor,
};
pub use commands::{
    GitCloseOutput, GitStatusOutput, PublishOutput, TargetPruneOutput, TargetShowOutput,
    TestChangedOutput, TrybuildChangedOutput, WorkspaceDoctorOutput, WorkspaceStatusOutput,
};
