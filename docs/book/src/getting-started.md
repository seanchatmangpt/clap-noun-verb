# Getting Started with Porting

This chapter covers the initial setup for porting ggen from regular clap to clap-noun-verb, including dependency management and understanding the framework APIs.

## Adding clap-noun-verb dependency

### 1. Update Cargo.toml

Add `clap-noun-verb` to your `Cargo.toml` dependencies:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
clap-noun-verb = "3.0.0"
clap-noun-verb-macros = "3.0.0"  # Required for #[verb] attribute macro
serde = { version = "1.0", features = ["derive"] }  # Required for JSON output
# ... your other dependencies
```

### 2. Verify Compatibility

Ensure your `clap` version is compatible with `clap-noun-verb` (requires clap 4.x):

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
clap-noun-verb = "3.0.0"
```

### 3. Optional: Update Dev Dependencies

If you have CLI tests, ensure they use compatible versions:

```toml
[dev-dependencies]
clap-noun-verb = "3.0.0"
# ... other dev dependencies
```

## Project structure

A typical clap-noun-verb project structure looks like:

```
ggen/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── commands/            # Command implementations
│   │   ├── mod.rs
│   │   ├── ai.rs            # AI commands
│   │   ├── marketplace.rs   # Marketplace commands
│   │   └── template.rs      # Template commands (if applicable)
│   ├── handlers/            # Command handlers
│   │   ├── mod.rs
│   │   ├── ai_handlers.rs
│   │   └── marketplace_handlers.rs
│   └── utils.rs             # Utilities
└── tests/
    └── cli_tests.rs         # CLI integration tests
```

### Alternative Structure (Flat)

For smaller projects, you might prefer a flatter structure:

```
ggen/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point with all commands
│   ├── handlers.rs          # Command handlers
│   └── utils.rs             # Utilities
```

### Recommended Approach (v2.0)

For ggen v2.0, we recommend a **modular structure with business logic separation**:

```
ggen/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── commands/            # CLI layer (sync wrappers)
│   │   ├── mod.rs
│   │   ├── ai.rs            # AI command wrappers
│   │   ├── marketplace.rs   # Marketplace command wrappers
│   │   ├── utils.rs         # Utility command wrappers
│   │   └── template.rs       # Template command wrappers
│   └── domain/              # Business logic (async functions)
│       ├── mod.rs
│       ├── ai.rs            # AI business logic
│       ├── marketplace.rs   # Marketplace business logic
│       ├── utils.rs         # Utility business logic
│       └── template.rs      # Template business logic
```

