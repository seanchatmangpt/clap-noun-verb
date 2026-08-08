// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # CapabilityRegistry Example
//!
//! Demonstrates `CapabilityPackage` + `CapabilityRegistry` — the runtime
//! registry of named, versioned CLI capability bundles.
//!
//! ## Capabilities witnessed
//!
//! - `CapabilityPackage::new()` + `validate()`
//! - `CapabilityRegistry::new()` + `add_package()` + `packages()` + `len()`
//! - `contains()` + `remove_package()`
//! - `PackAddedOutput` / `PackRemovedOutput` (CLI handler return types)
//!
//! **Doc**: docs/howto/capability-packing.md, docs/reference/api-catalog.md

use clap_noun_verb::{
    capability::{CapabilityPackage, CapabilityRegistry},
    Result,
};

fn main() -> Result<()> {
    let mut registry = CapabilityRegistry::new();
    assert!(registry.is_empty(), "new registry must be empty");
    println!("empty: {}", registry.is_empty());

    let pkg_a = CapabilityPackage::new("auth", "Authentication", "1.0.0", "login, logout, refresh");
    pkg_a.validate().map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;

    let pkg_b =
        CapabilityPackage::new("billing", "Billing operations", "2.1.0", "invoice, subscription");

    registry.add_package(pkg_a).map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;
    registry.add_package(pkg_b).map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;

    assert_eq!(registry.len(), 2, "registry must hold 2 packages");
    println!("len: {}", registry.len());

    assert!(registry.contains("auth"), "registry must contain 'auth'");
    assert!(registry.contains("billing"), "registry must contain 'billing'");
    println!(
        "contains auth: {} billing: {}",
        registry.contains("auth"),
        registry.contains("billing")
    );

    let pkgs = registry.packages();
    let mut ids: Vec<&str> = pkgs.iter().map(|p| p.id.as_str()).collect();
    ids.sort();
    println!("packages: {:?}", ids);
    assert_eq!(ids.len(), 2, "2 packages");

    let removed = registry
        .remove_package("auth")
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;
    println!("removed: {removed}");
    assert_eq!(registry.len(), 1, "registry must hold 1 package after remove");
    assert!(!registry.contains("auth"), "auth must be gone after remove");
    println!("len after remove: {}", registry.len());

    Ok(())
}
