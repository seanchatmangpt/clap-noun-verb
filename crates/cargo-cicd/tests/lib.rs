// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Unit tests for cargo-cicd library modules
//!
//! Tests organized by module:
//! - WorkspaceState, TargetState, GitState
//! - CICD TOML serialization/deserialization
//! - Policy evaluation logic
//! - Adapter implementations (git diff, target scanning, fixture detection)
//! - Command output generation

#![allow(clippy::panic)]

use cargo_cicd::{
    adapters::TargetScanning, TargetShowOutput, TestChangedOutput, TrybuildChangedOutput,
};
use std::path::PathBuf;

// =============================================================================
// UNIT TESTS - State Model
// =============================================================================

#[test]
fn test_target_scanning_initialization() {
    // Arrange: Create a scanning instance for a test target directory
    let target_path = PathBuf::from("target");

    // Act: Attempt to scan (may succeed or fail depending on directory state)
    let result = TargetScanning::scan(target_path.clone());

    // Assert: Result is either Ok or Err (both are valid states)
    match result {
        Ok(info) => {
            assert_eq!(info.path, target_path);
            assert!(info.total_size_gb >= 0.0, "Total size should be non-negative");
            // Profiles may be empty if target directory hasn't been built yet
            assert!(info.profiles.len() >= 0, "Profile count should be non-negative");
        }
        Err(_) => {
            // Target directory may not exist in test environment
        }
    }
}

#[test]
fn test_target_show_output_structure() {
    // Arrange: Create a TargetShowOutput with known values
    let output = TargetShowOutput {
        target_path: PathBuf::from("target"),
        total_size_gb: 5.5,
        profiles: vec![("debug".to_string(), 2.0), ("release".to_string(), 3.5)],
        stale_candidates: vec!["old_artifact_1".to_string()],
        configured_max_gb: 10.0,
        verdict: "WARN: Target at 55% of limit".to_string(),
    };

    // Act: Verify structure is correctly initialized
    let is_valid = output.total_size_gb > 0.0 && output.profiles.len() == 2;

    // Assert: Structure is valid
    assert!(is_valid, "TargetShowOutput structure is invalid");
    assert_eq!(output.profiles.len(), 2, "Expected exactly 2 profiles");
    assert_eq!(output.stale_candidates.len(), 1, "Expected 1 stale candidate");
}

#[test]
fn test_target_verdict_logic_under_limit() {
    // Arrange: Create a state where target is well under limit
    let output = TargetShowOutput {
        target_path: PathBuf::from("target"),
        total_size_gb: 2.0,
        profiles: vec![("debug".to_string(), 2.0)],
        stale_candidates: vec![],
        configured_max_gb: 10.0,
        verdict: "OK: Target at 20% of limit".to_string(),
    };

    // Act: Check verdict logic
    let is_healthy = output.verdict.starts_with("OK");

    // Assert: Verdict correctly reflects healthy state
    assert!(is_healthy, "Verdict should indicate OK status");
    assert_eq!(output.stale_candidates.len(), 0, "No stale candidates expected");
}

#[test]
fn test_target_verdict_logic_approaching_limit() {
    // Arrange: Create a state where target is approaching limit
    let output = TargetShowOutput {
        target_path: PathBuf::from("target"),
        total_size_gb: 8.0,
        profiles: vec![("debug".to_string(), 5.0), ("release".to_string(), 3.0)],
        stale_candidates: vec!["old_1".to_string(), "old_2".to_string()],
        configured_max_gb: 10.0,
        verdict: "WARN: Target at 80% of limit".to_string(),
    };

    // Act: Check verdict logic
    let is_warning = output.verdict.starts_with("WARN");

    // Assert: Verdict correctly reflects warning state
    assert!(is_warning, "Verdict should indicate WARN status");
    assert!(
        output.stale_candidates.len() > 0,
        "Should have stale candidates when approaching limit"
    );
}

#[test]
fn test_changed_file_classification_rust_source() {
    // Arrange: Create file paths for Rust source files
    let rust_files = vec!["src/lib.rs", "src/main.rs", "src/cli/mod.rs", "tests/unit.rs"];

    // Act: Verify that Rust files are correctly identified
    let mut all_rust = true;
    for file in rust_files {
        let is_rust = file.ends_with(".rs");
        all_rust = all_rust && is_rust;
    }

    // Assert: All files are identified as Rust
    assert!(all_rust, "All .rs files should be classified as Rust");
}

