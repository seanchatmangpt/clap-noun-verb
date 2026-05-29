// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! E2E tests for the global `--select` projection option
//!
//! Executes the compiled example binaries directly to validate global JSON path,
//! select, and JMESPath query projections.

use assert_cmd::Command;

#[test]
fn test_select_projection_basic_status() {
    // Test default output (no select)
    let mut cmd =
        Command::cargo_bin("tutorial_basic").expect("failed to get cargo bin tutorial_basic");
    let output = cmd.args(&["services", "status"]).output().expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json_val.get("services").is_some());
    assert_eq!(json_val.get("all_running"), Some(&serde_json::Value::Bool(true)));

    // Test select key "all_running"
    let mut cmd =
        Command::cargo_bin("tutorial_basic").expect("failed to get cargo bin tutorial_basic");
    let output = cmd
        .args(&["--select", "all_running", "services", "status"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json_val, serde_json::Value::Bool(true));

    // Test select array "services"
    let mut cmd =
        Command::cargo_bin("tutorial_basic").expect("failed to get cargo bin tutorial_basic");
    let output = cmd
        .args(&["--select", "services", "services", "status"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json_val, serde_json::json!(["web-server", "database", "redis"]));

    // Test JMESPath array index "services[0]"
    let mut cmd =
        Command::cargo_bin("tutorial_basic").expect("failed to get cargo bin tutorial_basic");
    let output = cmd
        .args(&["--select", "services[0]", "services", "status"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json_val, serde_json::Value::String("web-server".to_string()));

    // Test JSONPath format "$.all_running"
    let mut cmd =
        Command::cargo_bin("tutorial_basic").expect("failed to get cargo bin tutorial_basic");
    let output = cmd
        .args(&["--select", "$.all_running", "services", "status"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json_val, serde_json::Value::Bool(true));

    // Test JSONPath format with index "$.services[1]"
    let mut cmd =
        Command::cargo_bin("tutorial_basic").expect("failed to get cargo bin tutorial_basic");
    let output = cmd
        .args(&["--select", "$.services[1]", "services", "status"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json_val, serde_json::Value::String("database".to_string()));
}

#[test]
fn test_select_projection_with_yaml_format() {
    let mut cmd =
        Command::cargo_bin("tutorial_basic").expect("failed to get cargo bin tutorial_basic");
    let output = cmd
        .args(&["--format", "yaml", "--select", "services[0]", "services", "status"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("web-server"));
}
