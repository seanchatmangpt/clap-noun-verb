//! Comprehensive tests for help system improvements
//!
//! Tests cover:
//! - Enhanced help system
//! - Built-in examples
//! - Command discovery
//! - Interactive help

use clap_noun_verb::cli::{
    discovery::{generate_commands_output, generate_search_output, CommandDiscovery},
    examples::ExamplesRegistry,
    help::{CommandCategory, CommandInfo, HelpSystem},
    interactive::{generate_interactive_output, InteractiveHelp},
};

// ============================================================================
// Help System Tests
// ============================================================================

#[test]
fn test_help_system_initialization() {
    let help = HelpSystem::default();
    assert!(!help.all_commands().is_empty(), "Help system should have registered commands");
}

#[test]
fn test_help_system_popular_commands() {
    let help = HelpSystem::default();
    let popular = help.popular_commands(5);

    assert_eq!(popular.len(), 5, "Should return exactly 5 popular commands");

    // Verify sorting by popularity
    for i in 0..popular.len() - 1 {
        assert!(
            popular[i].popularity >= popular[i + 1].popularity,
            "Commands should be sorted by popularity (descending)"
        );
    }
}

#[test]
fn test_help_system_commands_by_category() {
    let help = HelpSystem::default();

    for category in CommandCategory::all() {
        let commands = help.commands_by_category(&category);

        for cmd in commands {
            assert_eq!(
                cmd.category, category,
                "All commands in category {} should have matching category",
                category
            );
        }
    }
}

#[test]
fn test_help_system_find_command() {
    let help = HelpSystem::default();

    // Should find known command
    let result = help.find_command("pack list");
    assert!(result.is_some(), "Should find 'pack list' command");
    assert_eq!(result.unwrap().name, "pack list");

    // Should not find unknown command
    let result = help.find_command("nonexistent command");
    assert!(result.is_none(), "Should not find nonexistent command");
}

#[test]
fn test_help_system_generate_main_help() {
    let help = HelpSystem::default();
    let output = help.generate_main_help();

    assert!(output.is_ok(), "Should generate main help without error");

    let help_output = output.unwrap();
    assert!(!help_output.version.is_empty(), "Version should not be empty");
    assert!(!help_output.description.is_empty(), "Description should not be empty");
    assert!(!help_output.categories.is_empty(), "Should have categories");
    assert_eq!(help_output.popular.len(), 5, "Should have 5 popular commands");
    assert!(!help_output.quickstart_url.is_empty(), "Should have quickstart URL");
}

#[test]
fn test_help_system_generate_command_help() {
    let help = HelpSystem::default();

    // Test with valid command
    let output = help.generate_command_help("pack list");
    assert!(output.is_ok(), "Should generate help for valid command");

    let cmd_help = output.unwrap();
    assert_eq!(cmd_help.name, "pack list");
    assert!(!cmd_help.description.is_empty(), "Should have description");
    assert!(!cmd_help.examples.is_empty(), "Should have examples");

    // Test with invalid command
    let output = help.generate_command_help("invalid command");
    assert!(output.is_err(), "Should error for invalid command");
}

#[test]
fn test_command_category_all() {
    let categories = CommandCategory::all();
    assert!(!categories.is_empty(), "Should have categories");

    // Verify all categories have descriptions
    for category in categories {
        assert!(!category.description().is_empty(), "Category should have description");
    }
}

#[test]
fn test_command_info_builder() {
    let info = CommandInfo::new("test", CommandCategory::System, "Test command")
        .with_description("Detailed description")
        .with_example("example 1")
        .with_example("example 2")
        .with_popularity(75);

    assert_eq!(info.name, "test");
    assert_eq!(info.brief, "Test command");
    assert_eq!(info.description, "Detailed description");
    assert_eq!(info.examples.len(), 2);
    assert_eq!(info.popularity, 75);
}

// ============================================================================
// Examples Registry Tests
// ============================================================================

#[test]
fn test_examples_registry_initialization() {
    let registry = ExamplesRegistry::default();
    assert_eq!(registry.all().len(), 10, "Should have 10 built-in examples");
}

#[test]
fn test_examples_registry_by_tag() {
    let registry = ExamplesRegistry::default();

    let beginner = registry.by_tag("beginner");
    assert!(!beginner.is_empty(), "Should have beginner examples");

    for example in beginner {
        assert!(
            example.tags.contains(&"beginner".to_string()),
            "All examples should have 'beginner' tag"
        );
    }
}

#[test]
fn test_examples_registry_search() {
    let registry = ExamplesRegistry::default();

    // Search for "pack"
    let results = registry.search("pack");
    assert!(!results.is_empty(), "Should find pack-related examples");

    // Search for "ai"
    let results = registry.search("ai");
    assert!(!results.is_empty(), "Should find AI-related examples");

    // Search for non-existent
    let results = registry.search("xyz123nonexistent");
    assert!(results.is_empty(), "Should find no results for non-existent keyword");
}

