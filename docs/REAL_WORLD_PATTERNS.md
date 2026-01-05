# clap-noun-verb Real-World Patterns & Best Practices

**Version**: 5.3.4
**Study Date**: 2026-01-05
**Codebase Lines**: ~4,245 lines (src/)

## Table of Contents

1. [CLI Design Patterns](#1-cli-design-patterns)
2. [Error Handling Patterns](#2-error-handling-patterns)
3. [Testing Patterns](#3-testing-patterns)
4. [Documentation Patterns](#4-documentation-patterns)
5. [Performance Patterns](#5-performance-patterns)
6. [Code Organization](#6-code-organization)
7. [Anti-Patterns to Avoid](#7-anti-patterns-to-avoid)
8. [Advanced Use Cases](#8-advanced-use-cases)
9. [Learning Path](#9-learning-path)
10. [Quick Reference Guide](#10-quick-reference-guide)

---

## 1. CLI Design Patterns

### 1.1 Attribute Macro Pattern (Recommended)

**Pattern**: Use `#[verb]` attributes for zero-boilerplate command registration.

```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    healthy: bool,
    services: Vec<String>,
}

/// Show service status
#[verb("status", "services")]
fn show_status() -> Result<Status> {
    Ok(Status {
        healthy: true,
        services: vec!["api".into(), "worker".into()],
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands
}
```

**When to use**:
- ✅ New projects (v5.3+)
- ✅ Simple to medium complexity CLIs
- ✅ When you want automatic type inference
- ✅ When you need JSON output by default

**Advantages**:
- Zero boilerplate
- Compile-time command discovery via `linkme`
- Automatic type inference from function signatures
- JSON output by default

### 1.2 Root-Level Verbs Pattern (v5.3.4+)

**Pattern**: Commands without noun prefix using `#[verb("name", "root")]`.

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct SyncOutput {
    status: String,
    files_synced: usize,
}

/// Synchronize files (appears as `mycli sync`)
#[verb("sync", "root")]
fn sync() -> clap_noun_verb::Result<SyncOutput> {
    Ok(SyncOutput {
        status: "success".to_string(),
        files_synced: 42,
    })
}
```

**When to use**:
- ✅ Common operations that don't fit noun hierarchy
- ✅ Top-level utilities (sync, init, version)
- ✅ Quick actions (build, test, deploy)

**Example CLI structure**:
```
mycli sync          # Root verb
mycli services status   # Noun-verb
mycli config get       # Noun-verb
```

### 1.3 Multiple Nouns Pattern

**Pattern**: Multiple command groups in single file with explicit noun names.

```rust
/// Show service status
#[verb("status", "services")] // Explicit noun
fn show_service_status() -> Result<ServiceStatus> { ... }

/// Show logs
#[verb("logs", "services")] // Explicit noun
fn show_service_logs(service: String) -> Result<Logs> { ... }

/// Start collector
#[verb("up", "collector")] // Different noun
fn start_collector() -> Result<CollectorStatus> { ... }
```

**When to use**:
- ✅ Related command groups in one file
- ✅ Medium-sized CLIs (5-15 commands)
- ❌ Large CLIs (prefer file-per-noun organization)

### 1.4 Nested Command Structure Pattern

**Pattern**: Organize commands hierarchically for complex CLIs.

```rust
/// Run tests
#[verb("run", "test")]
fn run_tests() -> Result<TestResult> { ... }

/// Check linting
#[verb("check", "lint")]
fn check_lint() -> Result<LintResult> { ... }

/// Check formatting
#[verb("check", "format")]
fn check_format() -> Result<FormatResult> { ... }
```

**CLI structure**:
```
mycli test run
mycli test watch
mycli lint check
mycli lint fix
mycli format check
mycli format apply
```

**When to use**:
- ✅ Complex CLIs with 20+ commands
- ✅ Multiple related operations per domain
- ✅ Clear conceptual hierarchy

### 1.5 Business Logic Separation Pattern

**Pattern**: Separate pure business logic from CLI layer.

```rust
// ===== Business Logic Layer (Pure Functions) =====
fn get_service_status() -> ServiceStatus {
    ServiceStatus {
        services: vec!["web".into(), "db".into()],
        healthy: true,
    }
}

// ===== CLI Layer (Validation + Delegation) =====
/// Show status of all services
#[verb("status", "services")]
fn show_status() -> Result<ServiceStatus> {
    // 1. Validate inputs (none here)
    // 2. Delegate to business logic
    Ok(get_service_status())
    // 3. Output shaping (auto JSON)
}
```

**Responsibilities**:

**Business Logic**:
- ✅ Pure functions (no I/O)
- ✅ Domain types and logic
- ✅ Testable independently
- ❌ No CLI dependencies

**CLI Layer**:
- ✅ Argument parsing and validation
- ✅ Delegation to business logic
- ✅ Output formatting
- ❌ Minimal business logic

---

## 2. Error Handling Patterns

### 2.1 Custom Error Type Pattern

**Pattern**: Use `thiserror` for custom error types with helpful messages.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NounVerbError {
    #[error("Command '{noun}' not found")]
    CommandNotFound { noun: String },

    #[error("Verb '{verb}' not found for noun '{noun}'")]
    VerbNotFound { noun: String, verb: String },

    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

pub type Result<T> = std::result::Result<T, NounVerbError>;
```

**Best practices**:
- ✅ Use named fields for contextual errors
- ✅ Provide clear, actionable error messages
- ✅ Include relevant context (noun, verb names)
- ✅ Use `{0}` for simple string errors

### 2.2 Builder Pattern for Errors

**Pattern**: Provide ergonomic constructors for common errors.

```rust
impl NounVerbError {
    pub fn command_not_found(noun: impl Into<String>) -> Self {
        Self::CommandNotFound { noun: noun.into() }
    }

    pub fn verb_not_found(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self::VerbNotFound { noun: noun.into(), verb: verb.into() }
    }

    pub fn validation_range_error(
        name: impl Into<String>,
        value: impl Into<String>,
        min: Option<&str>,
        max: Option<&str>,
    ) -> Self {
        let constraint = match (min, max) {
            (Some(min), Some(max)) => format!("Must be between {} and {}", min, max),
            (Some(min), None) => format!("Must be >= {}", min),
            (None, Some(max)) => format!("Must be <= {}", max),
            (None, None) => "Invalid value".to_string(),
        };
        Self::ValidationFailed(constraint)
    }
}
```

**Usage**:
```rust
return Err(NounVerbError::command_not_found("services"));
return Err(NounVerbError::validation_range_error("age", "5", Some("18"), Some("120")));
```

### 2.3 Result Type Pattern

**Pattern**: Use type alias for consistent Result types.

```rust
// In error.rs
pub type Result<T> = std::result::Result<T, NounVerbError>;

// Usage throughout codebase
fn show_status() -> Result<Status> { ... }
fn get_logs(service: String) -> Result<Logs> { ... }
```

**Benefits**:
- ✅ Consistent error handling
- ✅ Less typing
- ✅ Easy to change error type globally

### 2.4 Context-Rich Error Messages

**Pattern**: Include actionable context in errors.

```rust
// ❌ Bad: Vague error
Err(NounVerbError::Generic("Failed".into()))

// ✅ Good: Specific with context
Err(NounVerbError::ArgumentError {
    message: format!(
        "Invalid value '{}' for argument '{}'. Must be between {} and {}",
        value, name, min, max
    )
})

// ✅ Even better: Helper method
Err(NounVerbError::validation_range_error("age", "5", Some("18"), Some("120")))
```

---

## 3. Testing Patterns

### 3.1 Chicago TDD Pattern (State-Based Testing)

**Pattern**: Test observable outputs and state changes, not implementation.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_success() {
        // Arrange - Set up initial state
        let input = Input {
            data: "hello world".to_string(),
        };

        // Act - Execute the operation
        let output = process(input).unwrap();

        // Assert - Verify observable outputs
        assert_eq!(output.result, "PROCESSED: HELLO WORLD");
        assert_eq!(output.metadata.processed_length, 22);
        assert_eq!(output.metadata.transformations, 1);
    }
}
```

**Key principles**:
- ✅ **AAA pattern**: Arrange-Act-Assert
- ✅ **State-based**: Verify return values and state changes
- ✅ **Real collaborators**: Use real objects, minimize mocks
- ✅ **Observable behavior**: Test what code does, not how
- ❌ **No implementation details**: Don't test private methods

### 3.2 Error Path Testing Pattern

**Pattern**: Test error conditions and edge cases.

```rust
#[test]
fn test_process_empty_input_fails() {
    // Arrange
    let input = Input {
        data: "".to_string(),
    };

    // Act
    let result = process(input);

    // Assert - Verify specific error type and message
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        DomainError::ValidationFailed("Input data cannot be empty".to_string())
    );
}

#[test]
fn test_process_large_input_fails() {
    // Arrange
    let input = Input {
        data: "x".repeat(1_000_001),
    };

    // Act
    let result = process(input);

    // Assert - Use pattern matching for flexibility
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DomainError::ValidationFailed(_)));
}
```

### 3.3 Integration Test Pattern

**Pattern**: Test CLI commands end-to-end with real arguments.

```rust
#[test]
fn test_basic_noun_verb_cli() -> Result<()> {
    let cli = app! {
        name: "test-app",
        about: "Test CLI application",
        commands: [
            noun!("services", "Manage services", [
                verb!("status", "Show status", |_args: &VerbArgs| {
                    println!("Services are running");
                    Ok(())
                }),
            ]),
        ]
    };

    let command = cli.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "services"));

    Ok(())
}
```

### 3.4 Domain Separation Testing Pattern

**Pattern**: Test business logic independently of CLI.

```rust
// Test business logic directly (no CLI dependencies)
#[test]
fn test_business_logic_independence() {
    let status = get_service_status(); // Pure function
    assert_eq!(status.healthy, true);
    assert_eq!(status.services.len(), 2);
}

// Test CLI delegation
#[test]
fn test_cli_delegates_correctly() -> Result<()> {
    let cli_output = show_status()?;
    let business_output = get_service_status();

    assert_eq!(cli_output.services, business_output.services);
    assert_eq!(cli_output.healthy, business_output.healthy);
    Ok(())
}
```

### 3.5 File I/O Testing Pattern

**Pattern**: Use `tempfile` for testing file operations.

```rust
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_process_file_success() {
    // Arrange - Create temp file
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "test data").unwrap();

    // Act
    let result = process_file(input_file.path().to_path_buf());

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_process_file_missing_file_fails() {
    // Arrange
    let input_path = PathBuf::from("/nonexistent/file.txt");

    // Act
    let result = process_file(input_path);

    // Assert
    assert!(result.is_err());
    let err_msg = format!("{:#}", result.unwrap_err());
    assert!(err_msg.contains("Failed to read input file"));
}
```

---

## 4. Documentation Patterns

### 4.1 Docstring-Driven Help Text Pattern

**Pattern**: Extract help text from function docstrings.

```rust
/// Show logs for a service
///
/// # Arguments
/// * `service` - Service name (required)
/// * `lines` - Number of lines to show (default: 50)
#[verb("logs", "services")]
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    let lines = lines.unwrap_or(50);
    Ok(get_service_logs(service, lines))
}
```

**Help output**:
```
mycli services logs --help
Show logs for a service

USAGE:
    mycli services logs <SERVICE> [--lines <LINES>]

ARGUMENTS:
    <SERVICE>     Service name (required)

OPTIONS:
    --lines <LINES>    Number of lines to show (default: 50)
```

**Best practices**:
- ✅ First line: Brief description (appears in command list)
- ✅ `# Arguments` section: Describe each parameter
- ✅ Include defaults and constraints
- ✅ Be concise but informative

### 4.2 Type-Level Documentation Pattern

**Pattern**: Use meaningful type names that document themselves.

```rust
#[derive(Serialize, Debug)]
struct ServiceStatus {
    services: Vec<String>,
    all_running: bool,
}

#[derive(Serialize, Debug)]
struct RestartResult {
    service: String,
    success: bool,
    message: String,
}
```

**Benefits**:
- ✅ Self-documenting types
- ✅ Clear intent
- ✅ JSON output is readable

### 4.3 Example-Driven Documentation Pattern

**Pattern**: Provide runnable examples for each pattern.

```rust
//! # Examples
//!
//! Basic usage:
//!
//! ```rust
//! use clap_noun_verb::Result;
//! use clap_noun_verb_macros::verb;
//!
//! #[verb("status", "services")]
//! fn show_status() -> Result<Status> {
//!     Ok(Status { healthy: true })
//! }
//! ```
```

---

## 5. Performance Patterns

### 5.1 Zero-Copy String Access Pattern

**Pattern**: Use `&str` references instead of allocating `String`.

```rust
// ❌ Bad: Allocates for every argument
#[deprecated]
pub fn arg_names(&self) -> Vec<String> {
    self.matches.ids().map(|id| id.as_str().to_string()).collect()
}

// ✅ Good: Zero-copy references
pub fn arg_names_refs(&self) -> Vec<&str> {
    self.matches.ids().map(|id| id.as_str()).collect()
}
```

**Impact**: Saves 24 bytes per argument name + allocation overhead.

### 5.2 Shared Formatting Logic Pattern

**Pattern**: Extract common logic to reduce duplication.

```rust
/// Generic helper for formatting arrays (table/TSV)
fn format_object_array<F>(arr: &[serde_json::Value], formatter: F) -> String
where
    F: Fn(&serde_json::Value, &str) -> String,
{
    if arr.is_empty() {
        return String::new();
    }

    // Single implementation used by both table and TSV formatters
    // ...
}

// Table formatter
fn format_table(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(arr) => {
            format_object_array(arr, |val, _key| match val {
                serde_json::Value::Null => "-".to_string(),
                other => other.to_string(),
            })
        }
        // ...
    }
}

// TSV formatter
fn format_tsv(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(arr) => {
            format_object_array(arr, |val, _key| escape_tsv(&val.to_string()))
        }
        // ...
    }
}
```

### 5.3 Lazy Initialization Pattern

**Pattern**: Use `once_cell` for expensive one-time setup.

```rust
use once_cell::sync::Lazy;
use std::sync::Mutex;

static REGISTRY: Lazy<Mutex<CommandRegistry>> = Lazy::new(|| {
    Mutex::new(CommandRegistry::new())
});
```

### 5.4 Feature-Gated Dependencies Pattern

**Pattern**: Make expensive features optional to minimize compile time.

```toml
[features]
default = []  # Minimal dependencies (10 crates)
full = ["async", "io", "crypto", "rdf", ...]

[dependencies]
# Core (always required)
clap = "4.5"
serde = "1.0"
thiserror = "1.0"

# Optional (feature-gated)
tokio = { version = "1.40", optional = true }
sha2 = { version = "0.10", optional = true }
```

**Benefits**:
- ✅ Fast compilation for basic usage
- ✅ Pay-for-what-you-use
- ✅ Smaller binary size

---

## 6. Code Organization

### 6.1 Domain-Separated Architecture Pattern

**Pattern**: Separate CLI, Integration, and Domain layers.

```
my-cli/
├── src/
│   ├── cli/           # CLI layer (thin)
│   │   ├── commands.rs   # Argument parsing, I/O
│   │   └── mod.rs
│   ├── integration/   # Integration layer
│   │   ├── io.rs        # File I/O adapters
│   │   └── mod.rs
│   ├── domain/        # Domain layer (pure)
│   │   ├── logic.rs     # Business logic
│   │   └── mod.rs
│   └── main.rs
```

**Responsibilities**:

**CLI Layer** (`cli/`):
- Parse command-line arguments
- Open/close files
- Format output for users
- Convert domain errors to CLI errors
- **NO business logic**

**Integration Layer** (`integration/`):
- Adapt external systems (files, DB, HTTP)
- Implement ports/adapters
- Handle I/O concerns

**Domain Layer** (`domain/`):
- Pure business logic
- **ZERO CLI dependencies**
- No PathBuf, File I/O, println!
- Fully testable

### 6.2 Feature Module Organization Pattern

**Pattern**: Organize by feature, not by type.

```
src/
├── lib.rs              # Public API and re-exports
├── error.rs            # Error types
├── context.rs          # App context
├── format.rs           # Output formatting
├── noun.rs             # Noun command trait
├── verb.rs             # Verb command trait
├── registry.rs         # Command registry
├── router.rs           # Command routing
├── tree.rs             # Command tree
├── cli.rs              # CLI builder
└── autonomic/          # Feature: autonomic
    ├── mod.rs
    ├── hot_path.rs
    └── telemetry.rs
```

**Benefits**:
- ✅ Easy to find related code
- ✅ Clear module boundaries
- ✅ Feature flags align with modules

### 6.3 Public API Organization Pattern

**Pattern**: Re-export all public APIs from lib.rs.

```rust
// Core framework types (always available)
pub use builder::{build_cli, run_cli, CliBuilder};
pub use error::{NounVerbError, Result};
pub use noun::{NounCommand, NounContext};
pub use verb::{VerbArgs, VerbCommand, VerbContext};

// Feature-gated re-exports
#[cfg(feature = "async")]
pub use async_verb::{create_runtime, run_async};

#[cfg(feature = "completions")]
pub use completion::{generate_completion, Shell};
```

**Benefits**:
- ✅ Single import point: `use clap_noun_verb::*;`
- ✅ Clear public API surface
- ✅ Easy to version

---

## 7. Anti-Patterns to Avoid

### 7.1 Prohibited Patterns (Enforced by Lints)

**From `Cargo.toml` lints**:

```toml
[lints.clippy]
# Never use these in production
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
unimplemented = "deny"
todo = "deny"
```

**Why**:
- ❌ `unwrap()` - Can panic in production
- ❌ `expect()` - Can panic with "better" message (still panics!)
- ❌ `panic!()` - Crashes the program
- ❌ `unimplemented!()` - Not production-ready
- ❌ `todo!()` - Not production-ready

**Instead**:
```rust
// ❌ Bad
let value = map.get("key").unwrap();

// ✅ Good
let value = map.get("key")
    .ok_or_else(|| NounVerbError::missing_argument("key"))?;
```

### 7.2 Business Logic in CLI Anti-Pattern

**Anti-pattern**: Putting business logic in CLI functions.

```rust
// ❌ Bad: Business logic in CLI layer
#[verb("process", "data")]
fn process_data(input: String) -> Result<Output> {
    // This is business logic - doesn't belong here!
    let transformed = input.to_uppercase();
    let length = transformed.len();

    Ok(Output { result: transformed, length })
}
```

**Fix**: Separate business logic from CLI.

```rust
// ✅ Good: Pure business logic
fn transform_data(input: String) -> Output {
    let transformed = input.to_uppercase();
    Output {
        result: transformed,
        length: transformed.len(),
    }
}

// ✅ CLI layer: validation + delegation only
#[verb("process", "data")]
fn process_data(input: String) -> Result<Output> {
    // Validate (if needed)
    if input.is_empty() {
        return Err(NounVerbError::validation_error("input", "", Some("cannot be empty")));
    }

    // Delegate to business logic
    Ok(transform_data(input))
}
```

### 7.3 Direct Cargo Commands Anti-Pattern

**Anti-pattern**: Using `cargo` commands directly.

```bash
# ❌ Bad
cargo clippy
cargo test
cargo fmt

# ✅ Good
cargo make lint
cargo make test
cargo make format
```

**Why**: `cargo make` provides:
- ✅ Timeout wrappers
- ✅ Consistent flags
- ✅ Cross-platform compatibility
- ✅ Pre-configured settings

### 7.4 Implicit Noun Anti-Pattern

**Anti-pattern**: Forgetting explicit noun in multi-noun files.

```rust
// ❌ Bad: Missing explicit noun in multi-noun file
#[verb("status")] // Which noun does this belong to?
fn show_status() -> Result<Status> { ... }

// ✅ Good: Explicit noun
#[verb("status", "services")] // Clear!
fn show_service_status() -> Result<Status> { ... }

#[verb("status", "collector")] // Different noun
fn show_collector_status() -> Result<Status> { ... }
```

### 7.5 println! in Library Code Anti-Pattern

**Anti-pattern**: Using `println!` in library code.

```rust
// ❌ Bad: Can't control output
pub fn process(input: String) -> Result<Output> {
    println!("Processing: {}", input); // Bad in library!
    // ...
}

// ✅ Good: Return data, let CLI format
pub fn process(input: String) -> Result<Output> {
    // No println! - just return data
    Ok(Output { /* ... */ })
}

// ✅ In CLI layer: Format output
#[verb("process", "data")]
fn process_cmd(input: String) -> Result<Output> {
    let output = process(input)?;
    println!("Processed: {}", output.result); // OK in CLI
    Ok(output)
}
```

---

## 8. Advanced Use Cases

### 8.1 Async Command Handling

**Pattern**: Use `async` feature for async commands.

```rust
#[cfg(feature = "async")]
use tokio;

#[cfg(feature = "async")]
#[verb("fetch", "data")]
async fn fetch_data(url: String) -> Result<Data> {
    let response = reqwest::get(&url).await?;
    let data = response.json().await?;
    Ok(data)
}
```

**Cargo.toml**:
```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["async"] }
tokio = { version = "1.40", features = ["rt", "macros"] }
```

### 8.2 Custom Output Formats

**Pattern**: Support multiple output formats (JSON, YAML, TOML, table).

```rust
use clap_noun_verb::format::OutputFormat;

