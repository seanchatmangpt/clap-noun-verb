// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::panic)]

//! Unit tests for clap-noun-verb modules

use clap_noun_verb::tree::patterns;
use clap_noun_verb::{
    noun, verb, Cli, CommandTree, CommandTreeBuilder, NounCommand, NounContext, Registry, Result,
    VerbArgs, VerbCommand, VerbContext,
};

#[test]
fn test_noun_command_trait() -> Result<()> {
    // Arrange
    struct TestNoun;

    impl NounCommand for TestNoun {
        fn name(&self) -> &'static str {
            "test-noun"
        }
        fn about(&self) -> &'static str {
            "Test noun command"
        }
        fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
            vec![Box::new(TestVerb)]
        }
    }

    struct TestVerb;

    impl VerbCommand for TestVerb {
        fn name(&self) -> &'static str {
            "test-verb"
        }
        fn about(&self) -> &'static str {
            "Test verb command"
        }
        fn run(&self, _args: &VerbArgs) -> Result<()> {
            Ok(())
        }
    }

    let noun = TestNoun;

    // Act
    let name = noun.name();
    let about = noun.about();
    let verbs = noun.verbs();

    // Assert
    assert_eq!(name, "test-noun", "Noun name mismatch");
    assert_eq!(about, "Test noun command", "Noun about description mismatch");
    assert_eq!(verbs.len(), 1, "Expected exactly 1 verb command associated with noun");
    assert_eq!(verbs[0].name(), "test-verb", "Verb name mismatch inside noun command");

    Ok(())
}

#[test]
fn test_verb_command_trait() -> Result<()> {
    // Arrange
    struct TestVerb {
        #[allow(dead_code)]
        name: String,
        #[allow(dead_code)]
        about: String,
    }

    impl VerbCommand for TestVerb {
        fn name(&self) -> &'static str {
            "test-verb"
        }
        fn about(&self) -> &'static str {
            "Test verb command"
        }
        fn run(&self, _args: &VerbArgs) -> Result<()> {
            Ok(())
        }
    }

    let verb = TestVerb { name: "test".to_string(), about: "test".to_string() };

    // Act
    let name = verb.name();
    let about = verb.about();

    // Assert
    assert_eq!(name, "test-verb", "Verb name mismatch");
    assert_eq!(about, "Test verb command", "Verb about description mismatch");

    Ok(())
}

#[test]
fn test_verb_args_context() -> Result<()> {
    // Arrange
    let context = VerbContext::new("test-verb")
        .with_noun("test-noun")
        .with_data("key1", "value1")
        .with_data("key2", "value2");

    // Act
    let verb = context.verb.as_str();
    let noun = context.noun.as_deref();
    let val1 = context.get_data("key1");
    let val2 = context.get_data("key2");
    let val3 = context.get_data("key3");

    // Assert
    assert_eq!(verb, "test-verb", "Context verb mismatch");
    assert_eq!(noun, Some("test-noun"), "Context noun mismatch");
    assert_eq!(val1, Some(&"value1".to_string()), "Context data 'key1' mismatch");
    assert_eq!(val2, Some(&"value2".to_string()), "Context data 'key2' mismatch");
    assert_eq!(val3, None, "Expected context data 'key3' to be absent");

    Ok(())
}

#[test]
fn test_verb_args_creation() -> Result<()> {
    // Arrange
    let args = VerbArgs::new(clap::ArgMatches::default()).add_context("test-key", "test-value");

    // Act
    let verb = args.verb();
    let noun = args.noun();
    let context_val = args.get_context("test-key");

    // Assert
    assert_eq!(verb, "", "Expected empty verb for default ArgMatches");
    assert_eq!(noun, None, "Expected no noun for default ArgMatches");
    assert_eq!(
        context_val,
        Some(&"test-value".to_string()),
        "Context value for 'test-key' mismatch"
    );

    Ok(())
}

