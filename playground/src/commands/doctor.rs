//! Doctor commands - diagnostic surface
//!
//! Detects workspace integrity, lockfile truth, pack integrity, trust/profile conflicts.

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

use crate::domain::doctor::{Doctor, DiagnosticCheck};
use crate::outputs::{DoctorRunOutput, DoctorCheckOutput, DoctorEnvOutput};

/// Run all diagnostics
///
/// Runs all diagnostic checks and optionally auto-fixes issues.
///
/// # Arguments
/// * `fix` - Automatically fix issues found
#[verb("run")]
fn doctor_run(
    fix: bool,
) -> Result<DoctorRunOutput> {
    let doctor = Doctor::new()?;
    let results = doctor.run_all_diagnostics()?;

    if fix {
        doctor.auto_fix(&results)?;
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();

    Ok(DoctorRunOutput {
        checks_run: results.len(),
        passed,
        failed,
        results,
    })
}

/// Run specific check
///
/// Runs a single diagnostic check by name.
///
/// # Arguments
/// * `check_name` - Name of check to run (workspace-integrity, lockfile-exists, pack-integrity, policy-conflicts)
/// * `fix` - Automatically fix issues found
#[verb("check")]
fn doctor_check(
    check_name: String,
    fix: bool,
) -> Result<DoctorCheckOutput> {
    let doctor = Doctor::new()?;
    let result = doctor.run_check(&check_name)?;

    if fix && !result.passed {
        doctor.fix_check(&check_name)?;
    }

    Ok(DoctorCheckOutput {
        name: check_name,
        passed: result.passed,
        output: result.output,
        suggestions: result.suggestions,
    })
}

/// Show environment info
///
/// Displays workspace and environment information.
#[verb("env")]
fn doctor_env() -> Result<DoctorEnvOutput> {
    let doctor = Doctor::new()?;

    Ok(DoctorEnvOutput {
        workspace_root: doctor.workspace_root()?,
        ggen_version: env!("CARGO_PKG_VERSION").to_string(),
        lockfile_valid: doctor.check_lockfile_exists()?,
        pack_integrity: doctor.check_pack_integrity()?,
        policy_conflicts: doctor.check_policy_conflicts()?,
    })
}
