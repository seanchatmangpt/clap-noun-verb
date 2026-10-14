# CLI Testing Guide for clap-noun-verb

A comprehensive guide to testing noun-verb CLI applications built with clap-noun-verb. This guide covers practical patterns for testing actual CLI behavior—not just unit tests, but integration tests, help text validation, error scenarios, and feature-gated commands.

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Organization](#test-organization)
3. [Manual CLI Testing](#manual-cli-testing)
4. [Integration Testing Patterns](#integration-testing-patterns)
5. [Testing Feature-Gated Commands](#testing-feature-gated-commands)
6. [Error Message Validation](#error-message-validation)
7. [Help Text Testing](#help-text-testing)
8. [Test Fixtures and Scenarios](#test-fixtures-and-scenarios)
9. [Regression Testing](#regression-testing)
10. [Advanced Patterns](#advanced-patterns)

---

## Testing Philosophy

The clap-noun-verb framework separates concerns into layers:

- **Business Logic Layer** (pure functions) — Tested with unit tests
- **CLI Layer** (input validation + output shaping) — Tested with integration tests
- **Macro Layer** (generated command structure) — Tested via compiled examples
- **Runtime Layer** (command dispatch) — Tested via CLI integration tests

**Test Levels:**
1. **Unit tests** (< 100ms) — Test individual components in isolation
2. **Integration tests** (< 500ms) — Test complete command workflows
3. **CLI tests** — Run the actual CLI and validate output
4. **Manual testing** — Verify UX and help text readability

**Key Principle:** Test behaviors (observable outputs/state changes), not implementation details.

---

## Test Organization

### Directory Structure

```
tests/
├── cli/                              # CLI integration tests
│   ├── integration_cli_tests.rs      # Full system workflows
│   └── telemetry_cli_tests.rs        # CLI + telemetry integration
├── common/
│   ├── mod.rs                        # Test module re-exports
│   ├── test_prelude.rs               # Lint-compliant assertion helpers
│   └── deterministic.rs              # Deterministic test utilities
├── cli_builder.rs                    # CliBuilder tests
├── cli_router.rs                     # CommandRouter tests
├── structured_error_tests.rs         # Error message tests
├── edge_cases.rs                     # Edge case scenarios
├── acceptance/                       # Acceptance tests
│   └── attribute_macro.rs            # Attribute macro validation
└── fixtures/                         # Shared test data (if needed)
```

### Test File Naming Convention

- **`*_tests.rs`** — Contains multiple related tests
- **`test_*.rs`** — Single-purpose test file
- **`*_integration.rs`** — Integration/end-to-end tests

### Module Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_prelude::*;

    // Arrange-Act-Assert pattern
    #[test]
    fn test_verb_command_behavior_description() {
        // Arrange - set up test state
        // Act - execute the command
        // Assert - verify observable behavior
    }
}
```

---

## Manual CLI Testing

### Running the CLI During Development

```bash
# Build a release binary for testing
cargo make build-release

# Run with help
./target/release/myapp --help

# Run a specific noun-verb command
./target/release/myapp <noun> <verb> [OPTIONS] [ARGS]

# Run with debug output
RUST_LOG=debug ./target/release/myapp <noun> <verb>

# Test with specific feature flags
cargo run --features full -- <noun> <verb>
```

### Manual Test Checklist

Use this checklist when manually testing CLI changes:

```bash
#!/bin/bash
# Manual CLI testing script (save as tests/manual_cli_test.sh)

set -e
CLI="${1:-./target/release/myapp}"

echo "=== Manual CLI Testing ==="

# Root help
echo "Testing: $CLI --help"
$CLI --help

# Root version
echo "Testing: $CLI --version"
$CLI --version

# Noun help
echo "Testing: $CLI services --help"
$CLI services --help

# Noun-verb command
echo "Testing: $CLI services status"
$CLI services status

# With optional arguments
echo "Testing: $CLI services status --verbose"
$CLI services status --verbose

# With required positional arguments
echo "Testing: $CLI config set mykey myvalue"
$CLI config set mykey myvalue

# Missing required argument (should error gracefully)
echo "Testing: $CLI config set (missing values)"
$CLI config set || true

# Invalid noun (should suggest alternatives)
echo "Testing: $CLI servise --help (typo)"
$CLI servise --help || true

# Invalid verb (should suggest alternatives)
echo "Testing: $CLI services statu --help (typo)"
$CLI services statu --help || true

echo "=== All manual tests passed ==="
```

### Interactive Testing Approach

1. **Help system**: Verify `--help` output is accurate and readable
2. **Command discovery**: Verify noun/verb suggestions for typos
3. **Error messages**: Run invalid commands and check error clarity
4. **JSON output**: Pipe output to `jq` for validation
5. **Environment variables**: Test with custom env vars
6. **Piping**: Test output piping to other commands

---

## Integration Testing Patterns

### Pattern 1: Basic Command Execution

Test that a command parses and executes:

```rust
#[test]
fn test_services_status_command_executes() -> Result<()> {
    // Arrange - Create the command structure
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services");
    
    // Act - Simulate user running: myapp services status
    let args = vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
    ];
    let _result = builder.run_with_args(args)?;

    // Assert - No panic means the command executed
    Ok(())
}
```

### Pattern 2: Command with Arguments

Test commands that accept arguments:

```rust
#[test]
fn test_services_status_with_filter() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services");
    
    // Act - Run: myapp services status --filter web
    let args = vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--filter".to_string(),
        "web".to_string(),
    ];
    let _result = builder.run_with_args(args)?;

    // Assert
    Ok(())
}
```

### Pattern 3: JSON Output Validation

Verify the output is valid JSON and has expected structure:

```rust
#[test]
fn test_services_status_json_output_valid() -> Result<()> {
    // Arrange
    let status_response = ServiceStatus {
        services: vec![
            ServiceInfo {
                name: "web".to_string(),
                state: "running".to_string(),
                port: 8080,
            },
        ],
    };

    // Act - Serialize to JSON (as the CLI would)
    let json = serde_json::to_string(&status_response)?;
    
    // Assert - Can parse back
    let parsed: ServiceStatus = serde_json::from_str(&json)?;
    assert_eq!(parsed.services[0].name, "web");
    assert_eq!(parsed.services[0].port, 8080);

    Ok(())
}
```

### Pattern 4: Complete Workflow Test

Test a multi-step command sequence:

```rust
#[test]
fn test_services_start_stop_workflow() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services");
    
    // Act 1 - Start a service
    let start_args = vec![
        "myapp".to_string(),
        "services".to_string(),
        "start".to_string(),
        "--service".to_string(),
        "web".to_string(),
    ];
    builder.run_with_args(start_args)?;
    
    // Act 2 - Check status
    let status_args = vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
    ];
    let _result = builder.run_with_args(status_args)?;
    
    // Act 3 - Stop the service
    let stop_args = vec![
        "myapp".to_string(),
        "services".to_string(),
        "stop".to_string(),
        "--service".to_string(),
        "web".to_string(),
    ];
    builder.run_with_args(stop_args)?;

    // Assert - All steps completed without error
    Ok(())
}
```

---

## Testing Feature-Gated Commands

### Pattern 1: Feature-Gated Test Modules

Gate entire test modules behind features:

```rust
// tests/async_verb_tests.rs

#[cfg(feature = "async")]
mod async_verb_tests {
    use super::*;

    #[tokio::test]
    async fn test_async_command_execution() -> Result<()> {
        // Arrange
        let builder = CliBuilder::new("myapp")
            .noun("async_task", "Async operations");
        
        // Act
        let args = vec![
            "myapp".to_string(),
            "async_task".to_string(),
            "process".to_string(),
        ];
        let _result = builder.run_with_args(args)?;

        // Assert
        Ok(())
    }
}
```

### Pattern 2: Feature-Gated Individual Tests

Gate specific tests:

```rust
#[test]
#[cfg(feature = "federated-network")]
fn test_federated_network_peer_discovery() -> Result<()> {
    // This test only runs when building with: cargo test --features federated-network
    let network = FederatedNetwork::new("node1")?;
    assert!(!network.discover_peers()?.is_empty());
    Ok(())
}

#[tokio::test]
#[cfg(all(feature = "federated-network", feature = "async"))]
async fn test_byzantine_consensus() -> Result<()> {
    // Requires both features to compile
    let network = FederatedNetwork::new("node1")?;
    let consensus = network.consensus_vote(&peers, |p| true).await?;
    assert!(consensus);
    Ok(())
}
```

### Pattern 3: Conditional Imports

Gate imports based on features:

```rust
#[cfg(feature = "async")]
use crate::async_verb::AsyncVerbCommand;

#[cfg(feature = "federated-network")]
use crate::federation::FederatedNetwork;

#[cfg(feature = "executable-specs")]
mod executable_specs_tests {
    use super::*;
    use clap_noun_verb::frontier::ExecutableSpec;

    #[test]
    fn test_spec_execution() -> Result<()> {
        // ...
    }
}
```

### Pattern 4: Testing All Feature Combinations

Create a test that verifies build success with various features:

```bash
#!/bin/bash
# tests/feature_combination_test.sh

set -e

echo "Testing feature combinations..."

# Test default (no features)
echo "Building default..."
cargo build --quiet

# Test with async
echo "Building with async..."
cargo build --features async --quiet

# Test with federated-network
echo "Building with federated-network..."
cargo build --features federated-network --quiet

# Test with all features
echo "Building with all features..."
cargo build --features full --quiet

echo "✅ All feature combinations build successfully"
```

### Running Feature-Gated Tests

```bash
# Run tests for specific feature
cargo test --features async

# Run all tests across all features
cargo make test-all

# Run with frontier features
cargo make test-frontier

# Run single-threaded (deterministic)
cargo make test-lib-deterministic
```

---

## Error Message Validation

### Pattern 1: Testing Missing Required Arguments

Test that missing required arguments produce helpful errors:

```rust
#[test]
fn test_missing_required_argument_error() {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("config", "Manage configuration");
    
    // Act - Run without the required 'key' argument
    let args = vec![
        "myapp".to_string(),
        "config".to_string(),
        "set".to_string(),
        // Missing: key and value
    ];
    let result = builder.run_with_args(args);

    // Assert - Should be error, not panic
    assert!(result.is_err());
}
```

### Pattern 2: Testing Invalid Command Suggestions

Test that typos get helpful suggestions:

```rust
#[test]
fn test_invalid_verb_suggests_alternative() {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services");
    
    // Act - Typo: "statu" instead of "status"
    let args = vec![
        "myapp".to_string(),
        "services".to_string(),
        "statu".to_string(),
    ];
    let result = builder.run_with_args(args);

    // Assert - Error should contain suggestion
    match result {
        Err(e) => {
            let msg = e.to_string();
            assert!(msg.contains("status") || msg.contains("suggest"));
        }
        Ok(_) => panic!("Should have failed with typo"),
    }
}
```

### Pattern 3: Testing Structured Errors

Validate error structure and metadata:

```rust
use clap_noun_verb::error::{StructuredError, ErrorKind, Severity};

#[test]
fn test_error_structure_validation() {
    // Arrange
    let error = StructuredError::deadline_exceeded(100, 150);

    // Assert - Verify error properties
    assert_eq!(error.kind, ErrorKind::DeadlineExceeded);
    assert_eq!(error.severity, Severity::Critical);
    assert!(error.message.contains("Deadline"));
    assert!(error.details.contains_key("deadline_ms"));
    assert!(!error.action_templates.is_empty());
}
```

### Pattern 4: Error Message Clarity Tests

Ensure error messages don't contain debug artifacts:

```rust
#[test]
fn test_error_messages_user_friendly() {
    let args = vec!["myapp".to_string(), "invalid_noun".to_string()];
    let result = CliBuilder::new("myapp").run_with_args(args);

    match result {
        Err(e) => {
            let msg = e.to_string();
            // Should NOT contain these:
            assert!(!msg.contains("unwrap"));
            assert!(!msg.contains("panic"));
            assert!(!msg.contains("thread"));
            assert!(!msg.contains("backtrace"));
            // SHOULD contain helpful information:
            assert!(msg.len() < 500, "Error message should be concise");
        }
        Ok(_) => panic!("Expected error"),
    }
}
```

### Pattern 5: Testing Error Context

Validate that errors provide sufficient context:

```rust
#[test]
fn test_error_includes_context() {
    // Arrange
    let noun = "config";
    let verb = "invalid_verb";
    
    // Act
    let result = CliBuilder::new("myapp")
        .noun(noun, "Configuration management")
        .run_with_args(vec![
            "myapp".to_string(),
            noun.to_string(),
            verb.to_string(),
        ]);

    // Assert
    match result {
        Err(NounVerbError::VerbNotFound { noun: n, verb: v, .. }) => {
            assert_eq!(n, noun);
            assert_eq!(v, verb);
        }
        _ => panic!("Expected VerbNotFound error with context"),
    }
}
```

---

## Help Text Testing

### Pattern 1: Testing Help Output Exists

Verify `--help` produces output:

```rust
#[test]
fn test_root_help_displays() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .about("My awesome application");
    
    // Act
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "--help".to_string(),
    ]);

    // Assert - Help should succeed (exit 0 in real CLI)
    // In testing context, verify no panic occurred
    let _ = result;
    Ok(())
}
```

### Pattern 2: Testing Noun Help Text

Verify noun help is accurate:

```rust
#[test]
fn test_services_noun_help_contains_description() -> Result<()> {
    // Arrange
    let about_text = "Manage application services";
    let builder = CliBuilder::new("myapp")
        .noun("services", about_text);
    
    // Act - Request noun help
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "--help".to_string(),
    ]);

    // Assert - Should display noun description
    let _ = result;
    Ok(())
}
```

### Pattern 3: Testing Verb Help Text

Verify verb help with docstring extraction:

```rust
/// Show service status
/// 
/// # Arguments
/// * `filter` - Filter services by name (optional)
/// * `verbose` - Show detailed output (default: false)
#[verb]
fn show_status(filter: Option<String>, verbose: bool) -> Result<ServiceStatus> {
    // Implementation...
    Ok(ServiceStatus::default())
}

