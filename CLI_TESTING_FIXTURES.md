# CLI Testing Fixtures & Utilities

Reusable patterns for building test CLIs, fixtures, and common testing utilities.

## File Structure for Test Fixtures

```
tests/
├── fixtures/
│   ├── mod.rs              # Re-exports all fixtures
│   ├── cli_builders.rs     # CLI construction helpers
│   ├── test_data.rs        # Test data generators
│   ├── scenarios.rs        # Test scenario builders
│   └── helpers.rs          # Common assertion helpers
├── common/
│   └── mod.rs              # Shared utilities (already exists)
├── integration/
│   ├── commands.rs         # Command structure tests
│   ├── arguments.rs        # Argument validation tests
│   └── errors.rs           # Error handling tests
└── snapshots/
    ├── help_snapshot.txt   # Stored help output
    └── structure.json      # Command structure snapshot
```

## CLI Fixture Builders

### Basic CLI Builder

```rust
// tests/fixtures/cli_builders.rs

use clap::{Arg, ArgAction, Command};

/// Create a minimal test CLI with services and config nouns
pub fn create_basic_test_cli() -> Command {
    Command::new("testapp")
        .version("0.1.0")
        .about("Test application")
        .subcommand(
            Command::new("services")
                .about("Service management")
                .subcommand(Command::new("status").about("Show status"))
                .subcommand(Command::new("start").about("Start service"))
                .subcommand(Command::new("stop").about("Stop service"))
        )
        .subcommand(
            Command::new("config")
                .about("Configuration management")
                .subcommand(
                    Command::new("get")
                        .about("Get config value")
                        .arg(Arg::new("key").index(1).required(true))
                )
                .subcommand(
                    Command::new("set")
                        .about("Set config value")
                        .arg(Arg::new("key").index(1).required(true))
                        .arg(Arg::new("value").index(2).required(true))
                )
        )
}

/// Create a CLI with global arguments
pub fn create_cli_with_globals() -> Command {
    create_basic_test_cli()
        .arg(
            Arg::new("verbose")
                .short('v')
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Verbose output")
        )
        .arg(
            Arg::new("config-file")
                .long("config")
                .global(true)
                .help("Path to config file")
        )
}

/// Create a CLI with complex argument handling
pub fn create_cli_with_complex_args() -> Command {
    Command::new("complex")
        .subcommand(
            Command::new("process")
                .arg(
                    Arg::new("files")
                        .short('f')
                        .long("file")
                        .action(ArgAction::Append)
                        .help("Files to process (can use multiple times)")
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "yaml", "toml"])
                        .help("Output format")
                )
                .arg(
                    Arg::new("parallel")
                        .short('p')
                        .action(ArgAction::SetTrue)
                        .help("Process in parallel")
                )
        )
}

/// Create feature-gated CLI (for testing feature compilation)
#[cfg(feature = "experimental")]
pub fn create_experimental_cli() -> Command {
    create_basic_test_cli()
        .subcommand(Command::new("experimental"))
}
```

### Advanced CLI Builders

```rust
// tests/fixtures/cli_builders.rs (continued)

/// Builder pattern for constructing test CLIs
pub struct CliBuilder {
    name: String,
    nouns: Vec<(String, Vec<String>)>,  // (noun, [verbs])
    global_args: Vec<&'static str>,
}

impl CliBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nouns: Vec::new(),
            global_args: Vec::new(),
        }
    }

    /// Add a noun with verbs
    pub fn noun(mut self, name: &str, verbs: Vec<&str>) -> Self {
        self.nouns.push((
            name.to_string(),
            verbs.iter().map(|v| v.to_string()).collect(),
        ));
        self
    }

    /// Add global arguments
    pub fn with_globals(mut self, args: Vec<&'static str>) -> Self {
        self.global_args = args;
        self
    }

    /// Build the Command
    pub fn build(self) -> Command {
        let mut cmd = Command::new(&self.name);

        // Add global arguments
        for arg in self.global_args {
            cmd = cmd.arg(
                Arg::new(arg)
                    .long(arg)
                    .global(true)
            );
        }

        // Add nouns and verbs
        for (noun_name, verbs) in self.nouns {
            let mut noun_cmd = Command::new(&noun_name);
            for verb in verbs {
                noun_cmd = noun_cmd.subcommand(Command::new(verb));
            }
            cmd = cmd.subcommand(noun_cmd);
        }

        cmd
    }
}

// Usage example
#[test]
fn test_with_builder_pattern() {
    let cli = CliBuilder::new("app")
        .noun("users", vec!["list", "create", "delete"])
        .noun("roles", vec!["list", "assign"])
        .with_globals(vec!["verbose", "config"])
        .build();

    let matches = cli
        .try_get_matches_from(vec!["app", "--verbose", "users", "list"])
        .unwrap();

    assert!(matches.get_flag("verbose"));
}
```

