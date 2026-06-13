// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

mod common;

use clap_noun_verb_utils::display_json::{arg_matches_to_json, extract_command_schema, PrintJson};
use common::create_test_command;
use serde_json::Value;

#[test]
fn test_command_schema_serialization() -> Result<(), String> {
    let cmd = create_test_command();
    let schema = extract_command_schema(&cmd);

    assert_eq!(schema.name, "test-cli");
    assert_eq!(schema.about, Some("A test CLI for verification".to_string()));
    assert_eq!(schema.version, Some("1.0.0".to_string()));

    // Find verbose arg
    let verbose_arg = schema
        .arguments
        .iter()
        .find(|a| a.name == "verbose")
        .ok_or_else(|| "verbose argument not found in schema".to_string())?;
    assert!(verbose_arg.is_flag);
    assert!(!verbose_arg.required);

    // Find start subcommand
    let start_sub = schema
        .subcommands
        .iter()
        .find(|s| s.name == "start")
        .ok_or_else(|| "start subcommand not found in schema".to_string())?;
    assert_eq!(start_sub.about, Some("Start the service".to_string()));

    Ok(())
}

#[test]
fn test_arg_matches_to_json() -> Result<(), String> {
    let cmd = create_test_command();
    let matches = cmd
        .try_get_matches_from(vec![
            "test-cli",
            "--port",
            "8080",
            "--verbose",
            "--tag",
            "tag1",
            "--tag",
            "tag2",
        ])
        .map_err(|e| format!("Failed to parse matches: {}", e))?;

    let json_val = arg_matches_to_json(&matches);

    let port_val =
        json_val.get("port").ok_or_else(|| "port not found in JSON matches".to_string())?;
    assert_eq!(port_val, &Value::Number(8080.into()));

    let verbose_val =
        json_val.get("verbose").ok_or_else(|| "verbose not found in JSON matches".to_string())?;
    assert_eq!(verbose_val, &Value::Bool(true));

    let tags_val =
        json_val.get("tags").ok_or_else(|| "tags not found in JSON matches".to_string())?;
    assert_eq!(
        tags_val,
        &Value::Array(vec![Value::String("tag1".to_string()), Value::String("tag2".to_string())])
    );

    Ok(())
}

#[derive(serde::Serialize)]
struct DummyConfig {
    pub enabled: bool,
}

#[test]
fn test_print_json() -> Result<(), String> {
    let dummy = DummyConfig { enabled: true };
    dummy.print_json().map_err(|e| format!("print_json failed: {}", e))?;
    dummy.print_json_pretty().map_err(|e| format!("print_json_pretty failed: {}", e))?;
    Ok(())
}