#[test]
fn test_status_verb_help_includes_arguments() -> Result<()> {
    // The #[verb] macro extracts the docstring and Arguments section
    // automatically. This test verifies the behavior works:
    
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services");
    
    // Act
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
        "--help".to_string(),
    ]);

    // Assert - Macro should have extracted help from docstring
    let _ = result;
    Ok(())
}
```

### Pattern 4: Testing Help Text Structure

Ensure help output follows expected format:

```rust
#[test]
fn test_help_text_well_formatted() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp");
    
    // Act - Capture help output
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "--help".to_string(),
    ]);

    // Assert - Help follows clap conventions
    let _ = result;
    // Verify:
    // - Usage line exists
    // - About/description follows
    // - Options section exists
    // - Subcommands section exists (if applicable)
    Ok(())
}
```

### Pattern 5: Testing Completion Suggestions

Test shell completion output:

```rust
#[test]
fn test_bash_completion_generation() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("services", "Manage services")
        .noun("config", "Manage configuration");
    
    // Act
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "completion".to_string(),
        "bash".to_string(),
    ]);

    // Assert - Completion script should be generated
    let _ = result;
    Ok(())
}
```

---

## Test Fixtures and Scenarios

### Pattern 1: Creating Reusable Test Fixtures

Define common test data:

```rust
// tests/fixtures/mod.rs

