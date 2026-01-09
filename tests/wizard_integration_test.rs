//! Integration tests for wizard/interactive help CLI functionality
//!
//! Tests end-to-end wizard workflows including:
//! - Menu display and navigation
//! - Guided setup workflow
//! - Example display and search
//! - Category filtering
//! - Quickstart guide
//!
//! Chicago TDD Principles:
//! - State-based testing (verify outputs and state changes)
//! - Real collaborators (use actual InteractiveHelp implementation)
//! - Behavior verification (test observable outcomes)
//! - AAA pattern (Arrange-Act-Assert)

use clap_noun_verb::cli::help::CommandCategory;
use clap_noun_verb::cli::interactive::{
    generate_interactive_output, InteractiveHelp, MenuAction, MenuOption,
};
use serde_json;

// =============================================================================
// ARRANGE-ACT-ASSERT PATTERN TESTS
// =============================================================================

#[test]
fn test_interactive_help_initialization() {
    // Arrange: No setup needed

    // Act: Create new interactive help instance
    let help = InteractiveHelp::new();

    // Assert: Verify state
    let output = help.display_menu();
    assert!(output.is_ok(), "Display menu should succeed");

    let menu = output.unwrap();
    assert!(!menu.options.is_empty(), "Menu should have options");
    assert_eq!(menu.title, "Welcome to ggen Interactive Help", "Menu should have correct title");
}

#[test]
fn test_menu_has_all_required_options() {
    // Arrange
    let help = InteractiveHelp::new();

    // Act
    let output = help.display_menu().expect("Display menu should succeed");

    // Assert: Verify all required menu options exist
    let has_exit = output.options.iter().any(|o| matches!(o.action, MenuAction::Exit));
    let has_guided_setup =
        output.options.iter().any(|o| matches!(o.action, MenuAction::GuidedSetup));
    let has_quickstart = output.options.iter().any(|o| matches!(o.action, MenuAction::Quickstart));
    let has_examples =
        output.options.iter().any(|o| matches!(o.action, MenuAction::ShowExample(_)));
    let has_categories =
        output.options.iter().any(|o| matches!(o.action, MenuAction::ShowCategory(_)));

    assert!(has_exit, "Menu must have exit option");
    assert!(has_guided_setup, "Menu must have guided setup option");
    assert!(has_quickstart, "Menu must have quickstart option");
    assert!(has_examples, "Menu must have example options");
    assert!(has_categories, "Menu must have category options");
}

#[test]
fn test_menu_options_have_unique_keys() {
    // Arrange
    let help = InteractiveHelp::new();

    // Act
    let output = help.display_menu().expect("Display menu should succeed");

    // Assert: Verify all keys are unique
    let mut keys = std::collections::HashSet::new();
    for option in &output.options {
        assert!(keys.insert(&option.key), "Menu key '{}' must be unique", option.key);
    }
}

#[test]
fn test_menu_options_have_valid_text() {
    // Arrange
    let help = InteractiveHelp::new();

    // Act
    let output = help.display_menu().expect("Display menu should succeed");

    // Assert: Verify all options have non-empty text
    for option in &output.options {
        assert!(!option.key.is_empty(), "Option key must not be empty");
        assert!(!option.text.is_empty(), "Option text must not be empty");
        assert!(option.text.len() < 100, "Option text should be concise (< 100 chars)");
    }
}

// =============================================================================
// MENU ACTION TESTS
// =============================================================================

#[test]
fn test_menu_action_show_example() {
    // Arrange
    let help = InteractiveHelp::new();
    let output = help.display_menu().expect("Display menu should succeed");

    // Act: Find example action
    let example_option =
        output.options.iter().find(|o| matches!(o.action, MenuAction::ShowExample(_)));

    // Assert
    assert!(example_option.is_some(), "Should have at least one example option");

    if let Some(option) = example_option {
        if let MenuAction::ShowExample(example_name) = &option.action {
            assert!(!example_name.is_empty(), "Example name should not be empty");
        }
    }
}

#[test]
fn test_menu_action_show_category() {
    // Arrange
    let help = InteractiveHelp::new();
    let output = help.display_menu().expect("Display menu should succeed");

    // Act: Find category actions
    let category_options: Vec<_> =
        output.options.iter().filter(|o| matches!(o.action, MenuAction::ShowCategory(_))).collect();

    // Assert
    assert!(!category_options.is_empty(), "Should have at least one category option");

    // Verify categories are valid
    for option in category_options {
        if let MenuAction::ShowCategory(category) = &option.action {
            // Verify category has a description
            let description = category.description();
            assert!(!description.is_empty(), "Category should have a description");
        }
    }
}

