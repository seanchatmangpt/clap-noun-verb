//! Tests for completions subcommand configuration option
//!

use clap_noun_verb::{CliBuilder as MainCliBuilder, Result};
use clap_noun_verb::cli::builder::CliBuilder as OpinionatedCliBuilder;

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

    // Act & Assert - should run successfully
    let res = cli.run_with_args(vec![
        "testapp".to_string(),
        "completions".to_string(),
        "bash".to_string(),
    ]);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_opinionated_cli_builder_completions_execution() -> Result<()> {
    // Arrange
    let cli = OpinionatedCliBuilder::new("testapp")
        .about("Test app for completions")
        .version("1.2.3")
        .with_completions_subcommand();

    // Act & Assert - should run successfully
    let res = cli.run_with_args(vec![
        "testapp".to_string(),
        "completions".to_string(),
        "bash".to_string(),
    ]);
    assert!(res.is_ok());

    Ok(())
}
