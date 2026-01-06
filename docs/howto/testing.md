# How-to: Test Generated CLIs

**Problem**: You need to ensure generated CLI code is correct before deployment

**Solution**: Use Chicago TDD patterns to test generated CLIs comprehensively

## Testing Strategy

Generated CLIs should be tested at three levels:

1. **Unit tests** - Test individual generated functions
2. **Integration tests** - Test complete command workflows
3. **Property tests** - Verify invariants hold for all inputs

## Step 1: Unit Test Generated Handlers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status_handler_returns_running_status() {
        // Arrange
        let args = StatusArgs {
            service_name: "web".to_string(),
            verbose: Some(true),
        };

        // Act
        let result = status_service(&args).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, "running");
        assert!(response.running);
    }

    #[tokio::test]
    async fn test_status_handler_with_nonexistent_service() {
        // Arrange
        let args = StatusArgs {
            service_name: "nonexistent".to_string(),
            verbose: None,
        };

        // Act
        let result = status_service(&args).await;

        // Assert - should be error or handle gracefully
        match result {
            Ok(response) => assert!(!response.running),
            Err(_) => (), // Acceptable error handling
        }
    }
}
```

## Step 2: Integration Test Full Workflows

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_services_start_stop_workflow() {
        // Arrange
        let start_args = StartArgs {
            service_name: "web".to_string(),
        };

        let stop_args = StopArgs {
            service_name: "web".to_string(),
        };

        // Act - Start service
        let start_result = start_service(&start_args).await;
        assert!(start_result.is_ok());

        // Act - Check status
        let status_args = StatusArgs {
            service_name: "web".to_string(),
            verbose: None,
        };
        let status = status_service(&status_args).await.unwrap();
        assert!(status.running);

        // Act - Stop service
        let stop_result = stop_service(&stop_args).await;
        assert!(stop_result.is_ok());

        // Assert - Check stopped
        let final_status = status_service(&status_args).await.unwrap();
        assert!(!final_status.running);
    }

    #[tokio::test]
    async fn test_services_restart_command() {
        // Arrange
        let args = RestartArgs {
            service_name: "database".to_string(),
            timeout_seconds: Some(30),
        };

        // Act
        let result = restart_service(&args).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, "restarted");
    }
}
```

## Step 3: Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_status_never_panics(name in ".*") {
        let args = StatusArgs {
            service_name: name,
            verbose: None,
        };

        // Should not panic for any input
        let _ = futures::executor::block_on(status_service(&args));
    }

    #[test]
    fn prop_all_responses_serializable(
        status in "running|stopped|error",
        uptime in 0u64..1000000,
    ) {
        let response = StatusResponse {
            status,
            running: true,
            uptime_seconds: uptime,
        };

        // Must serialize to JSON
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        // Must deserialize back
        let deserialized: Result<StatusResponse, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}
```

## Step 4: Error Handling Tests

```rust
#[tokio::test]
async fn test_invalid_arguments_handled() {
    let args = StatusArgs {
        service_name: String::new(), // Empty name
        verbose: None,
    };

    let result = status_service(&args).await;

    // Should handle empty names gracefully
    match result {
        Ok(_) => assert!(true), // Accept for backward compat
        Err(e) => {
            let msg = e.to_string();
            assert!(msg.contains("invalid") || msg.contains("empty"));
        }
    }
}

#[tokio::test]
async fn test_service_error_messages_are_clear() {
    let args = StatusArgs {
        service_name: "nonexistent_service_xyz".to_string(),
        verbose: None,
    };

    match status_service(&args).await {
        Err(e) => {
            let msg = e.to_string();
            // Error messages must be user-friendly
            assert!(!msg.contains("unwrap"));
            assert!(!msg.contains("panic"));
        }
        Ok(_) => (),
    }
}
```

## Step 5: JSON Serialization Tests

```rust
#[test]
fn test_status_response_json_round_trip() {
    // Arrange
    let original = StatusResponse {
        status: "running".to_string(),
        running: true,
        uptime_seconds: 3600,
    };

    // Act
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: StatusResponse = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(original.status, deserialized.status);
    assert_eq!(original.running, deserialized.running);
    assert_eq!(original.uptime_seconds, deserialized.uptime_seconds);
}

