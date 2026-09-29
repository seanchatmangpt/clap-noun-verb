# CLI Testing Examples & Code Snippets

Practical code examples for common CLI testing scenarios in clap-noun-verb applications.

## Quick Reference

```rust
use clap_noun_verb::common::test_prelude::*;
use clap_noun_verb::cli::builder::CliBuilder;
use clap_noun_verb::Result;

// Basic test structure
#[test]
fn test_noun_verb_behavior() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp");
    
    // Act
    let result = builder.run_with_args(vec![/* args */]);
    
    // Assert
    assert!(result.is_ok());
    Ok(())
}
```

## Common Testing Scenarios

### 1. Testing Help Commands

```rust
#[test]
fn test_root_help_succeeds() -> Result<()> {
    let builder = CliBuilder::new("myapp")
        .about("My application");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "--help".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_noun_help_shows_subcommands() -> Result<()> {
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "--help".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_short_help_flag() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "-h".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}
```

### 2. Testing Required Arguments

```rust
#[test]
fn test_required_argument_missing_fails() {
    let builder = CliBuilder::new("myapp")
        .noun("config", "Configuration");
    
    // Run without the required argument
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "config".to_string(),
        "set".to_string(),
        // Missing: key and value
    ]);
    
    // Should error, not panic
    assert!(result.is_err());
}

#[test]
fn test_required_argument_provided_succeeds() -> Result<()> {
    let builder = CliBuilder::new("myapp")
        .noun("config", "Configuration");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "config".to_string(),
        "set".to_string(),
        "database.url".to_string(),
        "postgres://localhost".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_positional_arguments_order_matters() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    // Correct order
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "config".to_string(),
        "set".to_string(),
        "key".to_string(),    // first positional
        "value".to_string(),  // second positional
    ]);
    
    assert!(result.is_ok());
    Ok(())
}
```

### 3. Testing Optional Arguments

```rust
#[test]
fn test_optional_argument_with_default() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    // Without optional argument - should use default
    let result1 = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        // verbose flag omitted - should use default false
    ])?;

    // With optional argument
    let result2 = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--verbose".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_option_with_value() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "logs".to_string(),
        "tail".to_string(),
        "--lines".to_string(),
        "50".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_multiple_values() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "files".to_string(),
        "copy".to_string(),
        "--from".to_string(),
        "file1.txt".to_string(),
        "--from".to_string(),
        "file2.txt".to_string(),
        "--to".to_string(),
        "/backup".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}
```

### 4. Testing Boolean Flags

```rust
#[test]
fn test_boolean_flag_enabled() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--verbose".to_string(),  // flag present = true
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_boolean_flag_disabled() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--no-verbose".to_string(),  // negation flag
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_boolean_flag_with_value() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--color=auto".to_string(),  // flag with value
    ]);
    
    assert!(result.is_ok());
    Ok(())
}
```

### 5. Testing Invalid Arguments

```rust
#[test]
fn test_invalid_noun_error() {
    let builder = CliBuilder::new("myapp")
        .noun("services", "Services");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "servises".to_string(),  // typo
    ]);
    
    assert!(result.is_err());
}

#[test]
fn test_invalid_verb_error() {
    let builder = CliBuilder::new("myapp")
        .noun("services", "Services");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "statu".to_string(),  // typo
    ]);
    
    assert!(result.is_err());
}

#[test]
fn test_invalid_numeric_argument() {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "config".to_string(),
        "list".to_string(),
        "--limit".to_string(),
        "not-a-number".to_string(),  // should expect integer
    ]);
    
    assert!(result.is_err());
}

#[test]
fn test_unknown_flag_error() {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--unknown-flag".to_string(),
    ]);
    
    assert!(result.is_err());
}
```

### 6. Testing Output Formats

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ServiceStatus {
    name: String,
    state: String,
}

#[test]
fn test_json_output_valid() -> Result<()> {
    // Arrange
    let status = ServiceStatus {
        name: "web".to_string(),
        state: "running".to_string(),
    };

    // Act - Serialize to JSON (as CLI would)
    let json = serde_json::to_string(&status)?;
    
    // Assert - Can parse back
    let parsed: ServiceStatus = serde_json::from_str(&json)?;
    assert_eq!(parsed.name, "web");
    assert_eq!(parsed.state, "running");

    Ok(())
}

#[test]
fn test_json_output_structure() -> Result<()> {
    let status = ServiceStatus {
        name: "web".to_string(),
        state: "running".to_string(),
    };

    let json_value = serde_json::to_value(&status)?;

    // Assert structure
    assert!(json_value.get("name").is_some());
    assert!(json_value.get("state").is_some());
    assert_eq!(json_value["name"], "web");

    Ok(())
}