#[test]
fn test_registry_configuration() -> Result<()> {
    // Arrange
    let registry = Registry::new().name("test-app").about("Test application").version("1.0.0");

    // Act
    let command = registry.build_command();

    // Assert
    assert_eq!(command.get_name(), "test-app", "Registry application name mismatch");
    assert_eq!(
        command.get_about().map(|s| s.to_string()).unwrap_or_default(),
        "Test application",
        "Registry application about mismatch"
    );
    assert_eq!(
        command.get_version().unwrap_or(""),
        "1.0.0",
        "Registry application version mismatch"
    );

    Ok(())
}

#[test]
fn test_registry_noun_management() -> Result<()> {
    // Arrange
    let mut registry = Registry::new();
    let noun1 = noun!(
        "test1",
        "Test command 1",
        [verb!("action1", "Action 1", |_args: &VerbArgs| { Ok(()) }),]
    );
    let noun2 = noun!(
        "test2",
        "Test command 2",
        [verb!("action2", "Action 2", |_args: &VerbArgs| { Ok(()) }),]
    );

    // Act 1: Register nouns
    registry = registry.register_noun(noun1).register_noun(noun2);

    // Assert 1: Verify insertion
    assert_eq!(registry.noun_names().len(), 2, "Expected exactly 2 registered nouns");
    assert!(registry.has_noun("test1"), "Registry should contain 'test1' noun");
    assert!(registry.has_noun("test2"), "Registry should contain 'test2' noun");

    // Act 2: Remove a noun
    let removed = registry.remove_noun("test1");

    // Assert 2: Verify deletion
    assert!(removed.is_some(), "Expected remove_noun to return the removed noun");
    assert!(!registry.has_noun("test1"), "Registry should no longer contain 'test1' noun");
    assert!(registry.has_noun("test2"), "Registry should still contain 'test2' noun");

    Ok(())
}

#[test]
fn test_registry_command_structure() -> Result<()> {
    // Arrange
    let registry = Registry::new()
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
            [verb!("get", "Get config", |_args: &VerbArgs| { Ok(()) }),]
        ));

    // Act
    let structure = registry.command_structure();

    // Assert
    assert_eq!(structure.len(), 2, "Command structure should contain exactly 2 nouns");
    assert!(structure.contains_key("services"), "Command structure should map 'services'");
    assert!(structure.contains_key("config"), "Command structure should map 'config'");

    if let Some(services_verbs) = structure.get("services") {
        assert_eq!(services_verbs.len(), 2, "Expected 2 verbs under 'services'");
        assert!(
            services_verbs.contains(&"status".to_string()),
            "Expected 'status' verb under 'services'"
        );
        assert!(
            services_verbs.contains(&"restart".to_string()),
            "Expected 'restart' verb under 'services'"
        );
    } else {
        panic!("Missing 'services' key in command structure");
    }

    if let Some(config_verbs) = structure.get("config") {
        assert_eq!(config_verbs.len(), 1, "Expected 1 verb under 'config'");
        assert!(config_verbs.contains(&"get".to_string()), "Expected 'get' verb under 'config'");
    } else {
        panic!("Missing 'config' key in command structure");
    }

    Ok(())
}

#[test]
fn test_command_tree_basic() -> Result<()> {
    // Arrange
    let tree = CommandTree::new();

    // Act
    let names = tree.root_names();

    // Assert
    assert_eq!(names.len(), 0, "Expected newly initialized CommandTree to be empty");

    Ok(())
}

#[test]
fn test_command_tree_builder() -> Result<()> {
    // Arrange
    let builder = CommandTreeBuilder::new().add_root_with_handler(
        "version",
        "Show version",
        |_args: &VerbArgs| {
            println!("Version 1.0.0");
            Ok(())
        },
    );

    // Act
    let tree = CommandTree::from_builder(builder);
    let names = tree.root_names();

    // Assert
    assert_eq!(names.len(), 1, "Expected CommandTree to have exactly 1 root command");
    assert_eq!(names[0], "version", "Root command name mismatch");

    Ok(())
}

