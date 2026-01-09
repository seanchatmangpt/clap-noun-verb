//! Integration tests for wizard example display and search functionality
//!
//! Tests the example management system used by the interactive wizard:
//! - Example search and retrieval
//! - Example display formatting
//! - Example variation handling
//! - Example registry integration
//!
//! Chicago TDD Principles:
//! - State-based testing (verify example data structures)
//! - Real collaborators (use actual ExamplesRegistry)
//! - Behavior verification (test observable outputs)
//! - AAA pattern (Arrange-Act-Assert)

use clap_noun_verb::cli::examples::{Example, ExampleVariation, ExamplesRegistry};

// =============================================================================
// EXAMPLE REGISTRY TESTS
// =============================================================================

#[test]
fn test_examples_registry_initialization() {
    // Arrange & Act
    let registry = ExamplesRegistry::default();

    // Assert: Registry should have examples
    let all_examples = registry.all();
    assert!(!all_examples.is_empty(), "Registry should contain examples for wizard");
}

#[test]
fn test_example_search_by_keyword() {
    // Arrange
    let registry = ExamplesRegistry::default();

    // Act: Search for "pack" examples
    let results = registry.search("pack");

    // Assert
    assert!(!results.is_empty(), "Should find examples related to 'pack'");

    // Verify all results are relevant
    for example in results {
        let matches_title = example.title.to_lowercase().contains("pack");
        let matches_description = example.description.to_lowercase().contains("pack");
        let matches_command = example.command.to_lowercase().contains("pack");

        assert!(
            matches_title || matches_description || matches_command,
            "Example '{}' should match search term 'pack'",
            example.title
        );
    }
}

#[test]
fn test_example_search_case_insensitive() {
    // Arrange
    let registry = ExamplesRegistry::default();

    // Act: Search with different cases
    let results_lower = registry.search("ai");
    let results_upper = registry.search("AI");
    let results_mixed = registry.search("Ai");

    // Assert: All should return same results
    assert_eq!(results_lower.len(), results_upper.len(), "Search should be case-insensitive");
    assert_eq!(results_lower.len(), results_mixed.len(), "Search should be case-insensitive");
}

#[test]
fn test_example_structure_is_complete() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Act & Assert: Verify each example has required fields
    for example in examples {
        assert!(!example.title.is_empty(), "Example title must not be empty");
        assert!(!example.description.is_empty(), "Example description must not be empty");
        assert!(!example.command.is_empty(), "Example command must not be empty");
        assert!(!example.expected_output.is_empty(), "Example expected_output must not be empty");

        // Verify variations are well-formed
        for variation in &example.variations {
            assert!(!variation.description.is_empty(), "Variation description must not be empty");
            assert!(!variation.command.is_empty(), "Variation command must not be empty");
        }
    }
}

// =============================================================================
// EXAMPLE CONTENT VALIDATION TESTS
// =============================================================================

#[test]
fn test_example_commands_are_valid() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Act & Assert: Verify commands look valid
    for example in examples {
        let command = &example.command;

        // Should not have trailing whitespace
        assert_eq!(
            command.trim(),
            command,
            "Command should not have trailing whitespace: '{}'",
            command
        );

        // Should be reasonable length
        assert!(command.len() >= 5, "Command should be at least 5 chars: '{}'", command);
        assert!(command.len() <= 200, "Command should be concise (<= 200 chars): '{}'", command);
    }
}

#[test]
fn test_example_descriptions_are_informative() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Act & Assert: Verify descriptions are informative
    for example in examples {
        let description = &example.description;

        // Should be reasonable length
        assert!(
            description.len() >= 10,
            "Description should be informative (>= 10 chars): '{}'",
            example.title
        );

        // Should not be same as title
        assert_ne!(
            description.to_lowercase(),
            example.title.to_lowercase(),
            "Description should differ from title for example: '{}'",
            example.title
        );
    }
}