#[test]
fn test_menu_action_exit() {
    // Arrange
    let help = InteractiveHelp::new();
    let output = help.display_menu().expect("Display menu should succeed");

    // Act: Find exit option
    let exit_option = output.options.iter().find(|o| matches!(o.action, MenuAction::Exit));

    // Assert
    assert!(exit_option.is_some(), "Should have exit option");

    let exit = exit_option.unwrap();
    assert!(exit.text.to_lowercase().contains("exit"), "Exit option text should contain 'exit'");
}

// =============================================================================
// SERIALIZATION TESTS (State-based verification)
// =============================================================================

#[test]
fn test_interactive_output_json_serialization() {
    // Arrange
    let output = generate_interactive_output().expect("Should generate output");

    // Act: Serialize to JSON
    let json = serde_json::to_string(&output);

    // Assert
    assert!(json.is_ok(), "Should serialize to JSON");

    let json_str = json.unwrap();
    assert!(json_str.contains("title"), "JSON should contain title field");
    assert!(json_str.contains("options"), "JSON should contain options field");
    assert!(json_str.contains("subtitle"), "JSON should contain subtitle field");
}

#[test]
fn test_menu_option_json_serialization() {
    // Arrange
    let option = MenuOption {
        key: "1".to_string(),
        text: "Test option".to_string(),
        action: MenuAction::Exit,
    };

    // Act
    let json = serde_json::to_string(&option);

    // Assert
    assert!(json.is_ok(), "MenuOption should serialize to JSON");

    let json_str = json.unwrap();
    assert!(json_str.contains("key"), "JSON should contain key");
    assert!(json_str.contains("text"), "JSON should contain text");
    assert!(json_str.contains("action"), "JSON should contain action");
}

#[test]
fn test_menu_action_variants_serialization() {
    // Arrange: Test all MenuAction variants
    let actions = vec![
        MenuAction::Exit,
        MenuAction::GuidedSetup,
        MenuAction::Quickstart,
        MenuAction::ShowExample("test".to_string()),
        MenuAction::ShowCategory(CommandCategory::Pack),
    ];

    // Act & Assert: Each variant should serialize
    for action in actions {
        let json = serde_json::to_string(&action);
        assert!(json.is_ok(), "MenuAction variant should serialize: {:?}", action);
    }
}

// =============================================================================
// DETERMINISTIC BEHAVIOR TESTS (Regression)
// =============================================================================

#[test]
fn test_menu_structure_is_deterministic() {
    // Arrange & Act: Create multiple instances
    let help1 = InteractiveHelp::new();
    let help2 = InteractiveHelp::new();

    let output1 = help1.display_menu().expect("Display should succeed");
    let output2 = help2.display_menu().expect("Display should succeed");

    // Assert: Both should have identical structure
    assert_eq!(output1.title, output2.title, "Title should be deterministic");
    assert_eq!(output1.subtitle, output2.subtitle, "Subtitle should be deterministic");
    assert_eq!(
        output1.options.len(),
        output2.options.len(),
        "Number of options should be deterministic"
    );

    // Verify options are identical
    for (opt1, opt2) in output1.options.iter().zip(output2.options.iter()) {
        assert_eq!(opt1.key, opt2.key, "Option keys should match");
        assert_eq!(opt1.text, opt2.text, "Option text should match");
    }
}

#[test]
fn test_menu_output_is_idempotent() {
    // Arrange
    let help = InteractiveHelp::new();

    // Act: Call display_menu multiple times
    let output1 = help.display_menu().expect("First call should succeed");
    let output2 = help.display_menu().expect("Second call should succeed");
    let output3 = help.display_menu().expect("Third call should succeed");

    // Assert: All outputs should be identical
    assert_eq!(output1.title, output2.title);
    assert_eq!(output2.title, output3.title);
    assert_eq!(output1.options.len(), output2.options.len());
    assert_eq!(output2.options.len(), output3.options.len());
}

#[test]
fn test_generate_interactive_output_is_deterministic() {
    // Act: Generate output multiple times
    let output1 = generate_interactive_output().expect("Should generate");
    let output2 = generate_interactive_output().expect("Should generate");

    // Assert: Outputs should be identical
    assert_eq!(output1.title, output2.title);
    assert_eq!(output1.subtitle, output2.subtitle);
    assert_eq!(output1.options.len(), output2.options.len());

    // Verify JSON serialization is deterministic
    let json1 = serde_json::to_string(&output1).expect("Should serialize");
    let json2 = serde_json::to_string(&output2).expect("Should serialize");
    assert_eq!(json1, json2, "JSON output should be deterministic");
}

// =============================================================================
// COMMAND CATEGORY INTEGRATION TESTS
// =============================================================================