#[test]
fn test_command_tree_nested() -> Result<()> {
    // Arrange
    let builder = CommandTreeBuilder::new().add_root_with_children(
        "dev",
        "Development tools",
        vec![patterns::noun_verb_pattern(
            "test",
            "Testing utilities",
            vec![("run".to_string(), "Run tests".to_string(), Box::new(|_args: &VerbArgs| Ok(())))],
        )],
    );

    // Act
    let tree = CommandTree::from_builder(builder);
    let names = tree.root_names();
    let roots = tree.roots();
    let paths = roots[0].command_paths();

    // Assert
    assert_eq!(names.len(), 1, "Expected CommandTree to have 1 root");
    assert_eq!(names[0], "dev", "Root name mismatch");
    assert_eq!(paths.len(), 1, "Expected 1 command path under root");
    assert_eq!(paths[0], vec!["dev", "test", "run"], "Command path mismatch for nested tree");

    Ok(())
}

#[test]
fn test_cli_builder_basic() -> Result<()> {
    // Arrange
    let cli = Cli::new().name("test-cli").about("Test CLI");

    // Act
    let command = cli.build_command();

    // Assert
    assert_eq!(command.get_name(), "test-cli", "CLI name mismatch");
    assert_eq!(
        command.get_about().map(|s| s.to_string()).unwrap_or_default(),
        "Test CLI",
        "CLI about mismatch"
    );

    Ok(())
}

#[test]
fn test_cli_builder_with_nouns() -> Result<()> {
    // Arrange
    let cli = Cli::new()
        .name("multi-test")
        .about("Multi-command test")
        .noun(noun!(
            "cmd1",
            "Command 1",
            [verb!("action1", "Action 1", |_args: &VerbArgs| { Ok(()) }),]
        ))
        .noun(noun!(
            "cmd2",
            "Command 2",
            [verb!("action2", "Action 2", |_args: &VerbArgs| { Ok(()) }),]
        ));

    // Act
    let structure = cli.command_structure();

    // Assert
    assert_eq!(structure.len(), 2, "Expected exactly 2 nouns in structure");
    assert!(structure.contains_key("cmd1"), "Missing noun 'cmd1'");
    assert!(structure.contains_key("cmd2"), "Missing noun 'cmd2'");

    Ok(())
}

#[test]
fn test_cli_builder_introspection() -> Result<()> {
    // Arrange
    let cli = Cli::new().name("introspection-test").about("Introspection test").noun(noun!(
        "test",
        "Test command",
        [verb!("action", "Test action", |_args: &VerbArgs| { Ok(()) }),]
    ));

    // Act
    let has_test = cli.has_command("test");
    let has_nonexistent = cli.has_command("nonexistent");

    // Assert
    assert!(has_test, "Expected Cli to have command 'test'");
    assert!(!has_nonexistent, "Expected Cli to not have command 'nonexistent'");

    Ok(())
}

#[test]
fn test_noun_context_creation() -> Result<()> {
    // Arrange
    let context =
        NounContext::new("test-noun").with_data("key1", "value1").with_data("key2", "value2");

    // Act
    let noun = context.noun.as_str();
    let val1 = context.get_data("key1");
    let val2 = context.get_data("key2");
    let val3 = context.get_data("key3");

    // Assert
    assert_eq!(noun, "test-noun", "NounContext noun name mismatch");
    assert_eq!(val1, Some(&"value1".to_string()), "NounContext data 'key1' mismatch");
    assert_eq!(val2, Some(&"value2".to_string()), "NounContext data 'key2' mismatch");
    assert_eq!(val3, None, "Expected absent key 'key3' to return None");

    Ok(())
}

#[test]
fn test_verb_context_creation() -> Result<()> {
    // Arrange
    let context = VerbContext::new("test-verb").with_noun("test-noun").with_data("key1", "value1");

    // Act
    let verb = context.verb.as_str();
    let noun = context.noun.as_deref();
    let val1 = context.get_data("key1");

    // Assert
    assert_eq!(verb, "test-verb", "VerbContext verb name mismatch");
    assert_eq!(noun, Some("test-noun"), "VerbContext noun name mismatch");
    assert_eq!(val1, Some(&"value1".to_string()), "VerbContext data 'key1' mismatch");

    Ok(())
}

