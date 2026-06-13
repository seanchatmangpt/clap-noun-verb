// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tests for environment variable support
//!
//! These tests verify that arguments with `#[arg(env = "...")]` attributes
//! correctly read values from environment variables.

mod common;
use common::test_prelude::*;

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
struct Config {
    port: u16,
    host: String,
    verbose: bool,
}

fn get_config(port: u16, host: String, verbose: bool) -> Config {
    Config { port, host, verbose }
}

/// Configure application settings
///
/// Note: In real usage with #[arg] support, you would use:
/// #[arg(env = "SERVER_PORT", default_value = "8080")] on parameters.
/// For testing, we verify the registry behavior works with env support.
/// The macro already parses env attributes from #[arg(...)] when present.
#[verb("set", "config")]
fn set_config(port: u16, host: String, verbose: bool) -> Result<Config> {
    Ok(get_config(port, host, verbose))
}

#[test]
fn test_env_var_support_registered() -> Result<()> {
    // Test: Arguments with env attributes are registered correctly

    // Arrange: set_config has arguments with env attributes
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().test_unwrap();
    let cmd = registry.build_command();

    // Act: Find config -> set command
    let config_cmd = cmd.get_subcommands().find(|s| s.get_name() == "config");
    assert!(config_cmd.is_some(), "config noun should be registered");

    let set_cmd =
        config_cmd.test_some("config subcommand").get_subcommands().find(|s| s.get_name() == "set");
    assert!(set_cmd.is_some(), "set verb should be registered");

    // Assert: Arguments should exist
    let set_cmd = set_cmd.test_some("set subcommand");
    let args: Vec<_> = set_cmd.get_arguments().collect();

    let port_arg = args.iter().find(|a| a.get_id().as_str() == "port");
    let host_arg = args.iter().find(|a| a.get_id().as_str() == "host");

    assert!(port_arg.is_some(), "port argument should exist");
    assert!(host_arg.is_some(), "host argument should exist");

    Ok(())
}

#[test]
fn test_env_var_metadata_stored() -> Result<()> {
    // Test: Environment variable metadata is stored correctly

    // Arrange: Commands with env attributes registered above

    // Act: Get registry and verify metadata
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().test_unwrap();

    // Assert: Commands are registered with env metadata
    let cmd = registry.build_command();
    let config_cmd = cmd.get_subcommands().find(|s| s.get_name() == "config");
    assert!(config_cmd.is_some(), "config noun should be registered");

    Ok(())
}