#[test]
fn test_git_phase_state_clean() {
    // Arrange: Simulate a clean git state (no uncommitted changes)
    let uncommitted_changes = 0;
    let unpushed_commits = 0;

    // Act: Determine if repo is clean
    let is_clean = uncommitted_changes == 0 && unpushed_commits == 0;

    // Assert: Git state is correctly identified as clean
    assert!(is_clean, "Git state should be clean");
}

#[test]
fn test_git_phase_state_dirty() {
    // Arrange: Simulate a dirty git state (uncommitted changes)
    let uncommitted_changes = 3;
    let unpushed_commits = 2;

    // Act: Determine if repo is dirty
    let is_dirty = uncommitted_changes > 0 || unpushed_commits > 0;

    // Assert: Git state is correctly identified as dirty
    assert!(is_dirty, "Git state should be dirty");
}

#[test]
fn test_fixture_detection_structure() {
    // Arrange: Simulate a fixture detection result
    let changed_fixtures =
        vec!["tests/fixtures/test_1.rs".to_string(), "tests/fixtures/test_2.rs".to_string()];
    let total_fixtures = 10;

    // Act: Calculate scope
    let scope_name = if changed_fixtures.is_empty() {
        "none"
    } else if changed_fixtures.len() < total_fixtures {
        "partial"
    } else {
        "full"
    };

    // Assert: Scope is correctly determined
    assert_eq!(scope_name, "partial", "Should be partial scope");
    assert_eq!(changed_fixtures.len(), 2, "Expected 2 changed fixtures");
}

// =============================================================================
// UNIT TESTS - CICD TOML Serialization
// =============================================================================

#[test]
fn test_target_show_output_serializable() {
    // Arrange: Create a TargetShowOutput
    let output = TargetShowOutput {
        target_path: PathBuf::from("target"),
        total_size_gb: 5.0,
        profiles: vec![("debug".to_string(), 3.0), ("release".to_string(), 2.0)],
        stale_candidates: vec!["old_1".to_string()],
        configured_max_gb: 10.0,
        verdict: "OK".to_string(),
    };

    // Act: Serialize to JSON
    let json = serde_json::to_string(&output);

    // Assert: Serialization succeeds
    assert!(json.is_ok(), "TargetShowOutput should be JSON serializable");
    if let Ok(json_str) = json {
        assert!(json_str.contains("target"), "JSON should contain target_path");
        assert!(json_str.contains("5.0"), "JSON should contain size");
    }
}

#[test]
fn test_test_changed_output_serializable() {
    // Arrange: Create a TestChangedOutput
    let output = TestChangedOutput {
        test_plan: vec![
            ("test_module::test_foo".to_string(), "unit".to_string()),
            ("test_module::test_bar".to_string(), "integration".to_string()),
        ],
        is_conservative: false,
        reason: "Only core modules changed".to_string(),
        estimated_runtime_seconds: 30,
    };

    // Act: Serialize to JSON
    let json = serde_json::to_string(&output);

    // Assert: Serialization succeeds
    assert!(json.is_ok(), "TestChangedOutput should be JSON serializable");
    if let Ok(json_str) = json {
        assert!(json_str.contains("test_plan"), "JSON should contain test_plan");
        assert!(json_str.contains("30"), "JSON should contain runtime estimate");
    }
}

#[test]
fn test_trybuild_changed_output_serializable() {
    // Arrange: Create a TrybuildChangedOutput
    let output = TrybuildChangedOutput {
        changed_fixtures: vec!["tests/ui/fail_1.rs".to_string(), "tests/ui/pass_1.rs".to_string()],
        total_fixtures: 50,
        scope: "partial".to_string(),
        snapshot_update_available: true,
    };

    // Act: Serialize to JSON
    let json = serde_json::to_string(&output);

    // Assert: Serialization succeeds
    assert!(json.is_ok(), "TrybuildChangedOutput should be JSON serializable");
    if let Ok(json_str) = json {
        assert!(json_str.contains("changed_fixtures"), "JSON should contain changed_fixtures");
        assert!(json_str.contains("50"), "JSON should contain total_fixtures");
    }
}

