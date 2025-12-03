# Reference Examples

**Complete API demonstrations for looking up specific features**

These examples provide exhaustive coverage of clap-noun-verb APIs.

## Examples

### attribute_macro.rs - Complete Macro API
**Comprehensive #[noun] and #[verb] macro demonstration**

```bash
cargo run --example ref_attribute_macro -- services status
cargo run --example ref_attribute_macro -- services logs api
```

**Demonstrates:**
- `#[noun("name", "about")]` - Full noun syntax
- `#[verb("name")]` - Full verb syntax
- Doc comment parsing for help text
- Auto-discovery and registration

### framework.rs - Framework Integration
**How to integrate clap-noun-verb into larger applications**

```bash
cargo run --example ref_framework
```

**Demonstrates:**
- `clap_noun_verb::run()` - Standard execution
- `clap_noun_verb::build()` - Custom command building
- `CommandRegistry` - Accessing registered commands
- Custom error handling

### nested.rs - Multi-Level Nesting
**Deep command hierarchies (noun → sub-noun → verb)**

```bash
cargo run --example ref_nested -- level1 level2 action
cargo run --example ref_nested -- parent child grandchild
```

**Demonstrates:**
- Multi-level command nesting
- `[global]` arguments propagating to subcommands
- Help text at each level
- Command routing

### collector.rs - Command Collection
**Collecting commands from multiple modules**

```bash
cargo run --example ref_collector
```

**Demonstrates:**
- `linkme` distributed slice population
- Commands from multiple files
- Compile-time registration
- Auto-discovery patterns

### format.rs - Output Formats
**JSON, YAML, and custom output formats**

```bash
cargo run --example ref_format -- --format json
cargo run --example ref_format -- --format yaml
cargo run --example ref_format -- --format table
```

**Demonstrates:**
- JSON output (default)
- YAML output (optional)
- Table output (human-readable)
- Custom format handlers

### context.rs - Application Context
**Using AppContext for cross-cutting concerns**

```bash
cargo run --example ref_context
```

**Demonstrates:**
- `AppContext` structure
- Passing context to handlers
- State management
- Configuration access

## API Quick Reference

### Core Types

```rust
// Handler input (provided by macro)
pub struct HandlerInput {
    pub args: clap::ArgMatches,
    pub context: Option<AppContext>,
}

// Handler output (must implement Serialize)
pub struct HandlerOutput {
    pub data: serde_json::Value,
    pub status_code: u32,
}

// Result type
pub type Result<T> = std::result::Result<T, CliError>;
```

### Macro Syntax

```rust
// Noun declaration
#[noun("name", "About description")]

// Verb declaration
#[verb("name")]
fn handler(arg1: Type1, arg2: Type2) -> Result<Output> { }

// With explicit noun
#[verb("name", "noun_name")]
```

### Doc Comment Tags (v5.2.0)

```rust
/// # Arguments
/// * `arg` - Description [tag: value]
```

| Tag | Syntax | Purpose |
|-----|--------|---------|
| Group | `[group: name]` | Mutually exclusive |
| Requires | `[requires: arg]` | Dependency |
| Conflicts | `[conflicts: arg]` | Mutual exclusion |
| Env | `[env: VAR]` | Environment variable |
| Default | `[default: value]` | Default value |
| Value Hint | `[value_hint: type]` | Shell completion |
| Hide | `[hide]` | Hide from help |
| Heading | `[help_heading: name]` | Help organization |
| Global | `[global]` | Propagate to subcommands |
| Exclusive | `[exclusive]` | Cannot combine |

### Error Types

```rust
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Custom(String),
}
```

## Next Steps

1. See [docs/reference/api/](../../docs/reference/api/) for complete API documentation
2. Explore [advanced examples](../advanced/) for production patterns
3. Check [playground examples](../playground/) for RDF/MCP features
