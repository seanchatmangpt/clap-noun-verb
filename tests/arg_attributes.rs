// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable acceptance coverage for parameter `#[arg(...)]` metadata.

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ConfigResult {
    port: u16,
    output: String,
}

#[verb("set", "argmeta")]
fn set_config(
    #[arg(short = 'p', default_value = "8080", value_name = "PORT")]
    port: u16,
    #[arg(short = 'o', default_value = "stdout", value_name = "DEST")]
    output: String,
) -> Result<ConfigResult> {
    Ok(ConfigResult { port, output })
}

#[test]
fn parameter_attributes_reach_the_clap_command() -> Result<()> {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().map_err(|error| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "failed to lock registry: {error}"
        ))
    })?;
    let command = registry.build_command();
    let noun = command
        .get_subcommands()
        .find(|subcommand| subcommand.get_name() == "argmeta")
        .ok_or_else(|| clap_noun_verb::error::NounVerbError::execution_error("argmeta noun missing"))?;
    let verb = noun
        .get_subcommands()
        .find(|subcommand| subcommand.get_name() == "set")
        .ok_or_else(|| clap_noun_verb::error::NounVerbError::execution_error("set verb missing"))?;

    let port = verb
        .get_arguments()
        .find(|argument| argument.get_id().as_str() == "port")
        .ok_or_else(|| clap_noun_verb::error::NounVerbError::execution_error("port arg missing"))?;
    assert_eq!(port.get_short(), Some('p'));
    assert_eq!(port.get_default_values()[0].to_string_lossy(), "8080");
    assert_eq!(port.get_value_names().expect("PORT value name")[0].as_str(), "PORT");

    let output = verb
        .get_arguments()
        .find(|argument| argument.get_id().as_str() == "output")
        .ok_or_else(|| clap_noun_verb::error::NounVerbError::execution_error("output arg missing"))?;
    assert_eq!(output.get_short(), Some('o'));
    assert_eq!(output.get_default_values()[0].to_string_lossy(), "stdout");
    assert_eq!(output.get_value_names().expect("DEST value name")[0].as_str(), "DEST");

    Ok(())
}
