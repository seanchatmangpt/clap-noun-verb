//! Configuration management
//!
//! Get, set, and show configuration values.
//!
//! Following the golden rule: CLI validates, domain computes, integration connects.

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

use std::str::FromStr;
use clap_noun_verb::{OutputFormat, format_output};
use crate::domain::Config;

/// Emit deprecation warning for migrated commands
fn emit_deprecation(message: &str) {
    eprintln!("⚠️  DEPRECATED: {}", message);
    eprintln!("   This command will be removed in v27.0.0");
}

/// Get configuration value
///
/// Retrieves a single configuration value by key.
///
/// # Arguments
/// * `key` - Configuration key
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("get")]
fn get_config(key: String, format: Option<String>) -> Result<()> {
    emit_deprecation("Config moved to environment variables and policy profiles");

    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    let config = Config::default();
    let output = format_output(
        &crate::outputs::ConfigValueOutput {
            key: key.clone(),
            value: config.get(&key).map(|s| s.to_string()),
            valid_key: Config::is_valid_key(&key),
        },
        fmt,
    )
    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}

/// Set configuration value
///
/// Sets a configuration value. Note: Changes are not persisted.
///
/// # Arguments
/// * `key` - Configuration key
/// * `value` - Configuration value
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("set")]
fn set_config(key: String, value: String, format: Option<String>) -> Result<()> {
    emit_deprecation("Config moved to environment variables and policy profiles");

    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    let valid_key = Config::is_valid_key(&key);
    let output = format_output(
        &crate::outputs::ConfigSetOutput {
            key,
            value,
            valid_key,
            saved: true,
        },
        fmt,
    )
    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}

/// Show all configuration
///
/// Displays all configuration key-value pairs.
///
/// # Arguments
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("show")]
fn show_config(format: Option<String>) -> Result<()> {
    emit_deprecation("Config moved to environment variables and policy profiles");

    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    let config = Config::default();
    let entries: std::collections::HashMap<_, _> = config
        .all_entries()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let output = format_output(&crate::outputs::ConfigAllOutput { entries }, fmt)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}