#[verb("status", "services")]
fn show_status(format: Option<String>) -> Result<Status> {
    let status = get_service_status();

    let format = format
        .unwrap_or_else(|| "json".to_string())
        .parse::<OutputFormat>()?;

    let output = format.format(&status)?;
    println!("{}", output);

    Ok(status)
}
```

### 8.3 Global Application Context

**Pattern**: Share state across commands using `AppContext`.

```rust
use clap_noun_verb::context::AppContext;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_connection: Arc<Database>,
    config: Config,
}

fn main() -> Result<()> {
    // Create global context
    let state = AppState {
        db_connection: Arc::new(Database::connect()?),
        config: load_config()?,
    };

    let context = AppContext::new();
    context.insert(state)?;

    // Context is available to all handlers
    clap_noun_verb::run()
}

#[verb("query", "database")]
fn query_db() -> Result<QueryResult> {
    let ctx = AppContext::new();
    let state: AppState = ctx.get()?;

    let result = state.db_connection.query("SELECT * FROM users")?;
    Ok(result)
}
```

### 8.4 Multi-Platform CLIs

**Pattern**: Use platform-specific features with `cfg`.

```rust
#[cfg(target_os = "linux")]
#[verb("systemd", "services")]
fn manage_systemd(action: String) -> Result<SystemdStatus> {
    // Linux-specific systemd integration
}

