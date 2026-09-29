// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for `#[verb]` auto-registration and typed dispatch.

use clap_noun_verb::{cli::registry::CommandRegistry, Result};
use serde::Serialize;

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

#[clap_noun_verb_macros::verb("status", "services")]
fn services_status() -> Result<ServiceStatus> {
    Ok(ServiceStatus { running: true, uptime: 3600 })
}

#[clap_noun_verb_macros::verb("restart", "services")]
fn services_restart() -> Result<ServiceStatus> {
    Ok(ServiceStatus { running: true, uptime: 0 })
}

#[clap_noun_verb_macros::verb("set", "config")]
fn config_set(key: String, value: String, port: u16) -> Result<ConfigResult> {
    Ok(ConfigResult { key, value, port })
}

fn main() -> Result<()> {
    let registry = CommandRegistry::get();
    let registry = registry.lock().map_err(|error| {
        clap_noun_verb::NounVerbError::execution_error(format!("lock poisoned: {error}"))
    })?;

    let status =
        registry.execute_single_step(vec!["witness".into(), "services".into(), "status".into()])?;
    assert_eq!(status.data["running"].as_bool(), Some(true));
    assert_eq!(status.data["uptime"].as_u64(), Some(3600));

    let restart = registry.execute_single_step(vec![
        "witness".into(),
        "services".into(),
        "restart".into(),
    ])?;
    assert_eq!(restart.data["uptime"].as_u64(), Some(0));

    let config = registry.execute_single_step(vec![
        "witness".into(),
        "config".into(),
        "set".into(),
        "--key".into(),
        "debug".into(),
        "--value".into(),
        "false".into(),
        "--port".into(),
        "8080".into(),
    ])?;
    assert_eq!(config.data["key"].as_str(), Some("debug"));
    assert_eq!(config.data["port"].as_u64(), Some(8080));

    println!("Proc-macro registration admitted three typed dispatches");
    Ok(())
}
