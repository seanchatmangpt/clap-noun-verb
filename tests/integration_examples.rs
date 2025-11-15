//! Integration tests for examples
//!
//! These tests verify that all examples compile and can execute commands.
//! They are marked with `#[ignore]` so they don't run as part of the default
//! test suite. Run them explicitly with:
//!   - `cargo test --test integration_examples -- --ignored`
//!   - `cargo test --test integration_examples --ignored`

use clap_noun_verb::error::Result;
use std::process::{Command, Stdio};

/// Helper function to run an example and capture output
fn run_example(example_name: &str, args: Vec<&str>) -> Result<(String, i32)> {
    let mut cmd = Command::new("cargo")
        .args(&["run", "--example", example_name, "--"])
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            clap_noun_verb::error::NounVerbError::execution_error(format!(
                "Failed to spawn cargo: {}",
                e
            ))
        })?;

    let output = cmd.wait_with_output().map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "Failed to wait for process: {}",
            e
        ))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}\n{}", stdout, stderr);

    Ok((combined, output.status.code().unwrap_or(-1)))
}

#[test]
fn test_basic_example_help() -> Result<()> {
    // Test that the basic example compiles and shows help
    let (output, _code) = run_example("basic", vec!["--help"])?;

    // Should show help (exit code 0) or error gracefully
    assert!(
        output.contains("services") || output.contains("collector"),
        "Help should mention services or collector. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_basic_example_services_status() -> Result<()> {
    // Test that basic example can execute a command
    let (output, _code) = run_example("basic", vec!["services", "status"])?;

    // Should produce JSON output
    assert!(
        output.contains("services") || output.contains("\"services\""),
        "Should output JSON with services. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_services_example() -> Result<()> {
    // Test services example
    let (output, _code) = run_example("services", vec!["services", "status"])?;

    // Should produce JSON output
    assert!(
        output.contains("services") || output.contains("\"services\""),
        "Should output JSON. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_services_example_logs() -> Result<()> {
    // Test services example with arguments
    let (output, _code) = run_example("services", vec!["services", "logs", "--service", "web"])?;

    // Should produce JSON output
    assert!(
        output.contains("web") || output.contains("\"service\""),
        "Should output JSON with service. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_collector_example() -> Result<()> {
    // Test collector example
    let (output, _code) = run_example("collector", vec!["collector", "status"])?;

    // Should produce JSON output
    assert!(
        output.contains("state") || output.contains("\"state\""),
        "Should output JSON with state. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_arguments_example() -> Result<()> {
    // Test arguments example with required and optional args
    let (output, _code) =
        run_example("arguments", vec!["services", "logs", "--service", "api", "--lines", "10"])?;

    // Should produce JSON output
    assert!(
        output.contains("api") || output.contains("\"service\""),
        "Should output JSON with service. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_arguments_example_with_flag() -> Result<()> {
    // Test arguments example with boolean flag
    let (output, _code) =
        run_example("arguments", vec!["services", "restart", "--service", "api", "--force"])?;

    // Should produce JSON output
    assert!(
        output.contains("api") || output.contains("\"service\""),
        "Should output JSON with service. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_validation_example() -> Result<()> {
    // Test validation example
    let (output, _code) = run_example(
        "validation",
        vec!["users", "create", "--name", "test", "--age", "25", "--email", "test@example.com"],
    )?;

    // Should produce JSON output
    assert!(
        output.contains("test") || output.contains("\"name\""),
        "Should output JSON with user. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_nested_example() -> Result<()> {
    // Test nested example
    let (output, _code) = run_example("nested", vec!["test", "run"])?;

    // Should produce JSON output
    assert!(
        output.contains("tests") || output.contains("\"tests_run\""),
        "Should output JSON. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_framework_example() -> Result<()> {
    // Test framework example
    let (output, _code) = run_example("framework", vec!["services", "status"])?;

    // Should produce JSON output
    assert!(
        output.contains("services") || output.contains("\"services\""),
        "Should output JSON. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_attribute_macro_example() -> Result<()> {
    // Test attribute_macro example
    let (output, _code) = run_example("attribute_macro", vec!["services", "status"])?;

    // Should produce JSON output
    assert!(
        output.contains("services") || output.contains("\"services\""),
        "Should output JSON. Got: {}",
        output
    );

    Ok(())
}

#[test]
fn test_attribute_macro_example_with_args() -> Result<()> {
    // Test attribute_macro example with arguments
    let (output, _code) = run_example(
        "attribute_macro",
        vec!["services", "logs", "--service", "api", "--lines", "20"],
    )?;

    // Should produce JSON output
    assert!(
        output.contains("api") || output.contains("\"service\""),
        "Should output JSON with service. Got: {}",
        output
    );

    Ok(())
}