#[cfg(target_os = "windows")]
#[verb("service", "services")]
fn manage_windows_service(action: String) -> Result<ServiceStatus> {
    // Windows service integration
}

#[cfg(target_os = "macos")]
#[verb("launchd", "services")]
fn manage_launchd(action: String) -> Result<LaunchdStatus> {
    // macOS launchd integration
}
```

### 8.5 Dynamic Command Loading

**Pattern**: Load commands from plugins at runtime.

```rust
#[cfg(feature = "full")]
use clap_noun_verb::plugin::Plugin;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }

    fn register(&self, registry: &mut CommandRegistry) {
        registry.register_noun(noun!(
            "plugin",
            "Plugin commands",
            [verb!("action", "Plugin action", |_| Ok(()))]
        ));
    }
}
```

### 8.6 Custom Validation

**Pattern**: Implement custom validation logic.

```rust
use clap_noun_verb::error::NounVerbError;

fn validate_email(email: &str) -> Result<()> {
    if !email.contains('@') {
        return Err(NounVerbError::validation_error(
            "email",
            email,
            Some("must contain @ symbol")
        ));
    }
    Ok(())
}

#[verb("create", "users")]
fn create_user(email: String) -> Result<User> {
    validate_email(&email)?;
    Ok(User { email })
}
```

---

## 9. Learning Path

### 9.1 Beginner Path (Week 1-2)

**Goal**: Build simple noun-verb CLIs with attribute macros.

1. **Read**: `examples/tutorial/basic.rs`
2. **Build**: Simple status CLI
   ```rust
   #[verb("status", "services")]
   fn show_status() -> Result<Status> { ... }
   ```
3. **Learn**: AAA testing pattern
4. **Practice**: Add 2-3 commands to your CLI

**Resources**:
- `/examples/tutorial/` - Step-by-step tutorials
- `/tests/attribute_macro_acceptance.rs` - Test examples

### 9.2 Intermediate Path (Week 3-4)

**Goal**: Implement domain separation and error handling.

1. **Read**: `docs/examples/domain-separation/template/`
2. **Build**: CLI with domain layer separation
3. **Learn**: Error handling patterns with `NounVerbError`
4. **Practice**: Refactor existing CLI to separate layers

**Resources**:
- `/docs/examples/domain-separation/` - Full examples
- `/src/error.rs` - Error patterns
- `/tests/integration.rs` - Integration tests

### 9.3 Advanced Path (Week 5-6)

**Goal**: Master advanced features and patterns.

1. **Read**: `examples/reference/framework.rs`
2. **Build**: CLI with async commands, custom formats
3. **Learn**: Performance optimization patterns
4. **Practice**: Add telemetry, custom output formats

**Resources**:
- `/examples/reference/` - Advanced examples
- `/src/autonomic/` - Autonomic features
- `/benches/` - Performance benchmarks

### 9.4 Expert Path (Week 7+)

**Goal**: Contribute to framework, build complex CLIs.

1. **Read**: Framework source code (`/src/`)
2. **Build**: Plugin system, custom middleware
3. **Learn**: Proc macro internals
4. **Contribute**: Open issues, submit PRs

**Resources**:
- `/clap-noun-verb-macros/` - Macro implementation
- `/tests/` - Full test suite
- GitHub issues and discussions

---

## 10. Quick Reference Guide

### 10.1 Common Commands

```bash
# Development
cargo make check          # Quick compilation check
cargo make test           # Run all tests
cargo make lint           # Clippy linting

