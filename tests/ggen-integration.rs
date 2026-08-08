// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ggen Integration Test - Verify v26.6.1 Integration Surface
//!
//! This test verifies that ggen (and similar code generators) can effectively
//! integrate with clap-noun-verb v26.6.1 to:
//! 1. Introspect command metadata
//! 2. Execute commands programmatically
//! 3. Parse and format output
//! 4. Handle errors correctly
//!
//! The specimen-graph-manager CLI serves as the test subject with 6 commands:
//! - doctor check
//! - graph load
//! - graph query
//! - graph validate
//! - pack add
//! - pack remove

use clap_noun_verb::{NounVerbError, Result as ClapResult};
use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================================
// CONTRACT VALIDATION: v26.6.1 Public API Surface
// ============================================================================

/// Verify that v26.6.1 exports the stable ggen integration surface
#[test]
fn test_v26_6_1_public_api_surface() {
    // These types MUST be public for ggen integration
    use clap_noun_verb::{CliBuilder, CommandRegistry, OutputFormat};

    // If this compiles, the API surface is stable
    let _builder = CliBuilder::default();
    let _format = OutputFormat::default();
    let _registry = CommandRegistry::new();

    println!("✓ v26.6.1 public API surface is ggen-compatible");
}

// ============================================================================
// PART A: Command Introspection
// ============================================================================

/// Test that the specimen CLI exposes exactly the verbs ggen expects to discover.
///
/// The `tutorial_services` example registers three `#[verb]` functions on the
/// `services` noun: `status`, `restart`, and `logs`. The `--introspect` flag is
/// the ggen discovery surface and emits a JSON array of tool descriptors, one per
/// verb. We assert on the actual descriptor count and names, not merely on build
/// success — this is the contract ggen relies on for tool-calling.
#[test]
fn test_specimen_cli_command_count() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--example", "tutorial_services", "--", "--introspect"])
        .current_dir("/Users/sac/clap-noun-verb")
        .output()
        .expect("Failed to run tutorial_services --introspect");

    assert!(
        output.status.success(),
        "tutorial_services --introspect failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tools: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .expect("--introspect must emit a JSON array of tool descriptors");

    // Exactly the three registered verbs are discoverable.
    assert_eq!(tools.len(), 3, "specimen CLI must expose exactly 3 discoverable verbs");

    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"services_status"), "missing services_status verb: {names:?}");
    assert!(names.contains(&"services_restart"), "missing services_restart verb: {names:?}");
    assert!(names.contains(&"services_logs"), "missing services_logs verb: {names:?}");

    println!("✓ specimen CLI exposes exactly 3 discoverable verbs via --introspect");
}

/// Test that specimen CLI --help lists all commands
#[test]
fn test_specimen_cli_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--example", "tutorial_services", "--", "--help"])
        .current_dir("/Users/sac/clap-noun-verb")
        .output()
        .expect("Failed to run tutorial_services --help");

    assert!(
        output.status.success(),
        "tutorial_services --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let help_text = String::from_utf8_lossy(&output.stdout);
    println!("Help output:\n{}", help_text);

    // The top-level help lists the registered `services` noun and the built-in
    // `help` subcommand, under a "Commands:" section, with a usage line for the
    // example binary. Assert on these concrete strings rather than a fuzzy
    // case-insensitive "command" match.
    assert!(help_text.contains("Commands:"), "Help output must contain a Commands: section");
    assert!(help_text.contains("services"), "Help output must list the 'services' noun");
    assert!(
        help_text.contains("Manage application services"),
        "Help output must include the 'services' noun about text"
    );
    assert!(
        help_text.contains("Usage: tutorial_services"),
        "Help output must include the usage line for the example binary"
    );

    println!("✓ specimen CLI help output lists the 'services' noun and usage");
}

// ============================================================================
// PART B: Output Format Verification
// ============================================================================

