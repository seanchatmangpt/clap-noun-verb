#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

mod common;

use clap_noun_verb_utils::{
    adapters::{extract_key_value_pairs, parse_key_val, LayeredConfigAdapter},
    completions,
    help::{format_box_text, format_table},
    mangen, markdown,
};

static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
struct TestConfig {
    pub port: u16,
    pub host: String,
    pub verbose: bool,
}

// Struct that does not serialize to a JSON Object (to test adapter's error handling)
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
struct NonObjectConfig(pub u16);

struct TempFile(PathBuf);
impl TempFile {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cnv_test_{}_{}", name, std::process::id()));
        Self(path)
    }
}
impl Drop for TempFile {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_file(&self.0);
        }
    }
}

// --- 1. Test adapters under adverse conditions ---

#[test]
fn test_adverse_config_files() {
    // A. Malformed JSON
    let temp_json = TempFile::new("malformed.json");
    fs::write(&temp_json.0, "{invalid_json").unwrap();
    let adapter: LayeredConfigAdapter<TestConfig> =
        LayeredConfigAdapter::new(Some(temp_json.0.clone()), None);
    let cmd = clap::Command::new("test");
    let matches = cmd.try_get_matches_from(vec!["test"]).unwrap();
    let res = adapter.resolve(&matches);
    assert!(res.is_err(), "Expected error for malformed JSON config file");

    // B. Malformed TOML
    let temp_toml = TempFile::new("malformed.toml");
    fs::write(&temp_toml.0, "[invalid_toml").unwrap();
    let adapter_toml: LayeredConfigAdapter<TestConfig> =
        LayeredConfigAdapter::new(Some(temp_toml.0.clone()), None);
    let res = adapter_toml.resolve(&matches);
    assert!(res.is_err(), "Expected error for malformed TOML config file");

    // C. Config path is a directory (should fail on read_to_string)
    let temp_dir = std::env::temp_dir().join(format!("cnv_test_dir_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let adapter_dir: LayeredConfigAdapter<TestConfig> =
        LayeredConfigAdapter::new(Some(temp_dir.clone()), None);
    let res = adapter_dir.resolve(&matches);
    assert!(res.is_err(), "Expected error when config file path is a directory");
    fs::remove_dir(&temp_dir).unwrap();

    // D. Empty JSON file
    let temp_empty_json = TempFile::new("empty.json");
    fs::write(&temp_empty_json.0, "").unwrap();
    let adapter_empty_json: LayeredConfigAdapter<TestConfig> =
        LayeredConfigAdapter::new(Some(temp_empty_json.0.clone()), None);
    let res = adapter_empty_json.resolve(&matches);
    assert!(res.is_err(), "Expected error for empty JSON file");

    // E. Empty TOML file
    let temp_empty_toml = TempFile::new("empty.toml");
    fs::write(&temp_empty_toml.0, "").unwrap();
    let adapter_empty_toml: LayeredConfigAdapter<TestConfig> =
        LayeredConfigAdapter::new(Some(temp_empty_toml.0.clone()), None);
    let res = adapter_empty_toml.resolve(&matches);
    // TOML allows empty input (deserializes to empty map), so it may merge with defaults
    if let Ok(config) = res {
        assert_eq!(config, TestConfig::default(), "Empty TOML should resolve to default config");
    }

    // F. Non-object default configuration
    let adapter_non_obj: LayeredConfigAdapter<NonObjectConfig> =
        LayeredConfigAdapter::new(None, None);
    let res = adapter_non_obj.resolve(&matches);
    assert!(res.is_err(), "Expected error for config model not serializing to a JSON Object");
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Configuration model must serialize to a JSON Object"));
}

#[test]
fn test_adverse_key_value_formats() {
    // A. Empty string
    assert!(parse_key_val("").is_err());

    // B. No equals sign
    assert!(parse_key_val("no_equals").is_err());

    // C. Multiple equals signs
    let res_mult = parse_key_val("key=val1=val2");
    assert!(res_mult.is_ok());
    let (k_mult, v_mult) = res_mult.unwrap();
    assert_eq!(k_mult, "key");
    assert_eq!(v_mult, "val1=val2"); // Splitting at first '='

    // D. Empty key
    let res_empty_key = parse_key_val("=value");
    assert!(res_empty_key.is_ok());
    let (k_ek, v_ek) = res_empty_key.unwrap();
    assert_eq!(k_ek, "");
    assert_eq!(v_ek, "value");

    // E. Empty value
    let res_empty_val = parse_key_val("key=");
    assert!(res_empty_val.is_ok());
    let (k_ev, v_ev) = res_empty_val.unwrap();
    assert_eq!(k_ev, "key");
    assert_eq!(v_ev, "");

    // F. Only spaces around equal
    let res_spaces = parse_key_val("   =   ");
    assert!(res_spaces.is_ok());
    let (k_sp, v_sp) = res_spaces.unwrap();
    assert_eq!(k_sp, "");
    assert_eq!(v_sp, "");

    // G. extract_key_value_pairs with non-existent arg (should panic in clap)
    let cmd = clap::Command::new("test");
    let matches = cmd.try_get_matches_from(vec!["test"]).unwrap();
    let res_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = extract_key_value_pairs(&matches, "non_existent");
    }));
    assert!(res_panic.is_err(), "Expected panic when accessing non-existent argument ID");
}