#[test]
fn test_example_expected_output_provided() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Act & Assert: All examples should show expected output
    for example in examples {
        assert!(
            !example.expected_output.is_empty(),
            "Example '{}' should have expected output",
            example.title
        );

        // Expected output should be reasonable length
        assert!(
            example.expected_output.len() >= 5,
            "Expected output should be meaningful for: '{}'",
            example.title
        );
    }
}

// =============================================================================
// EXAMPLE VARIATION TESTS
// =============================================================================

#[test]
fn test_example_variations_are_distinct() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Act & Assert: Variations should differ from base command
    for example in examples {
        for variation in &example.variations {
            assert_ne!(
                variation.command, example.command,
                "Variation should differ from base command for: '{}'",
                example.title
            );

            // Variation should have descriptive text
            assert!(
                variation.description.len() >= 5,
                "Variation description should be meaningful for: '{}'",
                example.title
            );
        }
    }
}

#[test]
fn test_example_variations_have_unique_commands() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Act & Assert: Within each example, variations should be unique
    for example in examples {
        let mut commands = std::collections::HashSet::new();

        // Add base command
        commands.insert(&example.command);

        // Check variations
        for variation in &example.variations {
            assert!(
                commands.insert(&variation.command),
                "Variation commands should be unique for example: '{}'",
                example.title
            );
        }
    }
}

// =============================================================================
// SEARCH FUNCTIONALITY TESTS
// =============================================================================

#[test]
fn test_search_returns_relevant_results() {
    // Arrange
    let registry = ExamplesRegistry::default();

    // Act: Search for specific terms
    let search_terms = vec!["pack", "ai", "marketplace", "generate"];

    for term in search_terms {
        let results = registry.search(term);

        // Assert: Results should be relevant
        if !results.is_empty() {
            for example in results {
                let text = format!("{} {} {}", example.title, example.description, example.command)
                    .to_lowercase();

                assert!(
                    text.contains(term),
                    "Search result '{}' should contain term '{}'",
                    example.title,
                    term
                );
            }
        }
    }
}

#[test]
fn test_search_empty_string_returns_all() {
    // Arrange
    let registry = ExamplesRegistry::default();

    // Act
    let all_examples = registry.all();
    let search_results = registry.search("");

    // Assert: Empty search should return all examples
    assert_eq!(search_results.len(), all_examples.len(), "Empty search should return all examples");
}

#[test]
fn test_search_nonexistent_term_returns_empty() {
    // Arrange
    let registry = ExamplesRegistry::default();

    // Act: Search for term that doesn't exist
    let results = registry.search("xyznonexistentterm123");

    // Assert: Should return empty results gracefully
    assert!(results.is_empty(), "Search for nonexistent term should return empty results");
}

// =============================================================================
// EXAMPLE REGISTRY CONSISTENCY TESTS
// =============================================================================

#[test]
fn test_registry_is_deterministic() {
    // Arrange & Act: Create multiple registries
    let registry1 = ExamplesRegistry::default();
    let registry2 = ExamplesRegistry::default();

    let examples1 = registry1.all();
    let examples2 = registry2.all();

    // Assert: Should have same examples
    assert_eq!(examples1.len(), examples2.len(), "Registry should be deterministic");

    for (ex1, ex2) in examples1.iter().zip(examples2.iter()) {
        assert_eq!(ex1.title, ex2.title, "Example titles should match");
        assert_eq!(ex1.command, ex2.command, "Example commands should match");
    }
}

#[test]
fn test_registry_search_is_deterministic() {
    // Arrange
    let registry = ExamplesRegistry::default();

    // Act: Search multiple times
    let results1 = registry.search("pack");
    let results2 = registry.search("pack");
    let results3 = registry.search("pack");

    // Assert: Results should be identical
    assert_eq!(results1.len(), results2.len());
    assert_eq!(results2.len(), results3.len());

    for i in 0..results1.len() {
        assert_eq!(results1[i].title, results2[i].title);
        assert_eq!(results2[i].title, results3[i].title);
    }
}