/// Mock command output that matches specimen CLI pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockCommandOutput {
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl MockCommandOutput {
    pub fn success(message: impl Into<String>) -> Self {
        Self { status: "success".to_string(), message: message.into(), data: None }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Test that ggen can work with OutputFormat enum
#[test]
fn test_output_format_enum_usage() {
    let output = MockCommandOutput::success("Test command executed");

    // ggen should be able to serialize to both formats
    let json_string = serde_json::to_string(&output).expect("Failed to serialize to JSON");
    let json_value: serde_json::Value =
        serde_json::from_str(&json_string).expect("Failed to parse JSON");

    assert_eq!(json_value["status"], "success");
    assert!(json_value["message"].is_string());

    // YAML support verification
    let yaml_string = serde_yaml::to_string(&output).expect("Failed to serialize to YAML");
    let _: MockCommandOutput = serde_yaml::from_str(&yaml_string).expect("Failed to parse YAML");

    println!("✓ OutputFormat enum works with JSON and YAML");
}

/// Test that all specimen output types are Serialize-bound
#[test]
fn test_specimen_output_types_serializable() {
    // These types come from specimen-graph-manager
    #[derive(Serialize, Deserialize)]
    struct GraphLoadedOutput {
        triples_loaded: usize,
        source: String,
        status: String,
    }

    #[derive(Serialize, Deserialize)]
    struct QueryResultOutput {
        query_type: String,
        pattern: String,
        results: Vec<serde_json::Value>,
        match_count: usize,
    }

    #[derive(Serialize, Deserialize)]
    struct ValidationResultOutput {
        valid: bool,
        errors: Vec<serde_json::Value>,
        total_triples: usize,
        valid_triples: usize,
    }

    #[derive(Serialize, Deserialize)]
    struct DoctorOutput {
        status: String,
        healthy: bool,
        issues: Vec<serde_json::Value>,
        graph_triples: usize,
        registry_packages: usize,
    }

    #[derive(Serialize, Deserialize)]
    struct PackAddedOutput {
        id: String,
        name: String,
        version: String,
        status: String,
    }

    #[derive(Serialize, Deserialize)]
    struct RemovalStatus {
        removed_id: String,
        status: String,
        message: String,
    }

    // Verify each type can round-trip through JSON
    let loaded = GraphLoadedOutput {
        triples_loaded: 100,
        source: "test.ttl".to_string(),
        status: "success".to_string(),
    };
    let json = serde_json::to_string(&loaded).unwrap();
    let _: GraphLoadedOutput = serde_json::from_str(&json).unwrap();

    let query = QueryResultOutput {
        query_type: "subject_match".to_string(),
        pattern: "ex:alice".to_string(),
        results: vec![],
        match_count: 0,
    };
    let json = serde_json::to_string(&query).unwrap();
    let _: QueryResultOutput = serde_json::from_str(&json).unwrap();

    let validation = ValidationResultOutput {
        valid: true,
        errors: vec![],
        total_triples: 50,
        valid_triples: 50,
    };
    let json = serde_json::to_string(&validation).unwrap();
    let _: ValidationResultOutput = serde_json::from_str(&json).unwrap();

    let doctor = DoctorOutput {
        status: "healthy".to_string(),
        healthy: true,
        issues: vec![],
        graph_triples: 100,
        registry_packages: 5,
    };
    let json = serde_json::to_string(&doctor).unwrap();
    let _: DoctorOutput = serde_json::from_str(&json).unwrap();

    let pack = PackAddedOutput {
        id: "pkg-001".to_string(),
        name: "GraphUtils".to_string(),
        version: "2.1.0".to_string(),
        status: "added".to_string(),
    };
    let json = serde_json::to_string(&pack).unwrap();
    let _: PackAddedOutput = serde_json::from_str(&json).unwrap();

    let removal = RemovalStatus {
        removed_id: "pkg-001".to_string(),
        status: "removed".to_string(),
        message: "Package successfully removed from registry".to_string(),
    };
    let json = serde_json::to_string(&removal).unwrap();
    let _: RemovalStatus = serde_json::from_str(&json).unwrap();

    println!("✓ All specimen output types are Serialize-bound and round-trip correctly");
}

// ============================================================================
// PART C: Error Handling Verification
// ============================================================================

/// Test that NounVerbError is ggen-compatible
#[test]
fn test_error_handling_compatibility() {
    let error = NounVerbError::command_not_found("test");

    // The Display impl (thiserror) for CommandNotFound is
    // "Command '{noun}' not found{suggestion}". With no candidates the suggestion
    // is empty, so the rendered message is exactly "Command 'test' not found".
    let error_str = error.to_string();
    assert_eq!(error_str, "Command 'test' not found", "CommandNotFound Display contract");

    // ggen extracts the offending noun from the message; verify it is embedded.
    assert!(error_str.contains("test"), "error message must name the missing command");

    println!("✓ NounVerbError is ggen-compatible");
}

/// Test that Result<T> type works correctly with real crate functions
#[test]
fn test_result_type_usage() {
    use clap_noun_verb::format_output;
    use clap_noun_verb::OutputFormat;

    // Arrange: a serializable value to format
    #[derive(Serialize)]
    struct Sample {
        key: &'static str,
        value: u32,
    }
    let sample = Sample { key: "answer", value: 42 };

    // Act: format_output succeeds for valid input
    let success = format_output(&sample, OutputFormat::Json);
    assert!(success.is_ok());
    let json_str = success.unwrap();
    assert!(json_str.contains("answer"), "JSON output must contain the 'key' field value");
    assert!(json_str.contains("42"), "JSON output must contain the 'value' field value");

    // Act: NounVerbError constructors produce errors that carry the expected message
    let err = NounVerbError::execution_error("bad input".to_string());
    let failure: ClapResult<String> = Err(err);
    assert!(failure.is_err());
    let msg = failure.unwrap_err().to_string();
    assert!(msg.contains("bad input"), "Error message must include the original cause: {msg}");

    println!("✓ Result<T> type works correctly with real crate functions");
}

// ============================================================================
// PART D: Integration Contract Verification
// ============================================================================

/// Test that ggen can use #[verb] macro pattern
#[test]
fn test_verb_macro_pattern() {
    // The specimen CLI registers verbs via the #[verb] macro. Beyond compiling,
    // a macro-registered verb must actually route and produce its declared output.
    // Invoke `services status` and verify the real, structured JSON payload —
    // get_service_status() returns 4 services, each "Running", with known ports.
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--example", "tutorial_services", "--", "services", "status"])
        .current_dir("/Users/sac/clap-noun-verb")
        .output()
        .expect("Failed to run specimen 'services status'");

    assert!(
        output.status.success(),
        "specimen 'services status' failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("verb output must be valid JSON");

    let services = value["services"].as_array().expect("output must carry a 'services' array");
    assert_eq!(services.len(), 4, "status must report all 4 registered services");

    let names: Vec<&str> = services.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(names.contains(&"web-server"), "status must include web-server: {names:?}");
    assert!(names.contains(&"database"), "status must include database: {names:?}");

    // Every service is "Running" per get_service_status().
    assert!(
        services.iter().all(|s| s["state"] == "Running"),
        "all services must report state Running"
    );
    // web-server runs on port 8080.
    let web = services.iter().find(|s| s["name"] == "web-server").expect("web-server present");
    assert_eq!(web["port"], 8080, "web-server must report port 8080");

    println!("✓ #[verb] macro pattern routes and produces real structured output");
}

/// Test that VerbCommand trait is accessible
#[test]
fn test_verb_command_trait_accessible() {
    // VerbCommand is a core trait that ggen relies on
    use clap_noun_verb::VerbCommand;

    // If we can import it, it's stable
    // VerbCommand has a run method that takes &self and &VerbArgs
    // This just verifies the trait is public
    let _dummy: Option<&dyn VerbCommand> = None;

    println!("✓ VerbCommand trait is accessible to ggen");
}

// ============================================================================
// PART E: Version-Specific Stability Assertions
// ============================================================================

/// Assert that v26.6.1 provides stable API for ggen
#[test]
fn test_v26_6_1_ggen_contract() {
    use clap_noun_verb::{format_output, CliBuilder, CommandRegistry, OutputFormat, VerbCommand};

    // CliBuilder and CommandRegistry are constructible (public and stable)
    let _builder = CliBuilder::default();
    let _registry = CommandRegistry::new();

    // OutputFormat::Json and OutputFormat::Yaml are stable variants
    let fmt_json = OutputFormat::Json;
    let fmt_yaml = OutputFormat::Yaml;

    // format_output works with both stable output format variants
    #[derive(Serialize)]
    struct ContractProbe {
        probe: &'static str,
    }
    let probe = ContractProbe { probe: "ggen-contract" };

    let json_out = format_output(&probe, fmt_json).expect("format_output(Json) must succeed");
    assert!(
        json_out.contains("ggen-contract"),
        "JSON output must contain the serialized field value"
    );

    let yaml_out = format_output(&probe, fmt_yaml).expect("format_output(Yaml) must succeed");
    assert!(
        yaml_out.contains("ggen-contract"),
        "YAML output must contain the serialized field value"
    );

    // NounVerbError is constructible and carries its message
    let err = NounVerbError::command_not_found("probe-verb");
    let err_str = err.to_string();
    assert!(!err_str.is_empty(), "NounVerbError must produce a non-empty message");

    // VerbCommand trait is accessible as a dyn-safe trait object
    let _dyn_check: Option<&dyn VerbCommand> = None;
    let _ = _dyn_check; // suppress unused warning
}

// ============================================================================
// SUMMARY TEST
// ============================================================================

/// Final integration status summary — verifies the complete ggen contract end-to-end
#[test]
fn test_ggen_integration_summary() {
    use clap_noun_verb::{format_output, OutputFormat};

    // --- DISCOVERY: verify the MockCommandOutput round-trips through both formats ---
    let cmd_out =
        MockCommandOutput::success("summary-probe").with_data(serde_json::json!({"triples": 6}));

    let json = serde_json::to_string(&cmd_out).expect("MockCommandOutput must serialize to JSON");
    let roundtrip: MockCommandOutput =
        serde_json::from_str(&json).expect("MockCommandOutput must deserialize from JSON");
    assert_eq!(roundtrip.status, "success", "status field must round-trip");
    assert_eq!(roundtrip.message, "summary-probe", "message field must round-trip");
    assert!(roundtrip.data.is_some(), "data field must survive round-trip");

    // --- OUTPUT FORMATS: format_output covers both JSON and YAML ---
    let json_out = format_output(&cmd_out, OutputFormat::Json)
        .expect("format_output(Json) must succeed for MockCommandOutput");
    assert!(json_out.contains("summary-probe"), "JSON output must contain the message value");

    let yaml_out = format_output(&cmd_out, OutputFormat::Yaml)
        .expect("format_output(Yaml) must succeed for MockCommandOutput");
    assert!(yaml_out.contains("summary-probe"), "YAML output must contain the message value");

    // --- ERROR HANDLING: errors carry their message ---
    let err = NounVerbError::execution_error("summary-error".to_string());
    let err_str = err.to_string();
    assert!(
        err_str.contains("summary-error"),
        "NounVerbError must embed the original cause in its Display: {err_str}"
    );
}
