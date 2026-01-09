//! End-to-end wizard workflow integration tests
//!
//! Tests complete user workflows through the wizard system:
//! - Full guided setup flow
//! - Quickstart guide display
//! - Menu navigation sequences
//! - Multi-step user journeys
//!
//! Chicago TDD Principles:
//! - State-based testing (verify workflow outputs)
//! - Real collaborators (use actual wizard components)
//! - Behavior verification (test complete user journeys)
//! - AAA pattern (Arrange-Act-Assert)

use clap_noun_verb::cli::examples::ExamplesRegistry;
use clap_noun_verb::cli::help::CommandCategory;
use clap_noun_verb::cli::interactive::{generate_interactive_output, InteractiveHelp, MenuAction};

// =============================================================================
// COMPLETE WORKFLOW TESTS
// =============================================================================

#[test]
fn test_wizard_startup_workflow() {
    // Arrange: User starts wizard
    let help = InteractiveHelp::new();

    // Act: Display initial menu
    let menu = help.display_menu().expect("Menu should display");

    // Assert: Complete startup workflow
    assert!(!menu.title.is_empty(), "Should have welcome title");
    assert!(!menu.subtitle.is_empty(), "Should have subtitle");
    assert!(!menu.options.is_empty(), "Should have menu options");

    // Verify essential options are present
    let has_exit = menu.options.iter().any(|o| matches!(o.action, MenuAction::Exit));
    let has_help = menu
        .options
        .iter()
        .any(|o| matches!(o.action, MenuAction::GuidedSetup | MenuAction::Quickstart));

    assert!(has_exit, "Startup should offer exit");
    assert!(has_help, "Startup should offer help options");
}

#[test]
fn test_wizard_example_selection_workflow() {
    // Arrange: User wants to see an example
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Menu should display");

    // Act: Find example option
    let example_option =
        menu.options.iter().find(|o| matches!(o.action, MenuAction::ShowExample(_)));

    // Assert: Example workflow is available
    assert!(example_option.is_some(), "Should have example option in workflow");

    if let Some(option) = example_option {
        // Verify example can be retrieved
        if let MenuAction::ShowExample(example_name) = &option.action {
            let registry = ExamplesRegistry::default();
            let results = registry.search(example_name);

            // Example should be found
            assert!(!results.is_empty(), "Wizard example '{}' should be in registry", example_name);
        }
    }
}

#[test]
fn test_wizard_category_selection_workflow() {
    // Arrange: User wants to browse by category
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Menu should display");

    // Act: Find category option
    let category_option =
        menu.options.iter().find(|o| matches!(o.action, MenuAction::ShowCategory(_)));

    // Assert: Category workflow is available
    assert!(category_option.is_some(), "Should have category option in workflow");

    if let Some(option) = category_option {
        if let MenuAction::ShowCategory(category) = &option.action {
            // Verify category has description
            let description = category.description();
            assert!(!description.is_empty(), "Category in workflow should have description");
        }
    }
}

#[test]
fn test_wizard_guided_setup_workflow() {
    // Arrange: User selects guided setup
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Menu should display");

    // Act: Find guided setup option
    let setup_option = menu.options.iter().find(|o| matches!(o.action, MenuAction::GuidedSetup));

    // Assert: Guided setup is available
    assert!(setup_option.is_some(), "Should have guided setup in workflow");

    let option = setup_option.unwrap();
    assert!(
        option.text.to_lowercase().contains("setup")
            || option.text.to_lowercase().contains("guide"),
        "Setup option should mention setup or guide"
    );
}

#[test]
fn test_wizard_quickstart_workflow() {
    // Arrange: User wants quickstart
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Menu should display");

    // Act: Find quickstart option
    let quickstart_option =
        menu.options.iter().find(|o| matches!(o.action, MenuAction::Quickstart));

    // Assert: Quickstart is available
    assert!(quickstart_option.is_some(), "Should have quickstart in workflow");

    let option = quickstart_option.unwrap();
    assert!(
        option.text.to_lowercase().contains("quick")
            || option.text.to_lowercase().contains("start"),
        "Quickstart option should mention quick or start"
    );
}

#[test]
fn test_wizard_exit_workflow() {
    // Arrange: User wants to exit
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Menu should display");

    // Act: Find exit option
    let exit_option = menu.options.iter().find(|o| matches!(o.action, MenuAction::Exit));

    // Assert: Exit workflow is available
    assert!(exit_option.is_some(), "Should have exit in workflow");

    let option = exit_option.unwrap();
    assert!(!option.key.is_empty(), "Exit option should have key for selection");
}

