//! Built-in doctor command for epistemic, environment, and configuration validation
//!
//! This module provides a pre-packaged command for verifying that the
//! application environment, configuration, and dependencies are truthful,
//! reproducible, and lawful. It implements a medical pathology and diagnostic model.

use clap::{Arg, ArgAction, ArgMatches, Command};
#[cfg(feature = "config-formats")]
use crate::config::ConfigLoader;
use crate::error::{NounVerbError, Result};
use serde::Serialize;
use std::env;
use std::fmt;
use std::process::Command as ProcessCommand;

// ============================================================================
// DIAGNOSTIC TAXONOMY (Pathology & Severity)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "ℹ️  INFO"),
            Severity::Warning => write!(f, "⚠️  WARNING"),
            Severity::Error => write!(f, "❌ ERROR"),
            Severity::Critical => write!(f, "🚨 CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Pathology {
    pub name: &'static str,
    pub severity: Severity,
    pub symptoms: Vec<String>,
    pub diagnosis: String,
    pub treatment: String,
    pub auto_repairable: bool,
}

impl Pathology {
    pub fn new(
        name: &'static str,
        severity: Severity,
        diagnosis: impl Into<String>,
        treatment: impl Into<String>,
    ) -> Self {
        Self {
            name,
            severity,
            symptoms: Vec::new(),
            diagnosis: diagnosis.into(),
            treatment: treatment.into(),
            auto_repairable: false,
        }
    }

    pub fn with_symptom(mut self, symptom: impl Into<String>) -> Self {
        self.symptoms.push(symptom.into());
        self
    }

    pub fn repairable(mut self, can_repair: bool) -> Self {
        self.auto_repairable = can_repair;
        self
    }
}

// ============================================================================
// DIAGNOSTIC ENGINE
// ============================================================================

#[derive(Serialize)]
pub struct DiagnosticReport {
    pub healthy: bool,
    pub fix_requested: bool,
    pub pathologies: Vec<Pathology>,
}

pub struct DiagnosticEngine {
    pathologies: Vec<Pathology>,
    fix_requested: bool,
    quiet: bool,
}

impl DiagnosticEngine {
    pub fn new(fix_requested: bool, quiet: bool) -> Self {
        Self { pathologies: Vec::new(), fix_requested, quiet }
    }

    pub fn record(&mut self, pathology: Pathology) {
        self.pathologies.push(pathology);
    }

    pub fn has_errors(&self) -> bool {
        self.pathologies.iter().any(|p| p.severity >= Severity::Error)
    }

    pub fn log(&self, msg: &str) {
        if !self.quiet {
            println!("{}", msg);
        }
    }

    pub fn report_text(&self) {
        if self.pathologies.is_empty() {
            println!("🎉 System is healthy. No pathologies detected.");
            return;
        }

        println!("\n📊 Diagnostic Report ({} Pathologies Found)\n", self.pathologies.len());

        for (i, p) in self.pathologies.iter().enumerate() {
            println!("{}. [{}] {}", i + 1, p.severity, p.name);
            println!("   Diagnosis: {}", p.diagnosis);
            if !p.symptoms.is_empty() {
                println!("   Symptoms:");
                for symptom in &p.symptoms {
                    println!("     - {}", symptom);
                }
            }
            println!("   Treatment: {}", p.treatment);
            if p.auto_repairable {
                println!("   ⚙️  Auto-repair available (run with --fix)");
            }
            println!();
        }
    }

    pub fn report_json(&self) -> Result<()> {
        let report = DiagnosticReport {
            healthy: !self.has_errors(),
            fix_requested: self.fix_requested,
            pathologies: self.pathologies.clone(),
        };
        let json = serde_json::to_string_pretty(&report).map_err(|e| {
            NounVerbError::execution_error(format!("Failed to serialize report: {}", e))
        })?;
        println!("{}", json);
        Ok(())
    }
}

// ============================================================================
// REPAIR HANDLERS
// ============================================================================

/// Run a shell command and return true if successful
fn run_check(cmd: &str, args: &[&str]) -> bool {
    match ProcessCommand::new(cmd).args(args).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Run an auto-repair shell command
fn run_repair(cmd: &str, args: &[&str], desc: &str, quiet: bool) -> bool {
    if !quiet {
        println!("   ⚙️  Applying Treatment: {}...", desc);
    }
    match ProcessCommand::new(cmd).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                if !quiet {
                    println!("     ✅ Treatment successful.");
                }
                true
            } else {
                if !quiet {
                    println!("     ❌ Treatment failed.");
                    if !output.stderr.is_empty() {
                        println!(
                            "        Error: {}",
                            String::from_utf8_lossy(&output.stderr).trim()
                        );
                    }
                }
                false
            }
        }
        Err(e) => {
            if !quiet {
                println!("     ❌ Failed to execute treatment: {}", e);
            }
            false
        }
    }
}

