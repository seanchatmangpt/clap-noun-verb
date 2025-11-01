# Typer-Style API (v3.0.0)

## Overview

clap-noun-verb v3.0.0 implements a Typer-style API using Rust attribute macros. This provides the same simplicity and developer experience as Python's Typer library.

## Typer Pattern

### Python (Typer)

```python
import typer

app = typer.Typer()

@app.command()
def greet(name: str, age: int = 25):
    """Greet a user."""
    print(f"Hello, {name}! You are {age}")

if __name__ == "__main__":
    app()
```

### Rust (v3.0.0 Attribute Macros)

```rust
use clap_noun_verb_macros::{noun, verb};
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct GreetOutput {
    message: String,
    age: u32,
}

/// Greet a user
#[noun("users", "Manage users")]
#[verb("greet")]
fn greet(name: String, age: Option<u32>) -> Result<GreetOutput> {
    let age = age.unwrap_or(25);
    Ok(GreetOutput {
        message: format!("Hello, {}! You are {}", name, age),
        age,
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

**Key similarities:**
- ✅ Just add attribute/decorator to function
- ✅ Arguments inferred from function signature
- ✅ Help auto-generated from docstring
- ✅ Optional args use defaults
- ✅ JSON output by default

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
    clap_noun_verb::run() // Auto-discovers all #[verb] functions
}
```

**Usage:**
```bash
$ myapp services status
{"services":["api","worker"],"healthy":true}

$ myapp services logs api --lines 100
{"service":"api","lines":100,"entries":["log1","log2"]}
```

## Key Features

### 1. No Manual Registration

**Typer:**
```python
@app.command()
def greet(name: str):
    ...
```

**clap-noun-verb:**
```rust
#[verb("greet")]
fn greet(name: String) -> Result<Greeting> {
    ...
}
```

Both auto-discover functions at runtime/compile-time.

### 2. JSON by Default

Perfect for agents, MCP, and modern tooling. All output is automatically serialized to JSON.

### 3. Type Inference

Arguments are automatically inferred from function signatures:
- `String` → Required argument `--name`
- `Option<T>` → Optional argument `--name <value>`
- `bool` → Flag `--name` (true if present)
- `Vec<T>` → Multiple values

### 4. Docstring Help

Help text is automatically generated from Rust docstrings, including argument descriptions.

## Architecture: CLI = Input Validation + Output Shaping

**CRITICAL:** Functions with `#[verb]` are **ONLY** for:
1. **Input Validation** - Auto-validated from function signature
2. **Output Shaping** - Auto-serialized to JSON
3. **Delegation** - Delegate to pure business logic functions

**No business logic in CLI functions!** Business logic must be separate pure functions.

### Example with Separation

```rust
// ✅ Business Logic (Pure, Reusable)
fn get_status() -> Status {
    Status { services: vec!["api".to_string()], healthy: true }
}

// ✅ CLI Layer (Validation + Delegation + Output Shaping Only)
#[noun("services", "Manage services")]
#[verb("status")]
fn show_status() -> Result<Status> {
    // 1. Validate inputs (none here)
    // 2. Delegate to business logic
    Ok(get_status())
    // 3. Output shaping (auto-serializes to JSON)
}
```

## Benefits

1. ✅ **Decorator-like** - Familiar to Python/Typer users
2. ✅ **No manual registration** - Auto-discovery at compile time
3. ✅ **Minimal boilerplate** - Just add `#[verb(...)]` attribute
4. ✅ **Type-safe** - Compile-time checks
5. ✅ **JSON by default** - Perfect for agents/MCP
6. ✅ **Separation enforced** - CLI only validates/shapes, delegates to logic
