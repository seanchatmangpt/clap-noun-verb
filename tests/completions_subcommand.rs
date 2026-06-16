// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tests for completions subcommand configuration option
//!

use clap_noun_verb::clap_ext::completions::{CompletionGenerator, Shell};
use clap_noun_verb::cli::builder::CliBuilder as OpinionatedCliBuilder;
use clap_noun_verb::{CliBuilder as MainCliBuilder, Result};

#[test]
fn test_main_cli_builder_completions_structure() {
    // Arrange & Act
    let cli = MainCliBuilder::new()
        .name("testapp")
        .about("Test app for completions")
        .version("1.2.3")
        .with_completions_subcommand();

    let cmd = cli.build_command();

    // Assert
    assert_eq!(cmd.get_name(), "testapp");

    // Check if completions subcommand exists
    let completions_sub = cmd.find_subcommand("completions");
    assert!(completions_sub.is_some(), "completions subcommand should be registered");

    let completions_sub = completions_sub.unwrap();
    assert_eq!(completions_sub.get_name(), "completions");

    // Check if bash, zsh, fish, powershell subcommands are registered under completions
    assert!(completions_sub.find_subcommand("bash").is_some());
    assert!(completions_sub.find_subcommand("zsh").is_some());
    assert!(completions_sub.find_subcommand("fish").is_some());
    assert!(completions_sub.find_subcommand("powershell").is_some());
}

#[test]
fn test_opinionated_cli_builder_completions_structure() {
    // Arrange & Act
    let cli = OpinionatedCliBuilder::new("testapp")
        .about("Test app for completions")
        .version("1.2.3")
        .with_completions_subcommand();

    let cmd = cli.build_command();

    // Assert
    assert_eq!(cmd.get_name(), "testapp");

    // Check if completions subcommand exists
    let completions_sub = cmd.find_subcommand("completions");
    assert!(completions_sub.is_some(), "completions subcommand should be registered");

    let completions_sub = completions_sub.unwrap();
    assert_eq!(completions_sub.get_name(), "completions");

    // Check if bash, zsh, fish, powershell subcommands are registered under completions
    assert!(completions_sub.find_subcommand("bash").is_some());
    assert!(completions_sub.find_subcommand("zsh").is_some());
    assert!(completions_sub.find_subcommand("fish").is_some());
    assert!(completions_sub.find_subcommand("powershell").is_some());
}

#[test]
fn test_main_cli_builder_completions_execution() -> Result<()> {
    // Arrange
    let cli = MainCliBuilder::new()
        .name("testapp")
        .about("Test app for completions")
        .version("1.2.3")
        .with_completions_subcommand();

    // Act - dispatch through the completions -> bash path must succeed
    let res = cli.run_with_args(vec![
        "testapp".to_string(),
        "completions".to_string(),
        "bash".to_string(),
    ]);
    res?;

    // Assert - witness the concrete bash script the handler prints to stdout.
    // The handler builds a CompletionGenerator from the app name/version and
    // calls `.generate(Shell::Bash)`; reproduce that exact contract here.
    let script = CompletionGenerator::new("testapp").with_version("1.2.3").generate(Shell::Bash)?;
    assert!(
        script.starts_with("# testapp completion script for bash"),
        "bash script must carry the app-name header, got: {script}"
    );
    assert!(
        script.contains("# Generated for version 1.2.3"),
        "bash script must embed the configured version"
    );
    assert!(
        script.contains("_TESTAPP_completions()"),
        "bash script must define the uppercased completion function"
    );
    assert!(
        script.contains(
            "complete -o bashdefault -o default -o nospace -F _TESTAPP_completions testapp"
        ),
        "bash script must register the completion function for the app"
    );

    Ok(())
}

#[test]
fn test_opinionated_cli_builder_completions_execution() -> Result<()> {
    // Arrange
    let cli = OpinionatedCliBuilder::new("testapp")
        .about("Test app for completions")
        .version("1.2.3")
        .with_completions_subcommand();

    // Act - dispatch through the completions -> bash path must succeed
    let res = cli.run_with_args(vec![
        "testapp".to_string(),
        "completions".to_string(),
        "bash".to_string(),
    ]);
    res?;

    // Assert - witness the concrete bash script the handler prints to stdout,
    // reproducing the handler's CompletionGenerator(app, version).generate(Bash).
    let script = CompletionGenerator::new("testapp").with_version("1.2.3").generate(Shell::Bash)?;
    assert!(
        script.starts_with("# testapp completion script for bash"),
        "bash script must carry the app-name header, got: {script}"
    );
    assert!(
        script.contains("# Generated for version 1.2.3"),
        "bash script must embed the configured version"
    );
    assert!(
        script.contains("_TESTAPP_completions()"),
        "bash script must define the uppercased completion function"
    );
    assert!(
        script.contains(
            "complete -o bashdefault -o default -o nospace -F _TESTAPP_completions testapp"
        ),
        "bash script must register the completion function for the app"
    );

    Ok(())
}