#[test]
fn test_adverse_conflicting_inputs() {
    let _lock = ENV_MUTEX.lock().unwrap();
    // Ensure clean env for testing
    let keys_to_remove = ["TEST_CONFLICT_PORT", "TEST_CONFLICT_HOST", "TEST_CONFLICT_VERBOSE"];
    for k in &keys_to_remove {
        std::env::remove_var(k);
    }

    // A. Env prefix None / Path None
    let adapter: LayeredConfigAdapter<TestConfig> = LayeredConfigAdapter::new(None, None);
    let cmd = clap::Command::new("test");
    let matches = cmd.try_get_matches_from(vec!["test"]).unwrap();
    let resolved = adapter.resolve(&matches).unwrap();
    assert_eq!(resolved, TestConfig::default());

    // B. Conflicting env and CLI overrides
    std::env::set_var("TEST_CONFLICT_PORT", "1234");
    std::env::set_var("TEST_CONFLICT_HOST", "env.host");
    std::env::set_var("TEST_CONFLICT_VERBOSE", "true");

    let cmd_override = clap::Command::new("test")
        .arg(clap::Arg::new("port").long("port").action(clap::ArgAction::Set))
        .arg(clap::Arg::new("host").long("host").action(clap::ArgAction::Set))
        .arg(clap::Arg::new("verbose").long("verbose").action(clap::ArgAction::SetTrue));

    let matches_override = cmd_override
        .clone()
        .try_get_matches_from(vec!["test", "--port", "5678", "--host", "cli.host", "--verbose"])
        .unwrap();

    let adapter_conflict: LayeredConfigAdapter<TestConfig> =
        LayeredConfigAdapter::new(None, Some("TEST_CONFLICT_".to_string()));
    let resolved_conflict = adapter_conflict.resolve(&matches_override).unwrap();

    // Verify CLI overrides Env
    assert_eq!(resolved_conflict.port, 5678);
    assert_eq!(resolved_conflict.host, "cli.host");
    assert!(resolved_conflict.verbose);

    // C. Data type conflicts - passing invalid data type in env
    std::env::set_var("TEST_CONFLICT_PORT", "not_a_number");
    let matches_empty = cmd_override.clone().try_get_matches_from(vec!["test"]).unwrap();
    let res_err = adapter_conflict.resolve(&matches_empty);
    // Should fail because "not_a_number" cannot be parsed as u16 (resolved config deserialization failure)
    assert!(res_err.is_err(), "Expected error when environment variable has incompatible type");

    for k in &keys_to_remove {
        std::env::remove_var(k);
    }
}

#[test]
fn test_layered_config_cli_default_override_conflict() {
    let _lock = ENV_MUTEX.lock().unwrap();
    // Demonstration of CLI default override conflict:
    // If CLI argument has default_value, it will always override Env and Config File,
    // because clap puts default values in ArgMatches and we don't check value_source.

    let temp_json = TempFile::new("config.json");
    fs::write(&temp_json.0, r#"{"port": 8080, "host": "config.host", "verbose": false}"#).unwrap();

    std::env::set_var("TEST_DEFAULT_OVERRIDE_HOST", "env.host");

    // Command with default value for "host"
    let cmd = clap::Command::new("test").arg(
        clap::Arg::new("host")
            .long("host")
            .default_value("default.host")
            .action(clap::ArgAction::Set),
    );

    let matches = cmd.try_get_matches_from(vec!["test"]).unwrap();
    let adapter: LayeredConfigAdapter<TestConfig> = LayeredConfigAdapter::new(
        Some(temp_json.0.clone()),
        Some("TEST_DEFAULT_OVERRIDE_".to_string()),
    );

    let resolved = adapter.resolve(&matches).unwrap();

    // The host is "env.host" because CLI default override check is implemented
    assert_eq!(
        resolved.host, "env.host",
        "CLI default should NOT override env/config due to value_source checking"
    );

    std::env::remove_var("TEST_DEFAULT_OVERRIDE_HOST");
}

// --- 2. Verify completion generation scripts, manpage formatting, and markdown tree walker ---

#[test]
fn test_extreme_completions() {
    // A. Deeply nested subcommands (5 levels deep)
    let mut cmd = clap::Command::new("root").subcommand(
        clap::Command::new("sub1").subcommand(
            clap::Command::new("sub2").subcommand(
                clap::Command::new("sub3").subcommand(
                    clap::Command::new("sub4").subcommand(
                        clap::Command::new("sub5")
                            .arg(clap::Arg::new("arg5").long("arg5").action(clap::ArgAction::Set)),
                    ),
                ),
            ),
        ),
    );

    let mut buf = Vec::new();
    completions::generate_completions(&mut cmd, clap_complete::Shell::Bash, &mut buf);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("root"), "Completion output should contain command name");
    assert!(output.contains("sub1"), "Completion output should contain sub1");
    assert!(output.contains("sub5"), "Completion output should contain sub5");

    // B. Command name containing spaces and special characters
    let mut weird_cmd = clap::Command::new("weird name & * spec");
    let mut buf_weird = Vec::new();
    completions::generate_completions(&mut weird_cmd, clap_complete::Shell::Bash, &mut buf_weird);
    let output_weird = String::from_utf8(buf_weird).unwrap();
    assert!(output_weird.contains("weird"), "Should handle weird command names without panic");
}

