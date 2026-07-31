// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for evidence-derived capability standing and dependency closure.

use clap_noun_verb::{
    CapabilityPackage, CapabilityRegistry, CapabilityStanding, ProofSurface,
};

fn alive_proof(name: &str, rung: &str) -> ProofSurface {
    ProofSurface::new(name, rung, format!("receipt:{name}:{rung}"), true, true)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut core = CapabilityPackage::new("core", "Core", "26.7.62", "Core routing");
    core.record_proof(alive_proof("core-unit", "unit"))?;
    core.record_proof(alive_proof("core-integration", "integration"))?;
    core.record_proof(alive_proof("core-replay", "replay"))?;
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
    ))?;
    assert_eq!(cli.standing, CapabilityStanding::Blocked);

    let mut registry = CapabilityRegistry::new();
    registry.add_package(cli.clone())?;
    registry.add_package(core.clone())?;
    assert_eq!(registry.dependency_order()?, vec!["core", "cli"]);
    assert!(registry.add_package(core).is_err(), "duplicate IDs must refuse");

    cli.record_proof(alive_proof("cli-unit", "unit"))?;
    registry.update_package(cli)?;
    assert_eq!(registry.get("cli").map(|pkg| pkg.standing), Some(CapabilityStanding::Alive));

    println!("Capability order={:?}; duplicate ID refused", registry.dependency_order()?);
    Ok(())
}