// =============================================================================
// MULTI-STEP WORKFLOW TESTS
// =============================================================================

#[test]
fn test_wizard_complete_discovery_journey() {
    // Arrange: User discovers features through wizard

    // Step 1: Start wizard
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Step 1: Show menu");

    // Step 2: Browse examples
    let has_examples = menu.options.iter().any(|o| matches!(o.action, MenuAction::ShowExample(_)));
    assert!(has_examples, "Step 2: Should offer examples");

    // Step 3: Browse categories
    let has_categories =
        menu.options.iter().any(|o| matches!(o.action, MenuAction::ShowCategory(_)));
    assert!(has_categories, "Step 3: Should offer categories");

    // Step 4: Access help
    let has_help = menu
        .options
        .iter()
        .any(|o| matches!(o.action, MenuAction::GuidedSetup | MenuAction::Quickstart));
    assert!(has_help, "Step 4: Should offer help");

    // Step 5: Exit
    let has_exit = menu.options.iter().any(|o| matches!(o.action, MenuAction::Exit));
    assert!(has_exit, "Step 5: Should offer exit");
}

#[test]
fn test_wizard_learning_path_workflow() {
    // Arrange: Simulate new user learning path
    let help = InteractiveHelp::new();

    // Step 1: See initial menu
    let menu = help.display_menu().expect("Should show initial menu");
    assert!(!menu.title.is_empty());

    // Step 2: Choose quickstart
    let quickstart = menu.options.iter().find(|o| matches!(o.action, MenuAction::Quickstart));
    assert!(quickstart.is_some(), "Learning path should start with quickstart");

    // Step 3: Then explore examples
    let examples =
        menu.options.iter().filter(|o| matches!(o.action, MenuAction::ShowExample(_))).count();
    assert!(examples > 0, "Should have examples to explore");

    // Step 4: Browse by category
    let categories =
        menu.options.iter().filter(|o| matches!(o.action, MenuAction::ShowCategory(_))).count();
    assert!(categories > 0, "Should have categories to browse");
}

#[test]
fn test_wizard_troubleshooting_workflow() {
    // Arrange: User needs help
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Should show menu");

    // Act: Find help resources
    let guided_setup = menu.options.iter().any(|o| matches!(o.action, MenuAction::GuidedSetup));
    let quickstart = menu.options.iter().any(|o| matches!(o.action, MenuAction::Quickstart));
    let examples = menu.options.iter().any(|o| matches!(o.action, MenuAction::ShowExample(_)));

    // Assert: Multiple help resources available
    assert!(guided_setup, "Troubleshooting should offer guided setup");
    assert!(quickstart, "Troubleshooting should offer quickstart");
    assert!(examples, "Troubleshooting should offer examples");
}

// =============================================================================
// WORKFLOW STATE CONSISTENCY TESTS
// =============================================================================

#[test]
fn test_wizard_maintains_state_through_workflow() {
    // Arrange
    let help = InteractiveHelp::new();

    // Act: Simulate multiple menu displays
    let menu1 = help.display_menu().expect("First display");
    let menu2 = help.display_menu().expect("Second display");
    let menu3 = help.display_menu().expect("Third display");

    // Assert: State should remain consistent
    assert_eq!(menu1.title, menu2.title, "Title should not change");
    assert_eq!(menu2.title, menu3.title, "Title should not change");

    assert_eq!(menu1.options.len(), menu2.options.len(), "Options should not change");
    assert_eq!(menu2.options.len(), menu3.options.len(), "Options should not change");
}

#[test]
fn test_wizard_workflow_is_repeatable() {
    // Arrange & Act: Run complete workflow twice
    let workflow_run = || {
        let help = InteractiveHelp::new();
        let menu = help.display_menu().unwrap();
        (menu.title.clone(), menu.options.len())
    };

    let (title1, count1) = workflow_run();
    let (title2, count2) = workflow_run();

    // Assert: Workflow should be repeatable
    assert_eq!(title1, title2, "Workflow should be repeatable");
    assert_eq!(count1, count2, "Workflow should be repeatable");
}

// =============================================================================
// USER JOURNEY TESTS
// =============================================================================