## Test Data Generators

### Value Generators

```rust
// tests/fixtures/test_data.rs

/// Generate test values for various argument types
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Valid command names
    pub fn valid_nouns() -> Vec<&'static str> {
        vec!["services", "config", "users", "roles", "database"]
    }

    /// Invalid command names (typos, made-up)
    pub fn invalid_nouns() -> Vec<&'static str> {
        vec!["servces", "configg", "usr", "invalid", "xyz"]
    }

    /// Valid verbs
    pub fn valid_verbs() -> Vec<&'static str> {
        vec!["list", "create", "delete", "update", "status", "get", "set"]
    }

    /// Edge case string values
    pub fn edge_case_strings() -> Vec<&'static str> {
        vec![
            "",                           // empty
            "a",                          // single char
            "a".repeat(1000).leak(),     // very long
            "special!@#$%^&*()",         // special chars
            "with spaces",               // spaces
            "with\ttabs",                // tabs
            "with\nnewline",             // newlines
            "unicode: 你好",            // unicode
        ]
    }

    /// Valid numeric values
    pub fn valid_numbers() -> Vec<u32> {
        vec![0, 1, 42, 1000, 65535, u32::MAX]
    }

    /// Invalid numeric values
    pub fn invalid_numbers() -> Vec<&'static str> {
        vec!["abc", "-1", "3.14", "99999999999999999"]
    }

    /// Valid port numbers
    pub fn valid_ports() -> Vec<u16> {
        vec![1, 80, 443, 3000, 8080, 9000, 65535]
    }

    /// Invalid ports
    pub fn invalid_ports() -> Vec<&'static str> {
        vec!["0", "-1", "65536", "99999"]
    }

    /// Valid JSON values
    pub fn valid_json() -> Vec<&'static str> {
        vec![
            "{}",
            r#"{"key": "value"}"#,
            "[]",
            "[1, 2, 3]",
            r#"{"nested": {"key": "value"}}"#,
        ]
    }

    /// Invalid JSON
    pub fn invalid_json() -> Vec<&'static str> {
        vec![
            "{",
            "}",
            r#"{"key": value}"#,  // unquoted value
            "[1, 2, 3",           // unclosed
        ]
    }
}

#[test]
fn test_with_generated_data() {
    let cli = create_basic_test_cli();

    for invalid in TestDataGenerator::invalid_nouns() {
        let result = cli.clone()
            .try_get_matches_from(vec!["app", invalid]);
        assert!(result.is_err(), "Should reject invalid noun: {}", invalid);
    }
}
```

### Scenario Builders