// ============================================================================
// CLI INTEGRATION
// ============================================================================

/// Generate the `doctor` command
pub fn doctor_command() -> Command {
    Command::new("doctor")
        .about("Epistemic diagnostic and repair engine for the runtime and deployment stack")
        .arg(
            Arg::new("fix")
                .long("fix")
                .action(ArgAction::SetTrue)
                .help("Attempt autonomic repair on repairable pathologies"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .value_parser(["text", "json"])
                .default_value("text")
                .help("Output format (text or json)"),
        )
}

/// Handle the doctor command
pub fn handle_doctor_command(matches: &ArgMatches) -> Result<()> {
    let fix = matches.get_flag("fix");
    let format = matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("text");
    let is_json = format == "json";

    if !is_json {
        println!("🩺 clap-noun-verb Diagnostics & Repair Engine");
        println!("=============================================\n");
        if fix {
            println!("⚙️  Autonomic repair mode enabled.\n");
        }
    }

    let mut engine = DiagnosticEngine::new(fix, is_json);

    // 1. CONFIGURATION CHECKS (ConfigDrift, UnmappedKeys)
    #[cfg(feature = "config-formats")]
    check_configuration(&mut engine, fix);

    // 2. ENVIRONMENT CHECKS (EnvMissing)
    check_environment(&mut engine);

    // 3. REGISTRY CHECKS (RegistryLockFailed)
    check_registry(&mut engine);

    // 4. SOURCE & DEPLOYMENT TRUTH (GitDirty, FormattingDegraded, TestsFailing)
    if std::path::Path::new("Cargo.toml").exists() {
        check_source_truth(&mut engine, fix);
    }

    // 5. AI/WIZARD INTEGRATION (AiUnreachable)
    #[cfg(feature = "wizard")]
    check_ai_integration(&mut engine);

    // Report results
    if is_json {
        engine.report_json()?;
    } else {
        engine.report_text();
        if engine.has_errors() {
            println!("⚠️  System cannot be fully trusted here. Please address the errors above.");
        } else if !engine.pathologies.is_empty() {
            println!("ℹ️  System is trustworthy, but has minor warnings/info.");
        }
    }

    if engine.has_errors() {
        return Err(NounVerbError::execution_error(
            "Doctor discovered critical pathologies that prevent system trust.",
        ));
    }

    Ok(())
}

// ============================================================================
// SPECIFIC CHECK IMPLEMENTATIONS
// ============================================================================

#[cfg(feature = "config-formats")]
fn check_configuration(engine: &mut DiagnosticEngine, _fix: bool) {
    engine.log("🔍 Scanning Configuration...");
    let loader = ConfigLoader::new();
    if let Some(path) = loader.find_config_path() {
        match loader.load() {
            Ok(config) => {
                let registry = crate::cli::registry::CommandRegistry::get();
                if let Ok(reg) = registry.lock() {
                    // Check for unmapped keys (ConfigDrift)
                    let cmd = reg.build_command();
                    let flat_map = config.to_flat_map();

                    let mut valid_args = std::collections::HashSet::new();
                    collect_valid_args(&cmd, &mut valid_args, "");

                    let mut unknown_keys = Vec::new();
                    for key in flat_map.keys() {
                        if !valid_args.contains(key) {
                            unknown_keys.push(key.clone());
                        }
                    }

                    if !unknown_keys.is_empty() {
                        let mut pathology = Pathology::new(
                            "ConfigDrift",
                            Severity::Warning,
                            format!("Found {} unknown configuration keys in {}", unknown_keys.len(), path.display()),
                            "Remove unrecognized keys from the configuration file or register them as valid arguments in the CLI builder."
                        );
                        for key in unknown_keys {
                            pathology = pathology.with_symptom(format!("Unknown key: '{}'", key));
                        }
                        engine.record(pathology);
                    }
                }
            }
            Err(e) => {
                engine.record(
                    Pathology::new(
                        "ConfigParseFailed",
                        Severity::Error,
                        "Failed to parse configuration file.",
                        "Fix syntax errors in the configuration file.",
                    )
                    .with_symptom(e.to_string()),
                );
            }
        }
    } else {
        engine.record(
            Pathology::new(
                "ConfigMissing",
                Severity::Info,
                "No standard configuration file found (e.g., clap-nv.toml).",
                "Run `config init` to scaffold a default configuration file if desired.",
            )
            .repairable(true),
        );
    }
}

#[cfg(feature = "config-formats")]
fn collect_valid_args(
    cmd: &clap::Command,
    valid_args: &mut std::collections::HashSet<String>,
    prefix: &str,
) {
    for arg in cmd.get_arguments() {
        if let Some(long) = arg.get_long() {
            let key =
                if prefix.is_empty() { long.to_string() } else { format!("{}.{}", prefix, long) };
            valid_args.insert(key);
        }
    }

    for sub in cmd.get_subcommands() {
        let sub_name = sub.get_name();
        let new_prefix = if prefix.is_empty() {
            sub_name.to_string()
        } else {
            format!("{}.{}", prefix, sub_name)
        };
        collect_valid_args(sub, valid_args, &new_prefix);
    }
}

fn check_environment(engine: &mut DiagnosticEngine) {
    engine.log("🔍 Scanning Environment Profiles...");
    if let Err(_) = env::var("APP_ENV") {
        engine.record(Pathology::new(
            "EnvProfileUndefined",
            Severity::Info,
            "APP_ENV is not set. The system will use the [default] profile.",
            "Set APP_ENV (e.g., export APP_ENV=dev) to load environment-specific overrides.",
        ));
    }
}

fn check_registry(engine: &mut DiagnosticEngine) {
    engine.log("🔍 Scanning Semantic Registry...");
    let registry = crate::cli::registry::CommandRegistry::get();
    match registry.lock() {
        Ok(reg) => {
            let nouns = reg.get_nouns();
            if nouns.is_empty() {
                engine.record(Pathology::new(
                    "RegistryEmpty",
                    Severity::Warning,
                    "Command registry locked successfully, but no nouns are registered.",
                    "Ensure #[noun] and #[verb] macros are applied correctly or manually register commands."
                ));
            }
        }
        Err(_) => {
            engine.record(Pathology::new(
                "RegistryLockFailed",
                Severity::Critical,
                "Failed to acquire lock on the global CommandRegistry.",
                "Investigate potential deadlocks or threading issues during CLI initialization.",
            ));
        }
    }
}

fn check_source_truth(engine: &mut DiagnosticEngine, fix: bool) {
    engine.log("🔍 Scanning Codebase Epistemics (Source, Tests, Deployability)...");

    // Formatting
    if !run_check("cargo", &["fmt", "--", "--check"]) {
        let mut path = Pathology::new(
            "FormattingDegraded",
            Severity::Warning,
            "Source code formatting deviates from rustfmt standards.",
            "Run `cargo fmt` to repair codebase formatting.",
        )
        .repairable(true);

        if fix {
            if run_repair("cargo", &["fmt"], "cargo fmt", engine.quiet) {
                path.severity = Severity::Info;
                path.diagnosis = "Formatting was degraded but has been repaired.".to_string();
            } else {
                path = path.with_symptom("Auto-repair failed.");
            }
        }
        engine.record(path);
    }

    // Compilation
    if !run_check("cargo", &["check"]) {
        engine.record(Pathology::new(
            "CompilationFailed",
            Severity::Critical,
            "Source codebase fails to compile.",
            "Run `cargo check` manually and resolve compiler errors.",
        ));
    }

    // Tests (Only run if tests are requested to prevent long hangs on large projects, 
    // or run them silently but report if they fail)
    // We only check tests if compilation passes to avoid duplicate noisy output
    if !engine.has_errors() {
        engine.log("  ⏳ Running test suite... (this may take a moment)");
        if !run_check("cargo", &["test", "--all-features"]) {
            engine.record(Pathology::new(
                "TestsFailing",
                Severity::Error,
                "Test suite is failing. The system artifact cannot be fully trusted.",
                "Run `cargo test` and fix the failing regressions before deployment.",
            ));
        }
    }

    // Git Status
    match ProcessCommand::new("git").args(&["status", "--porcelain"]).output() {
        Ok(output) => {
            if !output.stdout.is_empty() {
                engine.record(Pathology::new(
                    "GitDirty",
                    Severity::Warning,
                    "Working tree contains uncommitted changes. Deployment reproducibility is compromised.",
                    "Commit or stash changes to ensure a clean, reproducible artifact."
                ).with_symptom(format!("{} modified file(s)", output.stdout.split(|&c| c == b'\n').count() - 1)));
            }
        }
        Err(_) => {
            engine.record(Pathology::new(
                "GitUnavailable",
                Severity::Info,
                "Git executable is missing or repository is uninitialized.",
                "Install Git or initialize a repository if tracking is desired.",
            ));
        }
    }
}

#[cfg(feature = "wizard")]
fn check_ai_integration(engine: &mut DiagnosticEngine) {
    engine.log("🔍 Scanning AI Ecosystem Integration...");
    let keys_to_check = ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GEMINI_API_KEY"];
    let mut found_keys = Vec::new();

    for key in keys_to_check {
        if env::var(key).is_ok() {
            found_keys.push(key);
        }
    }

    if found_keys.is_empty() {
        engine.record(Pathology::new(
            "AiUnreachable",
            Severity::Warning,
            "No AI provider API keys found in the environment. Wizard capabilities will fail.",
            "Export at least one provider key (e.g., OPENAI_API_KEY) in your environment or .env file."
        ));
    }
}