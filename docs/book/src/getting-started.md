# Getting Started with Porting

This chapter covers the initial setup for porting ggen from regular clap to clap-noun-verb, including dependency management and understanding the framework APIs.

## Adding clap-noun-verb dependency

### 1. Update Cargo.toml

Add `clap-noun-verb` to your `Cargo.toml` dependencies:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
clap-noun-verb = "3.0.0"
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

### Recommended Approach

For ggen, we recommend a **modular structure** to keep commands organized:

```
ggen/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI builder
│   ├── cli.rs               # CLI structure definition
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── ai.rs
│   │   └── marketplace.rs
│   └── handlers/
│       ├── mod.rs
│       ├── ai_handlers.rs
│       └── marketplace_handlers.rs
```

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

## Basic concepts

### 1. Nouns Group Related Verbs

Nouns are containers for related actions:

```rust,no_run
noun!("ai", "AI-powered generation", [
    verb!("project", ...),
    verb!("generate", ...),
    verb!("graph", ...),
])
```

This creates:
- `ggen ai project`
- `ggen ai generate`
- `ggen ai graph`

### 2. Verbs Perform Actions

Verbs execute specific actions on nouns:

```rust,no_run
verb!("project", "Generate complete projects", |args: &VerbArgs| {
    let name = args.get_one_str("name")?;
    
    use std::fs;
    use std::path::PathBuf;
    
    let project_dir = PathBuf::from(&name);
    fs::create_dir_all(&project_dir)?;
    println!("✓ Project '{}' created", name);
    
    Ok(())
}, args: [
    Arg::new("name").required(true),
])
```

### 3. Arguments Are Type-Safe

Extract arguments with type safety:

```rust,no_run
|args: &VerbArgs| {
    let name = args.get_one_str("name")?;  // Required string
    let rust = args.is_flag_set("rust");   // Flag
    let output = args.get_one_str_opt("output"); // Optional string
    Ok(())
}
```

### 4. Global Arguments Are Accessible

Global arguments are available to all verbs:

```rust,no_run
cli.global_args(vec![
    Arg::new("verbose").short('v').long("verbose").action(clap::ArgAction::Count),
    Arg::new("config").short('c').long("config"),
])
.noun(noun!("ai", ..., [
    verb!("project", ..., |args: &VerbArgs| {
        let verbose = args.get_global_flag_count("verbose");
        let config = args.get_global_str("config");
        // ...
    }),
]))
```

### 5. Error Handling Uses Result

All handlers return `Result<()>`:

```rust,no_run
verb!("project", ..., |args: &VerbArgs| -> Result<()> {
    let name = args.get_one_str("name")?;  // Returns Result<String>
    // If error, return early with ?
    do_something()?;
    Ok(())
})
```

### 6. Structure Validation (Optional)

Enable automatic structure validation:

```rust,no_run
cli.auto_validate(true)
```

This catches:
- Duplicate noun names
- Empty nouns (no verbs or sub-nouns)
- Duplicate verb names within a noun
- Duplicate sub-noun names
- Verb/sub-noun name conflicts

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
myapp services status
# Output: {"services":["api"],"healthy":true}
```

## Next Steps

Now that you understand the basics, you're ready to:

1. [Porting Commands Step-by-Step](porting-commands.md) - Port each command group with detailed examples
2. [Advanced Patterns](advanced-patterns.md) - Learn advanced techniques for complex scenarios