#[test]
fn test_category_options_reference_valid_categories() {
    // Arrange
    let help = InteractiveHelp::new();
    let output = help.display_menu().expect("Display should succeed");

    // Act: Extract all categories
    let categories: Vec<_> =
        output
            .options
            .iter()
            .filter_map(|o| {
                if let MenuAction::ShowCategory(cat) = &o.action {
                    Some(cat)
                } else {
                    None
                }
            })
            .collect();

    // Assert: All categories should be valid
    for category in categories {
        let description = category.description();
        assert!(!description.is_empty(), "Category {:?} should have description", category);

        // Verify category Display implementation works
        let display_str = format!("{}", category);
        assert!(!display_str.is_empty(), "Category should display as string");
    }
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_interactive_help_handles_default() {
    // Arrange & Act
    let help = InteractiveHelp::default();
    let output = help.display_menu();

    // Assert
    assert!(output.is_ok(), "Default instance should work");
    assert!(!output.unwrap().options.is_empty());
}

#[test]
fn test_generate_interactive_output_never_fails() {
    // Act: Generate output
    let result = generate_interactive_output();

    // Assert: Should always succeed
    assert!(result.is_ok(), "Interactive output generation should not fail");
}

// =============================================================================
// STATE CONSISTENCY TESTS
// =============================================================================

#[test]
fn test_menu_maintains_state_consistency() {
    // Arrange
    let help = InteractiveHelp::new();

    // Act: Perform multiple operations
    let _output1 = help.display_menu().expect("Should succeed");
    let _output2 = help.display_menu().expect("Should succeed");
    let output3 = help.display_menu().expect("Should succeed");

    // Assert: Final state should be consistent
    assert!(!output3.title.is_empty());
    assert!(!output3.subtitle.is_empty());
    assert!(!output3.options.is_empty());

    // Verify no corruption
    for option in &output3.options {
        assert!(!option.key.is_empty());
        assert!(!option.text.is_empty());
    }
}

#[test]
fn test_menu_options_are_well_formed() {
    // Arrange
    let help = InteractiveHelp::new();
    let output = help.display_menu().expect("Should succeed");

    // Act & Assert: Verify each option is well-formed
    for option in &output.options {
        // Key should be short and simple
        assert!(option.key.len() <= 3, "Key '{}' should be 1-3 characters", option.key);

        // Text should be readable
        assert!(option.text.len() >= 5, "Text should be descriptive (>= 5 chars)");
        assert!(option.text.len() <= 100, "Text should be concise (<= 100 chars)");

        // Should not have trailing/leading whitespace
        assert_eq!(option.key.trim(), option.key, "Key should be trimmed");
        assert_eq!(option.text.trim(), option.text, "Text should be trimmed");
    }
}

// =============================================================================
// COMPREHENSIVE WORKFLOW TESTS
// =============================================================================

#[test]
fn test_complete_wizard_workflow_simulation() {
    // Arrange: Simulate complete user workflow
    let help = InteractiveHelp::new();

    // Act: Step 1 - Display menu
    let menu = help.display_menu().expect("Display menu should work");

    // Assert: Menu is valid
    assert!(!menu.options.is_empty());

    // Act: Step 2 - Find and verify exit option
    let exit_option = menu.options.iter().find(|o| matches!(o.action, MenuAction::Exit));
    assert!(exit_option.is_some());

    // Act: Step 3 - Find and verify example option
    let example_option =
        menu.options.iter().find(|o| matches!(o.action, MenuAction::ShowExample(_)));
    assert!(example_option.is_some());

    // Act: Step 4 - Serialize for JSON output
    let json = serde_json::to_string(&menu);
    assert!(json.is_ok());

    // Assert: Complete workflow succeeded
    assert!(menu.title.contains("Interactive Help"));
}

#[test]
fn test_wizard_supports_multiple_interaction_patterns() {
    // Arrange
    let help = InteractiveHelp::new();
    let output = help.display_menu().expect("Should succeed");

    // Act: Count different action types
    let mut action_counts = std::collections::HashMap::new();
    for option in &output.options {
        let action_type = match &option.action {
            MenuAction::Exit => "exit",
            MenuAction::GuidedSetup => "guided_setup",
            MenuAction::Quickstart => "quickstart",
            MenuAction::ShowExample(_) => "show_example",
            MenuAction::ShowCategory(_) => "show_category",
        };
        *action_counts.entry(action_type).or_insert(0) += 1;
    }

    // Assert: Should support diverse interactions
    assert!(action_counts.len() >= 3, "Should support at least 3 interaction patterns");
    assert!(action_counts.get("exit").is_some(), "Must have exit action");
}

#[test]
fn test_wizard_menu_scalability() {
    // Arrange
    let help = InteractiveHelp::new();
    let output = help.display_menu().expect("Should succeed");

    // Act: Verify menu can handle reasonable number of options
    let option_count = output.options.len();

    // Assert: Menu should have reasonable bounds
    assert!(option_count >= 5, "Should have at least 5 options for usability");
    assert!(option_count <= 20, "Should have at most 20 options to avoid overwhelming users");
}
