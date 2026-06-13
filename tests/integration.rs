// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::panic)]

//! Integration tests for clap-noun-verb framework

use clap_noun_verb::tree::patterns;
use clap_noun_verb::{
    app, command_group, command_tree, noun, verb, Cli, CommandTree, CommandTreeBuilder,
    NounCommand, Registry, Result, VerbArgs, VerbCommand,
};

#[test]
fn test_basic_noun_verb_cli() -> Result<()> {
    // Arrange
    let cli = app! {
        name: "test-app",
        about: "Test CLI application",
        commands: [
            noun!("services", "Manage services", [
                verb!("status", "Show status", |_args: &VerbArgs| {
                    println!("Services are running");
                    Ok(())
                }),
            ]),
        ]
    };

    // Act
    let command = cli.build_command();
    let has_services = command.get_subcommands().any(|cmd| cmd.get_name() == "services");

    // Assert
    assert!(has_services, "Built command is missing the 'services' subcommand");

    Ok(())
}

#[test]
fn test_registry_functionality() -> Result<()> {
    // Arrange
    let registry = Registry::new()
        .name("registry-test")
        .about("Registry test application")
        .register_noun(noun!(
            "test",
            "Test commands",
            [verb!("run", "Run test", |_args: &VerbArgs| {
                println!("Running test");
                Ok(())
            }),]
        ));

    // Act
    let structure = registry.command_structure();
    let has_test = structure.contains_key("test");

    // Assert
    assert!(has_test, "Registry command structure is missing the 'test' noun");
    if let Some(verbs) = structure.get("test") {
        assert_eq!(verbs.len(), 1, "Expected exactly 1 verb associated with 'test' noun");
        assert!(verbs.contains(&"run".to_string()), "Expected verbs to contain 'run'");
    } else {
        panic!("Missing expected 'test' key in command structure");
    }

    Ok(())
}

#[test]
fn test_command_tree_hierarchy() -> Result<()> {
    // Arrange
    let builder = CommandTreeBuilder::new().add_root_with_children(
        "dev",
        "Development tools",
        vec![patterns::noun_verb_pattern(
            "test",
            "Testing utilities",
            vec![
                (
                    "run".to_string(),
                    "Run tests".to_string(),
                    Box::new(|_args: &VerbArgs| {
                        println!("Running tests...");
                        Ok(())
                    }),
                ),
                (
                    "watch".to_string(),
                    "Watch for changes".to_string(),
                    Box::new(|_args: &VerbArgs| {
                        println!("Watching for changes...");
                        Ok(())
                    }),
                ),
            ],
        )],
    );

    // Act
    let tree = CommandTree::from_builder(builder);
    let roots = tree.roots();
    let paths = roots[0].command_paths();

    // Assert
    assert_eq!(paths.len(), 2, "Expected exactly 2 command paths in hierarchy");
    assert!(
        paths
            .iter()
            .any(|path| path == &vec!["dev".to_string(), "test".to_string(), "run".to_string()]),
        "Expected paths to contain ['dev', 'test', 'run']"
    );
    assert!(
        paths
            .iter()
            .any(|path| path == &vec!["dev".to_string(), "test".to_string(), "watch".to_string()]),
        "Expected paths to contain ['dev', 'test', 'watch']"
    );

    Ok(())
}

#[test]
fn test_nested_command_routing() -> Result<()> {
    // Arrange
    let cli = Cli::new().name("nested-test").about("Nested command test").noun(
        noun!("dev", "Development tools", {
            noun!("test", "Testing utilities", [
                verb!("run", "Run tests", |_args: &VerbArgs| {
                    println!("Running tests...");
                    Ok(())
                }),
                verb!("watch", "Watch for changes", |_args: &VerbArgs| {
                    println!("Watching for changes...");
                    Ok(())
                }),
            ]),
        }),
    );

    // Act
    let command = cli.build_command();
    let has_dev = command.get_subcommands().any(|cmd| cmd.get_name() == "dev");

    // Assert
    assert!(has_dev, "Expected built command to routing subcommands through 'dev'");

    Ok(())
}

