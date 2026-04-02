//! Sync command - the sacred authoritative transformation
//!
//! Sync consumes resolved graph, validates, emits artifacts, writes receipts.
//! This is the authoritative seam - everything else supports this.

use std::collections::HashMap;
use std::time::Instant;

use clap_noun_verb_macros::verb;
use clap_noun_verb::{NounVerbError, Result};

use crate::domain::sync::{
    load_stage, resolve_stage, validate_stage, render_stage, emit_stage, receipt_stage,
    Lockfile, SyncPipeline,
    LoadInput, ResolveInput, ValidateInput, RenderInput, EmitInput, ReceiptInput,
};
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

    // Extract config values (SyncPipeline fields are private, so we capture
    // them before constructing the pipeline struct).
    let policy_profile = profile.unwrap_or_else(|| "default".to_string());

    // Build pipeline config (pure data, no methods that execute stages)
    let _pipeline = SyncPipeline::new()
        .with_lockfile(lockfile)
        .with_policy_profile(policy_profile.clone())
        .with_dry_run(dry_run)
        .with_force(force);

    let start = Instant::now();

    // Stage 1: Load
    let load_output = load_stage(LoadInput {
        lockfile_content: None,
        workspace_root: ".".to_string(),
    }).map_err(|e| NounVerbError::ExecutionError { message: e })?;

    // Stage 2: Resolve
    let resolve_output = resolve_stage(ResolveInput {
        lockfile: load_output.lockfile,
        workspace_metadata: load_output.workspace_metadata,
        force,
    }).map_err(|e| NounVerbError::ExecutionError { message: e })?;

    // Stage 3: Validate
    let validate_output = validate_stage(ValidateInput {
        lockfile: resolve_output.lockfile,
        dependency_graph: resolve_output.dependency_graph,
        policy_profile,
    }).map_err(|e| NounVerbError::ExecutionError { message: e })?;

    if !validate_output.passed {
        return Err(NounVerbError::ExecutionError {
            message: format!("Validation failed: {:?}", validate_output.violations),
        });
    }

    // Stage 4: Render
    let render_output = render_stage(RenderInput {
        lockfile: validate_output.lockfile,
        dependency_graph: HashMap::new(),
        template_vars: HashMap::new(),
    }).map_err(|e| NounVerbError::ExecutionError { message: e })?;

    // Stage 5: Emit
    let emit_output = emit_stage(EmitInput {
        rendered_artifacts: render_output.rendered_artifacts,
        lockfile: render_output.lockfile,
        dry_run,
    }).map_err(|e| NounVerbError::ExecutionError { message: e })?;

    // Stage 6: Receipt
    let duration_ms = start.elapsed().as_millis() as u64;
    let receipt_id = format!("sync-{}", chrono_now_ms());
    let receipt_output = receipt_stage(ReceiptInput {
        lockfile: emit_output.lockfile,
        artifact_paths: emit_output.artifact_paths.clone(),
        receipt_id: receipt_id.clone(),
        duration_ms,
    }).map_err(|e| NounVerbError::ExecutionError { message: e })?;

    let sync_result = receipt_output.sync_result;

    Ok(SyncOutput {
        lockfile: sync_result.lockfile_path,
        operations: sync_result.operations_completed,
        artifacts: sync_result.artifacts_emitted,
        receipt: sync_result.receipt_path,
        duration_ms: sync_result.duration_ms,
    })
}

/// Simple timestamp-based ID generator (avoids pulling in uuid or chrono as deps).
fn chrono_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
