# clap-noun-verb Quick Start Guide

Get started with clap-noun-verb in 5 simple steps. This guide will take you from installation to your first working CLI application in under 10 minutes.

## Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- Basic understanding of Rust syntax

## Step 1: Installation & Setup (2 minutes)

### Create a New Project

```bash
cargo new my-cli-app
cd my-cli-app
```

### Add Dependencies

Edit `Cargo.toml` and add clap-noun-verb:

```toml
[dependencies]
clap-noun-verb = "4.0.2"
clap-noun-verb-macros = "4.0.2"
serde = { version = "1.0", features = ["derive"] }
```

### Install Dependencies

```bash
cargo build
```

**Expected Output:**
```
    Updating crates.io index
   Compiling clap-noun-verb v4.0.2
   Compiling my-cli-app v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 8.2s
```

✅ **Success Check:** If compilation completes without errors, you're ready for Step 2!

---

## Step 2: List Available Commands (1 minute)

Before writing code, let's understand the noun-verb pattern by examining an example.

Create a simple `src/main.rs`:

```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    message: String,
}

/// Show system status
#[verb]
fn status() -> Result<Status> {
    Ok(Status {
        message: "System is running".to_string(),
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

### Test the Command

```bash
cargo run -- --help
```

**Expected Output:**
```
my-cli-app

USAGE:
    my-cli-app <COMMAND>

COMMANDS:
    main status    Show system status
    help           Print this message or the help of the given subcommand(s)

OPTIONS:
    -h, --help    Print help
```

✅ **Success Check:** You should see your `status` command listed!

**Common Mistake:** If you see "error: no verb functions found", make sure:
- You imported `clap_noun_verb_macros::verb`
- You added `#[verb]` before your function
- Your function returns `Result<T>` where `T: Serialize`

---

## Step 3: Generate Code Example (3 minutes)

Now let's create a more realistic CLI with multiple commands organized by nouns.

### Create a Services Module

Create `src/services.rs`:

```rust
//! Service management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct ServiceStatus {
    name: String,
    running: bool,
    port: u16,
}

#[derive(Serialize)]
struct ServiceList {
    services: Vec<ServiceStatus>,
}

/// Show status of all services
#[verb]
fn status() -> Result<ServiceList> {
    Ok(ServiceList {
        services: vec![
            ServiceStatus {
                name: "api".to_string(),
                running: true,
                port: 8080,
            },
            ServiceStatus {
                name: "worker".to_string(),
                running: true,
                port: 8081,
            },
        ],
    })
}

/// Restart a service
#[verb]
fn restart(
    /// Service name to restart
    service: String,
    /// Force restart even if service is unhealthy
    #[arg(short = 'f')]
    force: bool,
) -> Result<ServiceStatus> {
    Ok(ServiceStatus {
        name: service,
        running: true,
        port: 8080,
    })
}
```

### Update Main Module

Update `src/main.rs`:

```rust
mod services;

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
```

### Test Your CLI

```bash
# List all services
cargo run -- services status

# Restart a service
cargo run -- services restart api

# Restart with force flag
cargo run -- services restart worker -f
```

**Expected Output for `services status`:**
```json
{
  "services": [
    {
      "name": "api",
      "running": true,
      "port": 8080
    },
    {
      "name": "worker",
      "running": true,
      "port": 8081
    }
  ]
}
```

✅ **Success Check:** You should see JSON output with service information!