// =============================================================================
// EXAMPLE DATA QUALITY TESTS
// =============================================================================

#[test]
fn test_examples_have_no_duplicates() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Act: Check for duplicate titles
    let mut titles = std::collections::HashSet::new();

    // Assert
    for example in examples {
        assert!(
            titles.insert(&example.title),
            "Example titles should be unique, found duplicate: '{}'",
            example.title
        );
    }
}

#[test]
fn test_examples_cover_common_use_cases() {
    // Arrange
    let registry = ExamplesRegistry::default();

    // Act: Search for common wizard use cases
    let use_cases = vec![
        ("pack", "pack management"),
        ("ai", "AI features"),
        ("marketplace", "marketplace features"),
    ];

    // Assert: Should have examples for common use cases
    for (term, description) in use_cases {
        let results = registry.search(term);
        assert!(
            !results.is_empty(),
            "Should have examples for {}: searching '{}'",
            description,
            term
        );
    }
}

#[test]
fn test_example_registry_is_well_populated() {
    // Arrange
    let registry = ExamplesRegistry::default();
    let examples = registry.all();

    // Assert: Should have reasonable number of examples
    assert!(examples.len() >= 3, "Registry should have at least 3 examples for wizard");
    assert!(
        examples.len() <= 50,
        "Registry should have at most 50 examples to avoid overwhelming users"
    );
}

// =============================================================================
// INTEGRATION WITH WIZARD TESTS
// =============================================================================

#[test]
fn test_wizard_can_find_pack_examples() {
    // Arrange: Simulate wizard showing "pack list" example
    let registry = ExamplesRegistry::default();

    // Act
    let results = registry.search("pack list");

    // Assert: Should find relevant example
    assert!(!results.is_empty(), "Wizard should be able to find pack list examples");

    if let Some(example) = results.first() {
        assert!(example.command.contains("pack"));
        assert!(!example.expected_output.is_empty());
    }
}

#[test]
fn test_wizard_can_find_ai_examples() {
    // Arrange: Simulate wizard showing "ai generate" example
    let registry = ExamplesRegistry::default();

    // Act
    let results = registry.search("ai generate");

    // Assert
    assert!(!results.is_empty(), "Wizard should be able to find AI generation examples");

    if let Some(example) = results.first() {
        assert!(example.command.contains("ai") || example.title.to_lowercase().contains("ai"));
    }
}

#[test]
fn test_wizard_can_find_marketplace_examples() {
    // Arrange: Simulate wizard showing "marketplace search" example
    let registry = ExamplesRegistry::default();

    // Act
    let results = registry.search("marketplace search");

    // Assert
    assert!(!results.is_empty(), "Wizard should be able to find marketplace search examples");
}

// =============================================================================
// EXAMPLE SERIALIZATION TESTS
// =============================================================================

#[test]
fn test_example_json_serialization() {
    // Arrange
    let example = Example {
        title: "Test Example".to_string(),
        description: "A test example".to_string(),
        command: "test command".to_string(),
        expected_output: "output".to_string(),
        variations: vec![ExampleVariation {
            description: "variation".to_string(),
            command: "variation command".to_string(),
        }],
    };

    // Act
    let json = serde_json::to_string(&example);

    // Assert
    assert!(json.is_ok(), "Example should serialize to JSON");

    let json_str = json.unwrap();
    assert!(json_str.contains("title"));
    assert!(json_str.contains("command"));
    assert!(json_str.contains("variations"));
}

#[test]
fn test_example_variation_json_serialization() {
    // Arrange
    let variation = ExampleVariation {
        description: "Test variation".to_string(),
        command: "test command".to_string(),
    };

    // Act
    let json = serde_json::to_string(&variation);

    // Assert
    assert!(json.is_ok(), "ExampleVariation should serialize to JSON");

    let json_str = json.unwrap();
    assert!(json_str.contains("description"));
    assert!(json_str.contains("command"));
}
