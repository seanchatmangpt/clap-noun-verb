//! Sync command - the sacred authoritative transformation
//!
//! Sync consumes resolved graph, validates, emits artifacts, writes receipts.
//! This is the authoritative seam - everything else supports this.

use clap_noun_verb_macros::verb;
use clap_noun_verb::{NounVerbError, Result};

use crate::domain::sync::{Lockfile, SyncPipeline};
use crate::outputs::SyncOutput;

/// Run sync - sacred authoritative command
///
/// No subcommand - sync is singular and sacred.
///
/// # Arguments
/// * `dry_run` - Validate without applying
/// * `force` - Bypass policy checks (requires: ack)
/// * `ack` - Acknowledge safety bypass
/// * `profile` - Policy profile [default: default]
#[verb("sync")]
fn run_sync(
    dry_run: bool,
    force: bool,
    ack: Option<String>,
    profile: Option<String>,
) -> Result<SyncOutput> {
    // Validate constraints
    if force && ack.is_none() {
        return Err(NounVerbError::validation_error(
            "ack".to_string(),
            "missing".to_string(),
            Some("--force requires --ack to acknowledge safety bypass")
        ));
    }

    // Load or create lockfile
    let lockfile_path = std::path::Path::new("ggen.lock");
    let lockfile = Lockfile::load(lockfile_path)
        .unwrap_or_else(|_| Lockfile::new());

    // Build pipeline
    let pipeline = SyncPipeline::new()
        .with_lockfile(lockfile)
        .with_policy_profile(profile.unwrap_or_else(|| "default".to_string()))
        .with_dry_run(dry_run)
        .with_force(force);

    // Execute pipeline stages
    let result = pipeline
        .load()?
        .resolve()?
        .validate()?
        .render()?
        .emit()?
        .receipt()?;

    Ok(SyncOutput {
        lockfile: result.lockfile_path,
        operations: result.operations_completed,
        artifacts: result.artifacts_emitted,
        receipt: result.receipt_path,
        duration_ms: result.duration_ms,
    })
}
