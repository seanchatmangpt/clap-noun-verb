// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Output Formats Example
//!
//! Demonstrates every `OutputFormat` variant and the `format_output` convenience
//! function. The framework serializes handler return values automatically; this
//! example shows the format system directly for cases where callers select format
//! at runtime (e.g. via `--format` CLI flag).
//!
//! ## Capabilities witnessed
//!
//! - `OutputFormat::Json` — compact JSON
//! - `OutputFormat::JsonPretty` — pretty-printed JSON (the default)
//! - `OutputFormat::Yaml` — YAML, no external deps
//! - `OutputFormat::Table` — ASCII table
//! - `OutputFormat::Plain` — key: value pairs
//! - `OutputFormat::Tsv` — tab-separated values
//! - `OutputFormat::Quiet` — empty string (for automation pipelines)
//! - `OutputFormat::available_formats()` — static format name list
//! - `OutputFormat::from_str()` — parse from a CLI `--format` argument string
//! - `format_output(&data, format)` — convenience fn
//!
//! ## Run
//!
//! ```sh
//! cargo run --example output_formats
//! ```
//!
//! ## Expected output
//!
//! ```text
//! json:        {"name":"api","replicas":3,"healthy":true}
//! json-pretty: {\n  "name": "api", ...
//! yaml:        name: api\nreplicas: 3\nhealthy: true
//! table:       ASCII table with Name/Replicas/Healthy columns
//! plain:       name: api\nreplicas: 3\nhealthy: true
//! tsv:         name\treplicas\thealthy header + values row
//! quiet:       (empty — zero bytes)
//! from_str:    "yaml" parses to OutputFormat::Yaml
//! ```
//!
//! **Doc**: docs/tutorial/04-output-formats.md, docs/reference/api-catalog.md
//! **Reference**: docs/reference/api/types.md

use clap_noun_verb::{
    format::{format_output, OutputFormat},
    Result,
};
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
struct ServiceInfo {
    name: String,
    replicas: u32,
    healthy: bool,
}

fn main() -> Result<()> {
    let service = ServiceInfo { name: "api".into(), replicas: 3, healthy: true };

    // --- Witness: Json (compact) ---
    let json = format_output(&service, OutputFormat::Json)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(json.contains("\"name\":\"api\""), "Json must contain name field: {json}");
    assert!(!json.contains('\n'), "Json (compact) must not contain newlines");
    println!("json:        {json}");

    // --- Witness: JsonPretty (default via #[default]) ---
    let pretty = OutputFormat::JsonPretty
        .format(&service)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(pretty.contains('\n'), "JsonPretty must be multi-line");
    assert!(pretty.contains("\"replicas\": 3"), "JsonPretty must have replicas field");
    println!("json-pretty: {}", pretty.lines().next().unwrap_or(""));

    // --- Witness: Yaml ---
    let yaml = format_output(&service, OutputFormat::Yaml)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    // The built-in YAML serializer quotes string values: name: "api"
    assert!(yaml.contains("name:"), "Yaml must contain 'name:' key: {yaml}");
    assert!(yaml.contains("api"), "Yaml must contain the value 'api': {yaml}");
    assert!(yaml.contains("replicas: 3"), "Yaml must contain 'replicas: 3': {yaml}");
    println!("yaml:        {}", yaml.lines().next().unwrap_or(""));

    // --- Witness: Table ---
    let table = format_output(&service, OutputFormat::Table)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(!table.is_empty(), "Table output must not be empty");
    println!("table:       {} (lines)", table.lines().count());

    // --- Witness: Plain ---
    let plain = format_output(&service, OutputFormat::Plain)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(
        plain.contains("name") && plain.contains("api"),
        "Plain must contain key: value pairs: {plain}"
    );
    println!("plain:       {}", plain.lines().next().unwrap_or(""));

    // --- Witness: Tsv ---
    let tsv = format_output(&service, OutputFormat::Tsv)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(tsv.contains('\t'), "Tsv must contain tab separators: {tsv:?}");
    println!("tsv:         {} lines, has tabs", tsv.lines().count());

    // --- Witness: Quiet (used in CI pipelines — returns empty string) ---
    let quiet = format_output(&service, OutputFormat::Quiet)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(quiet.is_empty(), "Quiet format must produce empty output, got: {quiet:?}");
    println!("quiet:       (empty — {} bytes)", quiet.len());

    // --- Witness: available_formats() — static list for --format help text ---
    let formats = OutputFormat::available_formats();
    assert!(formats.contains(&"json"), "available_formats must include 'json'");
    assert!(formats.contains(&"yaml"), "available_formats must include 'yaml'");
    assert!(formats.contains(&"quiet"), "available_formats must include 'quiet'");
    assert_eq!(formats.len(), 7, "must have exactly 7 format names");
    println!("formats:     {:?}", formats);

    // --- Witness: FromStr — parse from a --format CLI arg value ---
    let parsed = OutputFormat::from_str("yaml").expect("'yaml' must parse to OutputFormat::Yaml");
    let roundtrip = format_output(&service, parsed)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(
        roundtrip.contains("name:") && roundtrip.contains("api"),
        "from_str roundtrip must produce yaml output"
    );
    println!("from_str:    \"yaml\" → Yaml, produces {} bytes", roundtrip.len());

    // --- Witness: Display (for --format help strings) ---
    assert_eq!(OutputFormat::Json.to_string(), "json");
    assert_eq!(OutputFormat::JsonPretty.to_string(), "json-pretty");
    assert_eq!(OutputFormat::Yaml.to_string(), "yaml");
    println!("Display:     Json=\"json\", JsonPretty=\"json-pretty\", Yaml=\"yaml\"");

    Ok(())
}