**Common Mistake:** If you see "error: unknown command 'services'", make sure:
- You declared `mod services;` in `src/main.rs`
- The file is named `src/services.rs` (the noun is auto-inferred from the filename!)
- You used `#[verb]` macro (not `#[verb("status", "services")]` - that's only needed for multi-noun files)

---

## Step 4: Customize Output (2 minutes)

clap-noun-verb supports multiple output formats out of the box.

### Add Format Flag

Update `src/main.rs` to support different output formats:

```rust
use clap_noun_verb::{Result, OutputFormat};
use clap::Parser;

mod services;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: json, yaml, table, tsv
    #[arg(long, default_value = "json")]
    format: String,
}

fn main() -> Result<()> {
    // Parse args with custom CLI
    let cli = Cli::parse();

    // Set output format
    let format = match cli.format.as_str() {
        "yaml" => OutputFormat::Yaml,
        "table" => OutputFormat::Table,
        "tsv" => OutputFormat::Tsv,
        _ => OutputFormat::Json,
    };

    // Run with format
    clap_noun_verb::run_with_format(format)
}
```

### Test Different Formats

```bash
# JSON output (default)
cargo run -- services status

# YAML output
cargo run -- services status --format yaml

# Table output
cargo run -- services status --format table

# TSV output (for spreadsheets)
cargo run -- services status --format tsv
```

**Expected Output for Table Format:**
```
┌─────────┬─────────┬──────┐
│ name    │ running │ port │
├─────────┼─────────┼──────┤
│ api     │ true    │ 8080 │
│ worker  │ true    │ 8081 │
└─────────┴─────────┴──────┘
```

✅ **Success Check:** You should see output in different formats!

**Common Mistake:** If table output looks broken:
- Make sure your terminal supports UTF-8
- Try using `--format json` if table rendering fails
- Check that all fields in your struct implement `Serialize`

---

## Step 5: Explore Advanced Features (2 minutes)

Now that you have a working CLI, explore these powerful features:

### A. Async Operations

Execute async operations from verb handlers:

```rust
use clap_noun_verb::async_verb::run_async;
use tokio::time::{sleep, Duration};

#[derive(Serialize)]
struct FetchResult {
    data: String,
}

#[verb]
fn fetch() -> Result<FetchResult> {
    run_async(async {
        // Your async code here
        sleep(Duration::from_millis(100)).await;

        Ok(FetchResult {
            data: "Fetched successfully".to_string(),
        })
    })
}
```

### B. Argument Attributes

Customize arguments with powerful attributes:

```rust
#[verb]
fn deploy(
    /// Environment to deploy to
    #[arg(short = 'e', env = "DEPLOY_ENV", default_value = "staging")]
    environment: String,

    /// Deployment tag
    #[arg(short = 't', required = true)]
    tag: String,

    /// Enable verbose logging
    #[arg(short = 'v', action = "count")]
    verbose: usize,

    /// Additional tags
    #[arg(long = "tag", multiple)]
    tags: Vec<String>,
) -> Result<DeployResult> {
    // Implementation
    Ok(DeployResult { success: true })
}
```

### C. Shell Completions

Generate shell completions for your CLI:

```bash
# Generate bash completions
cargo run -- --generate-completion bash > my-cli.bash
source my-cli.bash

# Generate zsh completions
cargo run -- --generate-completion zsh > _my-cli
```

### D. Application Context

Share state across commands:

```rust
use clap_noun_verb::AppContext;
use std::sync::Arc;

struct AppState {
    config: Config,
    db: Arc<Database>,
}

// In main
let context = AppContext::new();
context.insert(AppState { ... })?;

// In verb handler
#[verb]
fn query(args: &VerbArgs) -> Result<QueryResult> {
    let state: AppState = args.context.get()?;
    let db = &state.db;
    // Use database...
}
```

---

## Next Steps

Congratulations! You've built your first clap-noun-verb CLI application. Here's where to go next:

### Learn More

- **[CLI Reference](./CLI_REFERENCE.md)** - Complete command and argument reference
- **[CLI Cookbook](./CLI_COOKBOOK.md)** - Common recipes and patterns
- **[CLI Troubleshooting](./CLI_TROUBLESHOOTING.md)** - Common issues and solutions
- **[Examples Directory](../examples/)** - Working examples for advanced features

### Best Practices

1. **One noun per file** - Name files after the noun (`services.rs`, `database.rs`)
2. **Use descriptive names** - Verb function names should be clear (`show_status` not `status_show`)
3. **Add documentation** - Use `///` comments for help text
4. **Return structured data** - Always return `Result<T>` where `T: Serialize`
5. **Separate concerns** - Keep business logic separate from CLI layer

### Common Patterns

**Multi-file organization:**
```
src/
├── main.rs          # Entry point
├── services.rs      # services noun (auto-inferred)
├── database.rs      # database noun (auto-inferred)
└── config.rs        # config noun (auto-inferred)
```

**Explicit noun (multi-noun files):**
```rust
// When one file has multiple nouns
#[verb("status", "services")]
fn services_status() -> Result<Status> { ... }

#[verb("status", "database")]
fn database_status() -> Result<Status> { ... }
```

---

## Troubleshooting Quick Reference

| Problem | Solution |
|---------|----------|
| Command not found | Check `mod` declaration in `main.rs` |
| No JSON output | Ensure return type implements `Serialize` |
| Parse error | Check argument types match function signature |
| Compilation error | Make sure macros are imported: `use clap_noun_verb_macros::verb;` |
| Help text missing | Add `///` doc comments above function |

---

## Getting Help

- **Documentation**: [Full docs](../README.md)
- **Examples**: [Working examples](../examples/)
- **Issues**: [GitHub Issues](https://github.com/ruvnet/clap-noun-verb/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ruvnet/clap-noun-verb/discussions)

---

**Time to Complete**: ~10 minutes
**Difficulty**: Beginner
**Prerequisites**: Basic Rust knowledge

You're now ready to build production-grade CLIs with clap-noun-verb! 🚀