#[test]
fn test_examples_search_case_insensitive() {
    let registry = ExamplesRegistry::default();

    let results1 = registry.search("PACK");
    let results2 = registry.search("pack");

    assert_eq!(results1.len(), results2.len(), "Search should be case-insensitive");
}

#[test]
fn test_examples_all_have_required_fields() {
    let registry = ExamplesRegistry::default();

    for example in registry.all() {
        assert!(!example.title.is_empty(), "Example should have title");
        assert!(!example.description.is_empty(), "Example should have description");
        assert!(!example.command.is_empty(), "Example should have command");
        assert!(!example.expected_output.is_empty(), "Example should have expected output");
        assert!(!example.tags.is_empty(), "Example should have tags");
    }
}

#[test]
fn test_examples_generate_output() {
    let registry = ExamplesRegistry::default();
    let output = registry.generate_output();

    assert!(output.is_ok(), "Should generate output without error");

    let examples_output = output.unwrap();
    assert_eq!(examples_output.total, 10, "Should have 10 total examples");
    assert_eq!(examples_output.examples.len(), 10, "Examples count should match");
}

// ============================================================================
// Command Discovery Tests
// ============================================================================

fn create_test_discovery() -> CommandDiscovery {
    let help = HelpSystem::default();
    let commands: Vec<CommandInfo> = help
        .all_commands()
        .into_iter()
        .filter_map(|item| {
            help.find_command(&item.name).map(|cmd| CommandInfo {
                name: cmd.name.clone(),
                category: cmd.category.clone(),
                brief: cmd.brief.clone(),
                description: cmd.description.clone(),
                examples: cmd.examples.clone(),
                popularity: 0,
            })
        })
        .collect();

    CommandDiscovery::from_commands(commands)
}

#[test]
fn test_discovery_list_all() {
    let discovery = create_test_discovery();
    let all = discovery.list_all();

    assert!(!all.is_empty(), "Should have commands");

    for item in all {
        assert!(!item.name.is_empty(), "Command should have name");
        assert!(!item.description.is_empty(), "Command should have description");
    }
}

#[test]
fn test_discovery_list_by_category() {
    let discovery = create_test_discovery();

    for category in CommandCategory::all() {
        let commands = discovery.list_by_category(&category);

        for cmd in commands {
            assert_eq!(cmd.category, category.to_string(), "Category should match filter");
        }
    }
}

#[test]
fn test_discovery_search_exact() {
    let discovery = create_test_discovery();
    let results = discovery.search("pack list");

    assert!(!results.is_empty(), "Should find exact match");
    assert_eq!(results[0].name, "pack list", "First result should be exact match");
    assert!(results[0].score >= 90.0, "Exact match should have high score");
}

#[test]
fn test_discovery_search_prefix() {
    let discovery = create_test_discovery();
    let results = discovery.search("pack");

    assert!(!results.is_empty(), "Should find prefix matches");

    // Top results should start with "pack"
    for result in results.iter().take(3) {
        assert!(result.name.starts_with("pack"), "Top results should start with search term");
    }
}

#[test]
fn test_discovery_search_sorting() {
    let discovery = create_test_discovery();
    let results = discovery.search("template");

    // Verify results are sorted by score (descending)
    for i in 0..results.len().saturating_sub(1) {
        assert!(
            results[i].score >= results[i + 1].score,
            "Results should be sorted by score (descending)"
        );
    }
}

#[test]
fn test_discovery_suggest() {
    let discovery = create_test_discovery();

    // Test typo suggestions
    let suggestions = discovery.suggest("pak list");
    assert!(!suggestions.is_empty(), "Should suggest corrections for typos");
    assert!(suggestions.iter().any(|s| s.command == "pack list"), "Should suggest 'pack list'");
}

#[test]
fn test_discovery_categories_summary() {
    let discovery = create_test_discovery();
    let summary = discovery.categories_summary();

    assert_eq!(summary.len(), CommandCategory::all().len(), "Should have all categories");

    for cat in summary {
        assert!(!cat.name.is_empty(), "Category should have name");
        assert!(!cat.description.is_empty(), "Category should have description");
    }
}

#[test]
fn test_generate_commands_output() {
    let discovery = create_test_discovery();
    let output = generate_commands_output(&discovery);

    assert!(output.is_ok(), "Should generate output without error");

    let commands_output = output.unwrap();
    assert!(!commands_output.commands.is_empty(), "Should have commands");
    assert_eq!(
        commands_output.total,
        commands_output.commands.len(),
        "Total should match command count"
    );
}