# Validation
cargo make pre-commit     # Format + lint + unit tests
cargo make ci             # Full CI pipeline

# Documentation
cargo make doc            # Build docs (as docs.rs would)
cargo make doc-open       # Build and open docs

# Release
cargo make release-check  # Full release validation
```

### 10.2 Essential Patterns Cheat Sheet

```rust
// Basic verb
#[verb("status", "services")]
fn show_status() -> Result<Status> { Ok(...) }

// Root-level verb
#[verb("sync", "root")]
fn sync() -> Result<SyncOutput> { Ok(...) }

// With arguments
#[verb("logs", "services")]
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    let lines = lines.unwrap_or(50);
    Ok(...)
}

// Error handling
Err(NounVerbError::validation_error("age", "5", Some("must be >= 18")))

// Testing (AAA pattern)
#[test]
fn test_feature() -> Result<()> {
    // Arrange
    let input = Input { ... };

    // Act
    let output = process(input)?;

    // Assert
    assert_eq!(output.result, expected);
    Ok(())
}
```

### 10.3 Feature Flags Reference

```toml
# Minimal (default)
clap-noun-verb = "5.3"

# With async support
clap-noun-verb = { version = "5.3", features = ["async"] }

# With all features
clap-noun-verb = { version = "5.3", features = ["full"] }

