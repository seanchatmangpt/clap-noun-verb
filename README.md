# clap-noun-verb

A framework for building composable CLI patterns on top of clap using the **noun-verb pattern** (e.g., `services status`, `collector up`). Inspired by Python's Typer.

## What is clap-noun-verb?

**clap-noun-verb** is a framework that enables composition rather than providing specific compositions. It provides a high-level, ergonomic API for building CLI applications organized around the noun-verb pattern, where commands follow the structure `noun verb` (e.g., `services status`, `collector up`).

### Key Features

- **Attribute Macros**: `#[noun]` and `#[verb]` for zero-boilerplate command registration
- **Auto-Discovery**: Commands automatically discovered at compile time
- **Auto-Inference**: Verb names from function names, noun names from filenames
- **Type Inference**: Arguments inferred from function signatures
- **JSON Output**: All output automatically serialized to JSON (perfect for agents/MCP)
- **Async Support**: Execute async operations from sync handlers
- **Application Context**: Share typed state across all commands
- **Output Formats**: JSON, YAML, TOML, Table, and TSV support
- **Shell Completions**: Auto-generate completions for bash, zsh, fish, powershell, and elvish
- **Autonomic CLI Layer** (NEW in v3.8.0): Machine-grade interface with introspection, effect modeling, guards, and receipts for agents and MAPE-K loops

### The Noun-Verb Pattern

The noun-verb pattern structures commands hierarchically:

```
myapp
├── services
│   ├── status
│   ├── logs
│   └── restart
├── collector
│   ├── up
│   ├── down
│   └── status
└── dev
    ├── test
    └── lint
```

Where:
- **Nouns** are entities or concepts (e.g., `services`, `collector`, `dev`)
- **Verbs** are actions performed on nouns (e.g., `status`, `logs`, `up`)

This creates an intuitive, scalable command structure that's easy to understand and extend.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "3.7.1"
clap-noun-verb-macros = "3.7.1"
```

Create your first command:

```rust
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
        services: vec!["api".to_string(), "worker".to_string()],
        healthy: true,
    })
}

