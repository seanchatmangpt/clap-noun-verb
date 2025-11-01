//! Tests for CLI builder

use clap_noun_verb::cli::builder::CliBuilder;
use clap_noun_verb::error::Result;

#[test]
fn test_cli_builder_new() {
    let builder = CliBuilder::new("testapp");
    // Builder should be created successfully - verify by using it
    let result = builder.build_command();
    assert!(result.is_ok());
    let cmd = result.unwrap();
    assert_eq!(cmd.get_name(), "testapp");
}

#[test]
fn test_cli_builder_about() {
    let builder = CliBuilder::new("testapp").about("Test application");
    let result = builder.build_command();
    assert!(result.is_ok());
    let cmd = result.unwrap();
    assert_eq!(cmd.get_about(), Some("Test application"));
}

#[test]
fn test_cli_builder_version() {
    let builder = CliBuilder::new("testapp").version("1.0.0");
    let result = builder.build_command();
    assert!(result.is_ok());
    let cmd = result.unwrap();
    assert!(cmd.get_version().is_some());
    assert_eq!(cmd.get_version(), Some("1.0.0"));
}

#[test]
fn test_cli_builder_noun() {
    let builder = CliBuilder::new("testapp").noun("services", "Manage services");

    let result = builder.build_command();
    assert!(result.is_ok());
    let cmd = result.unwrap();

    // Should have one noun registered as subcommand
    let subcommands: Vec<_> = cmd.get_subcommands().collect();
    assert_eq!(subcommands.len(), 1);
    assert_eq!(subcommands[0].get_name(), "services");
}

#[test]
fn test_cli_builder_multiple_nouns() {
    let builder = CliBuilder::new("testapp")
        .noun("services", "Manage services")
        .noun("collector", "Manage collector");

    let result = builder.build_command();
    assert!(result.is_ok());
    let cmd = result.unwrap();

    // Should have two nouns registered
    let subcommands: Vec<_> = cmd.get_subcommands().collect();
    assert_eq!(subcommands.len(), 2);
    let subcommand_names: Vec<_> = subcommands.iter().map(|s| s.get_name()).collect();
    assert!(subcommand_names.contains(&"services"));
    assert!(subcommand_names.contains(&"collector"));
}

#[test]
fn test_cli_builder_build_command() -> Result<()> {
    let builder =
        CliBuilder::new("testapp").about("Test application").noun("services", "Manage services");

    let cmd = builder.build_command()?;

    // Command should have the correct name
    assert_eq!(cmd.get_name(), "testapp");

    // Command should have subcommands
    let subcommands: Vec<_> = cmd.get_subcommands().collect();
    assert_eq!(subcommands.len(), 1);
    assert_eq!(subcommands[0].get_name(), "services");

    Ok(())
}

#[test]
fn test_cli_builder_build_command_with_version() -> Result<()> {
    let builder = CliBuilder::new("testapp").version("2.0.0").noun("services", "Manage services");

    let cmd = builder.build_command()?;

    // Version should be set
    assert!(cmd.get_version().is_some());
    assert_eq!(cmd.get_version(), Some("2.0.0"));

    Ok(())
}

#[test]
fn test_cli_builder_method_chaining() {
    let builder = CliBuilder::new("testapp")
        .about("Test application")
        .version("1.0.0")
        .noun("services", "Manage services")
        .noun("collector", "Manage collector");

    // Verify by building command
    let result = builder.build_command();
    assert!(result.is_ok());
    let cmd = result.unwrap();
    assert_eq!(cmd.get_name(), "testapp");
    assert_eq!(cmd.get_about(), Some("Test application"));
    assert_eq!(cmd.get_version(), Some("1.0.0"));

    // Verify subcommands
    let subcommands: Vec<_> = cmd.get_subcommands().collect();
    assert_eq!(subcommands.len(), 2);
}