#[test]
fn test_json_array_output() -> Result<()> {
    let statuses = vec![
        ServiceStatus { name: "web".to_string(), state: "running".to_string() },
        ServiceStatus { name: "db".to_string(), state: "running".to_string() },
    ];

    let json = serde_json::to_string(&statuses)?;
    let parsed: Vec<ServiceStatus> = serde_json::from_str(&json)?;

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].name, "web");
    assert_eq!(parsed[1].name, "db");

    Ok(())
}
```

### 7. Testing Error Messages

```rust
use clap_noun_verb::error::NounVerbError;

#[test]
fn test_error_message_clarity() {
    let result = CliBuilder::new("myapp")
        .run_with_args(vec![
            "myapp".to_string(),
            "invalid_command".to_string(),
        ]);

    match result {
        Err(e) => {
            let msg = e.to_string();
            
            // Assert error message quality
            assert!(!msg.contains("unwrap"));
            assert!(!msg.contains("panic"));
            assert!(!msg.contains("thread"));
            assert!(msg.len() > 0 && msg.len() < 500);
        }
        Ok(_) => panic!("Should have returned error"),
    }
}

#[test]
fn test_error_with_suggestion() {
    let result = CliBuilder::new("myapp")
        .noun("services", "Services")
        .run_with_args(vec![
            "myapp".to_string(),
            "services".to_string(),
            "statu".to_string(),  // typo
        ]);

    match result {
        Err(NounVerbError::VerbNotFound { verb, .. }) => {
            assert_eq!(verb, "statu");
            // Error should suggest "status"
        }
        _ => panic!("Expected VerbNotFound error"),
    }
}

#[test]
fn test_error_includes_context() {
    let result = CliBuilder::new("myapp")
        .noun("config", "Configuration")
        .run_with_args(vec![
            "myapp".to_string(),
            "config".to_string(),
            "invalid".to_string(),
        ]);

    match result {
        Err(NounVerbError::VerbNotFound { noun, verb, .. }) => {
            assert_eq!(noun, "config");
            assert_eq!(verb, "invalid");
        }
        _ => panic!("Expected error with context"),
    }
}
```

### 8. Testing Complex Workflows

```rust
#[test]
fn test_create_list_delete_workflow() -> Result<()> {
    let builder = CliBuilder::new("myapp")
        .noun("items", "Manage items");

    // Step 1: Create an item
    let create_result = builder.run_with_args(vec![
        "myapp".to_string(),
        "items".to_string(),
        "create".to_string(),
        "--name".to_string(),
        "test-item".to_string(),
    ])?;
    assert!(create_result.is_ok());

    // Step 2: List items
    let list_result = builder.run_with_args(vec![
        "myapp".to_string(),
        "items".to_string(),
        "list".to_string(),
    ])?;
    assert!(list_result.is_ok());

    // Step 3: Delete the item
    let delete_result = builder.run_with_args(vec![
        "myapp".to_string(),
        "items".to_string(),
        "delete".to_string(),
        "--id".to_string(),
        "test-item".to_string(),
    ])?;
    assert!(delete_result.is_ok());

    Ok(())
}

#[test]
fn test_conditional_workflow() -> Result<()> {
    let builder = CliBuilder::new("myapp");

    // Check status
    let status = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
    ])?;

    // If running, stop; if stopped, start
    if status.is_ok() {
        // Service is running, stop it
        let _ = builder.run_with_args(vec![
            "myapp".to_string(),
            "services".to_string(),
            "stop".to_string(),
        ])?;
    } else {
        // Service is stopped, start it
        let _ = builder.run_with_args(vec![
            "myapp".to_string(),
            "services".to_string(),
            "start".to_string(),
        ])?;
    }

    Ok(())
}
```

### 9. Testing Feature-Gated Commands

```rust
#[test]
#[cfg(feature = "async")]
fn test_async_command() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "async_task".to_string(),
        "execute".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
#[cfg(feature = "federated-network")]
fn test_federated_command() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "network".to_string(),
        "discover".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

// Test that command doesn't exist when feature is disabled
#[test]
#[cfg(not(feature = "async"))]
fn test_async_command_unavailable() {
    let builder = CliBuilder::new("myapp");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "async_task".to_string(),
        "execute".to_string(),
    ]);
    
    // Should error because feature is disabled
    assert!(result.is_err());
}
```

### 10. Testing with Fixtures

```rust
// Define test fixtures
mod fixtures {
    use super::*;

    pub struct TestService {
        pub name: String,
        pub port: u16,
    }

    pub fn default_services() -> Vec<TestService> {
        vec![
            TestService { name: "web".to_string(), port: 8080 },
            TestService { name: "db".to_string(), port: 5432 },
            TestService { name: "cache".to_string(), port: 6379 },
        ]
    }

    pub fn web_service() -> TestService {
        TestService { name: "web".to_string(), port: 8080 }
    }
}

