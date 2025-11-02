# clap-noun-verb

A framework for building composable CLI patterns on top of clap using the **noun-verb pattern** (e.g., `services status`, `collector up`). Inspired by Python's Typer.

## v3.3.0 - Advanced clap Features & Typer-style Enhancements

- **Custom Value Parsers**: Auto-inferred parsers for `PathBuf`, `IpAddr`, `Url`, etc.
- **Enhanced Help**: `long_help`, `next_line_help`, help override support
- **Display Order**: Control argument order in help with `display_order`
- **Exclusive Groups**: Mutually exclusive argument groups
- **Trailing Varargs**: Support for trailing variable arguments
- **Negative Numbers**: Allow negative numbers in numeric arguments

## v3.2.0 - Complete clap Feature Support

- **Environment Variables**: `#[arg(env = "VAR_NAME")]` for env var fallback
- **Positional Arguments**: `#[arg(index = 0)]` for positional args
- **Enhanced Actions**: Count, SetFalse with auto-inference (`usize` → Count, `bool` → SetTrue)
- **Argument Groups**: Groups, requires, conflicts_with support
- **Better Help**: long_about, hide, help_heading support

## v3.0.0 - Attribute Macros & Auto-Discovery

- **Attribute Macros**: `#[noun]` and `#[verb]` for zero-boilerplate command registration
- **Auto-Discovery**: Commands automatically discovered at compile time
- **Auto-Inference**: Verb names from function names, noun names from filenames
- **Type Inference**: Arguments inferred from function signatures
- **JSON Output**: All output automatically serialized to JSON (perfect for agents/MCP)

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "3.3.0"
clap-noun-verb-macros = "3.3.0"
```

Use attribute macros to define commands:

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

## Type Inference

Arguments are automatically inferred from function signatures:

- `String` → Required argument `--name`
- `Option<T>` → Optional argument `--name <value>`
- `bool` → Flag `--name` (true if present, uses `SetTrue` action)
- `usize` → Count action `--name` (e.g., `-vvv` → 3)
- `Vec<T>` → Multiple values `--name <value1> <value2> ...` (uses `Append` action)

## Argument Attributes (v3.2.0)

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

**Usage:**
```bash
# Short flags with defaults
myapp config set --port 3000  # Uses 3000, not default 8080

# Environment variables
export SERVER_HOST=example.com
myapp config set  # Uses example.com from env

# Positional + named
myapp config set https://api.example.com --host api.example.com

# Count action (verbose)
myapp config set -vvv  # verbose = 3

# Multiple values
myapp config set --tags rust cli json

# Aliases
myapp config set -d  # Same as --verbose-debug or --debug
```

## Verb Registration

- **Single-noun files** (e.g., `services.rs`): Use `#[verb]` only - noun auto-inferred from filename
- **Multi-noun files**: Use `#[verb("verb_name", "noun_name")]` with explicit noun
- **Custom verb name**: Use `#[verb("custom_name")]` to override auto-inferred name

## Examples

```bash
cargo run --example attribute_macro -- services status
cargo run --example basic -- services status
```

See the [`examples/`](examples/) directory for more examples.

## Comparison with clap

### Direct clap (verbose):

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

### With clap-noun-verb (v3.0.0):

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

## Design Philosophy

**clap-noun-verb** is a **framework** that enables composition rather than providing specific compositions:

- **Composable by Design** - Users compose their own CLI patterns
- **Type-Safe** - Compile-time verification of command structure
- **Zero-Cost** - Thin wrapper over clap with no runtime overhead
- **JSON-First** - Optimized for modern tooling and AI agents

## Migration from clap

1. Replace builder/enum-based commands with `#[noun]` and `#[verb]` attributes
2. Add `#[derive(Serialize)]` to output types
3. Return `Result<T>` where `T: Serialize` for JSON output
4. Call `clap_noun_verb::run()` in `main()`

## Documentation

- [Examples](examples/) - Working examples
- [Book Documentation](docs/book/src/) - Comprehensive guide for porting CLI applications
- [Contributing](CONTRIBUTING.md) - Contribution guidelines
- [Changelog](CHANGELOG.md) - Version history

## License

MIT OR Apache-2.0

## Acknowledgments

- Inspired by Python's [Typer](https://typer.tiangolo.com/)
- Built on [clap](https://crates.io/crates/clap)
- Error handling with [thiserror](https://crates.io/crates/thiserror)