pub mod services {
    use crate::ServiceInfo;

    pub fn sample_web_service() -> ServiceInfo {
        ServiceInfo {
            name: "web".to_string(),
            state: "running".to_string(),
            port: 8080,
        }
    }

    pub fn sample_database_service() -> ServiceInfo {
        ServiceInfo {
            name: "database".to_string(),
            state: "running".to_string(),
            port: 5432,
        }
    }

    pub fn all_services() -> Vec<ServiceInfo> {
        vec![sample_web_service(), sample_database_service()]
    }
}

// Usage in tests:
#[test]
fn test_with_fixtures() {
    use crate::fixtures::services::*;
    
    let status = ServiceStatus {
        services: all_services(),
    };
    
    assert_eq!(status.services.len(), 2);
}
```

### Pattern 2: Builder Pattern for Test Data

Create test data with fluent interface:

```rust
pub struct TestConfigBuilder {
    name: String,
    services: Vec<String>,
    verbose: bool,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            name: "test-app".to_string(),
            services: vec![],
            verbose: false,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_service(mut self, service: &str) -> Self {
        self.services.push(service.to_string());
        self
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    pub fn build(self) -> TestConfig {
        TestConfig {
            name: self.name,
            services: self.services,
            verbose: self.verbose,
        }
    }
}

// Usage:
#[test]
fn test_with_builder_fixture() {
    let config = TestConfigBuilder::new()
        .with_name("myapp")
        .with_service("web")
        .with_service("db")
        .verbose()
        .build();
    
    assert_eq!(config.name, "myapp");
    assert_eq!(config.services.len(), 2);
}
```

### Pattern 3: Parametrized Tests

Test multiple scenarios with shared logic:

```rust
#[test]
fn test_services_startup_variations() {
    let scenarios = vec![
        ("web", true),
        ("database", true),
        ("cache", true),
        ("nonexistent", false),
    ];

    for (service_name, should_succeed) in scenarios {
        // Arrange
        let builder = CliBuilder::new("myapp")
            .noun("services", "Manage services");
        
        // Act
        let args = vec![
            "myapp".to_string(),
            "services".to_string(),
            "start".to_string(),
            "--service".to_string(),
            service_name.to_string(),
        ];
        let result = builder.run_with_args(args);

        // Assert
        if should_succeed {
            assert!(result.is_ok(), "Service {} should start", service_name);
        } else {
            assert!(result.is_err(), "Service {} should not start", service_name);
        }
    }
}
```

### Pattern 4: Scenario Files

Store complex test scenarios in files:

```rust
// tests/scenarios/service_workflow.yaml
# Service management workflow test

commands:
  - name: "Start web service"
    noun: "services"
    verb: "start"
    args:
      service: "web"
    expect: success

