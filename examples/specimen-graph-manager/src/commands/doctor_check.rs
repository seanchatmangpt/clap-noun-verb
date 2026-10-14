// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Doctor Check Command - Health check for graph manager system

use crate::output_models::DoctorOutput;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Perform a system health check
///
/// Delegates entirely to clap-noun-verb's real `diagnostics::health_check()`
/// (src/diagnostics/doctor.rs in the clap-noun-verb crate) -- the same,
/// already-tested probe that performs a real RDF graph accessibility check
/// via `check_graph_accessible()` and a real capability registry validation
/// via `check_registry_operational()`. No hardcoded/simulated results: this
/// used to construct `DoctorOutput::new(42, 5)` and three always-`true`
/// simulated checks; that fakery is gone.
///
/// `check_graph_accessible()` is gated behind clap-noun-verb's
/// `rdf-composition` feature (see clap-noun-verb's Cargo.toml `[features]`).
/// This crate's `Cargo.toml` does not enable it, so by default the real
/// probe honestly reports the graph store as not accessible -- an "error"
/// issue, not a silently-passing fake `true`.
///
/// # Example
/// ```text
/// specimen-graph-manager doctor check
/// ```
#[verb("check", "doctor")]
fn health_check() -> Result<DoctorOutput> {
    let real = clap_noun_verb::diagnostics::doctor::health_check()?;
    // Re-seed the local output type from the real probe's honest counts
    // (always 0/0 -- see clap-noun-verb's own `test_health_check_honest_counts`)
    // and replay each real issue through `add_issue()` so severity handling
    // (an "error"-level issue flips `healthy`/`status`) lives in exactly one
    // place: `DoctorOutput::add_issue()`, not duplicated here.
    let mut output = DoctorOutput::new(real.graph_triples, real.registry_packages);
    for issue in real.issues {
        output.add_issue(issue.level, issue.message);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_returns_result() {
        let result = health_check();
        assert!(result.is_ok());
    }

    #[test]
    fn test_health_check_reports_real_honest_counts() {
        // The real probe never fabricates counts -- it always reports 0/0
        // (see clap-noun-verb's own test_health_check_honest_counts). The
        // old fake implementation hardcoded 42/5; assert that is gone.
        let output = health_check().unwrap();
        assert_eq!(output.graph_triples, 0);
        assert_eq!(output.registry_packages, 0);
    }

    #[test]
    fn test_health_check_records_real_issues() {
        // The real probe always appends at least the info-level
        // "All core services operational" issue.
        let output = health_check().unwrap();
        assert!(!output.issues.is_empty());
        assert!(output
            .issues
            .iter()
            .any(|issue| issue.level == "info" && issue.message.contains("operational")));
    }

    #[test]
    fn test_health_check_registry_check_passes() {
        // check_registry_operational() is real and unconditional (not
        // feature-gated): a fresh CommandRegistry::new().validate() always
        // succeeds, so no "Capability registry is not responding" error
        // issue should ever appear.
        let output = health_check().unwrap();
        assert!(!output
            .issues
            .iter()
            .any(|issue| issue.message.contains("registry is not responding")));
    }
}
