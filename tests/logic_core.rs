//! Tests for logic core functions

use clap_noun_verb::error::Result;
use clap_noun_verb::logic::core::{make_core_function, CoreFunctionImpl};

#[test]
fn test_make_core_function() {
    // Arrange - Create a core function from closure
    let core_fn: CoreFunctionImpl<String, String> =
        make_core_function(|input: String| -> Result<String> {
            Ok(format!("Processed: {}", input))
        });

    // Act - Execute the function
    let result = core_fn("test".to_string());

    // Assert - Should execute successfully
    assert!(result.is_ok(), "Core function should not fail: {:?}", result);
    if let Ok(output) = result {
        assert_eq!(output, "Processed: test");
    }
}

#[test]
fn test_core_function_returns_error() {
    // Arrange - Create a core function that returns error
    let core_fn: CoreFunctionImpl<String, String> =
        make_core_function(|_input: String| -> Result<String> {
            Err(clap_noun_verb::error::NounVerbError::execution_error("Test error"))
        });

    // Act - Execute the function
    let result = core_fn("test".to_string());

    // Assert - Should return error
    assert!(result.is_err());
}

#[test]
fn test_core_function_with_different_types() {
    // Arrange - Create a core function with usize input
    let core_fn: CoreFunctionImpl<usize, usize> =
        make_core_function(|input: usize| -> Result<usize> { Ok(input * 2) });

    // Act - Execute the function
    let result = core_fn(21);

    // Assert - Should double the input
    assert!(result.is_ok(), "Core function should not fail: {:?}", result);
    if let Ok(output) = result {
        assert_eq!(output, 42);
    }
}