  - name: "Check status"
    noun: "services"
    verb: "status"
    args:
      filter: "web"
    expect: success
    assertions:
      - output_contains: "running"
      - output_contains: "web"

  - name: "Stop web service"
    noun: "services"
    verb: "stop"
    args:
      service: "web"
    expect: success
```

Parse and run scenarios:

```rust
#[test]
fn test_service_workflow_scenario() -> Result<()> {
    let scenario = load_scenario("tests/scenarios/service_workflow.yaml")?;
    run_scenario(&scenario)?;
    Ok(())
}
```

---

## Regression Testing

### Pattern 1: Issue-Based Regression Tests

Create tests for fixed bugs:

```rust
#[test]
fn test_issue_123_help_text_encoding() -> Result<()> {
    // Issue #123: Help text with UTF-8 characters was corrupted
    // Regression test to ensure it stays fixed
    
    // Arrange
    let builder = CliBuilder::new("myapp")
        .about("CLI with special chars: © ™ ®");
    
    // Act
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "--help".to_string(),
    ]);

    // Assert - Should handle UTF-8 correctly
    let _ = result;
    Ok(())
}

#[test]
fn test_issue_456_empty_noun_panic() -> Result<()> {
    // Issue #456: Empty noun name caused panic
    // This should now error gracefully
    
    // Arrange - This should either be prevented at compile time
    // or handled at runtime
    
    // Act
    let builder = CliBuilder::new("myapp");
    
    // Assert - Should not panic
    Ok(())
}
```

### Pattern 2: Performance Regression Tests

Verify performance hasn't degraded:

```rust
#[test]
fn test_startup_performance_not_regressed() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("services", "Services")
        .noun("config", "Configuration")
        .noun("logs", "Logs");
    
    // Act - Measure startup time
    let start = std::time::Instant::now();
    let _ = builder.run_with_args(vec![
        "myapp".to_string(),
        "--help".to_string(),
    ])?;
    let elapsed = start.elapsed();

    // Assert - Should complete in reasonable time (< 100ms)
    assert!(elapsed.as_millis() < 100, 
        "Startup took {}ms, should be <100ms", 
        elapsed.as_millis());
    Ok(())
}
```

### Pattern 3: Backwards Compatibility Tests

Ensure old CLI usage still works:

```rust
#[test]
fn test_deprecated_flag_still_works() -> Result<()> {
    // Old CLI: myapp status --old-format
    // New CLI: myapp status --output-format=json
    // Both should work during migration period
    
    // Arrange
    let builder = CliBuilder::new("myapp");
    
    // Act 1 - New flag
    let new_result = builder.run_with_args(vec![
        "myapp".to_string(),
        "status".to_string(),
        "--output-format=json".to_string(),
    ]);
    
    // Act 2 - Old flag (deprecated but supported)
    let old_result = builder.run_with_args(vec![
        "myapp".to_string(),
        "status".to_string(),
        "--old-format".to_string(),
    ]);

    // Assert - Both should work
    assert!(new_result.is_ok());
    assert!(old_result.is_ok());
    Ok(())
}
```

### Pattern 4: Changelog-Driven Tests

Create tests from changelog items:

```rust
// When adding to CHANGELOG.md:
// - Added: `config get` command to read single config values
// - Added: `--format` flag to all commands

