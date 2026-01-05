//! Integration tests for Federated Semantic Network macros
//!
//! These tests verify the behavior of federation macros using Chicago TDD:
//! - State-based testing with real collaborators
//! - AAA (Arrange-Act-Assert) pattern
//! - Observable outputs and state changes

use std::path::PathBuf;

/// Test capability advertisement macro parsing
///
/// Chicago TDD: State-based test verifying macro argument parsing
/// AAA Pattern:
/// - Arrange: N/A (compile-time test)
/// - Act: Macro expansion during compilation
/// - Assert: Compilation succeeds with correct structure
#[test]
fn test_advertise_capability_macro_compiles() {
    // This test verifies that the macro compiles correctly
    // The actual macro usage would be in a separate test crate
    // For now, we verify the parsing logic directly

    use clap_noun_verb_macros::macros::federated_network::parse_capability_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        capability_id = "test-capability",
        description = "Test capability description",
        inputs = ["input1:string", "input2:int"],
        outputs = ["output1:json"]
    };

    // Act
    let result = parse_capability_config(args);

    // Assert
    assert!(result.is_ok(), "Capability config parsing should succeed");
    let config = result.unwrap();
    assert_eq!(config.capability_id, "test-capability");
    assert_eq!(config.description, "Test capability description");
    assert_eq!(config.inputs.len(), 2);
    assert_eq!(config.outputs.len(), 1);
}

/// Test federated macro parsing
///
/// Chicago TDD: Verifies federated macro configuration parsing
/// AAA Pattern:
/// - Arrange: Create mock macro arguments
/// - Act: Parse configuration
/// - Assert: Verify parsed values match expectations
#[test]
fn test_federated_macro_parsing() {
    use clap_noun_verb_macros::macros::federated_network::parse_federated_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        discovery_url = "https://example.com/discovery",
        identity = "test-cli-v1.0",
        trust_anchor = "./certs/root.pem"
    };

    // Act
    let result = parse_federated_config(args);

    // Assert
    assert!(result.is_ok(), "Federated config parsing should succeed");
    let config = result.unwrap();
    assert_eq!(config.discovery_url, "https://example.com/discovery");
    assert_eq!(config.identity, "test-cli-v1.0");
    assert_eq!(config.trust_anchor, "./certs/root.pem");
}

/// Test remote invoke macro parsing
///
/// Chicago TDD: Verifies remote invocation configuration
/// AAA Pattern:
/// - Arrange: Create remote invoke arguments
/// - Act: Parse configuration
/// - Assert: Verify target, capability, and timeout are correct
#[test]
fn test_remote_invoke_macro_parsing() {
    use clap_noun_verb_macros::macros::federated_network::parse_remote_invoke_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        target = "remote-cli-v2.0",
        capability = "process-data",
        timeout_ms = 10000
    };

    // Act
    let result = parse_remote_invoke_config(args);

    // Assert
    assert!(result.is_ok(), "Remote invoke config parsing should succeed");
    let config = result.unwrap();
    assert_eq!(config.target, "remote-cli-v2.0");
    assert_eq!(config.capability, "process-data");
    assert_eq!(config.timeout_ms, 10000);
}

/// Test remote invoke default timeout
///
/// Chicago TDD: Verifies default timeout value when not specified
/// AAA Pattern:
/// - Arrange: Create config without timeout_ms
/// - Act: Parse configuration
/// - Assert: Default timeout is 5000ms
#[test]
fn test_remote_invoke_default_timeout() {
    use clap_noun_verb_macros::macros::federated_network::parse_remote_invoke_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        target = "remote-cli",
        capability = "test"
    };

    // Act
    let result = parse_remote_invoke_config(args);

    // Assert
    assert!(result.is_ok(), "Config parsing should succeed");
    let config = result.unwrap();
    assert_eq!(config.timeout_ms, 5000, "Default timeout should be 5000ms");
}

