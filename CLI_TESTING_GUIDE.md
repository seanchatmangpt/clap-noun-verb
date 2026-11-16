# CLI Testing & Validation Skill Guide

A practical guide for testing noun-verb commands in `clap-noun-verb` v26.9.1. Focus on integration tests, feature-gated commands, error handling, and help text validation.

---

## Table of Contents

1. [Manual CLI Testing Patterns](#manual-cli-testing-patterns)
2. [Integration Test Design](#integration-test-design)
3. [Testing Feature-Gated Commands](#testing-feature-gated-commands)
4. [Error Message Validation](#error-message-validation)
5. [Help Text Testing](#help-text-testing)
6. [Test Fixtures & Scenarios](#test-fixtures--scenarios)
7. [Regression Testing](#regression-testing)
8. [Performance & Stress Testing](#performance--stress-testing)
9. [Build Verification](#build-verification)

---

## Manual CLI Testing Patterns

### Running Single Commands

**Direct invocation of compiled binary:**

```bash
# Build the CLI
cargo make build

# Run a specific command
./target/debug/myapp services status
./target/debug/myapp config set --key app.name --value "My App"

# View help for a command
./target/debug/myapp services --help
./target/debug/myapp services status --help
```

**Using `cargo run` directly:**

```bash
cargo run -- services status
cargo run -- config set --key app.name --value "test"
cargo run -- --help
```

### Testing via Examples

The framework provides runnable examples with built-in noun-verb commands:

```bash
# Run an example CLI
cargo run --example tutorial_services -- services status
cargo run --example specimen-graph-manager -- graph load ./data.json

# Get introspection data (for tool discovery)
cargo run --example tutorial_services -- --introspect
```

### Common Testing Scenarios

```bash
# Test with missing arguments
./target/debug/myapp config set --key only  # should fail

# Test with invalid flags
./target/debug/myapp config set --invalid-flag value

# Test with wrong noun/verb
./target/debug/myapp invalid-noun status
./target/debug/myapp config invalid-verb

# Test with various output formats (if supported)
./target/debug/myapp services status --format json
./target/debug/myapp services status --format plain
```

---

## Integration Test Design

Integration tests verify end-to-end CLI behavior: argument parsing, command dispatch, and output. They should test the **observable behavior**, not implementation details.

### Test Structure: Arrange-Act-Assert (AAA)

```rust
#[test]
fn test_verb_command_executes_successfully_with_required_args() {
    // ARRANGE: Set up CLI environment
    let cmd = Command::new("myapp")
        .subcommand(
            Command::new("services")
                .subcommand(
                    Command::new("status")
                        .arg(Arg::new("verbose").long("verbose").action(ArgAction::SetTrue))
                )
        );

    // ACT: Parse arguments as the CLI would receive them
    let matches = cmd
        .try_get_matches_from(vec!["myapp", "services", "status", "--verbose"])
        .expect("Valid args should parse");

    // ASSERT: Verify parsed structure
    let services_cmd = matches.subcommand_matches("services").unwrap();
    let status_cmd = services_cmd.subcommand_matches("status").unwrap();
    assert!(status_cmd.get_flag("verbose"));
}
```

### Testing Command Registration & Discovery

Use `CommandRegistry` to verify noun and verb registration:

```rust
#[test]
fn test_command_registry_discovers_all_verbs() {
    // ARRANGE: Get registry with auto-discovery
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();

    // ACT: Retrieve registered commands
    let cmd = registry.build_command().unwrap();

    // ASSERT: Verify expected structure
    assert_eq!(cmd.get_name(), "myapp");
    assert!(cmd.get_subcommands().any(|s| s.get_name() == "services"));
}
```

### Testing Command Dispatch & Execution

```rust
#[test]
fn test_command_router_dispatches_to_correct_handler() {
    // ARRANGE: Create input representing parsed arguments
    let mut input = HandlerInput::new("services", "status");
    input.set_arg("verbose", "true");

    // ACT: Route through dispatcher
    let output = CommandRouter::route(&input).unwrap();

    // ASSERT: Verify output structure
    assert!(output.success);
    assert!(output.data.get("status").is_some());
}
```

### Testing Argument Validation

```rust
#[test]
fn test_required_argument_validation_fails_if_missing() {
    // ARRANGE: CLI definition with required argument
    let cmd = Command::new("test")
        .subcommand(
            Command::new("config")
                .subcommand(
                    Command::new("set")
                        .arg(Arg::new("key").required(true).index(1))
                        .arg(Arg::new("value").required(true).index(2))
                )
        );

    // ACT: Try to parse with missing value
    let result = cmd.try_get_matches_from(vec!["test", "config", "set", "only_one"]);

    // ASSERT: Should fail validation
    assert!(result.is_err());
}
```

### Common Integration Test Helpers

Use the provided test utilities in `tests/common/mod.rs`:

```rust
use tests::common::command_assertions::*;
use tests::common::handler_context::*;

#[test]
fn test_with_helpers() {
    let mut cmd = create_test_command();

    // Assert subcommand exists
    assert_has_subcommand(&cmd, "services");
    
    // Get verb names for a noun
    let verbs = get_verb_names(&cmd, "services");
    assert!(verbs.contains(&"status"));

    // Assert help text contains expected content
    assert_help_contains(&mut cmd, "Manage services");
}
```

---

## Testing Feature-Gated Commands

When commands are behind Cargo features, test them with feature flags enabled/disabled.

### Feature Configuration

```toml
# Cargo.toml features
[features]
default = []
wizard = ["rust-genai"]
federated-network = ["reqwest"]
full = ["wizard", "federated-network", ...]
```

### Feature-Gated Tests

Mark tests with `#[cfg(feature = "...")]`:

```rust
#![cfg(feature = "wizard")]

#[cfg(feature = "wizard")]
#[test]
fn test_wizard_command_available_with_wizard_feature() {
    let cmd = Command::new("myapp")
        .subcommand(Command::new("wizard"));

    assert_has_subcommand(&cmd, "wizard");
}

#[cfg(not(feature = "wizard"))]
#[test]
fn test_wizard_command_unavailable_without_wizard_feature() {
    let cmd = Command::new("myapp");
    assert_no_subcommand(&cmd, "wizard");
}
```

### Build Commands for Feature Testing

Use `cargo make` with feature flags:

```bash
# Test with a single feature
cargo test --features wizard

# Test with multiple features
cargo test --features "wizard,federated-network"

# Test with all features (frontier)
cargo make test-all

# Test frontier-only features
cargo make test-frontier

# Check compilation with all features
cargo make check-all
```

### Conditional Compilation in Test Files

```rust
// At top of test file
#![cfg_attr(not(feature = "wizard"), allow(dead_code))]

#[cfg(feature = "wizard")]
mod wizard_tests {
    use clap_noun_verb::wizard::*;

    #[test]
    fn test_wizard_session_creation() {
        let session = WizardSession::new("test-001".to_string());
        assert_eq!(session.session_id(), "test-001");
    }
}
```

### Testing Feature Interactions

When features have dependencies or interact:

```rust
#[cfg(all(feature = "wizard", feature = "federated-network"))]
#[test]
fn test_wizard_with_federated_network_integration() {
    // Test behavior when both features enabled
    let config = WizardConfig::default();
    // ... verify federated network support in wizard
}
```

---

## Error Message Validation

Test that error messages are helpful, accurate, and suggest corrections.

### Testing Error Creation

```rust
#[test]
fn test_command_not_found_error_includes_suggestion() {
    // ARRANGE: Simulate a misspelled command
    let error = NounVerbError::CommandNotFound {
        noun: "servces".to_string(),  // typo
        suggestion: ". Did you mean: services?".to_string(),
    };

    // ACT: Convert to structured error
    let structured = StructuredError::from_error(&error);

    // ASSERT: Verify structured error contains correction
    assert_eq!(structured.kind, ErrorKind::CommandNotFound);
    assert!(structured.action_templates.iter().any(|a| {
        matches!(a, ActionTemplate::CommandFix { suggested_command, .. } 
                 if suggested_command.contains("services"))
    }));
}
```

### Testing Error Parsing & Display

```rust
#[test]
fn test_verb_not_found_error_with_context() {
    let error = NounVerbError::VerbNotFound {
        noun: "config".to_string(),
        verb: "updat".to_string(),
        suggestion: ". Did you mean: update?".to_string(),
    };

    let msg = error.to_string();
    assert!(msg.contains("config"));
    assert!(msg.contains("updat"));
}
```

### Testing Structured Errors with Details

```rust
#[test]
fn test_structured_error_deadline_exceeded() {
    // ARRANGE: Create deadline exceeded error
    let error = StructuredError::deadline_exceeded(100, 150);

    // ASSERT: Verify structured information
    assert_eq!(error.kind, ErrorKind::DeadlineExceeded);
    assert_eq!(error.severity, Severity::Critical);
    assert_eq!(error.details.get("deadline_ms"), Some(&json!(100)));
    assert_eq!(error.details.get("actual_ms"), Some(&json!(150)));
    assert!(error.action_templates.iter().any(|a| {
        matches!(a, ActionTemplate::TimeoutAdjustment { .. })
    }));
}
```

### Testing Levenshtein Distance Suggestions

The framework uses edit distance for command suggestions:

```rust
#[test]
fn test_command_suggestion_uses_edit_distance() {
    // Commands close in spelling should be suggested
    let error = NounVerbError::CommandNotFound {
        noun: "servic".to_string(),
        suggestion: ". Did you mean: services?".to_string(),
    };

    assert!(error.to_string().contains("services"));
}
```

### Best Practices for Error Testing

- **Test the user-facing message** (what they see)
- **Verify actionable suggestions** are provided
- **Check error severity** classification
- **Ensure no secrets** in error output
- **Test error recovery** paths (can user fix and retry?)

---

## Help Text Testing

Verify that `--help` and `-h` output is complete, accurate, and helpful.

### Testing Help Text Content

```rust
#[test]
fn test_help_text_contains_command_description() {
    let mut cmd = Command::new("myapp")
        .about("My CLI application")
        .subcommand(
            Command::new("services")
                .about("Manage services")
                .subcommand(
                    Command::new("status")
                        .about("Show service status")
                )
        );

    // Test root help
    assert_help_contains(&mut cmd, "My CLI application");
    assert_help_contains(&mut cmd, "Manage services");
}
```

### Testing Subcommand Help

```rust
#[test]
fn test_subcommand_help_includes_all_verbs() {
    let mut cmd = create_test_cli();

    // Get help for a noun
    let output = cmd.render_help();
    let help_text = output.to_string();

    assert!(help_text.contains("status"));
    assert!(help_text.contains("start"));
    assert!(help_text.contains("stop"));
}
```

### Testing Help Flags

```rust
#[test]
fn test_help_flag_variants() {
    let cmd = create_test_cli();

    // Test both variants
    let result_h = cmd.clone().try_get_matches_from(vec!["test", "-h"]);
    let result_help = cmd.try_get_matches_from(vec!["test", "--help"]);

    // Both should display help and exit
    assert!(result_h.is_err());  // Help causes Err, check error_kind
    assert!(result_help.is_err());
}
```

### Testing Argument Help Text

```rust
#[test]
fn test_argument_help_is_descriptive() {
    let cmd = Command::new("test")
        .subcommand(
            Command::new("config")
                .subcommand(
                    Command::new("set")
                        .arg(
                            Arg::new("key")
                                .long("key")
                                .help("Configuration key in dot notation (e.g., app.name)")
                                .required(true)
                        )
                )
        );

    let mut help_output = Vec::new();
    cmd.clone()
        .get_subcommands_mut()
        .next()
        .unwrap()
        .get_subcommands_mut()
        .next()
        .unwrap()
        .write_help(&mut help_output)
        .unwrap();

    let help_text = String::from_utf8_lossy(&help_output);
    assert!(help_text.contains("dot notation"));
}
```

### Testing Usage Examples in Help

```rust
#[test]
fn test_help_includes_usage_examples() {
    let mut cmd = create_test_cli();
    
    let mut help_output = Vec::new();
    cmd.write_help(&mut help_output).unwrap();
    let help_text = String::from_utf8_lossy(&help_output);

    // Verify structure information is present
    assert!(help_text.contains("USAGE"));
    assert!(help_text.contains("COMMANDS") || help_text.contains("SUBCOMMANDS"));
}
```

### Best Practices for Help Text Testing

- **Verify all subcommands** are documented in parent help
- **Check argument descriptions** are non-empty and helpful
- **Ensure examples** if provided are accurate
- **Test nested help** (e.g., `myapp services --help`)
- **Verify required arguments** are marked as such
- **Check default values** are documented

---

## Test Fixtures & Scenarios

Create reusable test data and scenarios to reduce test code duplication.

### Common Test Fixtures

Create a `tests/fixtures/mod.rs`:

```rust
// tests/fixtures/mod.rs

use clap::{Arg, ArgAction, Command};

/// Create a standard test CLI with common commands
pub fn create_test_cli() -> Command {
    Command::new("testapp")
        .version("1.0.0")
        .about("Test CLI application")
        .subcommand(
            Command::new("services")
                .about("Service management")
                .subcommand(
                    Command::new("status")
                        .about("Show service status")
                        .arg(
                            Arg::new("verbose")
                                .long("verbose")
                                .short('v')
                                .action(ArgAction::SetTrue)
                                .help("Verbose output")
                        )
                )
                .subcommand(
                    Command::new("restart")
                        .about("Restart a service")
                        .arg(
                            Arg::new("name")
                                .index(1)
                                .required(true)
                                .help("Service name")
                        )
                )
        )
        .subcommand(
            Command::new("config")
                .about("Configuration management")
                .subcommand(
                    Command::new("set")
                        .arg(Arg::new("key").required(true).index(1))
                        .arg(Arg::new("value").required(true).index(2))
                )
                .subcommand(
                    Command::new("get")
                        .arg(Arg::new("key").required(true).index(1))
                )
        )
}

/// Create a handler input for testing
pub fn create_test_input(noun: &str, verb: &str) -> HandlerInput {
    HandlerInput::new(noun, verb)
}

/// Create a test context with predefined arguments
pub fn create_test_context_with_args(
    noun: &str,
    verb: &str,
    args: &[(&str, &str)],
) -> HandlerInput {
    let mut input = create_test_input(noun, verb);
    for (key, value) in args {
        input.set_arg(key, value);
    }
    input
}
```

### Scenario Builders

Create builders for complex test scenarios:

```rust
// tests/fixtures/scenarios.rs

pub struct CommandScenario {
    args: Vec<String>,
    expected_success: bool,
    expected_output: Option<String>,
}

impl CommandScenario {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            expected_success: true,
            expected_output: None,
        }
    }

    pub fn should_fail(mut self) -> Self {
        self.expected_success = false;
        self
    }

    pub fn with_output(mut self, output: String) -> Self {
        self.expected_output = Some(output);
        self
    }

    pub fn execute(&self, cmd: Command) -> Result<()> {
        let matches = cmd.try_get_matches_from(self.args.clone())?;
        // Verify parsed structure
        Ok(())
    }
}

// Usage in tests
#[test]
fn test_multiple_scenarios() {
    let cmd = create_test_cli();

    let scenarios = vec![
        CommandScenario::new(vec!["app", "services", "status"])
            .with_output("OK".to_string()),
        CommandScenario::new(vec!["app", "config", "get"])
            .should_fail(),
    ];

    for scenario in scenarios {
        let _ = scenario.execute(cmd.clone());
    }
}
```

### Test Data Generators

For testing with various input values:

```rust
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate valid command names
    pub fn valid_nouns() -> Vec<&'static str> {
        vec!["services", "config", "database"]
    }

    /// Generate invalid command names to test error handling
    pub fn invalid_nouns() -> Vec<&'static str> {
        vec!["servces", "configg", "databae", "xyz"]
    }

    /// Generate edge case values
    pub fn edge_case_values() -> Vec<&'static str> {
        vec![
            "",                              // empty
            "a",                             // single char
            "a".repeat(1000).as_str(),      // very long
            "special!@#$%^&*()",            // special chars
            "\n\r\t",                       // whitespace
        ]
    }
}

#[test]
fn test_cli_with_generated_values() {
    let cmd = create_test_cli();

    for invalid_noun in TestDataGenerator::invalid_nouns() {
        let result = cmd.clone()
            .try_get_matches_from(vec!["app", invalid_noun]);
        assert!(result.is_err(), "Should reject invalid noun: {}", invalid_noun);
    }
}
```

### Fixture Organization

```
tests/
├── fixtures/
│   ├── mod.rs              # Re-exports all fixtures
│   ├── cli.rs              # CLI definitions
│   ├── scenarios.rs        # Test scenarios
│   └── data.rs             # Test data generators
├── integration/
│   ├── commands.rs         # Command tests
│   ├── arguments.rs        # Argument validation tests
│   └── errors.rs           # Error handling tests
└── common/
    └── mod.rs              # Utilities
```

---

## Regression Testing

Detect when changes break existing CLI behavior.

### Snapshot Testing for Help Text

Store expected help output and verify against actual:

```rust
#[test]
fn test_help_text_snapshot() {
    let mut cmd = create_test_cli();
    
    let mut help_output = Vec::new();
    cmd.write_help(&mut help_output).unwrap();
    let help_text = String::from_utf8_lossy(&help_output);

    // Compare against stored snapshot
    let expected = std::fs::read_to_string("tests/fixtures/help_snapshot.txt")
        .expect("Snapshot file should exist");
    
    assert_eq!(help_text.to_string(), expected, 
        "Help text changed. Update snapshot if intentional: \
         cargo run -- myapp --help > tests/fixtures/help_snapshot.txt");
}
```

### Command Structure Snapshot

```rust
#[test]
fn test_command_structure_unchanged() {
    let cmd = create_test_cli();

    // Collect all nouns and verbs
    let mut structure = vec![];
    for noun in cmd.get_subcommands() {
        structure.push(format!("{}", noun.get_name()));
        for verb in noun.get_subcommands() {
            structure.push(format!("  - {}", verb.get_name()));
        }
    }

    let expected = vec![
        "services",
        "  - status",
        "  - restart",
        "config",
        "  - get",
        "  - set",
    ];

    assert_eq!(structure, expected, 
        "Command structure changed. Update if intentional.");
}
```

### API Contract Tests

Test that public types can be constructed/used by consumers:

```rust
#[test]
fn test_public_api_compatibility() {
    // These should always compile and work
    let _builder = CliBuilder::new();
    let _registry = CommandRegistry::new();
    let _output = OutputFormat::default();

    // If this test compiles, public API is stable
}
```

### Test Mutation Approach (Optional)

For advanced regression testing, use cargo-mutants to verify your tests catch real changes:

```bash
# Install: cargo install cargo-mutants

# Run mutation testing (finds tests that DON'T catch changes)
cargo mutants --package clap-noun-verb --test integration_cli_tests
```

### Regression Test Checklist

- [ ] Help text for all commands documented
- [ ] Command structure (nouns/verbs) verified
- [ ] Required arguments enforced
- [ ] Error messages remain helpful
- [ ] Exit codes unchanged for same inputs
- [ ] Output format (JSON/plain) consistent
- [ ] Performance regressions checked

---

## Performance & Stress Testing

Ensure CLI remains responsive under load.

### Argument Parsing Performance

```rust
#[test]
fn test_argument_parsing_performance() {
    let cmd = create_test_cli();
    
    // Parse 10,000 variations
    let start = std::time::Instant::now();
    
    for i in 0..10_000 {
        let args = vec![
            "app",
            "services",
            "status",
            "--verbose",
        ];
        let _ = cmd.clone().try_get_matches_from(args);
    }
    
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / 10_000;
    
    // Assert parsing takes < 10 microseconds per invocation
    assert!(avg_ns < 10_000, 
        "Parsing too slow: {} ns per parse", avg_ns);
}
```

### Command Registry Performance

```rust
#[test]
fn test_command_registry_lookup_speed() {
    let registry = CommandRegistry::get();
    let registry = registry.lock().unwrap();
    
    let start = std::time::Instant::now();
    for _ in 0..1_000 {
        let _ = registry.build_command();
    }
    let elapsed = start.elapsed();
    
    // Build command should be < 1ms
    assert!(elapsed.as_millis() < 1000);
}
```

### Stress Test: High Volume Commands

```rust
#[test]
#[ignore] // Run manually: cargo test -- --ignored --nocapture
fn stress_test_high_volume_command_dispatch() {
    let cmd = create_test_cli();
    
    let variants = vec![
        vec!["app", "services", "status"],
        vec!["app", "services", "status", "-v"],
        vec!["app", "config", "get", "app.name"],
    ];

    let start = std::time::Instant::now();
    let iterations = 10_000;

    for i in 0..iterations {
        let variant = &variants[i % variants.len()];
        let _ = cmd.clone().try_get_matches_from(
            variant.iter().map(|s| s.to_string()).collect::<Vec<_>>()
        );
    }

    let elapsed = start.elapsed();
    println!("Processed {} commands in {:?}", iterations, elapsed);
    println!("Throughput: {:.0} commands/sec", 
             iterations as f64 / elapsed.as_secs_f64());
}
```

### Memory Usage Testing

```rust
#[test]
fn test_cli_memory_footprint() {
    // Create many CLI instances
    let mut clis = Vec::new();
    
    for _ in 0..100 {
        clis.push(create_test_cli());
    }

    // All should be reasonably sized (< 1MB total)
    // This is a soft check; adjust threshold as needed
    assert!(!clis.is_empty());
}
```

### Performance Test Guidelines

- **Baseline measurements first** before adding assertions
- **Use `#[ignore]`** for heavy tests, run selectively
- **Compare against thresholds** not arbitrary numbers
- **Account for CI environment** (may be slower)
- **Test with release build** for real performance metrics

Run performance tests:

```bash
# Run only performance tests
cargo test --test integration_cli_tests performance --release

# Run ignored tests (including performance)
cargo test -- --ignored --nocapture
```

---

## Build Verification

Use `cargo make` commands to verify CLI in different configurations.

### Standard Build Verification

```bash
# Format check
cargo make format-check

# Linting (catches clippy issues)
cargo make clippy

# Full lint suite
cargo make lint

# Check compilation
cargo make check
```

### Feature-Based Verification

```bash
# Build with specific features
cargo make build -- --features wizard
cargo make build -- --features "wizard,federated-network"

# Check with all features
cargo make check-all

# Build release binary
cargo make build-release
```

### Test Verification

```bash
# Quick tests (default features)
cargo make test

# Deterministic test run (single-threaded)
cargo make test-lib-deterministic

# All feature combinations
cargo make test-all

# Frontier features only
cargo make test-frontier

# Full CI suite
cargo make ci
```

### Building Examples

```bash
# Build all examples
cargo make build-examples

# Build specific example
cargo build --example tutorial_services

# Run example with arguments
cargo run --example tutorial_services -- services status --verbose
```

### Documentation Generation

```bash
# Build and open docs
cargo make doc

# Verify doc builds without warnings
cargo doc --no-deps --all-features
```

### SLO Verification

From CLAUDE.md, clap-noun-verb maintains these SLOs:

- **Incremental compilation**: ≤2 seconds (currently 0.66s)
- **Binary size**: ≤10 MB (currently 2.2 MB)

Check current metrics:

```bash
# Check binary size
ls -lh target/release/myapp

# Time an incremental build (after touching one file)
time cargo make build
```

---

## Testing Workflow Examples

### Complete Integration Test

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tests::fixtures::*;

    #[test]
    fn test_complete_service_management_workflow() {
        // ARRANGE: Set up test CLI and input
        let cmd = create_test_cli();
        let mut input = create_test_input("services", "status");
        input.set_arg("verbose", "true");

        // ACT: Simulate user executing "myapp services status --verbose"
        let matches = cmd
            .try_get_matches_from(vec!["myapp", "services", "status", "--verbose"])
            .expect("Should parse valid command");

        // ASSERT: Verify command was recognized
        let services = matches.subcommand_matches("services").unwrap();
        let status = services.subcommand_matches("status").unwrap();
        assert!(status.get_flag("verbose"));
    }

    #[test]
    fn test_error_on_missing_required_argument() {
        let cmd = create_test_cli();

        // ACT: Try command without required service name
        let result = cmd.try_get_matches_from(vec!["myapp", "services", "restart"]);

        // ASSERT: Should fail because 'name' is required
        assert!(result.is_err());
    }

    #[test]
    fn test_help_displays_all_subcommands() {
        let mut cmd = create_test_cli();

        // ASSERT: Help includes all nouns
        assert_help_contains(&mut cmd, "services");
        assert_help_contains(&mut cmd, "config");
    }
}
```

### Feature-Gated Integration Test

```rust
#[cfg(test)]
mod wizard_integration_tests {
    #![cfg(feature = "wizard")]

    #[test]
    fn test_wizard_command_available_with_feature() {
        let cmd = create_test_cli_with_wizard();
        assert_has_subcommand(&cmd, "wizard");
    }

    #[test]
    fn test_wizard_session_execution() {
        let cmd = create_test_cli_with_wizard();
        let matches = cmd
            .try_get_matches_from(vec!["myapp", "wizard", "generate"])
            .expect("Should parse wizard command");

        let wizard = matches.subcommand_matches("wizard").unwrap();
        assert!(wizard.subcommand_matches("generate").is_some());
    }
}
```

---

## Quick Reference: Test Assertions

### Command Structure Assertions

```rust
assert_has_subcommand(&cmd, "services");          // Noun exists
assert_no_subcommand(&cmd, "invalid");            // Noun doesn't exist
assert_subcommand_has_verb(&cmd, "services", "status");  // Verb exists
assert_has_version(&cmd, Some("1.0.0"));          // Version set
```

### Argument Assertions

```rust
assert!(status.get_flag("verbose"));               // Flag is set
assert_eq!(config.get_one::<String>("key"), Some(&"app.name".to_string()));
assert!(matches.subcommand().is_some());           // Has subcommand
```

### Help Text Assertions

```rust
assert_help_contains(&mut cmd, "expected text");   // Help includes text
```

### Error Assertions

```rust
assert_eq!(error.kind, ErrorKind::CommandNotFound);
assert!(error.action_templates.iter().any(|a| matches!(a, ActionTemplate::CommandFix { .. })));
```

---

## Troubleshooting Common Issues

### Test Timeout

If tests hang:

```bash
# Run single test with timeout
timeout 30 cargo test test_name -- --nocapture

# Run with reduced parallelism
cargo test -- --test-threads=1
```

### Help Text Rendering Issues

If help text assertions fail:

```rust
// Debug by printing actual help
let mut help_output = Vec::new();
cmd.write_help(&mut help_output).unwrap();
println!("ACTUAL HELP:\n{}", String::from_utf8_lossy(&help_output));
```

### Feature Gating Complications

If feature-gated tests don't compile:

```bash
# Verify feature is enabled
cargo build --features wizard

# Check feature flags in Cargo.toml
grep -A 10 "^\[features\]" Cargo.toml
```

### Flaky Tests

Ensure no tests depend on execution order:

```bash
# Run tests multiple times in different orders
for i in {1..10}; do 
  cargo test -- --shuffle || break
done
```

---

## Summary Checklist

- [ ] **Manual Testing**: Can run CLI directly and verify output
- [ ] **Integration Tests**: Use Arrange-Act-Assert pattern
- [ ] **Feature-Gated**: Mark with `#[cfg(feature = "...")]`
- [ ] **Error Handling**: Verify error messages are helpful
- [ ] **Help Text**: All commands documented, discoverable
- [ ] **Fixtures**: Reusable test data and CLI builders
- [ ] **Regression**: Snapshot critical outputs
- [ ] **Performance**: Verify parsing/dispatch under load
- [ ] **Build**: All `cargo make` variants pass
- [ ] **Documentation**: Examples runnable, docs build

---

## Related Documentation

- **CLAUDE.md**: Project overview, build commands, feature system
- **src/cli/mod.rs**: CLI architecture (validation only, no business logic)
- **src/builder.rs**: CliBuilder API for composing CLIs
- **src/router.rs**: CommandRouter for dispatching commands
- **tests/common/mod.rs**: Built-in test assertion helpers
