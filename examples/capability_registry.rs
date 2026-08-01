// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for evidence-derived capability standing and dependency closure.

use clap_noun_verb::{
    CapabilityPackage, CapabilityRegistry, CapabilityStanding, ProofSurface,
};

fn alive_proof(name: &str, rung: &str) -> ProofSurface {
    ProofSurface::new(name, rung, format!("receipt:{name}:{rung}"), true, true)
}

fn io_error(error: String) -> std::io::Error {
    std::io::Error::other(error)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut core = CapabilityPackage::new("core", "Core", "26.7.62", "Core routing");
    core.record_proof(alive_proof("core-unit", "unit")).map_err(io_error)?;
    core.record_proof(alive_proof("core-integration", "integration"))
        .map_err(io_error)?;
    core.record_proof(alive_proof("core-replay", "replay")).map_err(io_error)?;
    assert_eq!(core.standing, CapabilityStanding::Alive);

    let mut cli = CapabilityPackage::new("cli", "CLI", "26.7.62", "Noun-verb interface")
        .with_default_verb("help")
        .with_dependency("core");
    cli.record_proof(ProofSurface::new(
        "cli-unit",
        "unit",
        "receipt:cli:unit",
        true,
        false,
    ))
    .map_err(io_error)?;
    assert_eq!(cli.standing, CapabilityStanding::Blocked);

    let mut registry = CapabilityRegistry::new();
    registry.add_package(cli.clone()).map_err(io_error)?;
    registry.add_package(core.clone()).map_err(io_error)?;
    assert_eq!(registry.dependency_order().map_err(io_error)?, vec!["core", "cli"]);
    assert!(registry.add_package(core).is_err(), "duplicate IDs must refuse");

    cli.record_proof(alive_proof("cli-unit", "unit")).map_err(io_error)?;
    registry.update_package(cli).map_err(io_error)?;
    assert_eq!(registry.get("cli").map(|pkg| pkg.standing), Some(CapabilityStanding::Alive));

    println!(
        "Capability order={:?}; duplicate ID refused",
        registry.dependency_order().map_err(io_error)?
    );
    Ok(())
}
