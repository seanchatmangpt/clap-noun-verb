//! Comprehensive test suite for ggen CLI error handling
//!
//! Tests validate that errors provide:
//! 1. Clear problem descriptions
//! 2. Actionable recovery steps
//! 3. Appropriate documentation links
//! 4. Correct error categories

// Import the ggen modules
#[path = "../examples/ggen/errors.rs"]
mod errors;

#[path = "../examples/ggen/validators.rs"]
mod validators;

use errors::{ErrorCategory, UserError};
use validators::*;

// ============================================================================
// Error Message Tests
// ============================================================================

#[test]
fn test_invalid_model_name_error_message() {
    let error = errors::invalid_model_name("gpt5");

    assert_eq!(error.category, ErrorCategory::Validation);
    assert!(error.problem.contains("gpt5"));
    assert!(error.problem.contains("not recognized"));
    assert!(error.solution.contains("supported models"));
    assert!(error.learn_more.is_some());
}

#[test]
fn test_missing_api_key_error_message() {
    let error = errors::missing_api_key("openai");

    assert_eq!(error.category, ErrorCategory::Configuration);
    assert!(error.problem.contains("openai"));
    assert!(error.problem.contains("API key"));
    assert!(error.solution.contains("OPENAI_API_KEY"));
    assert!(error.solution.contains("export") || error.solution.contains("config.toml"));
}

#[test]
fn test_invalid_prompt_error_message() {
    let error = errors::invalid_prompt("empty");

    assert_eq!(error.category, ErrorCategory::Validation);
    assert!(error.problem.contains("Invalid prompt"));
    assert!(error.solution.contains("-d") || error.solution.contains("--description"));
    assert!(error.solution.contains("example") || error.solution.contains("Tips"));
}

#[test]
fn test_no_search_results_error_message() {
    let error = errors::no_search_results("nonexistent-package");

    assert_eq!(error.category, ErrorCategory::NotFound);
    assert!(error.problem.contains("No packages found"));
    assert!(error.solution.contains("ggen marketplace"));
}

#[test]
fn test_missing_template_vars_error_message() {
    let required = vec!["name".to_string(), "author".to_string()];
    let provided = vec!["name".to_string()];

    let error = errors::missing_template_vars("test.tmpl", &required, &provided);

    assert_eq!(error.category, ErrorCategory::Validation);
    assert!(error.problem.contains("requires"));
    assert!(error.problem.contains("not provided"));
    assert!(error.solution.contains("author"));
}

#[test]
fn test_api_request_failed_401_error() {
    let error = errors::api_request_failed("OpenAI", 401, "Unauthorized");

    assert_eq!(error.category, ErrorCategory::Network);
    assert!(error.problem.contains("401"));
    assert!(error.solution.contains("API key"));
}

#[test]
fn test_api_request_failed_429_error() {
    let error = errors::api_request_failed("OpenAI", 429, "Rate limit");

    assert_eq!(error.category, ErrorCategory::Network);
    assert!(error.solution.contains("Rate limit") || error.solution.contains("retry"));
}

#[test]
fn test_package_not_found_error() {
    let error = errors::package_not_found("io.ggen.nonexistent");

    assert_eq!(error.category, ErrorCategory::NotFound);
    assert!(error.problem.contains("not found"));
    assert!(error.solution.contains("ggen marketplace search"));
}

// ============================================================================
// Validator Tests
// ============================================================================

#[test]
fn test_validate_model_name_valid() {
    assert!(validate_model_name("gpt-4-turbo").is_ok());
    assert!(validate_model_name("GPT-4-TURBO").is_ok()); // Case insensitive
    assert!(validate_model_name("claude-3-opus").is_ok());
    assert!(validate_model_name("  gpt-3.5-turbo  ").is_ok()); // Trimmed
}

#[test]
fn test_validate_model_name_with_suggestions() {
    // Common typos should suggest corrections
    let result = validate_model_name("gpt4");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.problem.contains("gpt-4-turbo"));
}

#[test]
fn test_validate_model_name_invalid() {
    let result = validate_model_name("invalid-model-xyz");
    assert!(result.is_err());
}

#[test]
fn test_validate_prompt_valid() {
    assert!(validate_prompt("Create a REST API handler").is_ok());
    assert!(validate_prompt("Generate authentication module").is_ok());
}

#[test]
fn test_validate_prompt_empty() {
    assert!(validate_prompt("").is_err());
    assert!(validate_prompt("   ").is_err());
}

#[test]
fn test_validate_prompt_too_short() {
    assert!(validate_prompt("short").is_err());
    assert!(validate_prompt("hi").is_err());
}

#[test]
fn test_validate_prompt_placeholder_text() {
    assert!(validate_prompt("TODO: add description").is_err());
    assert!(validate_prompt("TBD something").is_err());
    assert!(validate_prompt("XXX fix this").is_err());
}

