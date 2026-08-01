// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Agent-oriented CLI construction with deterministic JSON output.

use clap_noun_verb::{noun, run_cli_with_args, verb, OutputFormat, Result, VerbArgs};

fn build() -> impl FnOnce(clap_noun_verb::CliBuilder) -> clap_noun_verb::CliBuilder {
    |builder| {
        builder.name("agent-cli").version("26.7.62").noun(noun!(
            "capability",
            "Inspect capability standing",
            [verb!("describe", "Describe the core capability", |_args: &VerbArgs| {
                let output = serde_json::json!({
                    "capability": "core",
                    "standing": "PARTIAL_ALIVE",
                    "actuation_performed": false
                });
                let rendered = OutputFormat::Json
                    .format(&output)
                    .map_err(|error| clap_noun_verb::NounVerbError::execution_error(error.to_string()))?;
                println!("{rendered}");
                Ok(())
            })]
        ))
    }
}

fn main() -> Result<()> {
    run_cli_with_args(
        vec!["agent-cli".into(), "capability".into(), "describe".into()],
        build(),
    )?;
    println!("Agent CLI builder dispatched a typed read-only capability route");
    Ok(())
}
