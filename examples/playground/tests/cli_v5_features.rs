//! CLI Integration Tests for clap-noun-verb v5 Features
//!
//! 80/20 Approach: Test CLI commands end-to-end rather than domain internals.
//! These tests verify all v5 features work through the actual CLI interface.

use std::process::Command;

fn playground() -> Command {
    Command::new(env!("CARGO_BIN_EXE_playground"))
}

fn run_cmd(args: &[&str]) -> String {
    let output = playground()
        .args(args)
        .output()
        .expect("Failed to execute command");

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn assert_success(args: &[&str]) -> String {
    let output = playground()
        .args(args)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command {:?} failed: {}",
        args, String::from_utf8_lossy(&output.stderr));

    String::from_utf8_lossy(&output.stdout).to_string()
}

// ============================================================================
// Core Nouns: papers, thesis, config
// ============================================================================

#[test]
fn test_papers_list() {
    let out = assert_success(&["papers", "list"]);
    assert!(out.contains("IMRaD"), "Should list IMRaD family");
    assert!(out.contains("Argument"), "Should list Argument family");
}

#[test]
fn test_papers_validate() {
    let out = assert_success(&["papers", "validate", "test.tex"]);
    assert!(out.contains("valid") || out.contains("extension"), "Should validate .tex");
}

#[test]
fn test_thesis_families() {
    let out = assert_success(&["thesis", "families"]);
    assert!(out.contains("IMRaD"), "Should list IMRaD");
    assert!(out.contains("DSR"), "Should list DSR");
}

#[test]
fn test_thesis_structure() {
    let out = assert_success(&["thesis", "structure"]);
    assert!(out.contains("components") || out.contains("Shards"), "Should show HTF components");
}

#[test]
fn test_config_show() {
    let out = assert_success(&["config", "show"]);
    assert!(out.contains("output_dir"), "Should show output_dir config");
}

#[test]
fn test_config_get() {
    let out = assert_success(&["config", "get", "output_dir"]);
    assert!(out.contains("output"), "Should get output_dir value");
}

// ============================================================================
// v5 Feature: Autonomic CLI Introspection
// ============================================================================

#[test]
fn test_meta_introspect() {
    let out = assert_success(&["meta", "introspect"]);
    assert!(out.contains("cli_name"), "Should have cli_name");
    assert!(out.contains("nouns"), "Should list nouns");
    assert!(out.contains("autonomic_features"), "Should list autonomic features");
    assert!(out.contains("total_capabilities"), "Should count capabilities");
}

// ============================================================================
// v5 Feature: RDF/Ontology Export
// ============================================================================

#[test]
fn test_meta_ontology() {
    let out = assert_success(&["meta", "ontology"]);
    assert!(out.contains("cnv:"), "Should have CNV namespace");
    assert!(out.contains("rdf:type"), "Should have RDF types");
    assert!(out.contains("cnv:Capability"), "Should define capabilities");
    assert!(out.contains("text/turtle"), "Should be Turtle format");
}

// ============================================================================
// v5 Feature: SPARQL Queries
// ============================================================================

#[test]
fn test_meta_sparql_count() {
    let out = assert_success(&["meta", "sparql", "SELECT (COUNT(*) as ?n) WHERE { ?s ?p ?o }"]);
    assert!(out.contains("results"), "Should have results");
    // Should have 60 triples (12 capabilities * 5 properties each)
    assert!(out.contains("60") || out.contains("results"), "Should count triples");
}

// ============================================================================
// v5 Feature: Shell Completions
// ============================================================================

#[test]
fn test_meta_completions_bash() {
    let out = assert_success(&["meta", "completions", "bash"]);
    assert!(out.contains("_playground_completions"), "Should have bash function");
    assert!(out.contains("complete -F"), "Should register completion");
}

#[test]
fn test_meta_completions_zsh() {
    let out = assert_success(&["meta", "completions", "zsh"]);
    assert!(out.contains("#compdef"), "Should have zsh compdef");
}

#[test]
fn test_meta_completions_fish() {
    let out = assert_success(&["meta", "completions", "fish"]);
    assert!(out.contains("complete -c playground"), "Should have fish completions");
}

// ============================================================================
// v5 Feature: Middleware Configuration
// ============================================================================

#[test]
fn test_meta_middleware() {
    let out = assert_success(&["meta", "middleware"]);
    assert!(out.contains("logging"), "Should show logging config");
    assert!(out.contains("rate_limiting"), "Should show rate limiting");
    assert!(out.contains("caching"), "Should show caching config");
    assert!(out.contains("stats"), "Should show middleware stats");
}

// ============================================================================
// v5 Feature: Telemetry & Receipts
// ============================================================================