**Key Points**:
- **commands/**: Sync CLI wrappers that spawn async runtimes
- **domain/**: Async business logic functions (reusable, testable)
- **Separation**: CLI layer delegates to domain layer

## Understanding the framework APIs

### Core Types

### Attribute Macros (v3.0.0)

The recommended approach uses attribute macros:

```rust,no_run
// services.rs
//! Manage application services

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

/// Show service status
#[verb] // Verb "status" auto-inferred, noun "services" auto-inferred from filename
fn show_status() -> Result<Status> {
    Ok(Status {
        services: vec!["api".to_string()],
        healthy: true,
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

### Type Inference

Arguments are automatically inferred from function signatures:

```rust,no_run
/// Show logs for a service
/// 
/// # Arguments
/// * `service` - Service name (required)
/// * `lines` - Number of lines to show (default: 50)
#[verb] // Verb "logs" auto-inferred, noun "services" auto-inferred from filename (services.rs)
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    // service: String → Required argument --service
    // lines: Option<usize> → Optional argument --lines
    // Verb name "logs" auto-inferred from function name show_logs
    // Noun name "services" auto-inferred from filename services.rs
    Ok(Logs {
        service,
        lines: lines.unwrap_or(50),
        entries: vec![],
    })
}
```

## Async/Sync Compatibility

**Important**: `clap-noun-verb` v3.0.0 uses **sync-only** functions, but ggen has **async business logic**. We handle this by creating **sync CLI wrappers** that spawn async runtimes.

**Why Sync-Only?**: clap-noun-verb uses trait objects (`Box<dyn VerbCommand>`) for command registration. Rust trait objects cannot have async methods in stable Rust without `async-trait`, which would add overhead and violate the framework's zero-cost abstraction principle. **Async support is not planned for v3.1.0** - the sync wrapper pattern is the recommended approach.

### The Pattern

```rust,no_run
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer (Async - Reusable)
async fn run_diagnostics_async() -> DoctorOutput {
    // Async business logic here - can be I/O, network, etc.
    DoctorOutput {
        checks: vec![],
        overall: "OK".to_string(),
    }
}

// CLI Layer (Sync Wrapper - Delegates to Async Business Logic)
#[verb("doctor", "utils")]
fn utils_doctor() -> Result<DoctorOutput> {
    // Create runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    // Block on async business logic
    rt.block_on(async {
        run_diagnostics_async().await
    })
}
```

### Why This Pattern?

- **clap-noun-verb is sync-only**: Framework uses sync trait methods for `dyn` compatibility
- **ggen needs async**: Business logic performs async I/O operations (file system, network, AI APIs)
- **Solution**: Sync CLI wrapper spawns async runtime for business logic

### Recommended Structure

```
commands/
├── utils.rs          # CLI layer (sync wrappers)
└── domain/           # Business logic (async functions)
    └── utils.rs       # Domain logic (async, reusable)
```

**Example**:
```rust,no_run
// commands/utils.rs - CLI Layer (Sync)
#[verb("doctor", "utils")]
fn utils_doctor() -> Result<DoctorOutput> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    rt.block_on(async {
        crate::domain::utils::run_diagnostics().await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))
    })
}

// domain/utils.rs - Business Logic Layer (Async)
pub async fn run_diagnostics() -> Result<DoctorOutput> {
    // Async operations here
    Ok(DoctorOutput { checks: vec![], overall: "OK".to_string() })
}
```

## Basic concepts

### 1. Auto-Inference of Verb and Noun Names

The framework automatically infers verb names from function names and noun names from filenames:

```rust,no_run
// ai.rs
//! AI-powered generation

#[verb] // Verb "project" auto-inferred from function name ai_project
        // Noun "ai" auto-inferred from filename ai.rs
fn ai_project(name: String) -> Result<ProjectOutput> {
    Ok(ProjectOutput { name, rust: false })
}

#[verb] // Verb "generate" auto-inferred, noun "ai" auto-inferred
fn ai_generate(description: String) -> Result<String> {
    Ok(format!("Generated: {}", description))
}
```

This creates:
- `ggen ai project`
- `ggen ai generate`

### 2. Type Inference from Function Signatures

Arguments are automatically inferred from function signatures:

```rust,no_run
#[verb]
fn ai_project(
    name: String,           // → Required argument --name
    description: Option<String>,  // → Optional argument --description
    rust: bool              // → Flag --rust
) -> Result<ProjectOutput> {
    Ok(ProjectOutput { name, rust })
}
```

### 3. Automatic JSON Output

All return types are automatically serialized to JSON:

```rust,no_run
use serde::Serialize;

#[derive(Serialize)]
struct ProjectOutput {
    name: String,
    rust: bool,
}

#[verb]
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    Ok(ProjectOutput { name, rust })
    // Output automatically serialized to JSON
}
```

**Usage:**
```bash
$ ggen ai project my-app --rust
{"name":"my-app","rust":true}
```

### 4. Separation of Concerns

Business logic is separated from CLI layer:

```rust,no_run
// Business Logic Layer (Pure Functions - Reusable)
fn create_project(name: String, rust: bool) -> ProjectOutput {
    // Business logic here - can be used by CLI, API, Web, etc.
    ProjectOutput { name, rust }
}

// CLI Layer (Input Validation + Output Shaping Only)
#[verb]
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    Ok(create_project(name, rust))  // Delegate to business logic
}
```

### 5. Error Handling with Result

All handlers return `Result<T>` for proper error handling:

```rust,no_run
#[verb]
fn ai_project(name: String) -> Result<ProjectOutput> {
    // Use ? for error propagation
    let project = create_project(name)?;
    Ok(project)
}
```

### 6. Module Documentation

Module-level doc comments provide noun descriptions:

```rust,no_run
// ai.rs
//! AI-powered generation  ← This becomes the noun's "about" text

#[verb]
fn ai_project(name: String) -> Result<ProjectOutput> {
    Ok(ProjectOutput { name, rust: false })
}
```

## Example: Minimal Setup

Here's a minimal example to get started:

```rust,no_run
// services.rs
//! Manage application services

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

#[verb] // Verb "status" auto-inferred, noun "services" auto-inferred from filename
fn show_status() -> Result<Status> {
    Ok(Status {
        services: vec!["api".to_string()],
        healthy: true,
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

This creates a CLI with:
```bash
$ myapp services status
{"services":["api"],"healthy":true}
```

**Key Features Demonstrated:**
1. **Auto-inference**: Verb name "status" from function `show_status()`, noun name "services" from filename `services.rs`
2. **Type inference**: Arguments inferred from function signature (none in this case)
3. **JSON output**: Return type automatically serialized to JSON
4. **Module docs**: Noun description from `//! Manage application services` comment

## Next Steps

Now that you understand the basics, you're ready to:

1. [Porting Commands Step-by-Step](porting-commands.md) - Port each command group with detailed examples
2. [Advanced Patterns](advanced-patterns.md) - Learn advanced techniques for complex scenarios