```rust
// tests/fixtures/scenarios.rs

use crate::fixtures::cli_builders::*;
use clap::Command;

/// Represents a test scenario with expected results
pub struct CommandScenario {
    pub args: Vec<String>,
    pub should_succeed: bool,
    pub expected_subcommand: Option<String>,
    pub description: String,
}

impl CommandScenario {
    pub fn new(args: Vec<&str>, description: &str) -> Self {
        Self {
            args: args.iter().map(|s| s.to_string()).collect(),
            should_succeed: true,
            expected_subcommand: None,
            description: description.to_string(),
        }
    }

    pub fn should_fail(mut self) -> Self {
        self.should_succeed = false;
        self
    }

    pub fn expecting_subcommand(mut self, cmd: &str) -> Self {
        self.expected_subcommand = Some(cmd.to_string());
        self
    }

    /// Execute scenario and verify expectations
    pub fn verify(&self, cmd: Command) -> Result<(), String> {
        let result = cmd.try_get_matches_from(self.args.clone());

        if self.should_succeed {
            let matches = result.map_err(|e| format!(
                "Scenario '{}' should succeed but failed: {}",
                self.description, e
            ))?;

            if let Some(expected_cmd) = &self.expected_subcommand {
                if matches.subcommand_matches(expected_cmd).is_none() {
                    return Err(format!(
                        "Scenario '{}' expected subcommand '{}' but not found",
                        self.description, expected_cmd
                    ));
                }
            }

            Ok(())
        } else {
            if result.is_ok() {
                return Err(format!(
                    "Scenario '{}' should fail but succeeded",
                    self.description
                ));
            }
            Ok(())
        }
    }
}

/// Common test scenarios for any noun-verb CLI
pub fn standard_scenarios() -> Vec<CommandScenario> {
    vec![
        CommandScenario::new(
            vec!["app", "services", "status"],
            "Valid command with no arguments"
        ),
        CommandScenario::new(
            vec!["app", "config", "get", "app.name"],
            "Valid command with positional argument"
        ).expecting_subcommand("config"),
        CommandScenario::new(
            vec!["app", "invalid"],
            "Invalid noun should fail"
        ).should_fail(),
        CommandScenario::new(
            vec!["app", "services", "invalid"],
            "Invalid verb should fail"
        ).should_fail(),
    ]
}

#[test]
fn test_scenarios() {
    let cli = create_basic_test_cli();
    for scenario in standard_scenarios() {
        scenario.verify(cli.clone())
            .expect(&format!("Scenario failed: {}", scenario.description));
    }
}
```

## Assertion Helpers

```rust
// tests/fixtures/helpers.rs

use clap::Command;

/// Assert that help text was rendered
pub fn assert_help_was_displayed(result: &Result<clap::ArgMatches, clap::error::Error>) {
    assert!(
        result.is_err(),
        "Help flag should cause error (which triggers help display)"
    );

    if let Err(e) = result {
        // Help/version errors have specific kinds
        match e.kind() {
            clap::error::ErrorKind::DisplayHelp |
            clap::error::ErrorKind::DisplayVersion => {}
            _ => panic!("Expected help display, got: {:?}", e.kind()),
        }
    }
}

/// Collect all available nouns from a CLI
pub fn extract_nouns(cmd: &Command) -> Vec<String> {
    cmd.get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect()
}

/// Collect all verbs under a noun
pub fn extract_verbs(cmd: &Command, noun: &str) -> Vec<String> {
    cmd.get_subcommands()
        .find(|c| c.get_name() == noun)
        .map(|noun_cmd| {
            noun_cmd
                .get_subcommands()
                .map(|v| v.get_name().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Verify a command name doesn't exist
pub fn assert_command_not_found(cmd: &Command, name: &str) {
    assert!(
        !cmd.get_subcommands().any(|s| s.get_name() == name),
        "Command '{}' should not exist",
        name
    );
}

/// Count subcommands at a level
pub fn count_nouns(cmd: &Command) -> usize {
    cmd.get_subcommands().count()
}

pub fn count_verbs(cmd: &Command, noun: &str) -> usize {
    cmd.get_subcommands()
        .find(|c| c.get_name() == noun)
        .map(|n| n.get_subcommands().count())
        .unwrap_or(0)
}

#[test]
fn test_with_helpers() {
    let cli = create_basic_test_cli();

    // Test noun extraction
    let nouns = extract_nouns(&cli);
    assert!(nouns.contains(&"services".to_string()));
    assert!(nouns.contains(&"config".to_string()));

    // Test verb extraction
    let verbs = extract_verbs(&cli, "services");
    assert!(verbs.contains(&"status".to_string()));

    // Test counts
    assert_eq!(count_nouns(&cli), 2);
    assert_eq!(count_verbs(&cli, "services"), 3);
}
```

## Snapshot Storage

### Help Text Snapshot

```text
# tests/snapshots/help_snapshot.txt

testapp 0.1.0
Test application

USAGE:
    testapp [OPTIONS] [COMMAND]

COMMANDS:
    services    Service management
    config      Configuration management
    help        Print this message or the help of the given subcommand(s)

OPTIONS:
    -h, --help       Print help
    -V, --version    Print version
```

### Structure Snapshot

