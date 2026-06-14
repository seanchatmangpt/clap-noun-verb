// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Diagnostics Example
//!
//! Demonstrates `DoctorOutput`, `HealthIssue`, and `health_check()` —
//! the framework's built-in system health surface.
//!
//! ## Capabilities witnessed
//!
//! - `DoctorOutput::new(graph_triples, registry_packages)` + `add_issue()`
//! - `HealthIssue` fields: `level`, `message`
//! - `health_check()` — live check returning `Result<DoctorOutput>`
//!
//! **Doc**: docs/reference/api-catalog.md (diagnostics section)

use clap_noun_verb::{
    diagnostics::{DoctorOutput, HealthIssue},
    Result,
};
use clap_noun_verb::diagnostics::doctor::health_check;

fn main() -> Result<()> {
    // --- Witness: DoctorOutput::new() + add_issue() ---
    let mut doc = DoctorOutput::new(42, 3);
    doc.add_issue("warn", "connection pool near capacity");
    doc.add_issue("error", "registry checksum mismatch");

    println!("graph_triples: {}", doc.graph_triples);
    println!("registry_packages: {}", doc.registry_packages);
    println!("issues: {}", doc.issues.len());
    assert_eq!(doc.graph_triples, 42);
    assert_eq!(doc.registry_packages, 3);
    assert_eq!(doc.issues.len(), 2, "must have 2 issues");

    // --- Witness: HealthIssue fields ---
    let issue: &HealthIssue = &doc.issues[0];
    println!("issue[0]: level={} message={}", issue.level, issue.message);
    assert_eq!(issue.level, "warn");
    assert!(issue.message.contains("capacity"));

    let err_issue: &HealthIssue = &doc.issues[1];
    assert_eq!(err_issue.level, "error");
    println!("issue[1]: level={} message={}", err_issue.level, err_issue.message);

    // --- Witness: health_check() — live framework diagnostic ---
    let live = health_check()?;
    println!("health_check graph_triples: {}", live.graph_triples);
    println!("health_check registry_packages: {}", live.registry_packages);
    println!("health_check issues: {}", live.issues.len());

    Ok(())
}
