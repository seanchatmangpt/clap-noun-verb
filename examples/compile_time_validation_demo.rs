//! Compile-Time Validation Demonstration
//!
//! This example demonstrates the hyperadvanced compile-time error-proofing
//! implemented in clap-noun-verb v4.0+.
//!
//! To see the validation in action:
//! 1. Uncomment one of the error examples below
//! 2. Run `cargo build --example compile_time_validation_demo`
//! 3. Observe the helpful compile-time error message
//!
//! All errors are caught at COMPILE TIME with ZERO RUNTIME OVERHEAD.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// ============================================================================
// ✅ CORRECT EXAMPLES - These compile successfully
// ============================================================================

#[derive(Serialize, Debug)]
struct ServiceStatus {
    name: String,
    running: bool,
    uptime_seconds: u64,
}

#[derive(Serialize, Debug)]
struct ConfigData {
    port: u16,
    host: String,
}

/// Example 1: Correct - Result<T> where T: Serialize ✅
#[verb("status", "demo")]
fn show_status() -> Result<ServiceStatus> {
    Ok(ServiceStatus { name: "api".to_string(), running: true, uptime_seconds: 3600 })
}

/// Example 2: Correct - Option<T> where T: Serialize ✅
#[verb("config", "demo")]
fn get_config() -> Result<Option<ConfigData>> {
    Ok(Some(ConfigData { port: 8080, host: "localhost".to_string() }))
}

/// Example 3: Correct - Auto-inferred verb name ✅
#[verb]
fn show_health() -> Result<String> {
    Ok("Healthy".to_string())
}

/// Example 4: Correct - With parameters ✅
#[verb("list", "demo")]
fn list_services(limit: Option<usize>) -> Result<Vec<String>> {
    let limit = limit.unwrap_or(10);
    Ok((0..limit).map(|i| format!("service-{}", i)).collect())
}

// ============================================================================
// ❌ ERROR EXAMPLES - Uncomment to see compile-time validation
// ============================================================================

// ---------------------------------------------------------------------------
// GAP 3: Missing Return Type
// ---------------------------------------------------------------------------

/*
/// ❌ ERROR: Function must return a value that implements Serialize
#[verb]
fn missing_return_type() {
    println!("This will fail to compile");
}

// EXPECTED ERROR MESSAGE:
// error: Function 'missing_return_type' must return a value that implements serde::Serialize
//
// Expected return type patterns:
// - Result<T> where T: Serialize
// - Option<T> where T: Serialize
// - T where T: Serialize
//
// Hint: Add a return type like `Result<Status>` where Status derives Serialize
*/

// ---------------------------------------------------------------------------
// GAP 4: Invalid Attribute Syntax - Missing Quotes
// ---------------------------------------------------------------------------

/*
/// ❌ ERROR: Verb name must be a string literal
#[verb(status)]  // Missing quotes around 'status'
fn invalid_syntax_no_quotes() -> Result<String> {
    Ok("".to_string())
}

// EXPECTED ERROR MESSAGE:
// error: Argument 1 in #[verb] must be a string literal for function 'invalid_syntax_no_quotes'
//
// Found: status
// Expected: "status"
//
// Hint: Add double quotes around the identifier
*/

// ---------------------------------------------------------------------------
// GAP 4: Invalid Attribute Syntax - Too Many Arguments
// ---------------------------------------------------------------------------

/*
/// ❌ ERROR: Too many arguments
#[verb("status", "services", "extra")]
fn invalid_syntax_too_many() -> Result<String> {
    Ok("".to_string())
}

// EXPECTED ERROR MESSAGE:
// error: Too many arguments in #[verb] attribute for function 'invalid_syntax_too_many'
//
// Expected: 0, 1, or 2 arguments
// Found: 3 arguments
//
// Valid patterns:
// - #[verb]                    (0 args - auto-infer)
// - #[verb("status")]          (1 arg - verb name)
// - #[verb("status", "noun")] (2 args - verb + noun)
//
// Hint: Remove extra arguments
*/

// ---------------------------------------------------------------------------
// GAP 4: Invalid Attribute Syntax - Complex Expression
// ---------------------------------------------------------------------------

/*
/// ❌ ERROR: Complex expressions not allowed
#[verb(format!("status"))]  // Dynamic expressions not supported
fn invalid_syntax_complex() -> Result<String> {
    Ok("".to_string())
}

// EXPECTED ERROR MESSAGE:
// error: Argument 1 in #[verb] must be a string literal for function 'invalid_syntax_complex'
//
// Found: complex expression
// Expected: a string literal like "status" or "services"
//
// Hint: Use double-quoted string literals only
*/

// ---------------------------------------------------------------------------
// GAP 2: Duplicate Verb Detection
// ---------------------------------------------------------------------------

/*
/// First registration of "duplicate" ✅
#[verb("duplicate", "demo")]
fn first_duplicate() -> Result<String> {
    Ok("First".to_string())
}

/// ❌ ERROR: Duplicate verb registration
#[verb("duplicate", "demo")]  // Same noun+verb combination!
fn second_duplicate() -> Result<String> {
    Ok("Second".to_string())
}

// EXPECTED ERROR MESSAGE:
// error[E0428]: duplicate definitions with name `__VERB_DUPLICATE_CHECK_demo_duplicate_second_duplicate`
//   --> examples/compile_time_validation_demo.rs:XX:YY
//    |
// XX | #[verb("duplicate", "demo")]
//    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//    |
// note: previous definition of the constant `__VERB_DUPLICATE_CHECK_demo_duplicate_first_duplicate` here
//   --> examples/compile_time_validation_demo.rs:XX:YY
//    |
// XX | #[verb("duplicate", "demo")]
//    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
*/

// ---------------------------------------------------------------------------
// GAP 3: Return Type Not Serializable
// ---------------------------------------------------------------------------

/*
// This struct does NOT derive Serialize
struct NotSerializable {
    data: String,
}

/// ❌ ERROR: Return type must implement Serialize
#[verb]
fn return_non_serializable() -> Result<NotSerializable> {
    Ok(NotSerializable {
        data: "test".to_string(),
    })
}

// NOTE: This specific error will be caught by the Rust compiler later
// (when trying to call ::from_data() on the non-Serialize type),
// but our validation provides earlier and clearer feedback at the macro level.
//
// EXPECTED ERROR (from our validation):
// The return type has been validated to follow proper patterns.
// If the type doesn't implement Serialize, you'll get a clear error
// when the generated code tries to serialize it.
*/

// ============================================================================
// MAIN FUNCTION - Demonstrates all working examples
// ============================================================================

fn main() -> Result<()> {
    println!("=== Compile-Time Validation Demo ===\n");

    println!("All functions in this example have been validated at COMPILE TIME:");
    println!("  ✅ Return types implement Serialize (Gap 3)");
    println!("  ✅ Attribute syntax is correct (Gap 4)");
    println!("  ✅ No duplicate verb registrations (Gap 2)");
    println!("  ✅ All validations have ZERO runtime overhead\n");

    println!("To see validation in action:");
    println!("  1. Uncomment one of the error examples");
    println!("  2. Run: cargo build --example compile_time_validation_demo");
    println!("  3. Observe the helpful compile-time error message\n");

    println!("Examples of working functions:");
    println!("  - show_status()    → Returns ServiceStatus");
    println!("  - get_config()     → Returns Option<ConfigData>");
    println!("  - show_health()    → Auto-inferred verb name");
    println!("  - list_services()  → With optional parameters\n");

    println!("See docs/COMPILE_TIME_VALIDATION.md for complete documentation.");

    Ok(())
}
