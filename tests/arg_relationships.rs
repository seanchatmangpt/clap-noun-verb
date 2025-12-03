//! Comprehensive Argument Relationship Tests (Typer-like Doc Comment Syntax)
//!
//! Tests for Typer-like doc comment syntax for argument relationships:
//! - `[group: name]` - Argument groups (exclusive groups)
//! - `[requires: arg]` - Requires relationships
//! - `[conflicts: arg]` - Conflicts_with relationships
//!
//! This follows Python Typer's approach: relationships in doc comments, not code attributes.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
struct GroupTestOutput {
    json: bool,
    yaml: bool,
}

/// Test exclusive argument groups using Typer-like syntax
///
/// # Arguments
/// * `json` - Export as JSON [group: format]
/// * `yaml` - Export as YAML [group: format]
#[verb("group-test", "testcli")]
fn test_exclusive_groups(json: bool, yaml: bool) -> Result<GroupTestOutput> {
    Ok(GroupTestOutput { json, yaml })
}

#[derive(Serialize, Debug, PartialEq)]
struct RequiresTestOutput {
    format: Option<String>,
    filename: Option<String>,
}

/// Test requires relationships using Typer-like syntax
///
/// # Arguments
/// * `format` - Output format
/// * `filename` - Output filename [requires: format]
#[verb("requires-test", "testcli")]
fn test_requires(format: Option<String>, filename: Option<String>) -> Result<RequiresTestOutput> {
    Ok(RequiresTestOutput { format, filename })
}

#[derive(Serialize, Debug, PartialEq)]
struct ConflictsTestOutput {
    format: Option<String>,
    raw: bool,
}

/// Test conflicts_with relationships using Typer-like syntax
///
/// # Arguments
/// * `format` - Output format [conflicts: raw]
/// * `raw` - Raw output mode
#[verb("conflicts-test", "testcli")]
fn test_conflicts(format: Option<String>, raw: bool) -> Result<ConflictsTestOutput> {
    Ok(ConflictsTestOutput { format, raw })
}

#[test]
fn test_argument_groups_registered() {
    // Verify commands are registered
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let reg = registry.lock().unwrap();

    let verbs = reg.get_verbs("testcli");
    let verb_names: Vec<&str> = verbs.iter().map(|(name, _)| *name).collect();

    assert!(
        verb_names.contains(&"group-test"),
        "Expected 'group-test' verb to be registered. Found: {:?}",
        verb_names
    );
    assert!(
        verb_names.contains(&"requires-test"),
        "Expected 'requires-test' verb to be registered. Found: {:?}",
        verb_names
    );
    assert!(
        verb_names.contains(&"conflicts-test"),
        "Expected 'conflicts-test' verb to be registered. Found: {:?}",
        verb_names
    );
}

#[test]
fn test_commands_exist() {
    // Basic smoke test - verify commands can be discovered
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let reg = registry.lock().unwrap();

    let nouns = reg.get_nouns();
    let noun_names: Vec<&str> = nouns.iter().map(|(name, _)| *name).collect();

    assert!(
        noun_names.contains(&"testcli"),
        "Expected 'testcli' noun to be registered. Found nouns: {:?}",
        noun_names
    );
}

#[test]
fn test_group_metadata_extracted() {
    // Verify that group metadata is extracted from doc comments
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let reg = registry.lock().unwrap();

    let verbs = reg.get_verbs_with_metadata("testcli");
    let group_test = verbs.iter().find(|(name, _, _)| *name == "group-test");

    if let Some((_, _, args)) = group_test {
        // Check that json and yaml args have group set
        let json_arg = args.iter().find(|a| a.name == "json");
        let yaml_arg = args.iter().find(|a| a.name == "yaml");

        if let Some(json) = json_arg {
            assert_eq!(
                json.group,
                Some("format".to_string()),
                "Expected json arg to have group='format'"
            );
        }
        if let Some(yaml) = yaml_arg {
            assert_eq!(
                yaml.group,
                Some("format".to_string()),
                "Expected yaml arg to have group='format'"
            );
        }
    }
}

#[test]
fn test_requires_metadata_extracted() {
    // Verify that requires metadata is extracted from doc comments
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let reg = registry.lock().unwrap();

    let verbs = reg.get_verbs_with_metadata("testcli");
    let requires_test = verbs.iter().find(|(name, _, _)| *name == "requires-test");

    if let Some((_, _, args)) = requires_test {
        let filename_arg = args.iter().find(|a| a.name == "filename");

        if let Some(filename) = filename_arg {
            assert!(
                filename.requires.contains(&"format".to_string()),
                "Expected filename arg to require 'format'. Got: {:?}",
                filename.requires
            );
        }
    }
}

#[test]
fn test_conflicts_metadata_extracted() {
    // Verify that conflicts_with metadata is extracted from doc comments
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let reg = registry.lock().unwrap();

    let verbs = reg.get_verbs_with_metadata("testcli");
    let conflicts_test = verbs.iter().find(|(name, _, _)| *name == "conflicts-test");

    if let Some((_, _, args)) = conflicts_test {
        let format_arg = args.iter().find(|a| a.name == "format");

        if let Some(format) = format_arg {
            assert!(
                format.conflicts_with.contains(&"raw".to_string()),
                "Expected format arg to conflict with 'raw'. Got: {:?}",
                format.conflicts_with
            );
        }
    }
}