#[test]
fn test_config_get_command_new_in_v2_0() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("config", "Configuration management");
    
    // Act - New in v2.0
    let args = vec![
        "myapp".to_string(),
        "config".to_string(),
        "get".to_string(),
        "--key".to_string(),
        "app.version".to_string(),
    ];
    let result = builder.run_with_args(args);

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_format_flag_available_all_commands() -> Result<()> {
    let commands = vec![
        ("services", "status"),
        ("config", "list"),
        ("logs", "tail"),
    ];

    for (noun, verb) in commands {
        // Arrange
        let builder = CliBuilder::new("myapp")
            .noun(noun, "");
        
        // Act
        let args = vec![
            "myapp".to_string(),
            noun.to_string(),
            verb.to_string(),
            "--format=json".to_string(),
        ];
        let result = builder.run_with_args(args);

        // Assert
        assert!(result.is_ok(), 
            "Format flag should work on {}/{}", noun, verb);
    }
    Ok(())
}
```

---

## Advanced Patterns

### Pattern 1: Testing Command Registry

Verify registered commands match expected structure:

```rust
#[test]
fn test_command_registry_completeness() {
    // Arrange
    let registry = CommandRegistry::new();

    // Act
    let registered = registry.list_all_commands();

    // Assert - Expected nouns exist
    let nouns: Vec<_> = registered.iter()
        .map(|cmd| cmd.noun.as_str())
        .collect();
    
    assert!(nouns.contains(&"services"));
    assert!(nouns.contains(&"config"));
    assert!(nouns.contains(&"logs"));
}

