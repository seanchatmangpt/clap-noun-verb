//! Property-based tests for wizard/interactive help using proptest
//!
//! Tests invariants and properties that should hold for all inputs.
//! Uses proptest for generating random test data.
//!
//! Chicago TDD Principles:
//! - Property-based testing for robustness
//! - State-based verification
//! - Behavior verification across input space

use clap_noun_verb::cli::interactive::{InteractiveHelp, MenuAction, MenuOption};
use proptest::prelude::*;
use serde_json;

// =============================================================================
// PROPERTY-BASED TESTS FOR MENU OPTIONS
// =============================================================================

proptest! {
    /// Property: Menu should always be displayable regardless of state
    #[test]
    fn prop_menu_always_displayable(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let result = help.display_menu();
        prop_assert!(result.is_ok(), "Menu display should always succeed");

        let output = result.unwrap();
        prop_assert!(!output.options.is_empty(), "Menu should always have options");
    }

    /// Property: Menu title should always be non-empty
    #[test]
    fn prop_menu_title_non_empty(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        prop_assert!(!output.title.is_empty(), "Title must not be empty");
        prop_assert!(!output.subtitle.is_empty(), "Subtitle must not be empty");
    }

    /// Property: All menu keys should be unique
    #[test]
    fn prop_menu_keys_unique(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        let mut keys = std::collections::HashSet::new();
        for option in &output.options {
            prop_assert!(
                keys.insert(&option.key),
                "All keys must be unique, duplicate found: {}",
                option.key
            );
        }
    }

    /// Property: Menu serialization should always succeed
    #[test]
    fn prop_menu_serialization_succeeds(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        let json_result = serde_json::to_string(&output);
        prop_assert!(json_result.is_ok(), "Serialization should always succeed");

        let json = json_result.unwrap();
        prop_assert!(!json.is_empty(), "JSON should not be empty");
        prop_assert!(json.contains("title"), "JSON should contain title");
    }

    /// Property: Menu should be deterministic across multiple calls
    #[test]
    fn prop_menu_deterministic(call_count in 1..10usize) {
        let help = InteractiveHelp::new();

        let outputs: Vec<_> = (0..call_count)
            .map(|_| help.display_menu().unwrap())
            .collect();

        // All outputs should be identical
        for i in 1..outputs.len() {
            prop_assert_eq!(
                outputs[0].title,
                outputs[i].title,
                "Title should be deterministic"
            );
            prop_assert_eq!(
                outputs[0].options.len(),
                outputs[i].options.len(),
                "Option count should be deterministic"
            );
        }
    }
}

// =============================================================================
// PROPERTY-BASED TESTS FOR MENU ACTIONS
// =============================================================================

proptest! {
    /// Property: All menu actions should be serializable
    #[test]
    fn prop_all_actions_serializable(_iterations in 0..50u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        for option in &output.options {
            let json_result = serde_json::to_string(&option.action);
            prop_assert!(
                json_result.is_ok(),
                "Action {:?} should serialize",
                option.action
            );
        }
    }

    /// Property: Menu should have at least one exit option
    #[test]
    fn prop_menu_has_exit(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        let has_exit = output.options.iter()
            .any(|o| matches!(o.action, MenuAction::Exit));

        prop_assert!(has_exit, "Menu must always have an exit option");
    }

    /// Property: Menu options should have bounded text length
    #[test]
    fn prop_option_text_bounded(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        for option in &output.options {
            prop_assert!(
                option.text.len() >= 3,
                "Option text should be at least 3 chars: '{}'",
                option.text
            );
            prop_assert!(
                option.text.len() <= 200,
                "Option text should be at most 200 chars: '{}'",
                option.text
            );
        }
    }

    /// Property: Menu keys should be simple (1-3 characters)
    #[test]
    fn prop_menu_keys_simple(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        for option in &output.options {
            prop_assert!(
                !option.key.is_empty(),
                "Key must not be empty"
            );
            prop_assert!(
                option.key.len() <= 5,
                "Key should be simple (<=5 chars): '{}'",
                option.key
            );
        }
    }
}

// =============================================================================
// PROPERTY-BASED TESTS FOR STATE CONSISTENCY
// =============================================================================