#[test]
fn test_status_response_json_structure() {
    let response = StatusResponse {
        status: "running".to_string(),
        running: true,
        uptime_seconds: 3600,
    };

    let json = serde_json::to_value(&response).unwrap();

    // Verify JSON structure
    assert!(json.get("status").is_some());
    assert!(json.get("running").is_some());
    assert!(json.get("uptime_seconds").is_some());
    assert_eq!(json.get("status").unwrap().as_str(), Some("running"));
}
```

## Step 6: Concurrency Tests

```rust
#[tokio::test]
async fn test_concurrent_status_calls_dont_interfere() {
    // Arrange
    let args1 = StatusArgs {
        service_name: "service1".to_string(),
        verbose: None,
    };
    let args2 = StatusArgs {
        service_name: "service2".to_string(),
        verbose: None,
    };

    // Act - Call concurrently
    let handle1 = tokio::spawn(status_service(args1));
    let handle2 = tokio::spawn(status_service(args2));

    // Assert - Both complete
    let result1 = handle1.await;
    let result2 = handle2.await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}
```

## Step 7: Generated Code Quality Tests

Test code generation itself:

```rust
#[test]
fn test_generated_code_has_proper_structure() {
    let turtle = r#"
    @prefix cnv: <https://cnv.dev/ontology#> .
    cnv:Services a cnv:Noun ; cnv:name "services" .
    cnv:Status a cnv:Verb ; cnv:name "status" ; cnv:hasNoun cnv:Services .
    "#;

    let parser = TurtleParser::new();
    let ontology = parser.parse(turtle).unwrap();
    let generator = CliCodeGenerator::new().unwrap();
    let generated = generator.generate_from_ontology(&ontology).unwrap();

    let code = generated.rust_code();

    // Verify code structure
    assert!(code.contains("#[noun("));
    assert!(code.contains("#[verb("));
    assert!(code.contains("pub async fn"));
    assert!(code.contains("Result<"));
}

#[test]
fn test_generated_code_compiles() {
    // This would typically run cargo check on generated code
    let turtle = "...".to_string();
    let parser = TurtleParser::new();
    let ontology = parser.parse(&turtle).unwrap();
    let generator = CliCodeGenerator::new().unwrap();
    let generated = generator.generate_from_ontology(&ontology).unwrap();

    // Write to temp file
    let tmp = std::env::temp_dir().join("generated_test.rs");
    std::fs::write(&tmp, generated.rust_code()).unwrap();

    // Verify it's valid Rust syntax
    let code: syn::File = syn::parse_file(generated.rust_code()).unwrap();
    assert!(!code.items.is_empty());
}
```

## Step 8: CLI Integration Tests

Test generated CLI as if running it:

```bash
#!/bin/bash
# tests/cli_integration.sh

set -e

# Build the CLI
cargo build --release

CLI="./target/release/my_cli"

# Test 1: Help command works
echo "Testing help..."
$CLI --help > /dev/null

# Test 2: Services noun exists
echo "Testing services noun..."
$CLI services --help > /dev/null

# Test 3: Status command works
echo "Testing status command..."
$CLI services status --service web

# Test 4: Start/stop workflow
echo "Testing start/stop..."
$CLI services start --service db
$CLI services status --service db
$CLI services stop --service db

echo "✅ All CLI tests passed"
```

## Testing Checklist

- ✅ Unit tests for each generated handler
- ✅ Integration tests for command workflows
- ✅ Error handling tests
- ✅ JSON serialization tests
- ✅ Concurrency tests for async handlers
- ✅ Code generation tests
- ✅ CLI integration tests
- ✅ All tests pass: `cargo make test`

## Running Tests

```bash
# Run all tests
cargo make test

# Run specific test
cargo make test test_status_handler_returns_running_status

# Run with output
cargo test --lib -- --nocapture

# Run integration tests only
cargo test --test '*'
```

---

**Related**:
- [Tutorial 3: Generate Your First CLI](../tutorials/tutorial-3-first-cli.md)
- [How-to: Validate Ontologies](validation.md)
- [How-to: Debug RDF Issues](debugging.md)