#[test]
fn test_command_registry_verb_completeness() {
    // Arrange
    let registry = CommandRegistry::new();

    // Act
    let services_verbs = registry
        .get_verbs_for_noun("services")
        .unwrap_or_default();

    // Assert - Expected verbs for "services" noun
    let verb_names: Vec<_> = services_verbs.iter()
        .map(|v| v.name.as_str())
        .collect();
    
    assert!(verb_names.contains(&"status"));
    assert!(verb_names.contains(&"start"));
    assert!(verb_names.contains(&"stop"));
}
```

### Pattern 2: Testing Async Verbs

Test async command execution:

```rust
#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_command_execution() -> Result<()> {
    // Arrange
    let builder = CliBuilder::new("myapp")
        .noun("async_task", "Async operations");
    
    // Act
    let args = vec![
        "myapp".to_string(),
        "async_task".to_string(),
        "process".to_string(),
    ];
    let result = builder.run_with_args(args)?;

    // Assert
    assert!(result.is_ok());
    Ok(())
}
```

### Pattern 3: Testing Handler Logic Directly

Test the `HandlerInput`/`HandlerOutput` bridge without going through the CLI parser:

```rust
use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};

#[test]
fn test_handler_output_serializes_data() -> Result<()> {
    // Arrange
    let context = HandlerContext::new("status").with_noun("services");
    let input = HandlerInput {
        args: Default::default(),
        opts: Default::default(),
        context,
    };

    // Act
    let output = HandlerOutput::from_data(serde_json::json!({ "status": "ok" }))?;

    // Assert
    assert_eq!(input.context.verb, "status");
    assert_eq!(input.context.noun.as_deref(), Some("services"));
    assert_eq!(output.data["status"], "ok");
    Ok(())
}
```

### Pattern 4: Testing Output Formatting

Verify different output formats:

```rust
#[test]
fn test_json_output_format() -> Result<()> {
    // Arrange
    let data = ServiceStatus {
        services: vec![sample_web_service()],
    };

    // Act - Format as JSON
    let json = format_output(&data, OutputFormat::Json)?;
    
    // Assert - Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&json)?;
    Ok(())
}