#[test]
fn test_validate_template_vars_valid() {
    let vars = vec!["name=myproject".to_string(), "author=John Doe".to_string()];
    let result = validate_template_vars(&vars);

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].0, "name");
    assert_eq!(parsed[0].1, "myproject");
}

#[test]
fn test_validate_template_vars_invalid_format() {
    let vars = vec!["invalid".to_string()];
    assert!(validate_template_vars(&vars).is_err());

    let vars = vec!["key:value".to_string()]; // Wrong separator
    assert!(validate_template_vars(&vars).is_err());
}

#[test]
fn test_validate_template_vars_empty_key() {
    let vars = vec!["=value".to_string()];
    let result = validate_template_vars(&vars);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.problem.contains("empty key"));
}

#[test]
fn test_validate_package_id_valid() {
    assert!(validate_package_id("io.ggen.rust.axum").is_ok());
    assert!(validate_package_id("com.myorg.templates.api").is_ok());
    assert!(validate_package_id("org.example.my-package").is_ok());
}

#[test]
fn test_validate_package_id_too_short() {
    assert!(validate_package_id("io.ggen").is_err());
    assert!(validate_package_id("single").is_err());
}

#[test]
fn test_validate_package_id_invalid_characters() {
    assert!(validate_package_id("io.ggen.my@package").is_err());
    assert!(validate_package_id("io.ggen.my package").is_err());
    assert!(validate_package_id("io.ggen.my/package").is_err());
}

// ============================================================================
// Error Formatting Tests
// ============================================================================

#[test]
fn test_user_error_format_pretty() {
    let error = UserError::new(ErrorCategory::Validation, "Test problem", "Test solution");

    let formatted = error.format_pretty();
    assert!(formatted.contains("❌ Problem: Test problem"));
    assert!(formatted.contains("💡 Solution: Test solution"));
}

#[test]
fn test_user_error_with_docs_link() {
    let error = UserError::new(ErrorCategory::Validation, "Test problem", "Test solution")
        .with_docs("https://example.com/help");

    let formatted = error.format_pretty();
    assert!(formatted.contains("📚 Learn more: https://example.com/help"));
}

#[test]
fn test_error_categories_distinct() {
    // Ensure all error categories are unique
    let categories = vec![
        ErrorCategory::Validation,
        ErrorCategory::NotFound,
        ErrorCategory::Configuration,
        ErrorCategory::Network,
        ErrorCategory::Internal,
    ];

    for (i, cat1) in categories.iter().enumerate() {
        for (j, cat2) in categories.iter().enumerate() {
            if i == j {
                assert_eq!(cat1, cat2);
            } else {
                assert_ne!(cat1, cat2);
            }
        }
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_validation_error_provides_examples() {
    let error = errors::invalid_prompt("too short");

    // Should provide example usage
    assert!(error.solution.contains("ggen") || error.solution.contains("example"));
}

#[test]
fn test_file_error_provides_commands() {
    let error = errors::file_error("missing.txt", "read", "not found");

    // Should provide shell commands to help debug
    assert!(error.solution.contains("ls") || error.solution.contains("check"));
}

#[test]
fn test_config_error_provides_reset_option() {
    let error = errors::invalid_config("config.toml", "parse error");

    // Should offer config reset option
    assert!(error.solution.contains("reset") || error.solution.contains("example"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_validate_empty_package_id() {
    assert!(validate_package_id("").is_err());
}

#[test]
fn test_validate_whitespace_prompt() {
    assert!(validate_prompt("   \n\t   ").is_err());
}

#[test]
fn test_validate_template_vars_with_spaces() {
    let vars = vec!["key = value".to_string()]; // Spaces around =
    let result = validate_template_vars(&vars);

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed[0].0, "key");
    assert_eq!(parsed[0].1, "value");
}

// ============================================================================
// Performance Tests (Error creation should be fast)
// ============================================================================

#[test]
fn test_error_creation_performance() {
    use std::time::Instant;

    let start = Instant::now();

    for _ in 0..1000 {
        let _error = UserError::new(ErrorCategory::Validation, "Test problem", "Test solution");
    }

    let duration = start.elapsed();

    // Creating 1000 errors should take less than 10ms
    assert!(duration.as_millis() < 10, "Error creation too slow: {:?}", duration);
}

#[test]
fn test_error_formatting_performance() {
    use std::time::Instant;

    let error = UserError::new(ErrorCategory::Validation, "Test problem", "Test solution")
        .with_docs("https://example.com");

    let start = Instant::now();

    for _ in 0..1000 {
        let _formatted = error.format_pretty();
    }

    let duration = start.elapsed();

    // Formatting 1000 errors should take less than 10ms
    assert!(duration.as_millis() < 10, "Error formatting too slow: {:?}", duration);
}
