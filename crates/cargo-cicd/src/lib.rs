// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cargo CI/CD toolkit library
//!
//! Provides adapters and commands for:
//! - Target directory inspection and pruning
//! - Test selection based on changed files
//! - Trybuild fixture management

#![warn(
    missing_docs,
    missing_debug_implementations,
    unused_results,
    clippy::all
)]
#![allow(clippy::module_name_repetitions)]

pub mod adapters;
pub mod commands;

pub use adapters::{CargoMetadata, FixtureDetection, GitDiff, GitStatus, GitCommit, TargetScanning, TestPlan, WorkspaceDoctor};
pub use commands::{
    TargetPruneOutput, TargetShowOutput, TestChangedOutput, TrybuildChangedOutput, GitStatusOutput,
    GitCloseOutput, WorkspaceDoctorOutput, WorkspaceStatusOutput, PublishOutput,
};
