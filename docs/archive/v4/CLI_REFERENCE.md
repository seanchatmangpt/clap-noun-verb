# clap-noun-verb CLI Reference

Complete reference documentation for building CLI applications with clap-noun-verb v4.0.2.

## Table of Contents

- [Quick Reference](#quick-reference)
- [Command Structure](#command-structure)
- [Attribute Macros](#attribute-macros)
- [Argument Attributes](#argument-attributes)
- [Type Inference](#type-inference)
- [Output Formats](#output-formats)
- [Shell Completions](#shell-completions)
- [Application Context](#application-context)
- [Async Operations](#async-operations)
- [Error Handling](#error-handling)
- [Advanced Features](#advanced-features)

---

## Quick Reference

### Basic Command Definition

```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct Output { message: String }

/// Show status
#[verb]
fn status() -> Result<Output> {
    Ok(Output { message: "OK".to_string() })
}
```

**Usage:** `myapp main status`

### Common Patterns

| Pattern | Syntax | Example |
|---------|--------|---------|
| **Auto-inferred noun** | `#[verb]` in `services.rs` | `myapp services status` |
| **Explicit noun** | `#[verb("verb", "noun")]` | `myapp config set` |
| **Custom verb name** | `#[verb("custom-name")]` | `myapp services custom-name` |
| **Required argument** | `arg: String` | `--arg <value>` |
| **Optional argument** | `arg: Option<String>` | `--arg [value]` |
| **Flag** | `flag: bool` | `--flag` |
| **Count** | `verbose: usize` | `-vvv` |
| **Multiple values** | `items: Vec<String>` | `--items a b c` |

---

## Command Structure

### The Noun-Verb Pattern

Commands follow the structure: `<app> <noun> <verb> [args]`

```
myapp services status          # Check service status
myapp database migrate         # Run database migrations
myapp config set api-key       # Set configuration value
```

### File Organization

**Single-noun files** (recommended):
```
src/
├── main.rs          # Entry point
├── services.rs      # All 'services' verbs
├── database.rs      # All 'database' verbs
└── config.rs        # All 'config' verbs
```

**Multi-noun files** (when needed):
```rust
// framework.rs
#[verb("status", "services")]
fn services_status() -> Result<Status> { ... }

#[verb("status", "database")]
fn database_status() -> Result<Status> { ... }
```

### Auto-Discovery

The framework automatically discovers all commands at compile time:

```rust
// main.rs
mod services;   // Auto-discovers all #[verb] in services.rs
mod database;   // Auto-discovers all #[verb] in database.rs

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()  // Auto-discovers and runs all commands
}
```

---

## Attribute Macros

### `#[verb]` - Verb Registration

Registers a function as a CLI verb command.

#### Syntax

```rust
#[verb]                           // Auto-infer verb name from function, noun from filename
#[verb("custom-name")]            // Custom verb name, noun from filename
#[verb("verb", "noun")]           // Explicit verb and noun (for multi-noun files)
```

#### Examples

**Auto-inferred** (in `services.rs`):
```rust
/// Show service status
#[verb]
fn status() -> Result<ServiceStatus> { ... }
// Creates: myapp services status
```

**Custom verb name** (in `services.rs`):
```rust
/// List all services
#[verb("list")]
fn show_all_services() -> Result<ServiceList> { ... }
// Creates: myapp services list
```

**Explicit noun** (in `framework.rs`):
```rust
/// Show service status
#[verb("status", "services")]
fn services_status() -> Result<ServiceStatus> { ... }
// Creates: myapp services status
```

#### Rules

1. Function name becomes verb name (with underscores converted to hyphens)
2. Noun is auto-inferred from filename (e.g., `services.rs` → `services`)
3. Function must return `Result<T>` where `T: Serialize`
4. Doc comments become help text

---

## Argument Attributes

Configure CLI arguments using `#[arg(...)]` attributes.

### Core Attributes

#### `short` - Short Flag

```rust
#[verb]
fn deploy(
    #[arg(short = 'e')]
    environment: String,
) -> Result<DeployResult>
// Usage: myapp deploy -e production
```

#### `long` - Long Flag

```rust
#[verb]
fn deploy(
    #[arg(long = "env")]
    environment: String,
) -> Result<DeployResult>
// Usage: myapp deploy --env production
// Default: myapp deploy --environment production
```

#### `default_value` - Default Value

```rust
#[verb]
fn serve(
    #[arg(default_value = "8080")]
    port: String,
) -> Result<ServerStatus>
// Usage: myapp serve           (uses 8080)
//        myapp serve --port 3000
```

#### `env` - Environment Variable

```rust
#[verb]
fn connect(
    #[arg(env = "DATABASE_URL")]
    url: String,
) -> Result<ConnectionStatus>
// Reads from DATABASE_URL environment variable if not provided
```

#### `index` - Positional Argument

```rust
#[verb]
fn deploy(
    #[arg(index = 0)]
    environment: String,
    #[arg(index = 1)]
    version: String,
) -> Result<DeployResult>
// Usage: myapp deploy production v1.2.3
```

### Validation Attributes

#### `required` - Required Argument

```rust
#[verb]
fn delete(
    #[arg(required = true)]
    confirm: bool,
) -> Result<DeleteResult>
// Must be explicitly provided
```

#### `value_name` - Value Name in Help

```rust
#[verb]
fn upload(
    #[arg(value_name = "FILE")]
    path: String,
) -> Result<UploadResult>
// Help shows: --path <FILE>
```

#### `multiple` - Multiple Values

```rust
#[verb]
fn tag(
    #[arg(multiple)]
    tags: Vec<String>,
) -> Result<TagResult>
// Usage: myapp tag --tags tag1 tag2 tag3
```

### Relationship Attributes

#### `group` - Argument Groups

```rust
#[verb]
fn output(
    #[arg(group = "format")]
    json: bool,
    #[arg(group = "format")]
    yaml: bool,
) -> Result<OutputResult>
// Only one of --json or --yaml can be used
```

#### `requires` - Requires Another Argument

```rust
#[verb]
fn export(
    #[arg(requires = "output")]
    format: Option<String>,
    output: String,
) -> Result<ExportResult>
// --format requires --output to be present
```

#### `conflicts_with` - Conflicts with Argument

```rust
#[verb]
fn build(
    #[arg(conflicts_with = "release")]
    debug: bool,
    release: bool,
) -> Result<BuildResult>
// Cannot use --debug and --release together
```

### Display Attributes

#### `help` - Help Text

```rust
#[verb]
fn connect(
    #[arg(help = "Database connection URL")]
    url: String,
) -> Result<Connection>
```

#### `long_help` - Extended Help

```rust
#[verb]
fn deploy(
    #[arg(
        help = "Target environment",
        long_help = "Environment to deploy to. Must be one of: dev, staging, production"
    )]
    env: String,
) -> Result<DeployResult>
```

#### `hide` - Hide from Help

```rust
#[verb]
fn internal(
    #[arg(hide)]
    secret_flag: bool,
) -> Result<InternalResult>
// Not shown in help output
```

#### `display_order` - Display Order

```rust
#[verb]
fn config(
    #[arg(display_order = 1)]
    important: String,
    #[arg(display_order = 2)]
    less_important: String,
) -> Result<ConfigResult>
```

### Action Attributes

#### `action` - Argument Action

```rust
#[verb]
fn run(
    #[arg(action = "count")]
    verbose: usize,              // -vvv = 3

    #[arg(action = "set_true")]
    flag: bool,                  // --flag sets to true

    #[arg(action = "set_false")]
    no_cache: bool,              // --no-cache sets to false

    #[arg(action = "append")]
    items: Vec<String>,          // --items a --items b
) -> Result<RunResult>
```

### Advanced Attributes

#### `alias` - Argument Alias

```rust
#[verb]
fn debug(
    #[arg(short = 'd', alias = "dbg")]
    debug: bool,
) -> Result<DebugResult>
// Usage: --debug OR --dbg OR -d
```

#### `exclusive` - Exclusive Group

```rust
#[verb]
fn mode(
    #[arg(group = "mode", exclusive)]
    interactive: bool,
    #[arg(group = "mode", exclusive)]
    batch: bool,
) -> Result<ModeResult>
// Exactly one must be provided
```

#### `trailing_vararg` - Trailing Arguments

```rust
#[verb]
fn exec(
    #[arg(trailing_vararg)]
    command: Vec<String>,
) -> Result<ExecResult>
// Usage: myapp exec -- ls -la
```

#### `allow_negative_numbers` - Negative Numbers

```rust
#[verb]
fn calculate(
    #[arg(allow_negative_numbers)]
    value: i32,
) -> Result<CalcResult>
// Usage: myapp calculate --value -42
```

---

## Type Inference

Arguments are automatically inferred from function parameter types.

### Basic Types

| Rust Type | CLI Behavior | Example |
|-----------|--------------|---------|
| `String` | Required argument | `--name <value>` |
| `Option<T>` | Optional argument | `--name [value]` |
| `bool` | Flag (SetTrue action) | `--flag` |
| `usize` | Count action | `-vvv` → 3 |
| `Vec<T>` | Multiple values (Append) | `--items a b c` |
| `i32`, `u32`, etc. | Parsed number | `--count 42` |
| `f32`, `f64` | Parsed float | `--ratio 0.5` |

### Type Conversion Examples

**String argument:**
```rust
#[verb]
fn greet(name: String) -> Result<Greeting> {
    Ok(Greeting { message: format!("Hello, {}!", name) })
}
// Usage: myapp greet --name Alice
```

**Optional argument:**
```rust
#[verb]
fn connect(
    host: String,
    port: Option<u16>,
) -> Result<Connection> {
    let port = port.unwrap_or(8080);
    // ...
}
// Usage: myapp connect --host localhost
//        myapp connect --host localhost --port 3000
```

**Boolean flag:**
```rust
#[verb]
fn build(
    #[arg(short = 'r')]
    release: bool,
) -> Result<BuildResult> {
    // release is true if --release/-r is present
}
// Usage: myapp build -r
```

**Count action:**
```rust
#[verb]
fn run(
    #[arg(short = 'v')]
    verbose: usize,
) -> Result<RunResult> {
    // -v = 1, -vv = 2, -vvv = 3
}
// Usage: myapp run -vvv
```

**Multiple values:**
```rust
#[verb]
fn tag(tags: Vec<String>) -> Result<TagResult> {
    // Accepts multiple --tags arguments
}
// Usage: myapp tag --tags tag1 tag2 tag3
```

**Numbers:**
```rust
#[verb]
fn serve(
    port: u16,
    workers: Option<usize>,
    timeout: f64,
) -> Result<ServerStatus>
// Usage: myapp serve --port 8080 --workers 4 --timeout 30.5
```

---

## Output Formats

### Supported Formats

| Format | Description | Use Case |
|--------|-------------|----------|
| `json` | JSON (default) | APIs, scripts, agents |
| `yaml` | YAML | Configuration files |
| `toml` | TOML | Configuration files |
| `table` | ASCII table | Human-readable output |
| `tsv` | Tab-separated | Spreadsheets, data processing |

### Using OutputFormat

```rust
use clap_noun_verb::OutputFormat;
use serde::Serialize;

#[derive(Serialize)]
struct Result {
    name: String,
    value: i32,
}

let output = Result { name: "test".to_string(), value: 42 };

// JSON (default)
let json = OutputFormat::Json.format(&output)?;
// {"name":"test","value":42}

// YAML
let yaml = OutputFormat::Yaml.format(&output)?;
// name: test
// value: 42

// Table
let table = OutputFormat::Table.format(&output)?;
// ┌──────┬───────┐
// │ name │ value │
// ├──────┼───────┤
// │ test │ 42    │
// └──────┴───────┘

// TSV
let tsv = OutputFormat::Tsv.format(&output)?;
// name    value
// test    42
```

### Custom Format in CLI

```rust
use clap::{Parser, Subcommand};
use clap_noun_verb::OutputFormat;

#[derive(Parser)]
struct Cli {
    /// Output format: json, yaml, table, tsv
    #[arg(long, default_value = "json")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

fn main() -> clap_noun_verb::Result<()> {
    let cli = Cli::parse();

    let format = match cli.format.as_str() {
        "yaml" => OutputFormat::Yaml,
        "table" => OutputFormat::Table,
        "tsv" => OutputFormat::Tsv,
        _ => OutputFormat::Json,
    };

    // Use format...
}
```

---

## Shell Completions

Generate shell completions for your CLI.

### Supported Shells

- `bash` - Bash shell
- `zsh` - Zsh shell
- `fish` - Fish shell
- `powershell` - PowerShell
- `elvish` - Elvish shell

### Generating Completions

```rust
use clap_noun_verb::{generate_completion, Shell};
use clap::Command;

// Generate completion script
let mut cmd = build_my_cli();
let completion = generate_completion(&mut cmd, Shell::Bash, "myapp");
println!("{}", completion);

// Or print directly
use clap_noun_verb::print_completion;
print_completion(&mut cmd, Shell::Fish, "myapp")?;
```

### CLI Integration

Add completion generation to your CLI:

```rust
#[derive(Parser)]
struct Cli {
    /// Generate shell completion
    #[arg(long, value_name = "SHELL")]
    generate_completion: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(shell) = cli.generate_completion {
        let shell = match shell.as_str() {
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            "powershell" => Shell::PowerShell,
            "elvish" => Shell::Elvish,
            _ => return Err("Unknown shell".into()),
        };

        print_completion(&mut build_cli(), shell, "myapp")?;
        return Ok(());
    }

    // Normal command execution...
}
```

### Installation Examples

**Bash:**
```bash
myapp --generate-completion bash > ~/.myapp-completion.bash
echo "source ~/.myapp-completion.bash" >> ~/.bashrc
```

**Zsh:**
```bash
myapp --generate-completion zsh > ~/.zsh/completions/_myapp
echo "fpath=(~/.zsh/completions $fpath)" >> ~/.zshrc
```

**Fish:**
```bash
myapp --generate-completion fish > ~/.config/fish/completions/myapp.fish
```

---

## Application Context

Share typed state across all commands.

### Basic Usage

```rust
use clap_noun_verb::AppContext;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    config: Config,
    db: Arc<Database>,
}

fn main() -> Result<()> {
    // Create and populate context
    let context = AppContext::new();
    context.insert(AppState {
        config: load_config()?,
        db: Arc::new(connect_db()?),
    })?;

    // Run CLI with context
    clap_noun_verb::run_with_context(context)
}
```

### Using Context in Verbs

```rust
#[verb]
fn query(
    args: &VerbArgs,  // VerbArgs provides access to context
    table: String,
) -> Result<QueryResult> {
    // Get state from context
    let state: AppState = args.context.get()?;
    let db = &state.db;

    // Use shared state
    let results = db.query(&table)?;

    Ok(QueryResult { results })
}
```

### Multiple State Types

```rust
// Store multiple types in context
context.insert(Config { ... })?;
context.insert(Database { ... })?;
context.insert(Cache { ... })?;

// Retrieve by type
let config: Config = context.get()?;
let db: Database = context.get()?;
let cache: Cache = context.get()?;
```

---

## Async Operations

Execute async operations from synchronous verb handlers.

### Using `run_async`

```rust
use clap_noun_verb::async_verb::run_async;
use tokio::time::{sleep, Duration};

#[derive(Serialize)]
struct FetchResult {
    data: String,
}

#[verb]
fn fetch(url: String) -> Result<FetchResult> {
    run_async(async move {
        // Async operations here
        let response = reqwest::get(&url).await?;
        let data = response.text().await?;

        Ok(FetchResult { data })
    })
}
```

### Custom Runtime

```rust
use clap_noun_verb::async_verb::create_runtime;

#[verb]
fn process() -> Result<ProcessResult> {
    let runtime = create_runtime()?;

    runtime.block_on(async {
        // Async code...
        Ok(ProcessResult { success: true })
    })
}
```

### Async Examples

**HTTP request:**
```rust
#[verb]
fn download(url: String) -> Result<DownloadResult> {
    run_async(async move {
        let client = reqwest::Client::new();
        let bytes = client.get(&url)
            .send().await?
            .bytes().await?;

        Ok(DownloadResult {
            size: bytes.len(),
            url,
        })
    })
}
```

**Database query:**
```rust
#[verb]
fn migrate() -> Result<MigrateResult> {
    run_async(async {
        let pool = create_pool().await?;
        sqlx::migrate!()
            .run(&pool)
            .await?;

        Ok(MigrateResult { success: true })
    })
}
```

---

## Error Handling

### Error Types

```rust
use clap_noun_verb::{NounVerbError, Result};

// Framework Result type
pub type Result<T> = std::result::Result<T, NounVerbError>;

// Error variants
pub enum NounVerbError {
    InvalidCommand(String),
    ValidationError(String),
    ExecutionError(String),
    IoError(std::io::Error),
    // ... more variants
}
```

### Custom Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Database connection failed: {0}")]
    DatabaseError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

// Convert to NounVerbError
impl From<MyError> for NounVerbError {
    fn from(err: MyError) -> Self {
        NounVerbError::ExecutionError(err.to_string())
    }
}
```

### Error Context

```rust
#[verb]
fn connect(url: String) -> Result<Connection> {
    let conn = Database::connect(&url)
        .map_err(|e| NounVerbError::ExecutionError(
            format!("Failed to connect to {}: {}", url, e)
        ))?;

    Ok(Connection { url })
}
```

---

## Advanced Features

### Deprecation Warnings

Mark commands as deprecated:

```rust
use clap_noun_verb::deprecation::{Deprecation, DeprecationType};

let deprecation = Deprecation::new(DeprecationType::Verb)
    .since("3.5.0")
    .removed_in("4.0.0")
    .note("This verb has been replaced")
    .suggestion("Use 'new-verb' instead");

let warning = deprecation.warning_message("old-verb");
// ⚠️  Verb 'old-verb' is deprecated since v3.5.0
//    (will be removed in v4.0.0)
//
//    This verb has been replaced
//
//    💡 Suggestion: Use 'new-verb' instead
```

### Configuration Files

Load configuration from files:

```rust
use clap_noun_verb::config::Config;

let config = Config::from_file("config.toml")?;
let value: String = config.get("key")?;
```

### Validators

Validate arguments:

```rust
use clap_noun_verb::validators::{validate_email, validate_url};

#[verb]
fn subscribe(
    #[arg(validator = validate_email)]
    email: String,
    #[arg(validator = validate_url)]
    callback: String,
) -> Result<SubscribeResult>
```

---

## See Also

- [Quick Start Guide](./QUICKSTART.md) - Get started in 10 minutes
- [CLI Cookbook](./CLI_COOKBOOK.md) - Common recipes and patterns
- [Troubleshooting](./CLI_TROUBLESHOOTING.md) - Common issues and solutions
- [Examples](../examples/) - Working code examples

---

**Version:** 4.0.2
**Last Updated:** 2024-11-18
**License:** MIT OR Apache-2.0