#[test]
fn test_custom_command_implementation() -> Result<()> {
    // Arrange
    struct CustomServicesCommand;

    impl NounCommand for CustomServicesCommand {
        fn name(&self) -> &'static str {
            "custom-services"
        }
        fn about(&self) -> &'static str {
            "Custom services implementation"
        }
        fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
            vec![Box::new(CustomStatusCommand)]
        }
    }

    struct CustomStatusCommand;

    impl VerbCommand for CustomStatusCommand {
        fn name(&self) -> &'static str {
            "status"
        }
        fn about(&self) -> &'static str {
            "Show custom status"
        }
        fn run(&self, _args: &VerbArgs) -> Result<()> {
            println!("Custom status: All systems operational");
            Ok(())
        }
    }

    let cli =
        Cli::new().name("custom-test").about("Custom command test").noun(CustomServicesCommand);

    // Act
    let structure = cli.command_structure();
    let has_custom_services = structure.contains_key("custom-services");

    // Assert
    assert!(has_custom_services, "Expected structure to contain 'custom-services'");
    if let Some(verbs) = structure.get("custom-services") {
        assert!(
            verbs.contains(&"status".to_string()),
            "Expected custom services verbs to contain 'status'"
        );
    } else {
        panic!("Missing expected 'custom-services' key in command structure");
    }

    Ok(())
}

#[test]
fn test_verb_args_context() -> Result<()> {
    // Arrange
    let cli = app! {
        name: "context-test",
        about: "Context test application",
        commands: [
            noun!("test", "Test commands", [
                verb!("with-context", "Command with context", |args: &VerbArgs| {
                    // Act in Callback
                    let verb_name = args.verb();
                    let noun_name = args.noun();
                    let custom = args.get_context("custom");

                    // Assert in Callback
                    assert_eq!(verb_name, "with-context", "Verb name mismatch inside callback");
                    assert_eq!(noun_name, Some("test"), "Noun name mismatch inside callback");
                    if let Some(val) = custom {
                        assert_eq!(val, "test-value", "Context value mismatch inside callback");
                    }

                    println!("Context test passed");
                    Ok(())
                }),
            ]),
        ]
    };

    // Act
    let command = cli.build_command();
    let has_test = command.get_subcommands().any(|cmd| cmd.get_name() == "test");

    // Assert
    assert!(has_test, "Expected built command to have 'test' subcommand");

    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    // Arrange
    let cli = app! {
        name: "error-test",
        about: "Error handling test",
        commands: [
            noun!("test", "Test commands", [
                verb!("error", "Command that errors", |_args: &VerbArgs| {
                    Err(clap_noun_verb::NounVerbError::execution_error("Test error"))
                }),
            ]),
        ]
    };

    // Act
    let command = cli.build_command();
    let has_test = command.get_subcommands().any(|cmd| cmd.get_name() == "test");

    // Assert
    assert!(has_test, "Expected built command to contain 'test' subcommand");

    Ok(())
}

#[test]
fn test_cli_builder_method_chaining() -> Result<()> {
    // Arrange
    let cli = Cli::new()
        .name("method-chain-test")
        .about("Method chaining test")
        .noun(noun!(
            "first",
            "First command group",
            [verb!("action", "First action", |_args: &VerbArgs| {
                println!("First action executed");
                Ok(())
            }),]
        ))
        .noun(noun!(
            "second",
            "Second command group",
            [verb!("action", "Second action", |_args: &VerbArgs| {
                println!("Second action executed");
                Ok(())
            }),]
        ));

    // Act
    let structure = cli.command_structure();
    let has_first = structure.contains_key("first");
    let has_second = structure.contains_key("second");

    // Assert
    assert!(has_first, "Expected structure to contain 'first' noun");
    assert!(has_second, "Expected structure to contain 'second' noun");
    if let Some(first_verbs) = structure.get("first") {
        assert_eq!(first_verbs.len(), 1, "Expected first command group to have exactly 1 verb");
    } else {
        panic!("Missing expected 'first' key in command structure");
    }
    if let Some(second_verbs) = structure.get("second") {
        assert_eq!(second_verbs.len(), 1, "Expected second command group to have exactly 1 verb");
    } else {
        panic!("Missing expected 'second' key in command structure");
    }

    Ok(())
}

