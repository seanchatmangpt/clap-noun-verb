// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! cargo-cicd: Cargo CI/CD toolkit
//!
//! Commands:
//! - `cargo cicd target show` — Display target directory info
//! - `cargo cicd target prune` — Remove stale build artifacts
//! - `cargo cicd test changed` — Run tests for changed files
//! - `cargo cicd trybuild changed` — Update trybuild snapshots for changed fixtures

mod adapters;
mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cargo-cicd")]
#[command(about = "Cargo CI/CD toolkit: target, test, and trybuild management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Target management commands
    Target {
        #[command(subcommand)]
        command: TargetCommand,
    },
    /// Test management commands
    Test {
        #[command(subcommand)]
        command: TestCommand,
    },
    /// Trybuild fixture commands
    Trybuild {
        #[command(subcommand)]
        command: TrybuildCommand,
    },
}

#[derive(Subcommand)]
enum TargetCommand {
    /// Display target directory information
    Show {
        /// Path to target directory (default: ./target)
        #[arg(short, long)]
        target_dir: Option<PathBuf>,

        /// Maximum configured size in GB (default: 10.0)
        #[arg(short, long, default_value = "10.0")]
        max_gb: f64,
    },
    /// Prune stale artifacts from target directory
    Prune {
        /// Force prune without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum TestCommand {
    /// Show test plan for changed files
    Changed {
        /// Base ref for comparison (default: origin/main)
        #[arg(short, long)]
        base_ref: Option<String>,
    },
}

#[derive(Subcommand)]
enum TrybuildCommand {
    /// Show changed fixtures needing snapshot update
    Changed,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Target { command } => handle_target(command),
        Commands::Test { command } => handle_test(command),
        Commands::Trybuild { command } => handle_trybuild(command),
    }
}

fn handle_target(command: TargetCommand) -> Result<()> {
    match command {
        TargetCommand::Show { target_dir, max_gb } => {
            let output = commands::TargetShowOutput::execute(target_dir, max_gb)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        TargetCommand::Prune { force } => {
            let output = commands::TargetPruneOutput::execute(force)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
    }
}

fn handle_test(command: TestCommand) -> Result<()> {
    match command {
        TestCommand::Changed { base_ref } => {
            let output = commands::TestChangedOutput::execute(base_ref)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
    }
}

fn handle_trybuild(command: TrybuildCommand) -> Result<()> {
    match command {
        TrybuildCommand::Changed => {
            let output = commands::TrybuildChangedOutput::execute()?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
    }
}
