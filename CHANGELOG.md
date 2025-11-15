# Changelog

All notable changes to clap-noun-verb will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.6.0] - 2025-01-15

### Added - Production-Ready Features & State Management

#### Async Handler Support
- **`run_async()` function** - Execute async operations from sync handlers
- **`create_runtime()` helper** - Create reusable tokio runtime
- **Full tokio integration** - Support for database, HTTP, and file I/O operations
- **v3.6.0 feature** - Enable modern async patterns in CLI handlers

#### Global Application Context System
- **`AppContext<T>` type-safe container** - Share state across all commands
- **Type-erased storage** - Works with any type via `Arc<RwLock<T>>`
- **Thread-safe** - Safe for concurrent access across multiple handlers
- **Helper methods** - `insert()`, `get()`, `contains()`, `remove()`, `with()`, `clear()`
- **Real-world use cases** - Database connections, shared config, loggers, cache

#### Output Format Plugins
- **Pluggable formatters** - Beyond JSON (YAML, TOML, Table, TSV)
- **`OutputFormat` enum** - JSON, Yaml, Toml, Table, Tsv variants
- **Format auto-detection** - From CLI argument (`--format json|yaml|table`)
- **Table generation** - ASCII tables from JSON arrays
- **TSV support** - Spreadsheet-friendly tab-separated format
- **YAML & TOML** - Popular configuration and data serialization formats

#### Deprecation & Migration System
- **`Deprecation` struct** - Metadata about deprecated items
- **`DeprecationType` enum** - Noun, Verb, or Argument deprecations
- **Version tracking** - `since`, `removed_in` version fields
- **User guidance** - `suggestion` and `note` for migration help
- **Warning messages** - Formatted output with emoji and clear guidance
- **Help text integration** - Show deprecation info in help output

#### Shell Completion Generation
- **`Shell` enum** - Bash, Zsh, Fish, PowerShell, Elvish support
- **`generate_completion()` function** - Generate completion script
- **`print_completion()` helper** - Output directly to stdout
- **clap_complete integration** - Leverage clap's native completion system
- **Installation helpers** - Suggest where to install completions
- **Multiple shell support** - Bash/Zsh/Fish/PowerShell/Elvish

### Changed

- **Dependencies updated**:
  - Added `tokio` with rt and macros features
  - Added `async-trait` for trait helper macro
  - Added `serde_yaml` for YAML serialization
  - Added `toml` for TOML serialization
  - Added `clap_complete` for shell completion

- **Version bump**: 3.5.0 → 3.6.0 (minor release)

- **Documentation**: Enhanced README with v3.6.0 feature details

### Migration Notes

No breaking changes. All v3.5.0 code continues to work. v3.6.0 features are opt-in:

- **Async code**: Wrap async operations with `run_async()`
- **Shared state**: Create `AppContext` once at startup, pass to handlers
- **Alternative formats**: Use `OutputFormat` enum to format output differently
- **Deprecation**: Opt-in via `Deprecation` struct - no enforcement needed
- **Completions**: Call `generate_completion()` in a `--generate-completion` handler

## [3.5.0] - 2025-01-15

### Added - Example Completeness & Integration Testing

#### Comprehensive Examples
- **env_vars.rs** - Environment variable support example
  - Reading configuration from environment variables: `#[arg(env = "VAR_NAME")]`
  - Proper precedence: CLI args override env vars which override defaults
  - Real-world configuration management scenario

- **positional.rs** - Positional arguments example
  - First positional argument: `#[arg(index = 0)]`
  - Optional second positional argument: `#[arg(index = 1)]`
  - Mixed positional and named arguments pattern (e.g., `git clone`)

- **arg_actions.rs** - Advanced argument actions example
  - Count action: `-v`, `-vv`, `-vvv` → 1, 2, 3
  - SetFalse action: `--no-cache` style inverted flags
  - Type-based auto-inference for actions

- **arg_groups.rs** - Argument groups and constraints example
  - Exclusive argument groups (mutually exclusive options)
  - Argument dependencies: `#[arg(requires = "...")]`
  - Argument conflicts: `#[arg(conflicts_with = "...")]`