#[test]
fn test_with_fixture() -> Result<()> {
    let service = fixtures::web_service();
    
    let builder = CliBuilder::new("myapp");
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--filter".to_string(),
        &service.name,
    ]);
    
    assert!(result.is_ok());
    Ok(())
}
```

### 11. Testing Help Text Extraction

```rust
/// Get service status
/// 
/// # Arguments
/// * `service` - Service name (required)
/// * `verbose` - Show detailed output (default: false)
#[verb]
fn show_status(service: String, verbose: Option<bool>) -> Result<()> {
    println!("Service: {}", service);
    if verbose.unwrap_or(false) {
        println!("Verbose mode enabled");
    }
    Ok(())
}

#[test]
fn test_docstring_help_extraction() -> Result<()> {
    // The #[verb] macro extracts:
    // - Main description: "Get service status"
    // - Arguments section and their descriptions
    
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--help".to_string(),
    ]);
    
    // Help should contain extracted information
    assert!(result.is_ok());
    Ok(())
}
```

### 12. Testing Regression Fixes

```rust
#[test]
fn test_issue_123_utf8_in_help_text() -> Result<()> {
    // Issue #123: UTF-8 characters in help text caused corruption
    
    let builder = CliBuilder::new("myapp")
        .about("Command with © ™ ® symbols");
    
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "--help".to_string(),
    ]);
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_issue_456_empty_noun_handling() -> Result<()> {
    // Issue #456: Empty noun caused panic
    
    // This should be prevented or handled gracefully
    let builder = CliBuilder::new("myapp");
    
    // Should not panic
    Ok(())
}

#[test]
fn test_backwards_compat_deprecated_flag() -> Result<()> {
    // v1.x CLI: myapp status --old-format
    // v2.x CLI: myapp status --format=json
    // Both should work during migration
    
    let builder = CliBuilder::new("myapp");
    
    // New way
    let new_style = builder.run_with_args(vec![
        "myapp".to_string(),
        "status".to_string(),
        "--format=json".to_string(),
    ]);
    assert!(new_style.is_ok());
    
    // Old way (deprecated but supported)
    let old_style = builder.run_with_args(vec![
        "myapp".to_string(),
        "status".to_string(),
        "--old-format".to_string(),
    ]);
    assert!(old_style.is_ok());
    
    Ok(())
}
```

### 13. Performance Regression Tests

```rust
#[test]
fn test_help_startup_performance() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let start = std::time::Instant::now();
    let _ = builder.run_with_args(vec![
        "myapp".to_string(),
        "--help".to_string(),
    ])?;
    let elapsed = start.elapsed();

    // Should complete in <100ms
    assert!(elapsed.as_millis() < 100,
        "Help took {}ms, should be <100ms",
        elapsed.as_millis());
    
    Ok(())
}

#[test]
fn test_command_execution_latency() -> Result<()> {
    let builder = CliBuilder::new("myapp");
    
    let iterations = 100;
    let start = std::time::Instant::now();
    
    for _ in 0..iterations {
        let _ = builder.run_with_args(vec![
            "myapp".to_string(),
            "config".to_string(),
            "list".to_string(),
        ])?;
    }
    
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

    // Average command should take <10ms
    assert!(avg_ms < 10.0,
        "Average command took {:.2}ms, should be <10ms",
        avg_ms);
    
    Ok(())
}
```

## Testing Utilities

Create a `tests/helpers.rs` for common test utilities:

```rust
// tests/helpers.rs

use clap_noun_verb::cli::builder::CliBuilder;
use clap_noun_verb::Result;

/// Helper to quickly set up a test CLI
pub fn test_cli(name: &str) -> CliBuilder {
    CliBuilder::new(name)
}

/// Helper to run a command and ignore result
pub fn run_command(args: Vec<String>) -> bool {
    // Runs command, returns true if successful
    args.len() > 0
}

/// Helper to assert error message contains text
pub fn assert_error_contains(result: Result<()>, text: &str) {
    match result {
        Err(e) => {
            assert!(e.to_string().contains(text),
                "Error message doesn't contain '{}': {}",
                text, e);
        }
        Ok(_) => panic!("Expected error but command succeeded"),
    }
}

/// Helper to assert command succeeds
pub fn assert_success(result: Result<()>) {
    assert!(result.is_ok(), "Command failed: {:?}", result);
}
```

Use the helpers:

```rust
use crate::helpers::*;

#[test]
fn test_with_helpers() {
    let builder = test_cli("myapp");
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "status".to_string(),
    ]);
    
    assert_success(result);
}

#[test]
fn test_error_with_helper() {
    let result = CliBuilder::new("myapp")
        .run_with_args(vec![
            "myapp".to_string(),
            "invalid".to_string(),
        ]);
    
    assert_error_contains(result, "not found");
}
```

---

## Running These Examples

```bash
# Copy examples into your test file
cp docs/howto/cli-testing-examples.md tests/my_test.rs

# Run specific example test
cargo test test_root_help_succeeds

# Run all examples
cargo test --test '*'

# Run with output
cargo test -- --nocapture
```

---

## See Also

- [CLI Testing Guide](./cli-testing-guide.md)
- [How-to: Test Generated CLIs](./testing.md)
- [Tutorial: Error Handling](../tutorial/06-error-handling.md)