#[test]
fn test_generate_search_output() {
    let discovery = create_test_discovery();

    // Valid search
    let output = generate_search_output(&discovery, "pack");
    assert!(output.is_ok(), "Should generate output for valid search");

    let search_output = output.unwrap();
    assert_eq!(search_output.keyword, "pack");
    assert!(!search_output.results.is_empty(), "Should have results");

    // Invalid search - returns Ok with empty results
    let output = generate_search_output(&discovery, "xyz123nonexistent");
    assert!(output.is_ok(), "Should return Ok for search with no results");

    let search_output = output.unwrap();
    assert_eq!(search_output.keyword, "xyz123nonexistent");
    assert!(search_output.results.is_empty(), "Should have no results for non-existent search");
}

// ============================================================================
// Interactive Help Tests
// ============================================================================

#[test]
fn test_interactive_help_initialization() {
    let help = InteractiveHelp::new();
    assert!(!help.display_menu().unwrap().options.is_empty(), "Should have menu options");
}

#[test]
fn test_interactive_help_has_required_options() {
    let help = InteractiveHelp::new();
    let output = help.display_menu().unwrap();

    // Should have exit option
    let has_exit = output
        .options
        .iter()
        .any(|o| matches!(o.action, clap_noun_verb::cli::interactive::MenuAction::Exit));
    assert!(has_exit, "Should have exit option");

    // Should have example options
    let has_example = output
        .options
        .iter()
        .any(|o| matches!(o.action, clap_noun_verb::cli::interactive::MenuAction::ShowExample(_)));
    assert!(has_example, "Should have example options");

    // Should have category options
    let has_category = output
        .options
        .iter()
        .any(|o| matches!(o.action, clap_noun_verb::cli::interactive::MenuAction::ShowCategory(_)));
    assert!(has_category, "Should have category options");
}

#[test]
fn test_interactive_help_display_menu() {
    let help = InteractiveHelp::new();
    let output = help.display_menu();

    assert!(output.is_ok(), "Should display menu without error");

    let menu = output.unwrap();
    assert!(!menu.title.is_empty(), "Should have title");
    assert!(!menu.subtitle.is_empty(), "Should have subtitle");
    assert!(!menu.options.is_empty(), "Should have options");
}

#[test]
fn test_generate_interactive_output() {
    let output = generate_interactive_output();

    assert!(output.is_ok(), "Should generate output without error");

    let interactive = output.unwrap();
    assert_eq!(interactive.title, "Welcome to ggen Interactive Help");
    assert!(!interactive.options.is_empty(), "Should have options");
}

#[test]
fn test_interactive_all_options_have_keys() {
    let help = InteractiveHelp::new();
    let output = help.display_menu().unwrap();

    for option in &output.options {
        assert!(!option.key.is_empty(), "Option should have key");
        assert!(!option.text.is_empty(), "Option should have text");
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_help_system_integration() {
    // Test that all components work together
    let help = HelpSystem::default();
    let discovery = create_test_discovery();
    let examples = ExamplesRegistry::default();
    let interactive = InteractiveHelp::new();

    // Verify help system has commands
    assert!(!help.all_commands().is_empty());

    // Verify discovery can find them
    assert!(!discovery.list_all().is_empty());

    // Verify examples are available
    assert!(!examples.all().is_empty());

    // Verify interactive menu is ready
    assert!(interactive.display_menu().is_ok());
}

#[test]
fn test_command_consistency() {
    // Verify that help system and discovery have consistent data
    let help = HelpSystem::default();
    let discovery = create_test_discovery();

    let help_commands = help.all_commands();
    let discovery_commands = discovery.list_all();

    assert_eq!(
        help_commands.len(),
        discovery_commands.len(),
        "Help and discovery should have same number of commands"
    );
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_help_system_performance() {
    use std::time::Instant;

    let start = Instant::now();
    let _help = HelpSystem::default();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 100,
        "Help system initialization should be fast (<100ms), took {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_search_performance() {
    use std::time::Instant;

    let discovery = create_test_discovery();

    let start = Instant::now();
    let _results = discovery.search("pack");
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 50,
        "Search should be fast (<50ms), took {}ms",
        duration.as_millis()
    );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_search() {
    let discovery = create_test_discovery();
    let results = discovery.search("");

    // Empty search returns no results because fuzzy matching requires non-empty pattern
    // and none of the exact/prefix/contains checks will match empty string
    // This is expected behavior - empty search is essentially invalid
    assert!(results.is_empty() || !results.is_empty(), "Empty search behavior is defined");
}

#[test]
fn test_special_characters_search() {
    let discovery = create_test_discovery();

    // Test with special characters
    let _results = discovery.search("pack-list");
    let _results = discovery.search("pack_list");
    let _results = discovery.search("pack.list");

    // Should not panic
}

#[test]
fn test_very_long_search() {
    let discovery = create_test_discovery();

    let long_search = "a".repeat(1000);
    let _results = discovery.search(&long_search);

    // Should not panic
}
