//! Pack commands - law-bearing implementation units

use clap_noun_verb_macros::verb;
use clap_noun_verb::{Result, NounVerbError};
use crate::domain::pack::{Pack, PackStore, DependencyGraph};
use crate::outputs::{PackAddedOutput, PackRemovedOutput, PackListOutput, PackShowOutput, PackVerifyOutput, PackGraphOutput, PackUpdateOutput, PackInfo};

/// Add a pack
#[verb("add")]
fn add_pack(
    identifier: String,
    version: Option<String>,
    force: bool,
) -> Result<PackAddedOutput> {
    let store = PackStore::new()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    let pack = store.resolve(&identifier, version.as_deref())
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    store.install(&pack, force)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    Ok(PackAddedOutput {
        name: pack.name,
        version: pack.version,
        dependencies: pack.dependencies,
        installed_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Remove a pack
#[verb("remove")]
fn remove_pack(
    identifier: String,
    force: bool,
) -> Result<PackRemovedOutput> {
    let store = PackStore::new()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    store.remove(&identifier, force)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    Ok(PackRemovedOutput {
        identifier,
        removed_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// List installed packs
#[verb("list")]
fn list_packs() -> Result<PackListOutput> {
    let store = PackStore::new()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    let packs = store.list_all()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    Ok(PackListOutput {
        packs: packs.into_iter().map(|p| PackInfo {
            name: p.name,
            version: p.version,
        }).collect(),
    })
}

/// Show pack details
#[verb("show")]
fn show_pack(identifier: String) -> Result<PackShowOutput> {
    let store = PackStore::new()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    let details = store.show(&identifier)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    Ok(PackShowOutput {
        name: details.name,
        version: details.version,
        description: details.description,
        dependencies: details.dependencies,
        capabilities: details.capabilities,
    })
}

/// Verify pack integrity
#[verb("verify")]
fn verify_pack(identifier: String) -> Result<PackVerifyOutput> {
    let store = PackStore::new()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    let result = store.verify(&identifier)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    Ok(PackVerifyOutput {
        identifier,
        is_valid: result.valid,
        checksum: result.checksum,
        signature_valid: result.signature_valid,
        errors: result.errors,
    })
}

/// Show pack dependency graph
#[verb("graph")]
fn graph_packs(
    dot_format: bool,
) -> Result<PackGraphOutput> {
    let graph = DependencyGraph::load()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    Ok(PackGraphOutput {
        graph: graph.to_dot_format(),
    })
}

/// Update packs
#[verb("update")]
fn update_packs(
    dry_run: bool,
) -> Result<PackUpdateOutput> {
    let store = PackStore::new()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    let updates = store.check_updates()
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;

    if dry_run {
        return Ok(PackUpdateOutput::dry_run(updates));
    }

    let applied = store.apply_updates(updates)
        .map_err(|e| NounVerbError::ExecutionError { message: e })?;
    Ok(PackUpdateOutput::applied(applied))
}