#[test]
fn test_macro_expansion() -> Result<()> {
    // Arrange
    let test_noun = noun!(
        "test-noun",
        "Test noun",
        [verb!("test-verb", "Test verb", |_args: &VerbArgs| { Ok(()) }),]
    );

    // Act
    let name = test_noun.name();
    let about = test_noun.about();
    let verbs = test_noun.verbs();

    // Assert
    assert_eq!(name, "test-noun", "Macro-expanded noun name mismatch");
    assert_eq!(about, "Test noun", "Macro-expanded noun about mismatch");
    assert_eq!(verbs.len(), 1, "Expected exactly 1 verb under expanded noun");

    Ok(())
}

#[test]
fn test_error_types() -> Result<()> {
    // Arrange
    let cmd_error = clap_noun_verb::NounVerbError::command_not_found("missing-command");
    let verb_error = clap_noun_verb::NounVerbError::verb_not_found("services", "missing-verb");
    let structure_error = clap_noun_verb::NounVerbError::invalid_structure("Invalid structure");
    let exec_error = clap_noun_verb::NounVerbError::execution_error("Execution failed");
    let arg_error = clap_noun_verb::NounVerbError::argument_error("Invalid arguments");

    // Act
    let cmd_msg = cmd_error.to_string();
    let verb_msg = verb_error.to_string();
    let structure_msg = structure_error.to_string();
    let exec_msg = exec_error.to_string();
    let arg_msg = arg_error.to_string();

    // Assert
    assert!(
        cmd_msg.contains("Command 'missing-command' not found"),
        "Error string did not match expected: {}",
        cmd_msg
    );
    assert!(
        verb_msg.contains("Verb 'missing-verb' not found for noun 'services'"),
        "Error string did not match expected: {}",
        verb_msg
    );
    assert!(
        structure_msg.contains("Invalid structure"),
        "Error string did not match expected: {}",
        structure_msg
    );
    assert!(
        exec_msg.contains("Execution failed"),
        "Error string did not match expected: {}",
        exec_msg
    );
    assert!(
        arg_msg.contains("Invalid arguments"),
        "Error string did not match expected: {}",
        arg_msg
    );

    Ok(())
}

#[test]
fn test_patterns_helper() -> Result<()> {
    // Arrange
    let pattern = patterns::noun_verb_pattern(
        "test-noun",
        "Test noun pattern",
        vec![
            ("verb1".to_string(), "Verb 1".to_string(), Box::new(|_args: &VerbArgs| Ok(()))),
            ("verb2".to_string(), "Verb 2".to_string(), Box::new(|_args: &VerbArgs| Ok(()))),
        ],
    );

    // Act
    let name = pattern.name();
    let about = pattern.about();
    let children_len = pattern.children().len();

    // Assert
    assert_eq!(name, "test-noun", "Pattern name mismatch");
    assert_eq!(about, "Test noun pattern", "Pattern about mismatch");
    assert_eq!(children_len, 2, "Pattern children length mismatch");

    Ok(())
}

#[test]
fn test_build_cli_function() -> Result<()> {
    // Arrange
    let builder_fn = |cli: Cli| {
        cli.name("build-test").about("Build test CLI").noun(noun!(
            "test",
            "Test command",
            [verb!("action", "Test action", |_args: &VerbArgs| { Ok(()) }),]
        ))
    };

    // Act
    let (command, structure) = clap_noun_verb::build_cli(builder_fn);

    // Assert
    assert_eq!(command.get_name(), "build-test", "CLI built name mismatch");
    assert_eq!(
        command.get_about().map(|s| s.to_string()).unwrap_or_default(),
        "Build test CLI",
        "CLI built about mismatch"
    );
    assert_eq!(structure.len(), 1, "CLI structure length mismatch");
    assert!(structure.contains_key("test"), "CLI structure missing 'test' noun");

    Ok(())
}

#[test]
fn test_run_cli_function() -> Result<()> {
    // Arrange
    let builder_fn = |cli: Cli| {
        cli.name("run-test").about("Run test CLI").noun(noun!(
            "test",
            "Test command",
            [verb!("info", "Show info", |_args: &VerbArgs| {
                println!("Info command");
                Ok(())
            }),]
        ))
    };

    // Act
    let result = clap_noun_verb::run_cli(builder_fn);

    // Assert
    assert!(result.is_err(), "Expected execution to fail when running without CLI arguments");

    Ok(())
}
