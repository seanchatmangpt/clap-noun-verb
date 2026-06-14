// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests for compile-time validation (Poka-Yoke error-proofing)
//!
//! These tests demonstrate the four gaps that are now closed:
//! 1. Forgotten #[verb] detection
//! 2. Duplicate verb detection
//! 3. Return type must implement Serialize
//! 4. Enhanced attribute syntax validation
//!
//! Note: Most of these are compile-fail tests that should be run with trybuild.
//! For now, we demonstrate correct usage that SHOULD compile.

mod common;

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// ============================================================================
// GAP 3: Return Type Validation - CORRECT EXAMPLES (should compile)
// ============================================================================

#[derive(Serialize, Debug)]
struct ValidStatus {
    running: bool,
    uptime: u64,
}

/// Example 1: Result<T> where T: Serialize ✅
#[verb("status1", "test")]
fn correct_result_type() -> Result<ValidStatus> {
    Ok(ValidStatus { running: true, uptime: 3600 })
}

/// Example 2: Option<T> where T: Serialize ✅
#[verb("status2", "test")]
fn correct_option_type() -> Result<Option<ValidStatus>> {
    Ok(Some(ValidStatus { running: true, uptime: 3600 }))
}

/// Example 3: Direct type that implements Serialize ✅
#[verb("status3", "test")]
fn correct_direct_type() -> Result<String> {
    Ok("Running".to_string())
}

// ============================================================================
// GAP 4: Attribute Syntax Validation - CORRECT EXAMPLES
// ============================================================================

/// Example 4: Auto-inferred verb name ✅
#[verb]
fn show_health() -> Result<String> {
    Ok("Healthy".to_string())
}

/// Example 5: Explicit verb name ✅
#[verb("status")]
fn get_service_status() -> Result<String> {
    Ok("Active".to_string())
}

/// Example 6: Explicit verb + noun ✅
#[verb("list", "services")]
fn list_all_services() -> Result<Vec<String>> {
    Ok(vec!["api".to_string(), "worker".to_string()])
}

/// Example 7: With parameters (arg attributes parsed by #[verb] macro) ✅
#[verb("config")]
fn set_config(port: u16, host: Option<String>) -> Result<String> {
    Ok(format!("Config: {}:{}", host.unwrap_or_else(|| "localhost".to_string()), port))
}

// ============================================================================
// GAP 2: Duplicate Verb Detection
// ============================================================================

/// First registration of "unique_verb" for noun "test" ✅
#[verb("unique_verb1", "test")]
fn first_unique_verb() -> Result<String> {
    Ok("First".to_string())
}

/// Different verb name, no conflict ✅
#[verb("unique_verb2", "test")]
fn second_unique_verb() -> Result<String> {
    Ok("Second".to_string())
}

