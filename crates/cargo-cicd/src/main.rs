// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! cargo-cicd: Cargo CI/CD toolkit
//!
//! Commands:
//! - `cargo cicd target show` — Display target directory info
//! - `cargo cicd target prune` — Remove stale build artifacts
//! - `cargo cicd test changed` — Run tests for changed files
//! - `cargo cicd trybuild changed` — Update trybuild snapshots for changed fixtures
//! - `cargo cicd git status` — Show git repository status
//! - `cargo cicd git close` — Stage, commit, and optionally push changes
//! - `cargo cicd workspace doctor` — Run comprehensive workspace diagnostics
//! - `cargo cicd status` — Show workspace status and recommendations
//! - `cargo cicd publish` — Emit cicd.toml with process events

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
    /// Git management commands
    Git {
        #[command(subcommand)]
        command: GitCommand,
    },
    /// Workspace health and status commands
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
    /// Show workspace status and recommendations
    Status,
    /// Publish cicd.toml with process state
    Publish,
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

#[derive(Subcommand)]
enum GitCommand {
    /// Show git repository status
    Status,
    /// Stage, commit, and optionally push changes
    Close {
        /// Commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Files to stage (comma-separated)
        #[arg(short, long)]
        files: Option<String>,

        /// Push after commit
        #[arg(short, long)]
        push: bool,
    },
}

#[derive(Subcommand)]
enum WorkspaceCommand {
    /// Run comprehensive workspace diagnostics
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Target { command } => handle_target(command),
        Commands::Test { command } => handle_test(command),
        Commands::Trybuild { command } => handle_trybuild(command),
        Commands::Git { command } => handle_git(command),
        Commands::Workspace { command } => handle_workspace(command),
        Commands::Status => handle_status(),
        Commands::Publish => handle_publish(),
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

fn handle_git(command: GitCommand) -> Result<()> {
    match command {
        GitCommand::Status => {
            let output = commands::GitStatusOutput::execute()?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        GitCommand::Close { message, files, push } => {
            let default_message = "feat(cicd): phase boundary close";
            let commit_msg = message.as_deref().unwrap_or(default_message);
            let files_to_stage =
                files.map(|f| f.split(',').map(|s| s.trim().to_string()).collect());
            let output = commands::GitCloseOutput::execute(commit_msg, files_to_stage, push)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
    }
}

fn handle_workspace(command: WorkspaceCommand) -> Result<()> {
    match command {
        WorkspaceCommand::Doctor => {
            let output = commands::WorkspaceDoctorOutput::execute()?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
    }
}

fn handle_status() -> Result<()> {
    let output = commands::WorkspaceStatusOutput::execute()?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn handle_publish() -> Result<()> {
    let output = commands::PublishOutput::execute()?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
