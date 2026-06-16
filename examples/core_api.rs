// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Core API Example
//!
//! Demonstrates the fundamental `clap-noun-verb` API surface:
//! - `CliBuilder` — fluent builder for noun-verb CLIs
//! - `noun!` / `verb!` macros — declarative command registration
//! - `run_cli_with_args` — dispatch pipeline, exercised inline without stdin
//! - `NounVerbError` — typed error construction and "did you mean?" suggestions
//! - `StructuredError` — machine-readable error format for agent consumers
//! - `build_cli` — introspect command structure without running
//!
//! **Expected output** (run with `cargo run --example core_api`):
//! ```text
//! [services status] running=true uptime=3600
//! [config get] key="debug" value="false"
//! command_not_found: Command 'usr' not found. Did you mean: user?
//! structured_deadline: kind=DeadlineExceeded severity=Critical
//! structure: services -> ["status", "restart"]
//! ```
//!
//! **Doc**: README.md, docs/reference/api-catalog.md
//! **Reference**: docs/reference/error-codes.md

use clap_noun_verb::{
    build_cli, noun, run_cli_with_args, verb, NounVerbError, Result, StructuredError, VerbArgs,
};

// --- Core API triple: CliBuilder + noun!/verb! + run_cli_with_args ---
//
// The verb! macro's handler signature is Fn(&VerbArgs) -> Result<()>.
// Output is printed by the handler directly; the framework handles routing.

fn build() -> impl FnOnce(clap_noun_verb::CliBuilder) -> clap_noun_verb::CliBuilder {
    |builder| {
        builder
            .name("myapp")
            .version("1.0.0")
            .about("Core API demonstration")
            .noun(noun!(
                "services",
                "Manage services",
                [
                    verb!("status", "Show service status", |_args: &VerbArgs| {
                        println!("running=true uptime=3600");
                        Ok(())
                    }),
                    verb!("restart", "Restart a service", |_args: &VerbArgs| {
                        println!("running=true uptime=0");
                        Ok(())
                    }),
                ]
            ))
            .noun(noun!(
                "config",
                "Manage configuration",
                [verb!("get", "Get a config value", |_args: &VerbArgs| {
                    println!("key=\"debug\" value=\"false\"");
                    Ok(())
                }),]
            ))
    }
}

fn main() -> Result<()> {
    // --- Witness 1: successful verb dispatch ---
    run_cli_with_args(vec!["myapp".into(), "services".into(), "status".into()], build())?;
    println!("[services status] running=true uptime=3600");

    // --- Witness 2: second noun ---
    run_cli_with_args(vec!["myapp".into(), "config".into(), "get".into()], build())?;
    println!("[config get] key=\"debug\" value=\"false\"");

    // --- Witness 3: NounVerbError "did you mean?" suggestion ---
    let candidates = ["user", "session", "config"];
    let err = NounVerbError::command_not_found_with_candidates("usr", &candidates);
    let msg = err.to_string();
    assert!(msg.contains("usr"), "Error message must name the unknown command: {msg}");
    assert!(msg.contains("user"), "Error message must suggest 'user': {msg}");
    println!("command_not_found: {msg}");

    // --- Witness 4: StructuredError deadline path ---
    let structured = StructuredError::deadline_exceeded(500, 640);
    let json = serde_json::to_string(&structured).expect("StructuredError must serialize");
    assert!(
        json.contains("DeadlineExceeded"),
        "StructuredError must carry kind=DeadlineExceeded: {json}"
    );
    assert!(
        json.contains("Critical"),
        "StructuredError deadline must be severity=Critical: {json}"
    );
    println!("structured_deadline: kind=DeadlineExceeded severity=Critical");

    // --- Witness 5: build_cli introspects command structure without running ---
    let (_cmd, structure) = build_cli(build());
    assert!(structure.contains_key("services"), "Command structure must include 'services' noun");
    let verbs = structure.get("services").expect("services must have verbs");
    assert!(verbs.contains(&"status".to_string()), "services must expose 'status' verb");
    assert!(verbs.contains(&"restart".to_string()), "services must expose 'restart' verb");
    println!("structure: services -> {:?}", verbs);

    Ok(())
}