// Note: If we uncommented this, it would cause a compile error:
// #[verb("unique_verb1", "test")]  // ❌ Duplicate!
// fn duplicate_unique_verb() -> Result<String> {
//     Ok("Duplicate".to_string())
// }
// Error: duplicate definitions with name `__VERB_DUPLICATE_CHECK_test_unique_verb1_...`

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_prelude::*;

    #[test]
    fn test_correct_types_compile() {
        // Verify that each verb function above is callable and returns the declared type.
        // Compilation alone would not catch a function that silently panics or diverges.

        let r1: Result<ValidStatus> = correct_result_type();
        assert!(r1.is_ok(), "correct_result_type() must return Ok");
        let s1 = r1.test_unwrap();
        assert!(s1.running, "running must be true");
        assert_eq!(s1.uptime, 3600, "uptime must be 3600");

        let r2: Result<Option<ValidStatus>> = correct_option_type();
        assert!(r2.is_ok(), "correct_option_type() must return Ok");
        let s2 = r2.test_unwrap();
        assert!(s2.is_some(), "Option must be Some");
        assert_eq!(s2.test_unwrap().uptime, 3600, "inner uptime must be 3600");

        let r3: Result<String> = correct_direct_type();
        assert!(r3.is_ok(), "correct_direct_type() must return Ok");
        assert_eq!(r3.test_unwrap(), "Running");

        let r4: Result<String> = show_health();
        assert!(r4.is_ok(), "show_health() must return Ok");
        assert_eq!(r4.test_unwrap(), "Healthy");

        let r5: Result<String> = get_service_status();
        assert!(r5.is_ok(), "get_service_status() must return Ok");
        assert_eq!(r5.test_unwrap(), "Active");

        let r6: Result<Vec<String>> = list_all_services();
        assert!(r6.is_ok(), "list_all_services() must return Ok");
        let services = r6.test_unwrap();
        assert_eq!(services.len(), 2, "must return exactly two services");
        assert!(services.contains(&"api".to_string()));
        assert!(services.contains(&"worker".to_string()));

        let r7: Result<String> = set_config(8080, Some("127.0.0.1".to_string()));
        assert!(r7.is_ok(), "set_config() must return Ok");
        let cfg = r7.test_unwrap();
        assert!(cfg.contains("8080"), "config string must contain the port: {cfg}");
        assert!(cfg.contains("127.0.0.1"), "config string must contain the host: {cfg}");

        let r8: Result<String> = first_unique_verb();
        assert!(r8.is_ok(), "first_unique_verb() must return Ok");
        assert_eq!(r8.test_unwrap(), "First");

        let r9: Result<String> = second_unique_verb();
        assert!(r9.is_ok(), "second_unique_verb() must return Ok");
        assert_eq!(r9.test_unwrap(), "Second");
    }

    #[test]
    fn test_return_types_are_serializable() {
        // Verify that our types actually implement Serialize
        let status = ValidStatus { running: true, uptime: 100 };
        let json = serde_json::to_string(&status).test_unwrap();
        assert!(json.contains("running"));
        assert!(json.contains("uptime"));
    }
}

// ============================================================================
// Compile-Fail Examples (should be tested with trybuild)
// ============================================================================

// The following examples are commented out because they SHOULD fail to compile.
// In a real test suite, these would be in separate files tested with trybuild.

/*
// GAP 3: Missing return type ❌
#[verb]
fn missing_return_type() {
    println!("No return");
}
// Expected error:
// Function 'missing_return_type' must return a value that implements serde::Serialize
// Hint: Add a return type like `Result<Status>` where Status derives Serialize

// GAP 4: Invalid syntax - missing quotes ❌
#[verb(status)]  // Should be #[verb("status")]
fn invalid_syntax_no_quotes() -> Result<String> {
    Ok("".to_string())
}
// Expected error:
// Argument 1 in #[verb] must be a string literal
// Found: status
// Expected: "status"
// Hint: Add double quotes around the identifier

// GAP 4: Too many arguments ❌
#[verb("status", "services", "extra")]
fn invalid_syntax_too_many() -> Result<String> {
    Ok("".to_string())
}
// Expected error:
// Too many arguments in #[verb] attribute
// Expected: 0, 1, or 2 arguments
// Found: 3 arguments
// Hint: Remove extra arguments

// GAP 4: Invalid #[arg] syntax ❌
#[verb]
fn invalid_arg_syntax(
    #[arg(port)]  // Missing = and value
    port: u16,
) -> Result<String> {
    Ok("".to_string())
}
// Expected error:
// Invalid #[arg] attribute syntax
// Expected patterns:
// - #[arg(short = 'v')]
// - #[arg(env = "PORT", default_value = "8080")]
// Hint: Use key = value pairs with proper quoting

// GAP 2: Duplicate verb ❌
#[verb("duplicate", "test")]
fn first_duplicate() -> Result<String> {
    Ok("First".to_string())
}

#[verb("duplicate", "test")]
fn second_duplicate() -> Result<String> {
    Ok("Second".to_string())
}
// Expected error:
// duplicate definitions with name `__VERB_DUPLICATE_CHECK_test_duplicate_...`
*/