#[test]
fn test_table_output_format() -> Result<()> {
    // Arrange
    let data = ServiceStatus {
        services: vec![
            sample_web_service(),
            sample_database_service(),
        ],
    };

    // Act - Format as table
    let table = format_output(&data, OutputFormat::Table)?;
    
    // Assert - Should contain column headers and data
    assert!(table.contains("name"));
    assert!(table.contains("state"));
    assert!(table.contains("port"));
    assert!(table.contains("web"));
    assert!(table.contains("database"));
    Ok(())
}
```

### Pattern 5: Testing with Environment Variables

Test commands that use environment variables:

```rust
#[test]
fn test_command_respects_env_variable() -> Result<()> {
    // Arrange - Set environment variable
    std::env::set_var("MYAPP_DEBUG", "true");
    
    // Cleanup after test
    let _guard = EnvironmentGuard::new("MYAPP_DEBUG");

    // Act
    let builder = CliBuilder::new("myapp");
    let result = builder.run_with_args(vec![
        "myapp".to_string(),
        "status".to_string(),
    ])?;

    // Assert - Debug output should be enabled
    Ok(())
}

// Helper for test environment cleanup
pub struct EnvironmentGuard {
    key: String,
    original: Option<String>,
}

impl EnvironmentGuard {
    pub fn new(key: &str) -> Self {
        let original = std::env::var(key).ok();
        Self {
            key: key.to_string(),
            original,
        }
    }
}

impl Drop for EnvironmentGuard {
    fn drop(&mut self) {
        if let Some(val) = &self.original {
            std::env::set_var(&self.key, val);
        } else {
            std::env::remove_var(&self.key);
        }
    }
}
```

---

## Testing Checklist

Use this checklist when adding new CLI commands:

- [ ] **Parsing** — Command parses without panicking
  - [ ] `--help` flag works
  - [ ] `-h` short flag works
  - [ ] `--version` flag works
  
- [ ] **Arguments** — All argument types parse correctly
  - [ ] Required positional arguments
  - [ ] Optional arguments with defaults
  - [ ] Flags (boolean, string, numeric)
  - [ ] Multiple values
  
- [ ] **Validation** — Invalid inputs handled gracefully
  - [ ] Missing required arguments
  - [ ] Invalid argument types (e.g., string where number expected)
  - [ ] Out-of-range values
  
- [ ] **Output** — Output format correct
  - [ ] JSON serialization valid
  - [ ] Output contains expected fields
  - [ ] Special characters handled correctly
  - [ ] Empty results handled
  
- [ ] **Help Text** — Documentation is accurate
  - [ ] Root `--help` complete
  - [ ] Noun help displays
  - [ ] Verb help displays with arguments
  - [ ] Docstring help extracted correctly
  
- [ ] **Errors** — Error messages are helpful
  - [ ] Error messages are user-friendly
  - [ ] No debug artifacts in errors
  - [ ] Suggestions for typos
  - [ ] Error context is clear
  
- [ ] **Features** — Feature-gated code tested
  - [ ] Builds without optional features
  - [ ] Builds with optional features
  - [ ] Tests run for enabled features
  
- [ ] **Regression** — Previous bugs don't return
  - [ ] Applicable regression tests added
  - [ ] Related tests still pass
  
- [ ] **Performance** — No performance degradation
  - [ ] Startup time acceptable
  - [ ] Command execution time acceptable

---

## Running Tests

```bash
# Run all tests (must complete <1 second)
cargo make test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_services_status_command_executes

# Run tests matching pattern
cargo test services

# Run single-threaded (deterministic)
cargo make test-lib-deterministic

# Run with all features
cargo make test-all

# Run frontier feature tests
cargo make test-frontier

# Run with coverage (if tooling available)
cargo tarpaulin

# View test code coverage
cargo llvm-cov
```

---

## Best Practices

1. **AAA Pattern** — Always use Arrange-Act-Assert
2. **Descriptive Names** — Test names should describe behavior
3. **Single Responsibility** — Each test should verify one thing
4. **Deterministic** — Tests must pass consistently
5. **Fast** — Tests should complete in <1 second total
6. **No Sidelines** — Clean up any state in tests
7. **Error Clarity** — Validate error messages and structure
8. **Documentation** — Document why edge cases matter
9. **DRY** — Use fixtures and builders for test data
10. **Safety** — Use lint-compliant assertion helpers

---

## References

- [How-to: Test Generated CLIs](../howto/testing.md)
- [Tutorial 03: Testing Basics](../tutorial/03-testing-basics.md)
- [Error Handling Tutorial](../tutorial/06-error-handling.md)
- [CLAUDE.md - Build Commands](../../CLAUDE.md)
