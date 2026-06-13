// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use clap::{Arg, ArgAction, Command};
use clap_noun_verb_utils::adapters::LayeredConfigAdapter;
use clap_noun_verb_utils::display_json::arg_matches_to_json;
use clap_noun_verb_utils::number_parsing::{decimal_range, parse_duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::Duration as StdDuration;

// =========================================================================
// 1. Thread-safety and Panic-abort Safety of `arg_matches_to_json`
// =========================================================================

#[test]
fn test_arg_matches_to_json_thread_safety() {
    let cmd = Command::new("test-app")
        .arg(Arg::new("port").long("port").action(ArgAction::Set))
        .arg(Arg::new("verbose").long("verbose").action(ArgAction::SetTrue))
        .arg(Arg::new("tags").long("tag").action(ArgAction::Append));

    let matches = cmd
        .try_get_matches_from(vec![
            "test-app",
            "--port",
            "8080",
            "--verbose",
            "--tag",
            "admin",
            "--tag",
            "web",
        ])
        .unwrap();

    let shared_matches = Arc::new(matches);
    let mut handles = vec![];

    // Spawn 20 concurrent threads reading and parsing the same ArgMatches reference
    for _ in 0..20 {
        let matches_clone = Arc::clone(&shared_matches);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let json = arg_matches_to_json(&matches_clone);
                assert_eq!(json["port"], serde_json::Value::Number(8080.into()));
                assert_eq!(json["verbose"], serde_json::Value::Bool(true));
                assert_eq!(
                    json["tags"],
                    serde_json::Value::Array(vec![
                        serde_json::Value::String("admin".to_string()),
                        serde_json::Value::String("web".to_string())
                    ])
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked during concurrent read stress test");
    }
}

// =========================================================================
// 2. Overflow safety and empty input validation of `parse_duration`
//    and range bounds configuration safety in `decimal_range`
// =========================================================================

#[test]
fn test_parse_duration_verification() {
    // Empty input validation
    assert!(parse_duration("").is_err());
    assert!(parse_duration("   ").is_err());

    // Single segment overflow safety (u64::MAX minutes overflows u64 seconds)
    assert!(parse_duration("18446744073709551615m").is_err());

    // Segment addition overflow safety
    assert!(parse_duration("18446744073709551615s 1s").is_err());
    assert!(parse_duration("18446744073709551000s 1000s").is_err());

    // Valid duration parsing
    assert_eq!(parse_duration("1h 30m").unwrap(), StdDuration::from_secs(5400));
}

#[test]
fn test_decimal_range_verification() {
    // Range bounds configuration safety: min > max must return an Err, not panic or allow invalid range
    let parse_invalid = decimal_range(100, 50);
    assert!(parse_invalid("75").is_err());
    assert!(parse_invalid("120").is_err());
    assert!(parse_invalid("25").is_err());

    // Valid range configuration
    let parse_valid = decimal_range(10, 50);
    assert_eq!(parse_valid("25").unwrap(), 25);
    assert!(parse_valid("5").is_err());
    assert!(parse_valid("55").is_err());
}

// =========================================================================
// 3. LayeredConfigAdapter resolution and recursive merging
// =========================================================================

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
struct SubConfig {
    pub port: u16,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
struct NestedConfig {
    pub host: String,
    pub server: SubConfig,
    pub debug: bool,
}

#[test]
fn test_layered_config_adapter_overrides_and_nesting() {
    let file_path = std::env::temp_dir().join("test_config_verification_nested.json");

    // Write JSON config file representing file layer
    let json_content = r#"{
        "host": "file.host",
        "server": {
            "port": 8080,
            "path": "/file"
        },
        "debug": false
    }"#;
    std::fs::write(&file_path, json_content).unwrap();

    // Set Environment variables representing Env layer (using prefix VERIFY_)
    // Double underscore indicates nested path: VERIFY_SERVER__PORT -> server.port
    std::env::set_var("VERIFY_SERVER__PORT", "9090");
    std::env::set_var("VERIFY_DEBUG", "true");

    // CLI configuration (representing CLI layer) with default values
    let cmd = Command::new("verify-app")
        .arg(Arg::new("host").long("host").default_value("default.host").action(ArgAction::Set))
        .arg(Arg::new("server.path").long("path").action(ArgAction::Set));

    // Case A: CLI overrides not supplied.
    // Result should be:
    // host: default.host (CLI default value) -> wins? NO, CLI default should not override file/env. So host should be "file.host".
    // server.port: 9090 (from Env)
    // server.path: "/file" (from File)
    // debug: true (from Env)
    let matches_default = cmd.clone().try_get_matches_from(vec!["verify-app"]).unwrap();
    let adapter: LayeredConfigAdapter<NestedConfig> =
        LayeredConfigAdapter::new(Some(file_path.clone()), Some("VERIFY_".to_string()));

    let resolved_default = adapter.resolve(&matches_default).unwrap();
    assert_eq!(resolved_default.host, "file.host");
    assert_eq!(resolved_default.server.port, 9090);
    assert_eq!(resolved_default.server.path, "/file");
    assert!(resolved_default.debug);

    // Case B: CLI overrides supplied.
    // host: explicitly passed as "cli.host".
    // server.path: explicitly passed as "/cli".
    // Result: host -> cli.host, server.path -> /cli, server.port -> 9090, debug -> true
    let matches_override = cmd
        .clone()
        .try_get_matches_from(vec!["verify-app", "--host", "cli.host", "--path", "/cli"])
        .unwrap();

    let resolved_override = adapter.resolve(&matches_override).unwrap();
    assert_eq!(resolved_override.host, "cli.host");
    assert_eq!(resolved_override.server.port, 9090);
    assert_eq!(resolved_override.server.path, "/cli");
    assert!(resolved_override.debug);

    // Cleanup
    std::fs::remove_file(&file_path).ok();
    std::env::remove_var("VERIFY_SERVER__PORT");
    std::env::remove_var("VERIFY_DEBUG");
}
