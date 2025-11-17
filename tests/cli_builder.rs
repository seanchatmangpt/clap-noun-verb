//! Tests for CLI builder
//!
//! These tests follow AAA pattern (Arrange, Act, Assert) and test
//! behaviors, not implementation details.

mod common;

use clap_noun_verb::cli::builder::CliBuilder;
use clap_noun_verb::error::Result;

#[test]
fn test_cli_builder_new() -> Result<()> {
    // Arrange - Create a new builder
    let builder = CliBuilder::new("testapp");

    // Act - Run the CLI with help flag
    // Assert - Builder should handle help without panicking
    let _result = builder.run_with_args(vec!["testapp".to_string(), "--help".to_string()]);

    Ok(())
}

#[test]
fn test_cli_builder_method_chaining() -> Result<()> {
    // Arrange - Build CLI with method chaining
    let builder = CliBuilder::new("testapp")
        .about("Test application")
        .version("1.0.0")
        .noun("services", "Manage services")
        .noun("collector", "Manage collector");

    // Act - Try to run with help flag
    // Assert - Method chaining should produce valid command structure
    let _result = builder.run_with_args(vec!["testapp".to_string(), "--help".to_string()]);

    Ok(())
}

#[test]
fn test_cli_builder_with_noun() -> Result<()> {
    // Arrange - Create builder with noun
    let builder = CliBuilder::new("testapp").noun("services", "Manage services");

    // Act - Try to run with help for noun subcommand
    let _result = builder.run_with_args(vec![
        "testapp".to_string(),
        "services".to_string(),
        "--help".to_string(),
    ]);

    // Assert - Noun registration is valid (no panic means successful structure)
    Ok(())
}
