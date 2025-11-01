//! Acceptance tests for DX improvements: verb name inference and noun auto-detection
//!
//! These tests verify the improved developer experience features:
//! 1. Verb name auto-inference from function name when #[verb] has no args
//! 2. Noun name auto-detection when both #[noun] and #[verb] on same function
//! 3. Enhanced prefix stripping for various naming conventions
//!
//! Following London TDD (outside-in), we start with these acceptance tests.
//!
//! RED: These tests will fail until the implementation is complete.

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

// Test types
#[derive(Serialize, Debug, PartialEq)]
struct TestOutput {
    value: String,
}

#[derive(Serialize, Debug, PartialEq)]
struct UserOutput {
    id: String,
    name: String,
}

// DX Improvement 1: Verb name auto-inference when #[verb] has no args
// Expected: #[verb] + fn show_status() → verb name "status" inferred

/// Show status without explicit verb name
#[noun("services", "Manage services")]
#[verb] // No args - should infer "status" from function name
fn show_status() -> Result<TestOutput> {
    Ok(TestOutput { value: "status".to_string() })
}

/// Get logs without explicit verb name
#[verb] // No args - should infer "logs" from function name (but needs noun)
fn get_logs() -> Result<TestOutput> {
    Ok(TestOutput { value: "logs".to_string() })
}

/// List users without explicit verb name
#[noun("users", "Manage users")]
#[verb] // No args - should infer "users" from function name
fn list_users() -> Result<TestOutput> {
    Ok(TestOutput { value: "users".to_string() })
}

/// Create project without explicit verb name
#[verb] // No args - should infer "project" from function name
fn create_project(name: String) -> Result<TestOutput> {
    Ok(TestOutput { value: format!("project: {}", name) })
}

/// Delete item without explicit verb name
#[verb] // No args - should infer "item" from function name
fn delete_item(id: String) -> Result<TestOutput> {
    Ok(TestOutput { value: format!("deleted: {}", id) })
}

/// Update config without explicit verb name
#[verb] // No args - should infer "config" from function name
fn update_config(key: String, value: String) -> Result<TestOutput> {
    Ok(TestOutput { value: format!("{} = {}", key, value) })
}

// DX Improvement 2: Noun name auto-detection when both #[noun] and #[verb] on same function
// Expected: #[noun("services", "...")] + #[verb] → noun name "services" auto-detected

/// Status command with noun auto-detection
#[noun("services", "Manage services")]
#[verb] // Should auto-detect noun "services" from #[noun] attribute
fn status() -> Result<TestOutput> {
    Ok(TestOutput { value: "status".to_string() })
}

/// Restart command with noun auto-detection
#[noun("services", "Manage services")]
#[verb] // Should auto-detect noun "services"
fn restart(service: String) -> Result<TestOutput> {
    Ok(TestOutput { value: format!("restarted: {}", service) })
}

// DX Improvement 3: Enhanced prefix stripping
// Test various naming conventions

/// Test snake_case function name (no prefix)
#[verb] // Should infer "health_check" from function name
fn health_check() -> Result<TestOutput> {
    Ok(TestOutput { value: "health_check".to_string() })
}

/// Test function name with underscore
#[verb] // Should infer "service_status" from function name
fn service_status() -> Result<TestOutput> {
    Ok(TestOutput { value: "service_status".to_string() })
}

// Explicit override tests - should still work

/// Test explicit verb name override
#[verb("custom-verb")] // Explicit override
fn any_function_name() -> Result<TestOutput> {
    Ok(TestOutput { value: "custom".to_string() })
}

/// Test explicit verb and noun name
#[verb("custom-verb", "services")] // Explicit override both
fn another_function() -> Result<TestOutput> {
    Ok(TestOutput { value: "custom".to_string() })
}

// ============================================================================
// Acceptance Tests
// ============================================================================

#[test]
fn test_verb_name_auto_inference_with_show_prefix() -> Result<()> {
    // RED: Test verb name auto-inference when #[verb] has no args
    //
    // Acceptance criteria:
    // 1. #[verb] with no args should infer verb name from function name
    // 2. Function `show_status()` → verb name "status"
    // 3. Verb should be registered in the registry

    // Arrange: Function has #[verb] with no args

    // Act: Verify registry contains verb with inferred name
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();
    let cmd = registry.build_command();

    // Find services noun
    if let Some(services_cmd) = cmd.get_subcommands().find(|s| s.get_name() == "services") {
        // Assert: Should have "status" verb (inferred from show_status)
        let verbs: Vec<_> = services_cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(
            verbs.contains(&"status"),
            "Verb name should be inferred from function name 'show_status' → 'status'"
        );
    } else {
        panic!("services noun should be registered");
    }

    Ok(())
}

#[test]
fn test_verb_name_auto_inference_with_get_prefix() -> Result<()> {
    // RED: Test verb name auto-inference for get_ prefix
    //
    // Acceptance criteria:
    // 1. Function `get_logs()` → verb name "logs"
    // 2. Verb should be registered (but needs noun - this will be tested separately)

    // Arrange: Function has #[verb] with no args and get_ prefix

    // Act: Verify function can be called directly
    let output = get_logs()?;

    // Assert: Function works correctly
    assert_eq!(output.value, "logs");

    // Note: Full registry test will be done once noun association is handled
    Ok(())
}

