//! Pack commands - law-bearing implementation units

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use crate::domain::pack::{Pack, PackStore, DependencyGraph};
use crate::outputs::{PackAddedOutput, PackRemovedOutput, PackListOutput, PackShowOutput, PackVerifyOutput, PackGraphOutput, PackUpdateOutput};

/// Add a pack
#[verb("add")]
fn add_pack(
    identifier: String,
    version: Option<String>,
    force: bool,
) -> Result<PackAddedOutput> {
    let store = PackStore::new()?;
    let pack = store.resolve(&identifier, version.as_deref())?;
    store.install(&pack, force)?;

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
    let store = PackStore::new()?;
    store.remove(&identifier, force)?;

    Ok(PackRemovedOutput {
        identifier,
        removed_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// List installed packs
#[verb("list")]
fn list_packs() -> Result<PackListOutput> {
    let store = PackStore::new()?;
    Ok(PackListOutput {
        packs: store.list_all()?,
    })
}

/// Show pack details
#[verb("show")]
fn show_pack(identifier: String) -> Result<PackShowOutput> {
    let store = PackStore::new()?;
    Err("Not implemented".to_string())
}

/// Verify pack integrity
#[verb("verify")]
fn verify_pack(identifier: String) -> Result<PackVerifyOutput> {
    let store = PackStore::new()?;
    let result = store.verify(&identifier)?;

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
    let graph = DependencyGraph::load()?;
    Ok(PackGraphOutput {
        graph: graph.to_dot_format(),
    })
}

/// Update packs
#[verb("update")]
fn update_packs(
    dry_run: bool,
) -> Result<PackUpdateOutput> {
    let store = PackStore::new()?;
    let updates = store.check_updates()?;

    if dry_run {
        return Ok(PackUpdateOutput::dry_run(updates));
    }

    let applied = store.apply_updates(updates)?;
    Ok(PackUpdateOutput::applied(applied))
}