/// Test capability config with missing required fields
///
/// Chicago TDD: Verifies error handling for invalid configuration
/// AAA Pattern:
/// - Arrange: Create config missing required capability_id
/// - Act: Attempt to parse configuration
/// - Assert: Parsing fails with appropriate error
#[test]
fn test_capability_config_missing_required_field() {
    use clap_noun_verb_macros::macros::federated_network::parse_capability_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        description = "Test capability",
        inputs = ["input1:string"],
        outputs = ["output1:json"]
    };

    // Act
    let result = parse_capability_config(args);

    // Assert
    assert!(result.is_err(), "Should fail when capability_id is missing");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("capability_id"),
        "Error should mention missing capability_id"
    );
}

/// Test federated config with missing required fields
///
/// Chicago TDD: Verifies error handling for incomplete federated config
/// AAA Pattern:
/// - Arrange: Create config missing discovery_url
/// - Act: Attempt to parse configuration
/// - Assert: Parsing fails with appropriate error
#[test]
fn test_federated_config_missing_discovery_url() {
    use clap_noun_verb_macros::macros::federated_network::parse_federated_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        identity = "test-cli",
        trust_anchor = "./certs/root.pem"
    };

    // Act
    let result = parse_federated_config(args);

    // Assert
    assert!(result.is_err(), "Should fail when discovery_url is missing");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("discovery_url"),
        "Error should mention missing discovery_url"
    );
}

/// Test remote invoke config with missing required fields
///
/// Chicago TDD: Verifies error handling for incomplete remote invoke config
/// AAA Pattern:
/// - Arrange: Create config missing target
/// - Act: Attempt to parse configuration
/// - Assert: Parsing fails with appropriate error
#[test]
fn test_remote_invoke_config_missing_target() {
    use clap_noun_verb_macros::macros::federated_network::parse_remote_invoke_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        capability = "test-capability",
        timeout_ms = 5000
    };

    // Act
    let result = parse_remote_invoke_config(args);

    // Assert
    assert!(result.is_err(), "Should fail when target is missing");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("target"),
        "Error should mention missing target"
    );
}

/// Test capability config with empty inputs/outputs
///
/// Chicago TDD: Verifies handling of optional arrays
/// AAA Pattern:
/// - Arrange: Create config with empty inputs/outputs arrays
/// - Act: Parse configuration
/// - Assert: Config is valid with empty arrays
#[test]
fn test_capability_config_empty_inputs_outputs() {
    use clap_noun_verb_macros::macros::federated_network::parse_capability_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        capability_id = "test-capability",
        description = "Test",
        inputs = [],
        outputs = []
    };

    // Act
    let result = parse_capability_config(args);

    // Assert
    assert!(result.is_ok(), "Should succeed with empty inputs/outputs");
    let config = result.unwrap();
    assert_eq!(config.inputs.len(), 0, "Inputs should be empty");
    assert_eq!(config.outputs.len(), 0, "Outputs should be empty");
}

/// Test capability config description defaults to empty
///
/// Chicago TDD: Verifies optional description field
/// AAA Pattern:
/// - Arrange: Create config without description
/// - Act: Parse configuration
/// - Assert: Description defaults to empty string
#[test]
fn test_capability_config_optional_description() {
    use clap_noun_verb_macros::macros::federated_network::parse_capability_config;
    use quote::quote;

    // Arrange
    let args = quote! {
        capability_id = "test-capability",
        inputs = ["input:string"],
        outputs = ["output:json"]
    };

    // Act
    let result = parse_capability_config(args);

    // Assert
    assert!(result.is_ok(), "Should succeed without description");
    let config = result.unwrap();
    assert_eq!(config.description, "", "Description should default to empty");
}

/// Benchmark: Macro parsing performance
///
/// Verifies that macro parsing completes within performance SLO (< 10ms)
#[test]
fn bench_macro_parsing_performance() {
    use clap_noun_verb_macros::macros::federated_network::parse_capability_config;
    use quote::quote;
    use std::time::Instant;

    // Arrange
    let args = quote! {
        capability_id = "bench-capability",
        description = "Benchmark test",
        inputs = ["input1:string", "input2:int", "input3:bool"],
        outputs = ["output1:json", "output2:xml"]
    };

    // Act
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let _ = parse_capability_config(args.clone());
    }
    let elapsed = start.elapsed();
    let avg_per_parse = elapsed.as_micros() / iterations;

    // Assert
    assert!(
        avg_per_parse < 10_000,
        "Average parsing time should be < 10ms, got {}μs",
        avg_per_parse
    );
}
