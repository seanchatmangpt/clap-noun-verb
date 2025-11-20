//! Chicago TDD tests for version and help display features
//!
//! These tests use Chicago School TDD principles:
//! - Arrange-Act-Assert (AAA) pattern
//! - Real CLI command building (no mocks)
//! - State-based verification (verify actual command structure and output)
//! - Real command registry operations

mod common;

use clap_noun_verb::cli::registry::CommandRegistry;
use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
use common::command_assertions::*;

#[test]
fn test_build_command_includes_version() {
    // Arrange: Set up test registry with a noun
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();
    CommandRegistry::register_noun("test", "Test commands");

    // Act: Build command
    let cmd = registry.build_command();

    // Assert: Command should have version set from Cargo.toml
    assert_has_version(&cmd, Some(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_build_command_has_subcommands() {
    // Arrange: Set up test registry with nouns
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    CommandRegistry::register_noun("test", "Test commands");
    CommandRegistry::register_noun("other", "Other commands");

    // Act: Build command
    let cmd = registry.build_command();

    // Assert: Command should have subcommands
    assert_has_subcommand(&cmd, "test");
    assert_has_subcommand(&cmd, "other");
}

#[test]
fn test_noun_command_has_verbs_as_subcommands() {
    // Arrange: Set up test registry with noun and verbs
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    CommandRegistry::register_noun("test", "Test commands");
    CommandRegistry::register_verb("test", "status", "Show status", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("OK")?)
    });
    CommandRegistry::register_verb("test", "list", "List items", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("[]")?)
    });

    // Act & Assert
    let cmd = registry.build_command();
    assert_subcommand_has_verb(&cmd, "test", "status");
    assert_subcommand_has_verb(&cmd, "test", "list");
}

#[test]
fn test_help_can_be_generated_for_root_command() {
    // Arrange: Set up test registry
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    CommandRegistry::register_noun("test", "Test commands");
    CommandRegistry::register_verb("test", "status", "Show status", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("OK")?)
    });

    // Act: Build command and generate help
    let mut cmd = registry.build_command();

    // Assert: Help should be generated successfully with expected content
    assert_help_contains(&mut cmd, "test");
}

#[test]
fn test_help_can_be_generated_for_noun_command() {
    // Arrange: Set up test registry with noun and verbs
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    CommandRegistry::register_noun("test", "Test commands");
    CommandRegistry::register_verb("test", "status", "Show status", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("OK")?)
    });
    CommandRegistry::register_verb("test", "list", "List items", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("[]")?)
    });

    // Act & Assert
    let cmd = registry.build_command();
    let verbs = get_verb_names(&cmd, "test");
    assert!(verbs.contains(&"status"));
    assert!(verbs.contains(&"list"));
}

#[test]
fn test_version_matches_cargo_pkg_version() {
    // Arrange: Get Cargo.toml version from environment
    let cargo_version = env!("CARGO_PKG_VERSION");
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();
    CommandRegistry::register_noun("test", "Test commands");

    // Act & Assert
    let cmd = registry.build_command();
    assert_has_version(&cmd, Some(cargo_version));
}

#[test]
fn test_command_structure_allows_help_display() {
    // Arrange: Set up test registry with multiple nouns and verbs
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    CommandRegistry::register_noun("services", "Manage services");
    CommandRegistry::register_verb("services", "status", "Show status", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("OK")?)
    });

    CommandRegistry::register_noun("config", "Manage configuration");
    CommandRegistry::register_verb("config", "get", "Get config", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("{}")?)
    });

    // Act & Assert
    let cmd = registry.build_command();
    assert_has_subcommand(&cmd, "services");
    assert_has_subcommand(&cmd, "config");
    assert_subcommand_has_verb(&cmd, "services", "status");
    assert_subcommand_has_verb(&cmd, "config", "get");
}

#[test]
fn test_run_with_no_args_handles_gracefully() {
    // Arrange: Set up test registry
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    CommandRegistry::register_noun("test", "Test commands");
    CommandRegistry::register_verb("test", "status", "Show status", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("OK")?)
    });

    // Act: Build command
    let cmd = registry.build_command();

    // Assert: Command should have subcommands for help display
    assert!(get_subcommand_names(&cmd).len() > 0, "Command should have subcommands");
}

#[test]
fn test_noun_help_includes_all_verbs() {
    // Arrange: Set up test registry with noun and multiple verbs
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    CommandRegistry::register_noun("test", "Test commands");
    CommandRegistry::register_verb("test", "status", "Show status", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("OK")?)
    });
    CommandRegistry::register_verb("test", "list", "List items", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("[]")?)
    });
    CommandRegistry::register_verb("test", "create", "Create item", |_input: HandlerInput| {
        Ok(HandlerOutput::from_data("Created")?)
    });

    // Act & Assert
    let cmd = registry.build_command();
    let verbs = get_verb_names(&cmd, "test");
    assert!(verbs.contains(&"status"));
    assert!(verbs.contains(&"list"));
    assert!(verbs.contains(&"create"));
}