proptest! {
    /// Property: Multiple instances should produce identical menus
    #[test]
    fn prop_instances_produce_identical_menus(instance_count in 2..10usize) {
        let helps: Vec<_> = (0..instance_count)
            .map(|_| InteractiveHelp::new())
            .collect();

        let outputs: Vec<_> = helps.iter()
            .map(|h| h.display_menu().unwrap())
            .collect();

        // All should be identical
        for i in 1..outputs.len() {
            prop_assert_eq!(
                outputs[0].title,
                outputs[i].title,
                "All instances should have same title"
            );
            prop_assert_eq!(
                outputs[0].options.len(),
                outputs[i].options.len(),
                "All instances should have same option count"
            );
        }
    }

    /// Property: Default instance should equal new instance
    #[test]
    fn prop_default_equals_new(_iterations in 0..100u32) {
        let help_new = InteractiveHelp::new();
        let help_default = InteractiveHelp::default();

        let output_new = help_new.display_menu().unwrap();
        let output_default = help_default.display_menu().unwrap();

        prop_assert_eq!(
            output_new.title,
            output_default.title,
            "New and default should produce identical titles"
        );
        prop_assert_eq!(
            output_new.options.len(),
            output_default.options.len(),
            "New and default should have same option count"
        );
    }
}

// =============================================================================
// PROPERTY-BASED TESTS FOR JSON SERIALIZATION
// =============================================================================

proptest! {
    /// Property: Serialization should be deterministic
    #[test]
    fn prop_serialization_deterministic(repetitions in 1..10usize) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        let jsons: Vec<_> = (0..repetitions)
            .map(|_| serde_json::to_string(&output).unwrap())
            .collect();

        // All JSON strings should be identical
        for i in 1..jsons.len() {
            prop_assert_eq!(
                &jsons[0],
                &jsons[i],
                "Serialization should be deterministic"
            );
        }
    }

    /// Property: Deserialization round-trip should preserve structure
    #[test]
    fn prop_json_roundtrip_preserves_structure(_iterations in 0..50u32) {
        let help = InteractiveHelp::new();
        let original = help.display_menu().unwrap();

        // Serialize
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize
        let deserialized: Result<
            clap_noun_verb::cli::interactive::InteractiveOutput,
            _
        > = serde_json::from_str(&json);

        prop_assert!(deserialized.is_ok(), "Deserialization should succeed");

        let restored = deserialized.unwrap();
        prop_assert_eq!(
            original.title,
            restored.title,
            "Title should survive round-trip"
        );
        prop_assert_eq!(
            original.options.len(),
            restored.options.len(),
            "Option count should survive round-trip"
        );
    }
}

// =============================================================================
// PROPERTY-BASED TESTS FOR MENU OPTION STRUCTURE
// =============================================================================

proptest! {
    /// Property: All menu options should be well-formed
    #[test]
    fn prop_menu_options_well_formed(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        for option in &output.options {
            // Key validations
            prop_assert!(
                !option.key.is_empty(),
                "Key must not be empty"
            );
            prop_assert_eq!(
                option.key.trim(),
                option.key,
                "Key should have no leading/trailing whitespace"
            );

            // Text validations
            prop_assert!(
                !option.text.is_empty(),
                "Text must not be empty"
            );
            prop_assert_eq!(
                option.text.trim(),
                option.text,
                "Text should have no leading/trailing whitespace"
            );
        }
    }

    /// Property: Menu should have diverse action types
    #[test]
    fn prop_menu_has_diverse_actions(_iterations in 0..100u32) {
        let help = InteractiveHelp::new();
        let output = help.display_menu().unwrap();

        let mut action_types = std::collections::HashSet::new();
        for option in &output.options {
            let action_type = match &option.action {
                MenuAction::Exit => "exit",
                MenuAction::GuidedSetup => "guided_setup",
                MenuAction::Quickstart => "quickstart",
                MenuAction::ShowExample(_) => "show_example",
                MenuAction::ShowCategory(_) => "show_category",
            };
            action_types.insert(action_type);
        }

        prop_assert!(
            action_types.len() >= 2,
            "Menu should have at least 2 different action types for diversity"
        );
    }
}

// =============================================================================
// PROPERTY-BASED TESTS FOR ROBUSTNESS
// =============================================================================

proptest! {
    /// Property: Menu generation should never panic
    #[test]
    fn prop_menu_generation_never_panics(iterations in 0..1000u32) {
        for _ in 0..iterations {
            let _help = InteractiveHelp::new();
            let _output = _help.display_menu();
            // If we get here without panic, property holds
        }
    }

    /// Property: Menu should handle concurrent access safely
    #[test]
    fn prop_menu_thread_safe(thread_count in 1..10usize) {
        use std::sync::Arc;
        use std::thread;

        let help = Arc::new(InteractiveHelp::new());
        let mut handles = vec![];

        for _ in 0..thread_count {
            let help_clone = Arc::clone(&help);
            let handle = thread::spawn(move || {
                help_clone.display_menu()
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.join();
            prop_assert!(result.is_ok(), "Thread should not panic");

            let output = result.unwrap();
            prop_assert!(output.is_ok(), "Display should succeed in thread");
        }
    }
}