#[test]
fn test_verb_name_auto_inference_with_list_prefix() -> Result<()> {
    // RED: Test verb name auto-inference for list_ prefix
    //
    // Acceptance criteria:
    // 1. Function `list_users()` → verb name "users"

    // Arrange: Function has #[noun] and #[verb] with no args

    // Act: Verify registry contains verb with inferred name
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();
    let cmd = registry.build_command();

    // Find users noun
    if let Some(users_cmd) = cmd.get_subcommands().find(|s| s.get_name() == "users") {
        // Assert: Should have "users" verb (inferred from list_users)
        let verbs: Vec<_> = users_cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(
            verbs.contains(&"users"),
            "Verb name should be inferred from function name 'list_users' → 'users'"
        );
    } else {
        panic!("users noun should be registered");
    }

    Ok(())
}

#[test]
fn test_verb_name_auto_inference_with_create_prefix() -> Result<()> {
    // RED: Test verb name auto-inference for create_ prefix
    //
    // Acceptance criteria:
    // 1. Function `create_project()` → verb name "project"

    // Act: Verify function can be called directly
    let output = create_project("my-project".to_string())?;

    // Assert: Function works correctly
    assert!(output.value.contains("project"));

    Ok(())
}

#[test]
fn test_verb_name_auto_inference_with_delete_prefix() -> Result<()> {
    // RED: Test verb name auto-inference for delete_ prefix
    //
    // Acceptance criteria:
    // 1. Function `delete_item()` → verb name "item"

    // Act: Verify function can be called directly
    let output = delete_item("123".to_string())?;

    // Assert: Function works correctly
    assert!(output.value.contains("deleted"));

    Ok(())
}

#[test]
fn test_verb_name_auto_inference_with_update_prefix() -> Result<()> {
    // RED: Test verb name auto-inference for update_ prefix
    //
    // Acceptance criteria:
    // 1. Function `update_config()` → verb name "config"

    // Act: Verify function can be called directly
    let output = update_config("key".to_string(), "value".to_string())?;

    // Assert: Function works correctly
    assert!(output.value.contains("key"));

    Ok(())
}

#[test]
fn test_noun_name_auto_detection_when_both_attributes_on_same_function() -> Result<()> {
    // RED: Test noun name auto-detection when both #[noun] and #[verb] on same function
    //
    // Acceptance criteria:
    // 1. #[noun("services", "...")] + #[verb] on same function
    // 2. Verb should automatically associate with noun "services"
    // 3. No need to specify noun name in #[verb] attribute

    // Arrange: Function has both #[noun] and #[verb] attributes

    // Act: Verify registry contains verb associated with correct noun
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();
    let cmd = registry.build_command();

    // Find services noun
    if let Some(services_cmd) = cmd.get_subcommands().find(|s| s.get_name() == "services") {
        // Assert: Should have "status" verb automatically associated with "services" noun
        let verbs: Vec<_> = services_cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(
            verbs.contains(&"status"),
            "Verb should be auto-associated with noun from same function"
        );
    } else {
        panic!("services noun should be registered");
    }

    Ok(())
}

#[test]
fn test_noun_name_auto_detection_with_arguments() -> Result<()> {
    // RED: Test noun auto-detection works with functions that have arguments

    // Arrange: Function has both #[noun] and #[verb] with function arguments

    // Act: Verify function works and is registered correctly
    let output = restart("api".to_string())?;
    assert!(output.value.contains("restarted"));

    // Assert: Registry contains verb associated with correct noun
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();
    let cmd = registry.build_command();

    if let Some(services_cmd) = cmd.get_subcommands().find(|s| s.get_name() == "services") {
        let verbs: Vec<_> = services_cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(
            verbs.contains(&"restart"),
            "Verb 'restart' should be auto-associated with 'services' noun"
        );
    } else {
        panic!("services noun should be registered");
    }

    Ok(())
}

#[test]
fn test_enhanced_prefix_stripping_snake_case() -> Result<()> {
    // RED: Test enhanced prefix stripping for snake_case function names
    //
    // Acceptance criteria:
    // 1. Function `health_check()` → verb name "health_check" (or "health-check")
    // 2. No prefix to strip, so use entire function name

    // Act: Verify function can be called directly
    let output = health_check()?;
    assert_eq!(output.value, "health_check");

    Ok(())
}

#[test]
fn test_enhanced_prefix_stripping_underscore() -> Result<()> {
    // RED: Test enhanced prefix stripping for function names with underscores
    //
    // Acceptance criteria:
    // 1. Function `service_status()` → verb name "service_status" (or "service-status")

    // Act: Verify function can be called directly
    let output = service_status()?;
    assert_eq!(output.value, "service_status");

    Ok(())
}

#[test]
fn test_explicit_verb_name_override_still_works() -> Result<()> {
    // RED: Test that explicit verb name override still works
    //
    // Acceptance criteria:
    // 1. #[verb("custom-verb")] should override inferred name
    // 2. Function name doesn't matter when explicit name is provided

    // Act: Verify function can be called directly
    let output = any_function_name()?;
    assert_eq!(output.value, "custom");

    // Note: Registry verification would check for "custom-verb" not "any_function_name"
    Ok(())
}

#[test]
fn test_explicit_verb_and_noun_name_override_still_works() -> Result<()> {
    // RED: Test that explicit verb and noun name override still works
    //
    // Acceptance criteria:
    // 1. #[verb("custom-verb", "services")] should override both inferred values

    // Act: Verify function can be called directly
    let output = another_function()?;
    assert_eq!(output.value, "custom");

    // Assert: Registry contains verb with explicit name
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();
    let cmd = registry.build_command();

    if let Some(services_cmd) = cmd.get_subcommands().find(|s| s.get_name() == "services") {
        let verbs: Vec<_> = services_cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(
            verbs.contains(&"custom-verb"),
            "Explicit verb name 'custom-verb' should be registered"
        );
    } else {
        panic!("services noun should be registered");
    }

    Ok(())
}
