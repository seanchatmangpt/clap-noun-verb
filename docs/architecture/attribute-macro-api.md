# Attribute Macro API (v3.0.0)

## Overview

Rust's **attribute macros** provide decorator-style syntax similar to Python's Typer. With v3.0.0, you can define CLI commands directly on functions using `#[noun]` and `#[verb]` attributes.

## Basic Usage

```rust
use clap_noun_verb_macros::{noun, verb};
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

/// Show service status
#[noun("services", "Manage services")]
#[verb("status")]
fn show_status() -> Result<Status> {
    Ok(Status {
        services: vec!["api".to_string(), "worker".to_string()],
        healthy: true,
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

## How It Works

1. **`#[noun("name", "description")]`** - Registers a noun command
2. **`#[verb("name")]`** - Registers a verb command with auto-discovery
3. **Auto-discovery** - All `#[verb]` functions are automatically discovered at compile time using `linkme`
4. **Auto-serialization** - Return types automatically serialized to JSON
5. **Auto-inference** - Arguments inferred from function signature

## Argument Type Inference

```rust
#[verb("logs", "services")]
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    // service: String → Required argument --service
    // lines: Option<usize> → Optional argument --lines
    Ok(Logs {
        service,
        lines: lines.unwrap_or(50),
        entries: vec![],
    })
}
```

**Type inference rules:**
- `String` → Required argument `--name`
- `Option<T>` → Optional argument `--name <value>`
- `bool` → Flag `--name` (true if present)
- `Vec<T>` → Multiple values `--name <value1> <value2> ...`

## Verb Registration

- Function has both `#[noun]` and `#[verb]` → Verb registered to that noun
- Function has only `#[verb]` → Specify noun: `#[verb("name", "noun_name")]`

## Docstring Help Generation

```rust
/// Show logs for a service
/// 
/// # Arguments
/// * `service` - Service name (required)
/// * `lines` - Number of lines to show (default: 50)
#[verb("logs", "services")]
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    // Help text auto-generated from docstring
}
```

## Separation of Concerns

**CRITICAL:** Functions with `#[verb]` are **ONLY** for:
1. **Input Validation** - Auto-validated from function signature
2. **Output Shaping** - Auto-serialized to JSON
3. **Delegation** - Delegate to pure business logic functions

**Example with separation:**

```rust
// ✅ Business Logic Layer (Pure, Reusable)
fn get_service_status() -> Status {
    Status {
        services: vec!["api".to_string()],
        healthy: true,
    }
}

// ✅ CLI Layer (Input Validation + Output Shaping Only)
#[noun("services", "Manage services")]
#[verb("status")]
fn show_status() -> Result<Status> {
    // 1. Validate inputs (auto-validated from signature - none here)
    // 2. Delegate to business logic
    Ok(get_service_status())
    // 3. Output shaping (auto-serializes to JSON)
}
```

## Complete Example

```rust
use clap_noun_verb_macros::{noun, verb};
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

#[derive(Serialize)]
struct Logs {
    service: String,
    lines: usize,
    entries: Vec<String>,
}

/// Show status of all services
#[noun("services", "Manage services")]
#[verb("status")]
fn show_status() -> Result<Status> {
    Ok(Status {
        services: vec!["api".to_string(), "worker".to_string()],
        healthy: true,
    })
}

/// Show logs for a service
/// 
/// # Arguments
/// * `service` - Service name (required)
/// * `lines` - Number of lines to show (default: 50)
#[verb("logs", "services")]
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    Ok(Logs {
        service,
        lines: lines.unwrap_or(50),
        entries: vec!["log1".to_string(), "log2".to_string()],
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
{"service":"api","lines":100,"entries":["log1","log2"]}
```

## Benefits

1. ✅ **Zero boilerplate** - Just add attributes
2. ✅ **Auto-discovery** - Commands discovered at compile time
3. ✅ **Type inference** - Arguments inferred from function signature
4. ✅ **JSON by default** - Perfect for agents/MCP
5. ✅ **Separation enforced** - CLI only validates/shapes, delegates to logic
