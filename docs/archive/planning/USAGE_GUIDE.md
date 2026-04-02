# ggen-clap-noun-verb Usage Guide

**Complete step-by-step guide from Turtle specification to working CLI**

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Workflow](#workflow)
4. [Step 1: Create Turtle Specification](#step-1-create-turtle-specification)
5. [Step 2: Validate Specification](#step-2-validate-specification)
6. [Step 3: Generate Code](#step-3-generate-code)
7. [Step 4: Implement Business Logic](#step-4-implement-business-logic)
8. [Step 5: Test and Validate](#step-5-test-and-validate)
9. [Step 6: Build and Deploy](#step-6-build-and-deploy)
10. [Error Handling and Debugging](#error-handling-and-debugging)
11. [Advanced Topics](#advanced-topics)

---

## Overview

ggen-clap-noun-verb transforms Turtle/RDF specifications into production-ready Rust CLI applications using the clap-noun-verb framework.

**Workflow**: `Turtle Spec → Code Generation → Business Logic → Tests → Production CLI`

---

## Prerequisites

### Required Tools
```bash
# Rust 1.74+
rustc --version

# cargo-make
cargo install cargo-make

# ggen (if not already available)
cd /home/user/clap-noun-verb/vendors/ggen
cargo build --release
```

### Project Setup
```bash
# Clone clap-noun-verb
git clone https://github.com/sac/clap-noun-verb
cd clap-noun-verb

# Ensure ggen is available
ls vendors/ggen
```

---

## Workflow

```
┌──────────────────┐
│ 1. Create        │
│    Turtle Spec   │──┐
└──────────────────┘  │
                      │
┌──────────────────┐  │
│ 2. Validate      │<─┘
│    Specification │──┐
└──────────────────┘  │
                      │
┌──────────────────┐  │
│ 3. Generate      │<─┘
│    Rust Code     │──┐
└──────────────────┘  │
                      │
┌──────────────────┐  │
│ 4. Implement     │<─┘
│    Business Logic│──┐
└──────────────────┘  │
                      │
┌──────────────────┐  │
│ 5. Test &        │<─┘
│    Validate      │──┐
└──────────────────┘  │
                      │
┌──────────────────┐  │
│ 6. Build &       │<─┘
│    Deploy        │
└──────────────────┘
```

---

## Step 1: Create Turtle Specification

### 1.1 Choose a Template

Start with an example from `/home/user/clap-noun-verb/examples/turtle-specs/`:

```bash
# Copy a template
cp examples/turtle-specs/calculator.ttl my-cli.ttl
```

### 1.2 Define Your CLI Application

```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix clap: <http://clap-noun-verb.io/ontology#> .
@prefix my: <http://example.org/my-cli#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# CLI Application Definition
my:MyCLI a clap:CliApplication ;
    clap:name "my-cli" ;
    clap:version "1.0.0" ;
    clap:author "Your Name" ;
    clap:about "My custom CLI application" ;
    clap:nouns (my:TaskNoun) .
```

### 1.3 Define Nouns (Domain Entities)

```turtle
# Noun: task
my:TaskNoun a clap:Noun ;
    clap:name "task" ;
    clap:about "Task management operations" ;
    clap:verbs (my:CreateTaskVerb my:ListTasksVerb my:CompleteTaskVerb) .
```

### 1.4 Define Verbs (Actions)

```turtle
# Verb: create
my:CreateTaskVerb a clap:Verb ;
    clap:name "create" ;
    clap:about "Create a new task" ;
    clap:arguments (my:TitleArg my:DescriptionArg my:PriorityArg) ;
    clap:flags (my:VerboseFlag) ;
    clap:returnType my:TaskResponse ;
    clap:example "my-cli task create --title 'New Task' --priority high" .
```

### 1.5 Define Arguments

```turtle
# Required argument
my:TitleArg a clap:Argument ;
    clap:name "title" ;
    clap:shortName "t" ;
    clap:about "Task title" ;
    clap:valueType xsd:string ;
    clap:required true ;
    clap:position 1 .

# Optional argument with default
my:PriorityArg a clap:Argument ;
    clap:name "priority" ;
    clap:shortName "p" ;
    clap:about "Task priority (low, medium, high)" ;
    clap:valueType xsd:string ;
    clap:required false ;
    clap:defaultValue "medium" .
```

### 1.6 Add Validation

```turtle
# Validation rule
my:ValidPriority a clap:Validation ;
    clap:field "priority" ;
    clap:constraint "value in ['low', 'medium', 'high']" ;
    clap:errorMessage "Priority must be low, medium, or high" .
```

### 1.7 Define Complex Types (Optional)

```turtle
# Complex return type
my:TaskResponse a clap:ComplexType ;
    clap:fields (my:TaskIdField my:TaskTitleField my:TaskStatusField) .
```

---

## Step 2: Validate Specification

### 2.1 Syntax Validation

Check Turtle syntax:
```bash
# Using rapper (if installed)
rapper -i turtle my-cli.ttl

# Or use ggen's built-in validator
cd /home/user/clap-noun-verb/crates/ggen-clap-noun-verb
cargo run -- validate ../../my-cli.ttl
```

### 2.2 Semantic Validation

Verify ontology compliance:
```bash
# Validate against clap-noun-verb ontology
cargo run -- validate --schema ../../docs/clap-capabilities.ttl ../../my-cli.ttl
```

### 2.3 Fix Common Issues

**Missing Required Fields**:
```turtle
# ❌ Missing clap:name
my:MyNoun a clap:Noun ;
    clap:about "Description" .

# ✅ Correct
my:MyNoun a clap:Noun ;
    clap:name "my-noun" ;
    clap:about "Description" .
```

**Invalid Type References**:
```turtle
# ❌ Undefined type
clap:valueType my:CustomType .

# ✅ Use XSD types
clap:valueType xsd:string .
```

---

## Step 3: Generate Code

### 3.1 Run Code Generator

```bash
cd /home/user/clap-noun-verb/crates/ggen-clap-noun-verb

# Generate code
cargo run -- generate \
    --input ../../my-cli.ttl \
    --output ../../my-cli \
    --template clap-derive
```

### 3.2 Verify Generated Structure

```bash
cd ../../my-cli
tree -L 2

# Expected output:
# my-cli/
# ├── Cargo.toml
# ├── src/
# │   ├── main.rs
# │   ├── cli.rs
# │   ├── nouns/
# │   ├── verbs/
# │   ├── validators.rs
# │   └── error.rs
# └── tests/
#     └── integration_test.rs
```

### 3.3 Inspect Generated Code

**Main Entry Point** (`src/main.rs`):
```rust
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    cli.execute()?;
    Ok(())
}
```

**CLI Structure** (`src/cli.rs`):
```rust
#[derive(Parser)]
#[command(name = "my-cli")]
#[command(version = "1.0.0")]
pub struct Cli {
    #[command(subcommand)]
    pub noun: NounCommand,
}

#[derive(Subcommand)]
pub enum NounCommand {
    Task(TaskArgs),
}
```

---

## Step 4: Implement Business Logic

### 4.1 Locate Verb Handlers

Generated verb handlers have placeholder implementations:

```rust
// src/verbs/create_task.rs
impl CreateTaskArgs {
    pub fn execute(&self) -> Result<TaskResponse, CliError> {
        // TODO: Implement business logic
        unimplemented!("CreateTask::execute")
    }
}
```

### 4.2 Implement Logic

Replace placeholders with real implementations:

```rust
impl CreateTaskArgs {
    pub fn execute(&self) -> Result<TaskResponse, CliError> {
        // Validate priority (generated validator)
        validate_priority(&self.priority)?;

        // Create task
        let task = Task {
            id: generate_id(),
            title: self.title.clone(),
            description: self.description.clone(),
            priority: Priority::from_str(&self.priority)?,
            status: TaskStatus::Open,
            created_at: Utc::now(),
        };

        // Save to storage
        task.save()?;

        // Return response
        Ok(TaskResponse {
            id: task.id,
            title: task.title,
            status: task.status.to_string(),
        })
    }
}
```

### 4.3 Add Domain Logic

Create domain modules:

```rust
// src/domain/task.rs
pub struct Task {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn save(&self) -> Result<(), DomainError> {
        // Save to database or file
        Ok(())
    }
}
```

---

## Step 5: Test and Validate

### 5.1 Run Compilation Checks

```bash
# Quick compilation check
cargo make check
```

**Fix Andon Signal (Compiler Errors)**:
- Stop the line
- Fix errors immediately
- Re-run `cargo make check`

### 5.2 Run Unit Tests

```bash
# Run unit tests only
cargo make test-unit
```

**Generated Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_with_valid_input() {
        // Arrange
        let args = CreateTaskArgs {
            title: "Test Task".to_string(),
            description: Some("Test description".to_string()),
            priority: "high".to_string(),
            verbose: false,
        };

        // Act
        let result = args.execute();

        // Assert
        assert!(result.is_ok());
    }
}
```

### 5.3 Add Integration Tests

```rust
// tests/integration_test.rs
#[test]
fn test_cli_create_task() {
    let output = Command::new("cargo")
        .args(&["run", "--", "task", "create", "--title", "Test", "--priority", "high"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}
```

### 5.4 Run Linter

```bash
# Run clippy
cargo make lint
```

**Fix Andon Signals (Warnings)**:
- Address clippy warnings
- Use `#[allow(...)]` only for false positives
- Re-run `cargo make lint`

### 5.5 Full Test Suite

```bash
# Run all tests
cargo make test

# With coverage
cargo make test-coverage
```

---

## Step 6: Build and Deploy

### 6.1 Development Build

```bash
# Debug build
cargo build

# Run development version
./target/debug/my-cli task create --title "Test"
```

### 6.2 Release Build

```bash
# Optimized release build
cargo build --release

# Binary location
ls -lh target/release/my-cli
```

### 6.3 Install Locally

```bash
# Install to ~/.cargo/bin
cargo install --path .

# Run installed version
my-cli task create --title "Production Task"
```

### 6.4 Package for Distribution

```bash
# Create distributable binary
cargo make release-validate

# Package with metadata
cargo package

# Publish to crates.io (optional)
cargo publish
```

---

## Error Handling and Debugging

### Common Errors

#### 1. Turtle Syntax Error
```
Error: Parsing error at line 42: Expected '.' after triple
```

**Solution**:
- Check Turtle syntax (every triple must end with `.`)
- Validate with `rapper -i turtle spec.ttl`

#### 2. Missing Validation
```
Error: Validation 'ValidPriority' not found
```

**Solution**:
- Define validation in Turtle spec
- Ensure validator is referenced in verb

#### 3. Compilation Error
```
error[E0425]: cannot find value `validate_priority` in scope
```

**Solution**:
- Implement validator in `src/validators.rs`
- Import validator in verb module

#### 4. Test Failure
```
test test_create_task ... FAILED
```

**Solution**:
- Check test assertions
- Verify business logic implementation
- Use `cargo test -- --nocapture` for output

### Debugging Tips

**Enable Verbose Logging**:
```bash
RUST_LOG=debug cargo run -- task create --title "Test"
```

**Inspect Generated Code**:
```bash
# View generated CLI structure
bat src/cli.rs

# View generated validators
bat src/validators.rs
```

**Check Cargo.toml**:
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
thiserror = "2.0"
```

---

## Advanced Topics

### Custom Validators

Add complex validation logic:

```rust
// src/validators.rs
pub fn validate_priority(priority: &str) -> Result<(), ValidationError> {
    match priority {
        "low" | "medium" | "high" => Ok(()),
        _ => Err(ValidationError::InvalidPriority(priority.to_string())),
    }
}

pub fn validate_date_range(start: &str, end: &str) -> Result<(), ValidationError> {
    let start_date = DateTime::parse_from_rfc3339(start)?;
    let end_date = DateTime::parse_from_rfc3339(end)?;

    if start_date < end_date {
        Ok(())
    } else {
        Err(ValidationError::InvalidDateRange)
    }
}
```

### Async Operations

For async verbs:

```rust
// src/verbs/fetch_task.rs
impl FetchTaskArgs {
    pub async fn execute(&self) -> Result<TaskResponse, CliError> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/tasks/{}", self.base_url, self.id))
            .send()
            .await?;

        let task: Task = response.json().await?;
        Ok(task.into())
    }
}
```

### Middleware

Add cross-cutting concerns:

```rust
// src/middleware.rs
pub fn with_logging<F, T>(f: F) -> impl Fn() -> Result<T, CliError>
where
    F: Fn() -> Result<T, CliError>,
{
    move || {
        log::info!("Starting operation");
        let result = f();
        log::info!("Operation complete");
        result
    }
}
```

### Output Formatting

Support multiple formats:

```rust
// src/formatters.rs
pub enum OutputFormat {
    Json,
    Yaml,
    Text,
}

pub fn format_output<T: Serialize>(data: &T, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(data).unwrap(),
        OutputFormat::Yaml => serde_yaml::to_string(data).unwrap(),
        OutputFormat::Text => format!("{:#?}", data),
    }
}
```

---

## Quick Reference

### Common Commands

```bash
# Validate Turtle spec
rapper -i turtle spec.ttl

# Generate code
cargo run -- generate --input spec.ttl --output cli-name

# Build and test
cargo make check
cargo make test

# Lint code
cargo make lint

# Release build
cargo build --release
```

### File Locations

- Turtle specs: `/home/user/clap-noun-verb/examples/turtle-specs/`
- Generated CLIs: `/home/user/clap-noun-verb/examples/generated-from-turtle/`
- Documentation: `/home/user/clap-noun-verb/docs/`

---

## Next Steps

1. **Create Your Specification**: Start with a template from `examples/turtle-specs/`
2. **Generate Code**: Use ggen to create your CLI
3. **Implement Logic**: Add business logic to verb handlers
4. **Test Thoroughly**: Use Chicago TDD to validate behavior
5. **Deploy**: Build and distribute your CLI

---

## Related Documentation

- [Turtle Specifications README](/home/user/clap-noun-verb/examples/turtle-specs/README.md) - Specification format guide
- [Generated CLI README](/home/user/clap-noun-verb/examples/generated-from-turtle/README.md) - Generated code guide
- [EXAMPLES_SHOWCASE.md](/home/user/clap-noun-verb/docs/EXAMPLES_SHOWCASE.md) - Before/after examples
- [TURTLE_SPECIFICATION_GUIDE.md](/home/user/clap-noun-verb/docs/TURTLE_SPECIFICATION_GUIDE.md) - Complete syntax reference
- [ggen-clap-noun-verb-quickstart.md](/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-quickstart.md) - Implementation guide

---

**Need Help?** Check the [troubleshooting section](#error-handling-and-debugging) or consult the [quickstart guide](/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-quickstart.md).