/// Show logs for a service
#[verb] // Verb "logs" auto-inferred, noun "services" auto-inferred from filename
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    Ok(Logs {
        service,
        lines: lines.unwrap_or(50),
        entries: vec![],
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

**Usage:**
```bash
$ myapp services status
{"services":["api","worker"],"healthy":true}

$ myapp services logs api --lines 100
{"service":"api","lines":100,"entries":[]}
```

## How-to Guides

### How to configure arguments

Use `#[arg(...)]` attributes to configure arguments:

```rust
#[verb("config")]
fn set_config(
    // Short flag with default value
    #[arg(short = 'p', default_value = "8080")]
    port: u16,
    
    // Environment variable fallback
    #[arg(env = "SERVER_HOST", default_value = "localhost")]
    host: String,
    
    // Positional argument (index 0)
    #[arg(index = 0)]
    url: String,
    
    // Count action (auto-inferred for usize, but can be explicit)
    #[arg(short = 'v', action = "count")]
    verbose: usize,
    
    // Multiple values
    #[arg(multiple)]
    tags: Vec<String>,
    
    // Custom value name in help
    #[arg(value_name = "FILE")]
    output: String,
    
    // Aliases
    #[arg(short = 'd', alias = "debug")]
    verbose_debug: bool,
    
    // Argument groups (exclusive)
    #[arg(group = "format")]
    json: bool,
    #[arg(group = "format")]
    yaml: bool,
    
    // Requires another argument
    #[arg(requires = "output")]
    format: Option<String>,
    
    // Conflicts with another argument
    #[arg(conflicts_with = "format")]
    raw: bool,
) -> Result<Config> {
    Ok(get_config(port, host, url, verbose, tags, output))
}
```

### How to use async operations

Execute async operations from within synchronous verb handlers using `run_async()`:

```rust
use clap_noun_verb::async_verb::run_async;
use clap_noun_verb::VerbArgs;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct Output {
    message: String,
}

#[verb("fetch")]
fn fetch_data(args: &VerbArgs) -> Result<Output> {
    run_async(async {
        // Your async code here
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Database queries, HTTP calls, etc.
        let data = fetch_from_api().await?;

        Ok(Output {
            message: data.into(),
        })
    })
}
```

### How to share state across commands

Use `AppContext` to share typed state across all commands:

```rust
use clap_noun_verb::AppContext;
use std::sync::Arc;

struct AppState {
    db: Arc<Database>,
    config: Config,
}

// At startup
let context = AppContext::new();
context.insert(AppState { ... })?;

// In handlers
#[verb("query")]
fn query_database(args: &VerbArgs) -> Result<QueryResult> {
    let context = // get from somewhere
    let state: AppState = context.get()?;
    let db = &state.db;
    // Use database connection...
}
```

### How to format output

Generate output in multiple formats:

```rust
use clap_noun_verb::OutputFormat;
use serde::Serialize;

#[derive(Serialize)]
struct Result {
    name: String,
    value: i32
}

let output = Result {
    name: "test".to_string(),
    value: 42
};

// JSON (default)
let json = OutputFormat::Json.format(&output)?;

// YAML
let yaml = OutputFormat::Yaml.format(&output)?;

// Table format
let table = OutputFormat::Table.format(&output)?;

// TSV for spreadsheets
let tsv = OutputFormat::Tsv.format(&output)?;
```

Supported formats: `json`, `yaml`, `toml`, `table`, `tsv`

### How to generate shell completions

Auto-generate shell completions for supported shells:

```rust
use clap_noun_verb::{generate_completion, Shell};
use clap::Command;

let mut cmd = my_cli_command();
let completion = generate_completion(&mut cmd, Shell::Bash, "myapp");
println!("{}", completion);

// Or print directly
print_completion(&mut cmd, Shell::Fish, "myapp")?;
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`

Installation example for bash:
```bash
# Output completions
myapp --generate-completion bash > myapp.bash

# Source in .bashrc
source myapp.bash
```

### How to mark commands as deprecated

Mark commands as deprecated with helpful migration messages:

```rust
use clap_noun_verb::deprecation::{Deprecation, DeprecationType};

let deprecation = Deprecation::new(DeprecationType::Verb)
    .since("3.5.0")
    .removed_in("4.0.0")
    .note("This verb has been replaced for clarity")
    .suggestion("Use 'new-verb' instead");

let warning = deprecation.warning_message("old-verb");
// Output:
// ⚠️  Verb 'old-verb' is deprecated since v3.5.0 (will be removed in v4.0.0)
//
//   This verb has been replaced for clarity
//
//   💡 Suggestion: Use 'new-verb' instead
```

## Reference

### Type Inference

Arguments are automatically inferred from function signatures:

- `String` → Required argument `--name`
- `Option<T>` → Optional argument `--name <value>`
- `bool` → Flag `--name` (true if present, uses `SetTrue` action)
- `usize` → Count action `--name` (e.g., `-vvv` → 3)
- `Vec<T>` → Multiple values `--name <value1> <value2> ...` (uses `Append` action)

### Argument Attributes

Available `#[arg(...)]` attributes:

- `short = 'c'` - Short flag character
- `long = "name"` - Long flag name (defaults to parameter name)
- `default_value = "value"` - Default value as string
- `env = "VAR_NAME"` - Environment variable fallback
- `index = 0` - Positional argument index
- `action = "count"` - Custom action (count, set_true, set_false, append)
- `multiple` - Accept multiple values
- `value_name = "FILE"` - Custom value name in help
- `alias = "name"` - Argument aliases
- `group = "group_name"` - Argument group membership
- `requires = "other_arg"` - Requires another argument
- `conflicts_with = "other_arg"` - Conflicts with another argument
- `hide` - Hide from help text
- `help = "..."` - Custom help text
- `long_help = "..."` - Long help text
- `next_line_help` - Next line help formatting
- `display_order = 1` - Display order in help
- `exclusive` - Exclusive group flag
- `trailing_vararg` - Trailing variable arguments
- `allow_negative_numbers` - Allow negative numbers

### Verb Registration

- **Single-noun files** (e.g., `services.rs`): Use `#[verb]` only - noun auto-inferred from filename
- **Multi-noun files**: Use `#[verb("verb_name", "noun_name")]` with explicit noun
- **Custom verb name**: Use `#[verb("custom_name")]` to override auto-inferred name

### Available Output Formats

- `OutputFormat::Json` - JSON format (default)
- `OutputFormat::Yaml` - YAML format
- `OutputFormat::Toml` - TOML format
- `OutputFormat::Table` - ASCII table format
- `OutputFormat::Tsv` - Tab-separated values format

### Supported Shells for Completions

- `Shell::Bash` - Bash completions
- `Shell::Zsh` - Zsh completions
- `Shell::Fish` - Fish shell completions
- `Shell::PowerShell` - PowerShell completions
- `Shell::Elvish` - Elvish completions

## Explanation

### Design Philosophy

**clap-noun-verb** is a **framework** that enables composition rather than providing specific compositions:

- **Composable by Design** - Users compose their own CLI patterns
- **Type-Safe** - Compile-time verification of command structure
- **Zero-Cost** - Thin wrapper over clap with no runtime overhead
- **JSON-First** - Optimized for modern tooling and AI agents

### Comparison with clap

#### Direct clap (verbose):

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Services {
        #[command(subcommand)]
        command: ServiceCommands,
    },
}

#[derive(Subcommand)]
enum ServiceCommands {
    Status,
    Logs { service: String },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Services { command } => match command {
            ServiceCommands::Status => println!("Services running"),
            ServiceCommands::Logs { service } => println!("Logs for {}", service),
        },
    }
}
```

#### With clap-noun-verb:

```rust
// services.rs
//! Manage application services

#[verb] // Verb "status" and noun "services" auto-inferred!
fn show_status() -> Result<Status> { /* ... */ }

#[verb] // Verb "logs" and noun "services" auto-inferred!
fn show_logs(service: String) -> Result<Logs> { /* ... */ }

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

**Benefits:**
- ✅ Zero boilerplate - Just add attributes
- ✅ Auto-discovery - Commands automatically registered
- ✅ Better organization - Commands grouped by functionality
- ✅ JSON output - Perfect for agents/MCP

### Migration from clap

1. Replace builder/enum-based commands with `#[noun]` and `#[verb]` attributes
2. Add `#[derive(Serialize)]` to output types
3. Return `Result<T>` where `T: Serialize` for JSON output
4. Call `clap_noun_verb::run()` in `main()`

## Examples

```bash
cargo run --example attribute_macro -- services status
cargo run --example basic -- services status
```

See the [`examples/`](examples/) directory for more examples.

## Documentation

- [Examples](examples/) - Working examples
- [Book Documentation](docs/book/src/) - Comprehensive guide for porting CLI applications
- [Autonomic CLI Layer](AUTONOMIC.md) - Machine-grade interface for agents and MAPE-K loops
- [Contributing](CONTRIBUTING.md) - Contribution guidelines
- [Changelog](CHANGELOG.md) - Version history

## License

MIT OR Apache-2.0

## Acknowledgments

- Inspired by Python's [Typer](https://typer.tiangolo.com/)
- Built on [clap](https://crates.io/crates/clap)
- Error handling with [thiserror](https://crates.io/crates/thiserror)