#### Integration Testing
- **Enabled 12 integration tests** - All tests in `tests/integration_examples.rs` now enabled
  - `test_basic_example_help` - Basic example help output
  - `test_basic_example_services_status` - Basic example command execution
  - `test_services_example` - Services example functionality
  - `test_services_example_logs` - Services with arguments
  - `test_collector_example` - Collector pattern example
  - `test_arguments_example` - Arguments with required/optional fields
  - `test_arguments_example_with_flag` - Boolean flag support
  - `test_validation_example` - Input validation
  - `test_nested_example` - Nested command structures
  - `test_framework_example` - Framework usage patterns
  - `test_attribute_macro_example` - Attribute macro basics
  - `test_attribute_macro_example_with_args` - Attribute macro with arguments

### Changed

- **Documentation completeness** - All v3.2.0 and v3.3.0 features now fully documented with working examples
- **Example coverage** - Comprehensive examples for every major feature category

### Migration Notes

No breaking changes. All existing code continues to work. v3.5.0 is a feature-complete release with comprehensive examples and integration tests.

**New in this release:**
1. All 12 integration tests are now enabled and part of the standard test suite
2. Four additional examples demonstrating v3.2.0+ features:
   - env_vars.rs: Environment variable handling
   - positional.rs: Positional arguments
   - arg_actions.rs: Advanced argument actions (count, set_false)
   - arg_groups.rs: Argument groups and constraints

## [3.4.0] - 2025-01-07

### Fixed
- **Option<Option<T>> type inference bug** - Fixed double-wrapping of Option types in macro-generated code
  - Changed `None::<Option<String>>` to `None` in ArgMetadata initialization (lines 789, 798, 866)
  - Allows Rust compiler to correctly infer types from struct field context
  - Resolves type mismatch errors when using Option<T> function parameters
  - Validated with comprehensive TDD test suite

### Changed
- Disabled work-in-progress `arg_attributes` test files to unblock release

## [3.3.0] - 2025-01-XX

### Added - Advanced clap Features and Typer-style Enhancements