#[test]
fn test_command_group_macro() -> Result<()> {
    // Arrange
    let group = command_group!(
        "test-group",
        "Test command group",
        [
            verb!("first", "First command", |_args: &VerbArgs| {
                println!("First command");
                Ok(())
            }),
            verb!("second", "Second command", |_args: &VerbArgs| {
                println!("Second command");
                Ok(())
            }),
        ]
    );

    // Act
    let name = group.name();
    let about = group.about();
    let verbs_count = group.verbs().len();

    // Assert
    assert_eq!(name, "test-group", "Command group name mismatch");
    assert_eq!(about, "Test command group", "Command group about mismatch");
    assert_eq!(verbs_count, 2, "Expected exactly 2 verbs in the group");

    Ok(())
}

#[test]
fn test_command_tree_macro() -> Result<()> {
    // Arrange
    let mut cli = Cli::new().name("tree-test").about("Tree test");
    let tree_noun = noun!(
        "root",
        "Root command",
        [verb!("leaf", "Leaf command", |_args: &VerbArgs| {
            println!("Leaf command");
            Ok(())
        }),]
    );

    // Act
    cli = command_tree!(cli => tree_noun);
    let command = cli.build_command();
    let has_root = command.get_subcommands().any(|cmd| cmd.get_name() == "root");

    // Assert
    assert!(has_root, "Expected built command to contain 'root' subcommand");

    Ok(())
}

#[test]
fn test_registry_introspection() -> Result<()> {
    // Arrange
    let registry = Registry::new()
        .name("introspection-test")
        .about("Introspection test")
        .register_noun(noun!(
            "services",
            "Service management",
            [
                verb!("status", "Show status", |_args: &VerbArgs| { Ok(()) }),
                verb!("restart", "Restart service", |_args: &VerbArgs| { Ok(()) }),
            ]
        ))
        .register_noun(noun!(
            "config",
            "Configuration management",
            [
                verb!("get", "Get config value", |_args: &VerbArgs| { Ok(()) }),
                verb!("set", "Set config value", |_args: &VerbArgs| { Ok(()) }),
            ]
        ));

    // Act
    let noun_count = registry.noun_names().len();
    let has_services = registry.has_noun("services");
    let has_config = registry.has_noun("config");
    let structure = registry.command_structure();

    // Assert
    assert_eq!(noun_count, 2, "Expected exactly 2 noun names in registry");
    assert!(has_services, "Expected registry to contain 'services' noun");
    assert!(has_config, "Expected registry to contain 'config' noun");
    assert_eq!(structure.len(), 2, "Expected structure length to be exactly 2");

    if let Some(services_verbs) = structure.get("services") {
        assert_eq!(services_verbs.len(), 2, "Expected 'services' to have exactly 2 verbs");
    } else {
        panic!("Missing expected 'services' key in command structure");
    }

    if let Some(config_verbs) = structure.get("config") {
        assert_eq!(config_verbs.len(), 2, "Expected 'config' to have exactly 2 verbs");
    } else {
        panic!("Missing expected 'config' key in command structure");
    }

    Ok(())
}

#[test]
fn test_verb_args_functionality() -> Result<()> {
    // Arrange
    let cli = app! {
        name: "args-test",
        about: "Arguments test",
        commands: [
            noun!("test", "Test commands", [
                verb!("with-args", "Command with arguments", |args: &VerbArgs| {
                    // Act in Callback
                    let verb_name = args.verb();
                    let noun_name = args.noun();

                    // Assert in Callback
                    assert_eq!(verb_name, "with-args", "Verb name mismatch inside callback");
                    assert_eq!(noun_name, Some("test"), "Noun name mismatch inside callback");

                    println!("Arguments test passed");
                    Ok(())
                }),
            ]),
        ]
    };

    // Act
    let command = cli.build_command();
    let has_test = command.get_subcommands().any(|cmd| cmd.get_name() == "test");

    // Assert
    assert!(has_test, "Expected built command to contain 'test' subcommand");

    Ok(())
}
