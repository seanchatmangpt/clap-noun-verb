// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

mod common;

use clap_noun_verb_utils::adapters::{
    extract_key_value_pairs, parse_key_val, LayeredConfigAdapter,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, PartialEq)]
struct TestConfig {
    pub port: u16,
    pub host: String,
    pub verbose: bool,
}

#[test]
fn test_parse_key_val() -> Result<(), String> {
    let (k, v) = parse_key_val("foo=bar")?;
    assert_eq!(k, "foo");
    assert_eq!(v, "bar");

    let (k2, v2) = parse_key_val("  env = production  ")?;
    assert_eq!(k2, "env");
    assert_eq!(v2, "production");

    assert!(parse_key_val("invalid").is_err());
    Ok(())
}

#[test]
fn test_extract_key_value_pairs() -> Result<(), String> {
    let cmd = clap::Command::new("test")
        .arg(clap::Arg::new("kv").long("kv").action(clap::ArgAction::Append));
    let matches = cmd
        .try_get_matches_from(vec!["test", "--kv", "a=1", "--kv", "b=2"])
        .map_err(|e| format!("matches failed: {}", e))?;

    let map = extract_key_value_pairs(&matches, "kv")?;
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), "1".to_string());
    expected.insert("b".to_string(), "2".to_string());
    assert_eq!(map, expected);
    Ok(())
}

struct CleanupFile(PathBuf);
impl Drop for CleanupFile {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_file(&self.0);
        }
    }
}

#[test]
fn test_layered_config_adapter() -> Result<(), String> {
    // 1. Setup config file path
    let file_path = std::env::temp_dir().join("test_config_layered_adapter.json");
    let _cleanup = CleanupFile(file_path.clone());

    // Write default JSON config to file
    let json_content = r#"{"port": 8080, "host": "127.0.0.1", "verbose": false}"#;
    fs::write(&file_path, json_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    // 2. Setup env variables (prefix APP_)
    std::env::set_var("APP_PORT", "9090");
    std::env::set_var("APP_HOST", "10.0.0.1");

    // 3. Setup CLI args (port will not be overridden, verbose will be overridden to true, host overridden to 192.168.1.1)
    let cmd = clap::Command::new("test")
        .arg(clap::Arg::new("verbose").long("verbose").action(clap::ArgAction::SetTrue))
        .arg(clap::Arg::new("host").long("host").action(clap::ArgAction::Set));

    let matches = cmd
        .try_get_matches_from(vec!["test", "--verbose", "--host", "192.168.1.1"])
        .map_err(|e| format!("CLI matches failed: {}", e))?;

    // 4. Resolve layered config
    let adapter: LayeredConfigAdapter<TestConfig> =
        LayeredConfigAdapter::new(Some(file_path), Some("APP_".to_string()));

    let resolved = adapter.resolve(&matches).map_err(|e| format!("Resolve failed: {}", e))?;

    // Assert priority layers:
    // host: File ("127.0.0.1") -> Env ("10.0.0.1") -> CLI ("192.168.1.1") => CLI wins
    assert_eq!(resolved.host, "192.168.1.1");
    // port: File (8080) -> Env (9090) -> CLI (none) => Env wins
    assert_eq!(resolved.port, 9090);
    // verbose: File (false) -> Env (none) -> CLI (true) => CLI wins
    assert!(resolved.verbose);

    // Cleanup env
    std::env::remove_var("APP_PORT");
    std::env::remove_var("APP_HOST");

    Ok(())
}