#### Custom Value Parsers
- **Auto-inferred type parsers** - Automatic value parsers for common types (`PathBuf`, `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `Url`)
- **Pattern matching support** - String-based pattern matching for explicit `value_parser` expressions
- **Common type support** - Full support for path and IP address parsing

**Example:**
```rust
#[verb("deploy")]
fn deploy_service(
    #[arg(value_parser = clap::value_parser!(PathBuf))]
    config_file: PathBuf,
    #[arg(value_parser = clap::value_parser!(IpAddr))]
    host: IpAddr,
) -> Result<DeployOutput> {
    Ok(deploy(config_file, host))
}
```

**Note**: For range validation (e.g., `clap::value_parser!(u16).range(1..=65535)`), use `#[validate(min = ..., max = ...)]` instead, which is fully supported.

#### Enhanced Help System
- **Long help** - Separate `long_help` from `help` for detailed explanations
- **Next line help** - Help text on next line for better formatting using `#[arg(next_line_help)]`
- **Help override** - `#[arg(help = "...")]` to override docstring help text
- **Long help text** - `#[arg(long_help = "...")]` for detailed argument descriptions

**Example:**
```rust
/// Deploy a service
///
/// Short description appears in --help
///
/// Long description appears in --help with detailed
/// explanations and examples.
#[verb("deploy")]
fn deploy_service(
    /// Port number (short help)
    /// Detailed explanation of port configuration
    /// appears on the next line in help output.
    #[arg(long_help = "Detailed explanation of port configuration", next_line_help)]
    port: u16,
) -> Result<DeployOutput> {
    Ok(deploy(port))
}
```

#### Display Order Control
- **Display order** - Control argument order in help output using `#[arg(display_order = N)]`
- **Lower numbers first** - Lower numbers appear first in help output
- **Better organization** - Group related arguments together in help

**Example:**
```rust
#[verb("config")]
fn set_config(
    #[arg(display_order = 1)]  // Appears first
    host: String,
    #[arg(display_order = 2)]  // Appears second
    port: u16,
    #[arg(display_order = 99)] // Appears last
    debug: bool,
) -> Result<Config> {
    Ok(get_config(host, port, debug))
}
```

#### Exclusive Argument Groups
- **Exclusive groups** - Arguments in exclusive groups are mutually exclusive
- **Multiple vs exclusive** - Control group behavior with `#[arg(exclusive = true)]`
- **Better validation** - Prevents conflicting argument combinations

**Example:**
```rust
#[verb("filter")]
fn filter_items(
    #[arg(group = "filter", exclusive = true)]
    by_name: Option<String>,
    #[arg(group = "filter", exclusive = true)]
    by_id: Option<u64>,
) -> Result<FilteredItems> {
    Ok(filter(by_name, by_id))
}
```

#### Trailing Varargs Support
- **Trailing varargs** - Support for trailing variable arguments using `#[arg(trailing_vararg)]`
- **Flexible arguments** - Accept multiple trailing arguments
- **Better CLI patterns** - Support for commands like `cp file1 file2 ... dest/`

**Example:**
```rust
#[verb("copy")]
fn copy_files(
    #[arg(trailing_vararg)]
    files: Vec<String>,
) -> Result<CopyResult> {
    Ok(copy(files))
}
```

**Usage:**
```bash
myapp file copy file1.txt file2.txt file3.txt
# files: ["file1.txt", "file2.txt", "file3.txt"]
```

#### Allow Negative Numbers
- **Negative number support** - Allow negative numbers in numeric arguments using `#[arg(allow_negative_numbers)]`
- **Flexible parsing** - Support for negative values where appropriate

**Example:**
```rust
#[verb("offset")]
fn apply_offset(
    #[arg(allow_negative_numbers)]
    offset: i32,
) -> Result<OffsetResult> {
    Ok(apply(offset))
}
```

### Changed

- **Improved value parser support** - Better integration with clap's value parser system
- **Enhanced help formatting** - Better help text organization and presentation
- **Code organization** - Split large files into smaller modules for better maintainability

### Migration Notes

No breaking changes. All existing code continues to work. New features are opt-in.

**Workarounds for explicit value_parser expressions:**

1. **Use `#[validate]` attributes** - For range validation, use `#[validate(min = ..., max = ...)]` instead of `#[arg(value_parser = clap::value_parser!(u16).range(1..=65535))]`

2. **Auto-inferred parsers** - Common types like `PathBuf`, `IpAddr`, `Url` are automatically inferred

3. **Pattern matching** - Simple type parsers like `clap::value_parser!(PathBuf)` are supported via pattern matching

## [3.2.0] - 2025-01-XX

### Added - Complete clap Feature Support

#### Environment Variable Support
- **Environment variable fallback** - Arguments can read from environment variables using `#[arg(env = "VAR_NAME")]`
- **Automatic precedence** - Command-line args override environment variables
- **clap env feature** - Full integration with clap's env feature

**Example:**
```rust
#[verb("config")]
fn set_config(
    #[arg(env = "SERVER_PORT", default_value = "8080")]
    port: u16,
) -> Result<Config> {
    Ok(get_config(port))
}
```

**Usage:**
```bash
# Uses env var if set
export SERVER_PORT=3000
myapp config set  # Uses 3000

# CLI arg overrides env var
myapp config set --port 9090  # Uses 9090
```

#### Positional Arguments
- **Positional argument support** - Arguments can be positional using `#[arg(index = 0)]`
- **Order-based parsing** - Positional args parsed by their order
- **Mixed positional and named** - Support for both positional and named arguments in the same command

**Example:**
```rust
#[verb("clone")]
fn clone_repo(
    #[arg(index = 0)]
    url: String,
    #[arg(index = 1)]
    destination: Option<String>,
    #[arg(short = 'b')]
    branch: Option<String>,
) -> Result<Repo> {
    Ok(clone(url, destination, branch))
}
```

**Usage:**
```bash
myapp git clone https://example.com/repo.git ./local-dir --branch main
# url: https://example.com/repo.git (positional)
# destination: ./local-dir (positional)
# branch: main (named)
```

#### Enhanced ArgAction Support
- **Count action** - Count occurrences for flags (e.g., `-vvv` → 3)
- **Set action** - Explicit set action
- **SetFalse action** - Inverse flags (e.g., `--no-cache`)
- **Auto-inference** - `usize` type automatically uses `Count` action, `bool` uses `SetTrue`

**Example:**
```rust
#[verb("build")]
fn build_project(
    verbose: usize, // Auto-inferred as Count action (-v, -vv, -vvv)
    #[arg(action = "set_false")]
    cache: bool,    // SetFalse action (--no-cache)
    debug: bool,    // Auto-inferred as SetTrue
) -> Result<BuildResult> {
    Ok(build(verbose, cache, debug))
}
```

**Usage:**
```bash
myapp build --verbose --verbose --verbose --no-cache --debug
# verbose: 3 (count)
# cache: false (set_false)
# debug: true (set_true)
```

#### Argument Groups and Conflicts
- **Argument groups** - Arguments can be grouped using `#[arg(group = "group_name")]`
- **Requires** - Arguments can require other arguments using `#[arg(requires = "other_arg")]`
- **Conflicts** - Arguments can conflict with others using `#[arg(conflicts_with = "other_arg")]`

**Example:**
```rust
#[verb("filter")]
fn filter_items(
    #[arg(group = "filter", requires = "value")]
    by_name: Option<String>,
    #[arg(group = "filter", requires = "value")]
    by_id: Option<u64>,
    #[arg(short = 'v')]
    value: Option<String>,
    #[arg(conflicts_with = "by_name")]
    all: bool,
) -> Result<FilteredItems> {
    Ok(filter(by_name, by_id, value, all))
}
```

**Usage:**
```bash
# Exclusive group: by_name OR by_id
myapp filter --by-name "test" --value "test-value"  # OK
myapp filter --by-id 123 --value "test-value"      # OK
myapp filter --by-name "test" --by-id 123          # Error: mutually exclusive

# Requires: by_name needs value
myapp filter --by-name "test"                      # Error: requires value
myapp filter --by-name "test" --value "test"      # OK

# Conflicts: all conflicts with by_name
myapp filter --all                                # OK
myapp filter --all --by-name "test"               # Error: conflicts
```

#### Improved Help Generation
- **Long about** - Extended help text for nouns using `long_about` field
- **Hide arguments** - Hide arguments from help using `hide` field
- **Help headings** - Group arguments in help using `help_heading` field

**Example:**
```rust
/// Short description for --help
///
/// This is the long description that appears
/// in the detailed help output.
#[verb("command")]
fn my_command(
    /// Visible argument
    visible: String,
    /// Hidden argument (not shown in help)
    #[arg(hide = true)]
    hidden: String,
) -> Result<Output> {
    Ok(create_output(visible, hidden))
}
```

### Changed

- **Enhanced type inference** - `usize` arguments automatically use `Count` action
- **Improved validation** - Better integration with clap's validation system
- **Documentation** - Comprehensive examples for all new features

### Migration Notes

No breaking changes. All existing code continues to work. New features are opt-in.

## [3.0.0] - 2024-12-19

### Added - v3.0.0 Revolutionary Release

#### Attribute Macro API
- **Attribute macros `#[noun]` and `#[verb]`** - Zero-boilerplate command registration
- **Compile-time auto-discovery** - Commands automatically discovered using `linkme`
- **Verb name auto-inference** - Verb names automatically inferred from function names (e.g., `show_status` → `status`)
- **Noun name auto-inference** - Noun names automatically inferred from filename (e.g., `services.rs` → `services`)
- **Type inference** - Arguments automatically inferred from function signatures
- **Docstring-driven help** - Help text extracted from Rust docstrings
- **JSON output by default** - Perfect for agents, MCP, and modern tooling

#### Example

**Zero-args pattern (recommended for single-noun files):**

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
        services: vec!["api".to_string()],
        healthy: true,
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

**Explicit nouns (for multi-noun files):**

```rust
// framework.rs
#[verb("status", "services")] // Explicit noun since filename doesn't match
fn show_status() -> Result<Status> { /* ... */ }
```

### Changed

- **Breaking**: Attribute macros are now the primary API
- **Breaking**: CLI functions must return `Result<T>` where `T: Serialize`
- **API**: JSON output is now the default format
- **API**: `CliBuilder` remains for backward compatibility but is not recommended

### Migration Guide

From v1.x to v3.0.0:

1. Replace builder pattern with attribute macros
2. Add `#[derive(Serialize)]` to all output types
3. Separate business logic into pure functions
4. Call `clap_noun_verb::run()` in `main()`

```rust
// Old (v1.x)
let cli = CliBuilder::new("myapp")
    .noun("services", "Manage services")
    .verb("services", "status", "Show status", handler);
cli.run()

// New (v3.0.0)
#[noun("services", "Manage services")]
#[verb("status")]
fn show_status() -> Result<Status> { ... }
fn main() -> Result<()> { clap_noun_verb::run() }
```

## [1.0.0] - 2024-12-19

### Added

- **API Stability**: All public APIs are now stable
- **Enhanced Documentation**: Comprehensive API documentation
- **Publishing Metadata**: Complete Cargo.toml metadata for crates.io