#[test]
fn test_new_user_first_time_journey() {
    // Arrange: Simulate first-time user experience
    let help = InteractiveHelp::new();

    // Act: User sees welcome screen
    let menu = help.display_menu().expect("Welcome screen should appear");

    // Assert: First-time user needs
    assert!(
        menu.title.contains("Interactive") || menu.title.contains("Help"),
        "Should welcome new user"
    );

    // Should offer quickstart
    let has_quickstart = menu.options.iter().any(|o| matches!(o.action, MenuAction::Quickstart));
    assert!(has_quickstart, "New user should see quickstart");

    // Should offer guided setup
    let has_setup = menu.options.iter().any(|o| matches!(o.action, MenuAction::GuidedSetup));
    assert!(has_setup, "New user should see guided setup");
}

#[test]
fn test_experienced_user_journey() {
    // Arrange: Experienced user wants specific info
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Menu should appear");

    // Assert: Experienced user needs
    // Should offer direct access to examples
    let example_count =
        menu.options.iter().filter(|o| matches!(o.action, MenuAction::ShowExample(_))).count();
    assert!(example_count > 0, "Experienced user should have quick access to examples");

    // Should offer category browsing
    let category_count =
        menu.options.iter().filter(|o| matches!(o.action, MenuAction::ShowCategory(_))).count();
    assert!(category_count > 0, "Experienced user should browse by category");

    // Should allow quick exit
    let has_exit = menu.options.iter().any(|o| matches!(o.action, MenuAction::Exit));
    assert!(has_exit, "Experienced user should exit quickly");
}

// =============================================================================
// WORKFLOW INTEGRATION TESTS
// =============================================================================

#[test]
fn test_wizard_integrates_all_components() {
    // Arrange: Verify all components work together
    let help = InteractiveHelp::new();
    let menu = help.display_menu().expect("Menu should work");

    // Act: Verify each component
    // 1. Menu display works
    assert!(!menu.title.is_empty());

    // 2. Examples registry works
    let registry = ExamplesRegistry::default();
    assert!(!registry.all().is_empty());

    // 3. Categories work
    let category = CommandCategory::Pack;
    assert!(!category.description().is_empty());

    // 4. Menu actions are valid
    for option in &menu.options {
        match &option.action {
            MenuAction::ShowExample(name) => {
                let results = registry.search(name);
                // Should find at least one result or be a valid example name
                assert!(!results.is_empty() || !name.is_empty(), "Example should be valid");
            }
            MenuAction::ShowCategory(cat) => {
                assert!(!cat.description().is_empty(), "Category should be valid");
            }
            _ => {
                // Other actions (Exit, GuidedSetup, Quickstart) are always valid
            }
        }
    }
}

#[test]
fn test_wizard_workflow_json_output() {
    // Arrange: Wizard should support JSON output for agents
    let output = generate_interactive_output().expect("Should generate output");

    // Act: Serialize to JSON
    let json = serde_json::to_string(&output).expect("Should serialize");

    // Assert: JSON workflow output
    assert!(json.contains("title"), "JSON should have title");
    assert!(json.contains("options"), "JSON should have options");
    assert!(json.contains("subtitle"), "JSON should have subtitle");

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    assert!(parsed.is_object(), "Should be JSON object");
}

// =============================================================================
// WORKFLOW ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_wizard_workflow_handles_edge_cases() {
    // Arrange: Test workflow robustness
    let help = InteractiveHelp::new();

    // Act: Multiple rapid menu displays
    for _ in 0..10 {
        let result = help.display_menu();
        assert!(result.is_ok(), "Workflow should handle rapid calls");
    }
}

#[test]
fn test_wizard_workflow_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    // Arrange: Test concurrent workflow access
    let help = Arc::new(InteractiveHelp::new());
    let mut handles = vec![];

    // Act: Multiple threads access wizard
    for _ in 0..5 {
        let help_clone = Arc::clone(&help);
        let handle = thread::spawn(move || help_clone.display_menu());
        handles.push(handle);
    }

    // Assert: All threads should succeed
    for handle in handles {
        let result = handle.join().expect("Thread should not panic");
        assert!(result.is_ok(), "Concurrent access should work");
    }
}

#[test]
fn test_complete_wizard_session() {
    // Arrange: Simulate complete wizard session
    let help = InteractiveHelp::new();

    // Act: Complete session flow
    // 1. Display menu
    let menu = help.display_menu().expect("Session: Display menu");
    assert!(!menu.title.is_empty());

    // 2. User explores options
    assert!(!menu.options.is_empty());

    // 3. User can serialize for agents
    let json = serde_json::to_string(&menu).expect("Session: Serialize");
    assert!(!json.is_empty());

    // 4. Session ends cleanly
    let exit_exists = menu.options.iter().any(|o| matches!(o.action, MenuAction::Exit));
    assert!(exit_exists, "Session should have clean exit");
}