# Custom combination
clap-noun-verb = { version = "5.3", features = ["async", "io", "validators"] }
```

**Available features**:
- `async` - Async runtime (tokio, futures)
- `io` - Advanced I/O (clio)
- `crypto` - Cryptographic hashing
- `validators` - URL/regex validators
- `config-formats` - YAML/TOML support
- `completions` - Shell completion
- `autonomic` - Introspection & telemetry
- `agent2028` - Trillion-agent ecosystems
- `rdf` - RDF/Ontology with MCP
- `kernel` - Deterministic execution
- `full` - All features

### 10.4 Common Pitfalls & Solutions

| Problem | Solution |
|---------|----------|
| "Command not found" | Add explicit noun: `#[verb("cmd", "noun")]` |
| Compile error in macro | Check return type is `Result<T>` where `T: Serialize` |
| Panic in production | Never use `unwrap()`, use `?` or `ok_or_else()` |
| Slow compilation | Use minimal features, only enable what you need |
| Test failing | Follow AAA pattern, verify observable outputs |
| Help text missing | Add docstring above function |
| Can't access arguments | Use `VerbArgs` methods: `get_one_str()`, `get_one()` |

---

## Conclusion

This guide provides comprehensive real-world patterns from the clap-noun-verb v5.3.4 codebase. Key takeaways:

1. **Prefer attribute macros** (`#[verb]`) for zero boilerplate
2. **Separate business logic** from CLI layer
3. **Use Chicago TDD** (state-based, AAA pattern)
4. **Follow error handling patterns** with builder methods
5. **Organize by domain**, not by type
6. **Test observable behavior**, not implementation
7. **Use feature flags** to minimize dependencies
8. **Never use panic/unwrap** in production code

**Next steps**:
1. Start with tutorial examples (`/examples/tutorial/`)
2. Build a simple CLI with 2-3 commands
3. Add tests following AAA pattern
4. Refactor to separate business logic
5. Explore advanced features as needed

**Remember**: Good CLIs are built incrementally. Start simple, add complexity only when needed.