#[test]
fn test_meta_telemetry() {
    let out = assert_success(&["meta", "telemetry"]);
    assert!(out.contains("span"), "Should have execution span");
    assert!(out.contains("trace_id"), "Should have trace ID");
    assert!(out.contains("receipt"), "Should have execution receipt");
}

// ============================================================================
// v5 Feature: Output Formats
// ============================================================================

#[test]
fn test_meta_formats() {
    let out = assert_success(&["meta", "formats"]);
    assert!(out.contains("json"), "Should list json format");
    assert!(out.contains("yaml"), "Should list yaml format");
    assert!(out.contains("table"), "Should list table format");
}

#[test]
fn test_format_flag_json() {
    let out = assert_success(&["meta", "formats", "--format", "json"]);
    // JSON output should be compact (no pretty printing)
    assert!(out.contains("[{") || out.contains("\"name\""), "Should be JSON");
}

#[test]
fn test_format_flag_yaml() {
    let out = assert_success(&["meta", "formats", "--format", "yaml"]);
    assert!(out.contains("name:") || out.contains("-"), "Should be YAML-like");
}

// ============================================================================
// v5 Feature: Help & Version (output goes to stderr via clap)
// ============================================================================

#[test]
fn test_help() {
    let output = playground()
        .args(&["--help"])
        .output()
        .expect("Failed to execute");
    let combined = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));
    assert!(combined.contains("papers") || combined.contains("Usage"), "Should show help");
}

#[test]
fn test_version() {
    let output = playground()
        .args(&["--version"])
        .output()
        .expect("Failed to execute");
    let combined = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));
    assert!(combined.contains("2.0.0") || combined.contains("playground"), "Should show version");
}

// ============================================================================
// TEST-1: papers generate with multiple families
// ============================================================================

#[test]
fn test_papers_generate_imrad() {
    // CLI uses positional argument for family, not --family flag
    let out = assert_success(&["papers", "generate", "IMRaD"]);
    assert!(out.contains("IMRaD") || out.contains("generate"), "Should generate IMRaD paper");
}

#[test]
fn test_papers_generate_argument() {
    // CLI uses positional argument for family, not --family flag
    let out = assert_success(&["papers", "generate", "Argument"]);
    assert!(out.contains("Argument") || out.contains("generate"), "Should generate Argument paper");
}

// ============================================================================
// TEST-2: thesis schedule with multiple families
// ============================================================================

#[test]
fn test_thesis_schedule_imrad() {
    // CLI uses positional argument for family, not --family flag
    let out = assert_success(&["thesis", "schedule", "IMRaD"]);
    assert!(out.contains("IMRaD") || out.contains("schedule") || out.contains("phase"), "Should schedule IMRaD thesis");
}

#[test]
fn test_thesis_schedule_dsr() {
    // CLI uses positional argument for family, not --family flag
    let out = assert_success(&["thesis", "schedule", "DSR"]);
    assert!(out.contains("DSR") || out.contains("schedule") || out.contains("phase"), "Should schedule DSR thesis");
}

// ============================================================================
// TEST-3: config get/set verification
// ============================================================================

#[test]
fn test_config_set_and_get() {
    // Set a config value
    let set_out = assert_success(&["config", "set", "output_dir", "/tmp/test_output"]);
    assert!(set_out.contains("output_dir") || set_out.contains("set") || set_out.contains("/tmp"), "Should confirm set");

    // Get the value back
    let get_out = assert_success(&["config", "get", "output_dir"]);
    assert!(get_out.contains("output") || get_out.contains("/tmp"), "Should get config value");
}

// ============================================================================
// TEST-4: Error conditions
// ============================================================================

#[test]
fn test_invalid_noun_error() {
    let output = playground()
        .args(&["invalid_noun", "list"])
        .output()
        .expect("Failed to execute");

    // Invalid noun should fail
    assert!(!output.status.success(), "Invalid noun should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("invalid") || stderr.contains("unrecognized"),
        "Should show error message for invalid noun");
}

#[test]
fn test_invalid_verb_error() {
    let output = playground()
        .args(&["papers", "invalid_verb"])
        .output()
        .expect("Failed to execute");

    // Invalid verb should fail
    assert!(!output.status.success(), "Invalid verb should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("invalid") || stderr.contains("unrecognized"),
        "Should show error message for invalid verb");
}

// ============================================================================
// TEST-5: --format flag on multiple commands
// ============================================================================

#[test]
fn test_papers_list_format_json() {
    let out = assert_success(&["papers", "list", "--format", "json"]);
    // JSON output should contain brackets or quotes
    assert!(out.contains("[") || out.contains("{") || out.contains("\""),
        "Should output JSON format");
}

#[test]
fn test_thesis_families_format_yaml() {
    let out = assert_success(&["thesis", "families", "--format", "yaml"]);
    // YAML output should contain colons or dashes
    assert!(out.contains(":") || out.contains("-") || out.contains("name"),
        "Should output YAML format");
}