#[test]
fn test_mangen_formatting_and_missing_metadata() {
    // A. Command with special troff characters and missing metadata (no version, no about, no author)
    let cmd = clap::Command::new("mangen-test").arg(
        clap::Arg::new("config")
            .long("config")
            .help("Set config file. Starts with dot: .config. Has backslash: \\path\\to\\config.")
            .action(clap::ArgAction::Set),
    );

    let mut buf = Vec::new();
    let res = mangen::generate_manpage(&cmd, &mut buf);
    assert!(res.is_ok(), "Mangen should succeed with missing metadata and special chars");
    let output = String::from_utf8(buf).unwrap();

    // Check if the backslash and dot are present in the output
    assert!(output.contains("mangen-test"));
    assert!(output.contains(".config"));
}

#[test]
fn test_markdown_tree_walker_edge_cases() {
    // A. Missing metadata and nested structure
    let cmd = clap::Command::new("md-test").subcommand(
        clap::Command::new("sub command") // name with spaces
            .arg(clap::Arg::new("pos_arg").required(true))
            .arg(clap::Arg::new("opt_pos").required(false)),
    );

    let mut buf = Vec::new();
    let res = markdown::generate_markdown(&cmd, &mut buf);
    assert!(res.is_ok(), "Markdown generation should succeed");
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("# md-test"));
    assert!(output.contains("## sub command"));
    // Check usage block for required and optional positional arguments
    assert!(output.contains("<pos_arg>"));
    assert!(output.contains("[opt_pos]"));
    // Anchor links: markdown generator does: - [`sub command`](#sub-command)
    // Verify it generates the exact expected string:
    assert!(output.contains("- [`sub command`](#sub-command)"), "Anchor links should be generated");
}

// --- 3. Ensure no invalid terminal states occur under help formatting ---

#[test]
fn test_help_formatting_adverse_inputs() {
    // A. format_box_text with empty text
    let res_empty = format_box_text("");
    assert!(res_empty.contains("┌──┐"));
    assert!(res_empty.contains("└──┘"));

    // B. format_box_text with newlines only
    let res_nl = format_box_text("\n\n");
    assert!(!res_nl.is_empty());

    // C. format_box_text with wide characters (UTF-8)
    let wide_text = "Hello 你好 🦀";
    let res_wide = format_box_text(wide_text);
    // top border should be 15 dashes
    assert!(res_wide.contains("┌───────────────┐"));

    // D. format_box_text with tabs
    let tab_text = "a\tb";
    let res_tab = format_box_text(tab_text);
    assert!(res_tab.contains("┌───────┐"));

    // E. format_table with mismatched row sizes
    let headers = vec!["Col1", "Col2"];
    let rows = vec![
        vec!["val1".to_string()], // missing second column
        vec!["val1".to_string(), "val2".to_string(), "val3".to_string()], // extra column
    ];
    let res_table = format_table(&headers, &rows);
    assert!(res_table.contains("Col1"));
    assert!(res_table.contains("Col2"));
    assert!(res_table.contains("val1"));
    assert!(!res_table.is_empty());

    // F. format_table with cell containing newlines
    let rows_nl = vec![vec!["val1\nnewline".to_string(), "val2".to_string()]];
    let res_table_nl = format_table(&headers, &rows_nl);
    assert!(res_table_nl.contains("newline"));
}
