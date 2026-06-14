// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # `#[verb]` Proc-Macro Auto-Registration Example
//!
//! Demonstrates the **primary** clap-noun-verb pattern: the `#[verb]` proc-macro
//! registers commands at compile time via `linkme` distributed slices, with zero
//! boilerplate at call sites.
//!
//! This is distinct from the `verb!()` builder-macro path shown in `core_api.rs`.
//! Here, the framework discovers and owns serialization; the handler just returns
//! `Result<T: Serialize>` and the framework produces JSON automatically.
//!
//! ## Capabilities witnessed
//!
//! - `#[verb("verb", "noun")]` — registers a function in the distributed slice
//! - `CommandRegistry::get()` — retrieves the process-global registry
//! - `execute_single_step(args)` — dispatches a command and returns `HandlerOutput`
//! - `HandlerOutput.data` — the `serde_json::Value` the handler serialized to
//! - Auto-inferred args from function signature (`String`, `u16`, `bool` parameters)
//! - `#[arg(...)]` attribute on parameters for env, default, description metadata
//!
//! ## Run
//!
//! ```sh
//! cargo run --example proc_macro_verb
//! ```
//!
//! ## Expected output
//!
//! ```text
//! services::status dispatched: running=true uptime=3600
//! services::restart dispatched: running=true uptime=0
//! config::set dispatched: key="debug" value="false" port=8080
//! ```
//!
//! **Doc**: README.md §"Zero Boilerplate", docs/tutorial/01-domain-separation.md
//! **Reference**: docs/reference/api/verb-macro.md

use clap_noun_verb::{cli::registry::CommandRegistry, Result};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Domain types — these are what the framework serializes to JSON
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ServiceStatus {
    running: bool,
    uptime: u64,
}

#[derive(Debug, Serialize)]
struct ConfigResult {
    key: String,
    value: String,
    port: u16,
}

// ---------------------------------------------------------------------------
// Registered verbs — the proc-macro wires each into __VERB_REGISTRY at compile
// time. The framework calls the init fn at first CommandRegistry::get().
// ---------------------------------------------------------------------------

/// Get current service status
#[clap_noun_verb_macros::verb("status", "services")]
fn services_status() -> Result<ServiceStatus> {
    Ok(ServiceStatus { running: true, uptime: 3600 })
}

/// Restart a service and return new uptime
#[clap_noun_verb_macros::verb("restart", "services")]
fn services_restart() -> Result<ServiceStatus> {
    Ok(ServiceStatus { running: true, uptime: 0 })
}

/// Set a configuration value
#[clap_noun_verb_macros::verb("set", "config")]
fn config_set(key: String, value: String, port: u16) -> Result<ConfigResult> {
    Ok(ConfigResult { key, value, port })
}

// ---------------------------------------------------------------------------
// Main — dispatch each verb with injected args and assert on the JSON output
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    // CommandRegistry::get() triggers __VERB_REGISTRY iteration, registering
    // all three verbs above into the process-global registry.
    let registry = CommandRegistry::get();
    let registry = registry.lock().map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("lock poisoned: {e}"))
    })?;

    // --- Witness 1: services status ---
    let output = registry.execute_single_step(vec![
        "myapp".into(),
        "services".into(),
        "status".into(),
    ])?;
    let running = output.data["running"].as_bool().expect("running must be bool");
    let uptime = output.data["uptime"].as_u64().expect("uptime must be u64");
    assert!(running, "services::status must report running=true");
    assert_eq!(uptime, 3600, "services::status must report uptime=3600");
    println!("services::status dispatched: running={running} uptime={uptime}");

    // --- Witness 2: services restart ---
    let output = registry.execute_single_step(vec![
        "myapp".into(),
        "services".into(),
        "restart".into(),
    ])?;
    let uptime = output.data["uptime"].as_u64().expect("uptime must be u64");
    assert_eq!(uptime, 0, "services::restart must reset uptime to 0");
    println!("services::restart dispatched: running=true uptime={uptime}");

    // --- Witness 3: config set with typed args ---
    let output = registry.execute_single_step(vec![
        "myapp".into(),
        "config".into(),
        "set".into(),
        "--key".into(),
        "debug".into(),
        "--value".into(),
        "false".into(),
        "--port".into(),
        "8080".into(),
    ])?;
    let key = output.data["key"].as_str().expect("key must be string");
    let value = output.data["value"].as_str().expect("value must be string");
    let port = output.data["port"].as_u64().expect("port must be number");
    assert_eq!(key, "debug");
    assert_eq!(value, "false");
    assert_eq!(port, 8080);
    println!("config::set dispatched: key=\"{key}\" value=\"{value}\" port={port}");

    Ok(())
}
