# clap-noun-verb v6.0.0 Migration Guide

**From**: v5.5.0
**To**: v6.0.0
**Difficulty**: Intermediate (2-4 hours for typical projects)
**Breaking Changes**: Yes (see all sections below)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Telemetry API Migration](#telemetry-api-migration)
3. [Command Handler Consolidation](#command-handler-consolidation)
4. [Macro Signature Updates](#macro-signature-updates)
5. [Feature Gate Reorganization](#feature-gate-reorganization)
6. [Error Type Restructuring](#error-type-restructuring)
7. [Testing Patterns](#testing-patterns)
8. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Step 1: Update Dependencies

```toml
[dependencies]
clap-noun-verb = "6.0"
clap-noun-verb-macros = "6.0"
```

```bash
cargo update clap-noun-verb
cargo check
```

### Step 2: Address Compiler Errors

The compiler will flag all breaking API changes. Follow the sections below to fix each category.

### Step 3: Run Test Suite

```bash
cargo test
cargo make test
```

### Step 4: Verify Telemetry (if used)

If your code uses telemetry, follow the [Telemetry API Migration](#telemetry-api-migration) section.

---

## Telemetry API Migration

### Breaking Change

The v5 `TelemetryManager` facade has been completely rewritten in v6.

**v5.5.0 Code** (will not compile in v6):

```rust
use clap_noun_verb::TelemetryManager;

fn my_handler() -> Result<Output> {
    let tm = TelemetryManager::instance();
    let start = Instant::now();

    // ... do work ...

    let duration = start.elapsed();
    tm.record_span("my_operation", duration);  // BROKEN IN v6.0.0

    Ok(output)
}
```

### Migration Path

**Step 1: Update Import**

```rust
// OLD (v5.5.0)
use clap_noun_verb::TelemetryManager;

// NEW (v6.0.0)
use clap_noun_verb::telemetry::TelemetryManager;
use clap_noun_verb::telemetry::SpanBuilder;
```

**Step 2: Update Span Recording**

```rust
// v6.0.0 - New fluent API
TelemetryManager::v2()
    .span_builder("my_operation")
    .with_duration(duration)
    .with_attributes(vec![
        ("operation_type", "handler"),
        ("result", "success"),
    ])
    .record()?;
```

**Step 3: Full Example with Error Handling**

```rust
use clap_noun_verb::{Result, telemetry::TelemetryManager};
use std::time::Instant;

#[verb("process")]
fn process_data(input: String) -> Result<ProcessOutput> {
    let span_start = Instant::now();

    // ... do work ...
    let result = do_processing(&input)?;

    // Record telemetry
    let duration = span_start.elapsed();
    TelemetryManager::v2()
        .span_builder("process_data")
        .with_duration(duration)
        .with_attributes(vec![
            ("input_length", &input.len().to_string()),
            ("status", "completed"),
        ])
        .record()
        .ok();  // Don't fail if telemetry fails

    Ok(result)
}
```

### Common Migration Scenarios

#### Scenario 1: Conditional Telemetry

**v5.5.0**:
```rust
#[cfg(feature = "telemetry")]
{
    tm.record_span("operation", duration);
}
```

**v6.0.0**:
```rust
#[cfg(feature = "autonomic")]
{
    TelemetryManager::v2()
        .span_builder("operation")
        .with_duration(duration)
        .record()
        .ok();
}
```

#### Scenario 2: Nested Spans

**v6.0.0** supports nested spans via context:

```rust
use clap_noun_verb::telemetry::{TelemetryManager, SpanContext};

fn outer_operation() -> Result<Output> {
    let tm = TelemetryManager::v2();

    let span = tm.span_builder("outer")
        .start()?;

    // Inner operations inherit trace context
    inner_operation()?;

    span.end()?;
    Ok(output)
}

fn inner_operation() -> Result<()> {
    let tm = TelemetryManager::v2();
    tm.span_builder("inner")
        .record()?;
    Ok(())
}
```

---

## Command Handler Consolidation

### Breaking Change

v5 used separate trait interfaces (`VerbHandler`, `NounHandler`). v6 unifies them into a single `CommandHandler` trait.

**v5.5.0 Code** (will not compile in v6):

```rust
use clap_noun_verb::handlers::{VerbHandler, HandlerInput, HandlerOutput};

struct MyHandler;

impl VerbHandler for MyHandler {
    fn handle(&self, input: HandlerInput) -> Result<HandlerOutput> {
        Ok(HandlerOutput::json(json!({"status": "ok"})))
    }
}
```

### Migration Path

**Step 1: Update Trait Implementation**

```rust
// NEW (v6.0.0)
use clap_noun_verb::{
    CommandHandler,
    CommandArgs,
    CommandOutput,
    CommandMetadata,
};

pub struct MyHandler;

impl CommandHandler for MyHandler {
    fn execute(&self, args: &CommandArgs) -> Result<CommandOutput> {
        // Process command arguments
        let result = serde_json::json!({
            "status": "ok",
            "args": args.positional()
        });

        Ok(CommandOutput::json(result))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "my_command".to_string(),
            description: "My command handler".to_string(),
            ..Default::default()
        }
    }
}
```

**Step 2: Register Handler with Builder**

```rust
// v6.0.0 - AgentCliBuilder v2
use clap_noun_verb::AgentCliBuilder;

let cli = AgentCliBuilder::new("myapp")
    .register_handler("process", Arc::new(MyHandler))
    .build()?;
```

**Step 3: Full Example**

```rust
use clap_noun_verb::{
    CommandHandler,
    CommandArgs,
    CommandOutput,
    CommandMetadata,
    AgentCliBuilder,
    Result,
};
use std::sync::Arc;

pub struct DataProcessHandler;

impl CommandHandler for DataProcessHandler {
    fn execute(&self, args: &CommandArgs) -> Result<CommandOutput> {
        let file = args.get_one_str("file")?;
        let format = args.get_one_str_opt("format");

        // Process file
        let result = serde_json::json!({
            "file": file,
            "format": format,
            "processed": true
        });

        Ok(CommandOutput::json(result))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "data_process".to_string(),
            description: "Process data files".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        }
    }
}

fn setup_cli() -> Result<()> {
    let cli = AgentCliBuilder::new("data-tool")
        .register_handler("process", Arc::new(DataProcessHandler))
        .build()?;

    Ok(())
}
```

---

## Macro Signature Updates

### Breaking Change 1: Argument Constraint Syntax

The `#[arg]` constraint syntax has been unified under `#[param]`.

**v5.5.0 Code** (still compiles but deprecated):

```rust
#[verb("filter")]
fn filter_items(
    #[arg(group = "format", exclusive = true)]
    json: bool,

    #[arg(requires = "output")]
    verbose: bool,

    #[arg(conflicts_with = "json")]
    yaml: bool,

    output: Option<String>,
) -> Result<FilteredItems> {
    Ok(filter_impl(json, yaml, verbose, output))
}
```

**v6.0.0 New Syntax** (recommended):

```rust
#[verb]
fn filter_items(
    /// Output format (JSON) [group: format exclusive]
    json: bool,

    /// Verbose output [requires: output]
    verbose: bool,

    /// Output format (YAML) [conflicts: json]
    yaml: bool,

    /// Output file path
    output: Option<String>,
) -> Result<FilteredItems> {
    Ok(filter_impl(json, yaml, verbose, output))
}
```

### Breaking Change 2: Inline Documentation

v6 supports richer inline documentation with auto-detection of constraints.

**v6.0.0 Enhanced Pattern**:

```rust
/// Manage services
///
/// Process background services with start/stop/restart operations.
/// Supports filtering by status or service name.
#[noun]
#[verb("Process service state")]
fn services_status(
    /// Only show running services [default: false]
    running_only: bool,

    /// Service name to query [requires: running_only]
    name: Option<String>,
) -> Result<ServiceStatus> {
    // Implementation
    Ok(ServiceStatus::default())
}
```

### Migration Guide for Macros

Replace constraint attributes with doc comment tags:

| v5.5.0 Attribute | v6.0.0 Doc Tag | Meaning |
|-----------------|-----------------|---------|
| `#[arg(group = "x")]` | `[group: x]` | Part of exclusive group |
| `#[arg(requires = "x")]` | `[requires: x]` | Requires another argument |
| `#[arg(conflicts_with = "x")]` | `[conflicts: x]` | Conflicts with argument |
| `#[arg(env = "VAR")]` | `[env: VAR]` | Read from environment |
| `#[arg(default_value = "x")]` | `[default: x]` | Default value |
| `#[arg(hide = true)]` | `[hide]` | Hide from help |
| `#[arg(exclusive = true)]` | `[exclusive]` | Cannot combine with others |

**Full Before/After**:

```rust
// v5.5.0
#[verb("export")]
fn export_data(
    #[arg(short = 'f', long = "format", default_value = "json", group = "output", exclusive = true)]
    format: String,

    #[arg(short = 'o', long = "output", requires = "format")]
    output_file: Option<String>,
) -> Result<ExportOutput> {
    // ...
}

// v6.0.0
#[verb]
fn export_data(
    /// Export format [default: json] [group: output exclusive]
    #[arg(short, long)]
    format: String,

    /// Output file path [requires: format]
    #[arg(short, long)]
    output_file: Option<String>,
) -> Result<ExportOutput> {
    // ...
}
```

---

## Feature Gate Reorganization

### Breaking Change

Feature flags have been reorganized for clarity in v6.0.0.

**v5.5.0 Pattern** (fragmented):

```toml
[dependencies]
clap-noun-verb = { version = "5.5", features = [
    "frontier-learning",
    "frontier-discovery",
    "frontier-federated",
    "autonomic",
    "mcp-sdk"
]}
```

**v6.0.0 Pattern** (consolidated):

```toml
[dependencies]
clap-noun-verb = { version = "6.0", features = [
    "frontier",  # All frontier packages included
    "telemetry", # Telemetry and tracing
    "mcp"        # MCP integration
]}
```

### Feature Mapping

| v5.5.0 Features | v6.0.0 Feature |
|-----------------|-----------------|
| `frontier-learning` | `frontier` |
| `frontier-discovery` | `frontier` |
| `frontier-federated` | `frontier` |
| `frontier-economic` | `frontier` |
| `frontier-quantum` | `frontier` |
| All frontier-* | `frontier` |

### Migration

Update your `Cargo.toml`:

```toml
# Before
clap-noun-verb = { version = "5.5", features = [
    "frontier-learning",
    "frontier-discovery"
] }

# After
clap-noun-verb = { version = "6.0", features = ["frontier"] }
```

No code changes required - features are backward compatible functionally.

---

## Error Type Restructuring

### Breaking Change

Error types have been simplified with shorter variant names and new categories.

**v5.5.0 Code** (will not compile in v6):

```rust
use clap_noun_verb::Error;

match err {
    Error::ParsingFailed(e) => eprintln!("Parse: {}", e),
    Error::ExecutionFailed(e) => eprintln!("Exec: {}", e),
    Error::TelemetryError(e) => eprintln!("Telemetry: {}", e),
    _ => eprintln!("Other: {}", err),
}
```

### v6.0.0 Updated Error Handling

```rust
use clap_noun_verb::Error;

match err {
    // Simplified names
    Error::Parsing(e) => eprintln!("Parse: {}", e),
    Error::Execution(e) => eprintln!("Exec: {}", e),

    // New error categories
    Error::Plugin(e) => eprintln!("Plugin: {}", e),
    Error::Telemetry(e) => eprintln!("Telemetry: {}", e),
    Error::Configuration(e) => eprintln!("Config: {}", e),

    Error::Other(e) => eprintln!("Other: {}", e),
}
```

### Complete Error Type Reference

```rust
pub enum Error {
    // Core errors
    Parsing(String),        // Argument parsing failed
    Execution(String),      // Handler execution failed

    // Plugin system
    Plugin(PluginError),    // Plugin loading/execution error
    PluginNotFound(String), // Plugin not in registry

    // Configuration
    Configuration(String),  // Invalid configuration

    // Telemetry
    Telemetry(String),      // Telemetry operation failed

    // I/O
    Io(std::io::Error),

    // Generic
    Other(String),
}
```

### Migration Pattern

```rust
// v5.5.0
fn handle_error(err: clap_noun_verb::Error) {
    use clap_noun_verb::Error::*;

    match err {
        ParsingFailed(msg) => { /* ... */ },
        ExecutionFailed(msg) => { /* ... */ },
        _ => { /* ... */ }
    }
}

// v6.0.0
fn handle_error(err: clap_noun_verb::Error) {
    use clap_noun_verb::Error::*;

    match err {
        Parsing(msg) => { /* ... */ },
        Execution(msg) => { /* ... */ },
        Plugin(plugin_err) => { /* ... */ },
        Configuration(msg) => { /* ... */ },
        _ => { /* ... */ }
    }
}
```

---

## Testing Patterns

### Updated Chicago TDD Integration

v6 continues Chicago TDD patterns with enhanced event testing.

**v5.5.0 Pattern** (still works in v6):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap_noun_verb::testing::TestContext;

    #[test]
    fn test_command_execution() {
        let ctx = TestContext::new();
        let result = my_handler();

        assert!(result.is_ok());
    }
}
```

**v6.0.0 Enhanced Pattern** (new):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap_noun_verb::testing::{TestContext, EventCollector};

    #[test]
    fn test_command_with_events() {
        let ctx = TestContext::new();
        let mut events = EventCollector::new();

        // Subscribe to events
        let result = my_handler_with_events(&mut events);

        // Verify execution and events
        assert!(result.is_ok());
        assert!(events.contains_event_type("completed"));
        assert!(!events.has_errors());
    }

    #[test]
    fn test_state_machine_transitions() {
        use clap_noun_verb::ConfigState;

        let config = ConfigState::new("valid_value".to_string());
        let ready = config.validate().expect("validation failed");
        let _initialized = ready.apply();

        // Compile error if states transition wrong:
        // let config = ConfigState::new("x".to_string());
        // let _ = config.apply();  // ERROR: apply() not available on Config state
    }
}
```

### Property-Based Testing

v6 includes proptest integration for comprehensive argument fuzzing:

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use clap_noun_verb::testing::proptest_args;

    proptest! {
        #[test]
        fn test_handler_never_panics(
            args in proptest_args()
        ) {
            // Property: handler should never panic with any args
            let _ = my_handler(&args);
        }
    }
}
```

---

## Troubleshooting

### Issue 1: "trait CommandHandler not found"

**Problem**: You're using old `VerbHandler` trait.

**Solution**: Replace with `CommandHandler`:

```rust
// OLD (broken in v6)
impl VerbHandler for MyHandler {
    fn handle(&self, input: HandlerInput) -> Result<HandlerOutput> { }
}

// NEW
impl CommandHandler for MyHandler {
    fn execute(&self, args: &CommandArgs) -> Result<CommandOutput> { }
}
```

### Issue 2: "cannot find macro `span!`"

**Problem**: v5 telemetry macros removed in v6.

**Solution**: Use new TelemetryManager API:

```rust
// OLD (broken in v6)
span!("operation", duration);

// NEW
use clap_noun_verb::telemetry::TelemetryManager;
TelemetryManager::v2()
    .span_builder("operation")
    .with_duration(duration)
    .record()?;
```

### Issue 3: "unknown attribute `#[arg(...)]` with constraint syntax"

**Problem**: Old constraint syntax no longer works.

**Solution**: Use doc comment tags:

```rust
// OLD
#[arg(requires = "other_arg")]

// NEW
/// My argument [requires: other_arg]
```

### Issue 4: Compilation Error with Feature Flags

**Problem**: Using individual `frontier-*` features that don't exist in v6.

**Solution**: Replace with unified `frontier` feature:

```toml
# OLD
features = ["frontier-learning", "frontier-discovery"]

# NEW
features = ["frontier"]
```

### Issue 5: "TelemetryManager::instance() deprecated"

**Problem**: v5 singleton pattern removed.

**Solution**: Use new builder pattern:

```rust
// OLD
let tm = TelemetryManager::instance();
tm.record_span("op", duration);

// NEW
use clap_noun_verb::telemetry::TelemetryManager;
TelemetryManager::v2()
    .span_builder("op")
    .with_duration(duration)
    .record()?;
```

### Seeking Help

If you encounter migration issues:

1. Check the [Release Notes](./v6_0_0_RELEASE_NOTES.md) for feature details
2. Review [COMMON_MISTAKES.md](../COMMON_MISTAKES.md) for error patterns
3. Open issue on [GitHub](https://github.com/seanchatmangpt/clap-noun-verb/issues)

---

## Summary Checklist

- [ ] Updated `clap-noun-verb` to `6.0` in `Cargo.toml`
- [ ] Migrated telemetry code (if used) to new `TelemetryManager` API
- [ ] Converted `VerbHandler` to `CommandHandler` (if custom handlers)
- [ ] Updated macro constraint syntax to doc comment tags
- [ ] Consolidated feature flags to `frontier` (if applicable)
- [ ] Updated error handling for new error types
- [ ] Ran `cargo test` and verified all tests pass
- [ ] Tested with `cargo make` if using build system
- [ ] Verified production telemetry (if enabled)

**Expected migration time**: 2-4 hours depending on project size.

**Questions?** See [Troubleshooting](#troubleshooting) or open a GitHub issue.
