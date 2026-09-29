// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for `DoctorOutput` and `HealthIssue`.

use clap_noun_verb::{DoctorOutput, HealthIssue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut report = DoctorOutput::new(3, 2);
    assert!(report.healthy);
    assert_eq!(report.status, "healthy");

    report.add_issue("warning", "Replay receipt expires soon");
    assert!(report.healthy, "warnings do not collapse health standing");

    report.add_issue("error", "Capability dependency is unavailable");
    assert!(!report.healthy);
    assert_eq!(report.status, "unhealthy");
    assert_eq!(report.issues.len(), 2);

    let explicit =
        HealthIssue { level: "info".to_string(), message: "Static contract admitted".to_string() };
    report.issues.push(explicit);

    let json = serde_json::to_string(&report)?;
    assert!(json.contains("Capability dependency is unavailable"));
    assert!(json.contains("Static contract admitted"));

    println!("Doctor status={} issues={}", report.status, report.issues.len());
    Ok(())
}
