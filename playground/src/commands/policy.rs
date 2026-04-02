//! Policy commands - governance and execution constraints

use clap_noun_verb_macros::verb;
use clap_noun_verb::{Result, NounVerbError};
use crate::domain::policy::{PolicyProfile, PolicyValidator};
use crate::outputs::{PolicyListOutput, PolicyShowOutput, PolicyValidateOutput};

/// List policy profiles
#[verb("list")]
fn list_policies() -> Result<Vec<PolicyListOutput>> {
    Ok(PolicyProfile::all_available()
        .into_iter()
        .map(|p| PolicyListOutput {
            name: p.name,
            description: p.description,
            strict: p.strict,
        })
        .collect())
}

/// Show policy details
#[verb("show")]
fn show_policy(name: String) -> Result<PolicyShowOutput> {
    let profile = PolicyProfile::find(&name)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    Ok(PolicyShowOutput {
        name: profile.name(),
        description: profile.description(),
        strict: profile.strict(),
        rules: profile.rules().len(),
    })
}

/// Validate against policy
#[verb("validate")]
fn validate_policy(
    profile: Option<String>,
    lockfile: Option<String>,
) -> Result<PolicyValidateOutput> {
    let validator = PolicyValidator::new();
    let profile_name = profile.unwrap_or_else(|| "default".to_string());
    let lockfile_ref = lockfile.as_deref();
    let result = validator.validate(&profile_name, lockfile_ref)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    Ok(PolicyValidateOutput {
        profile: result.profile.clone(),
        valid: result.valid,
        violations: result.violations.clone(),
        warnings: result.warnings.clone(),
    })
}
