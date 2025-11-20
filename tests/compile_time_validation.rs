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

    #[test]
    fn test_correct_types_compile() {
        // If this compiles, all the correct examples above are valid
        assert!(true);
    }

    #[test]
    fn test_return_types_are_serializable() {
        // Verify that our types actually implement Serialize
        let status = ValidStatus { running: true, uptime: 100 };
        let json = serde_json::to_string(&status).unwrap();
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