```json
// tests/snapshots/structure.json

{
  "name": "testapp",
  "version": "0.1.0",
  "nouns": [
    {
      "name": "services",
      "description": "Service management",
      "verbs": [
        {"name": "status", "description": "Show status"},
        {"name": "start", "description": "Start service"},
        {"name": "stop", "description": "Stop service"}
      ]
    },
    {
      "name": "config",
      "description": "Configuration management",
      "verbs": [
        {"name": "get", "description": "Get config value"},
        {"name": "set", "description": "Set config value"}
      ]
    }
  ]
}
```

## Comprehensive Test Template

```rust
// tests/integration/complete_example.rs

#[cfg(test)]
mod complete_cli_test {
    use crate::fixtures::*;

    #[test]
    fn test_cli_discovery() {
        // ARRANGE
        let cli = create_basic_test_cli();

        // ACT
        let nouns = extract_nouns(&cli);

        // ASSERT
        assert_eq!(nouns.len(), 2);
        assert!(nouns.contains(&"services".to_string()));
    }

    #[test]
    fn test_valid_commands() {
        let cli = create_basic_test_cli();

        for scenario in standard_scenarios() {
            scenario.verify(cli.clone())
                .expect(&scenario.description);
        }
    }

    #[test]
    fn test_invalid_commands() {
        let cli = create_basic_test_cli();

        let invalid_scenarios = vec![
            CommandScenario::new(
                vec!["app", "invalid"],
                "Invalid noun"
            ).should_fail(),
            CommandScenario::new(
                vec!["app", "services", "invalid"],
                "Invalid verb"
            ).should_fail(),
        ];

        for scenario in invalid_scenarios {
            scenario.verify(cli.clone())
                .expect(&scenario.description);
        }
    }

    #[test]
    fn test_help_displays() {
        let cli = create_basic_test_cli();

        let result = cli.try_get_matches_from(vec!["app", "--help"]);
        assert_help_was_displayed(&result);
    }
}
```

## Integration with Existing Test Structure

Add to `tests/common/mod.rs`:

```rust
// Re-export fixtures
pub mod fixtures {
    pub mod cli_builders {
        pub use crate::fixtures::cli_builders::*;
    }
    pub mod test_data {
        pub use crate::fixtures::test_data::*;
    }
    pub mod scenarios {
        pub use crate::fixtures::scenarios::*;
    }
    pub mod helpers {
        pub use crate::fixtures::helpers::*;
    }
}
```

Then in tests:

```rust
use tests::common::fixtures::*;

#[test]
fn my_test() {
    let cli = cli_builders::create_basic_test_cli();
    // ...
}
```

## Performance Fixture for Load Testing

```rust
// tests/fixtures/performance.rs

pub struct PerformanceFixture {
    pub iterations: usize,
    pub timeout_ms: u128,
}

impl PerformanceFixture {
    pub fn default_load() -> Self {
        Self {
            iterations: 10_000,
            timeout_ms: 5000,  // 5 seconds for all iterations
        }
    }

    pub fn stress_test() -> Self {
        Self {
            iterations: 100_000,
            timeout_ms: 30000,
        }
    }

    pub fn quick_smoke() -> Self {
        Self {
            iterations: 100,
            timeout_ms: 100,
        }
    }
}

#[test]
#[ignore]  // Run manually: cargo test -- --ignored
fn stress_test_parsing_performance() {
    let fixture = PerformanceFixture::stress_test();
    let cli = create_basic_test_cli();

    let start = std::time::Instant::now();

    for i in 0..fixture.iterations {
        let _ = cli.clone()
            .try_get_matches_from(vec!["app", "services", "status"]);
    }

    let elapsed = start.elapsed();
    println!("Completed {} parses in {:?}", fixture.iterations, elapsed);
    
    assert!(
        elapsed.as_millis() < fixture.timeout_ms,
        "Performance degradation: {} > {} ms",
        elapsed.as_millis(),
        fixture.timeout_ms
    );
}
```

## Maintenance Tips

1. **Keep fixtures simple** - One responsibility per fixture
2. **Document generators** - Show what data is produced
3. **Update snapshots** - When intentional changes made:
   ```bash
   cargo run -- app --help > tests/snapshots/help_snapshot.txt
   ```
4. **Version fixtures** - If breaking changes, version them
5. **Share patterns** - Document and reuse across projects