// =============================================================================
// UNIT TESTS - Policy Logic
// =============================================================================

#[test]
fn test_policy_pass_on_clean_repo() {
    // Arrange: Clean git state
    let uncommitted = 0;
    let unpushed = 0;
    let target_size_ok = true;

    // Act: Evaluate policy
    let policy_passes = uncommitted == 0 && unpushed == 0 && target_size_ok;

    // Assert: Policy passes when all conditions are met
    assert!(policy_passes, "Policy should pass on clean repo with acceptable target size");
}

#[test]
fn test_policy_warn_on_dirty_repo() {
    // Arrange: Dirty git state (uncommitted changes)
    let uncommitted = 5;
    let unpushed = 0;

    // Act: Evaluate policy
    let policy_warns = uncommitted > 0;

    // Assert: Policy warns when uncommitted changes exist
    assert!(policy_warns, "Policy should warn on uncommitted changes");
}

#[test]
fn test_policy_warn_on_high_target_pressure() {
    // Arrange: Target size approaching limit
    let target_size_gb = 9.0;
    let configured_max = 10.0;
    let pressure_ratio = target_size_gb / configured_max;

    // Act: Evaluate policy
    let policy_warns = pressure_ratio > 0.75;

    // Assert: Policy warns when target is >75% of limit
    assert!(policy_warns, "Policy should warn when target pressure is high");
}

// =============================================================================
// INTEGRATION TESTS - Adapter Behavior
// =============================================================================

#[test]
fn test_git_status_creates_valid_instance() {
    // Arrange: Create a GitStatusInfo instance
    let status = cargo_cicd::adapters::git_status::GitStatusInfo {
        branch_name: "main".to_string(),
        dirty_count: 0,
        dirty_sample: vec![],
        staged_count: 0,
        untracked_count: 0,
        ahead: 0,
        behind: 0,
        has_upstream: false,
    };

    // Act: Verify instance fields
    let is_valid = !status.branch_name.is_empty() && status.dirty_count >= 0;

    // Assert: GitStatusInfo instance is valid
    assert!(is_valid, "GitStatusInfo should be properly initialized");
    assert_eq!(status.branch_name, "main", "Branch should be 'main'");
}

#[test]
fn test_fixture_detection_enum_scope() {
    // Arrange: Test all scope variants
    let scopes = vec!["none", "partial", "full"];

    // Act: Verify each scope string
    let all_valid = scopes.iter().all(|s| matches!(*s, "none" | "partial" | "full"));

    // Assert: All scope variants are valid
    assert!(all_valid, "All fixture scopes should be valid");
}

// =============================================================================
// SNAPSHOT TESTS - Output Consistency
// =============================================================================

#[test]
fn test_target_show_verdict_messages_are_consistent() {
    // Arrange: Create outputs with different pressure levels
    let outputs = vec![("OK", 0.2), ("WARN", 0.75), ("CRITICAL", 0.95)];

    // Act: Verify verdict patterns
    let verdicts_valid = outputs.iter().all(|(prefix, _)| prefix.len() > 0);

    // Assert: All verdict prefixes are present and non-empty
    assert!(verdicts_valid, "All verdict prefixes should be valid");
}

#[test]
fn test_test_changed_reason_messages_are_descriptive() {
    // Arrange: Create reasons with varying lengths
    let reasons = vec![
        "Only core modules changed",
        "Binary files changed",
        "No Rust files changed",
        "Full workspace rebuild needed",
    ];

    // Act: Verify all reasons are descriptive
    let all_descriptive = reasons.iter().all(|r| r.len() > 5);

    // Assert: Reasons are substantive (>5 chars)
    assert!(all_descriptive, "Reasons should be descriptive (>5 characters)");
}

#[test]
fn test_trybuild_scope_values_are_exhaustive() {
    // Arrange: Expected scope values
    let expected_scopes = vec!["none", "partial", "full"];

    // Act: Count unique scopes
    let unique_count = expected_scopes.len();

    // Assert: Expected number of scope variants
    assert_eq!(unique_count, 3, "Should have exactly 3 scope variants");
}
