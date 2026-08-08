# clap-noun-verb

[![Crates.io](https://img.shields.io/crates/v/clap-noun-verb)](https://crates.io/crates/clap-noun-verb)
[![Documentation](https://docs.rs/clap-noun-verb/badge.svg)](https://docs.rs/clap-noun-verb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

`clap-noun-verb` is a declarative, type-safe, and agent-ready noun-verb CLI framework for Rust, built on top of the powerful `clap` library. It enables developers to structure their command-line tools around domain entities (nouns) and actions (verbs), offering zero-boilerplate registration, compile-time validation, automatic JSON output, and machine-grade interfaces out-of-the-box.

---

## 1. Introduction & Overview

The **noun-verb pattern** organizes commands hierarchically, separating domain concepts from operations. Instead of a flat list of commands (e.g., `login`, `create-user`, `list-services`), commands are grouped logically:

```bash
myapp session login             # Noun: session, Verb: login
myapp services status           # Noun: services, Verb: status
myapp users create --name Bob   # Noun: users,    Verb: create
```

This maps directly to your domain model, where a **Noun** is a resource/entity, and a **Verb** is an action on that resource.

### Key Framework Principles

* **Zero Boilerplate:** Commands are registered simply by adding the `#[verb]` attribute to your handler functions. No manual routing or CLI mapping code is needed.
* **Compile-Time Auto-Discovery:** Using `linkme` distributed slices, all command handlers are automatically discovered and registered at compile time.
* **Automatic Type Inference & Validation:** CLI arguments and option flags are inferred directly from Rust function parameter types. Help text is extracted from doc comments.
* **JSON Output by Default:** All command outputs are automatically formatted as JSON, providing a clean, parseable interface for shell scripts, AI agents, Model Context Protocol (MCP) servers, and other automation tooling.
* **Minimalist Architecture:** The core framework compiles with minimal dependencies, ensuring fast builds and small binary sizes. Extra capabilities are modularized behind Cargo feature flags.

---

## 2. Installation & Cargo Features

Add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "26.7.4"
clap-noun-verb-macros = "26.7.4"
serde = { version = "1.0", features = ["derive"] }
```

### Cargo Feature Flags

All core features (dispatch, chaining `++`, stdin extraction `@-`, completions, `--introspect`, structured errors, telemetry, validators, graph, capability, and diagnostics) are available by default. Advanced functionality can be enabled via modular cargo feature flags:

| Feature Flag | Default | Extra Dependencies | Description |
|:---|:---:|:---|:---|
| **`repl`** | No | `rustyline` | Enables an interactive REPL shell mode (`Repl::new(registry).run()`). |
| **`otel`** | No | `tracing`, `tracing-subscriber`, `tracing-opentelemetry`, `opentelemetry`, `opentelemetry_sdk` | OpenTelemetry instrumentation for CLI dispatch paths (spans, tracing). |
| **`process-data`** | No | None | Process data pipeline features. |
| **`autonomic`** | No | None | Autonomic CI/CD policies (implies `process-data`). |
| **`contrib`** | No | None | Contributor features/verbs (implies `process-data`). |
| **`federated-network`** | No | None | Federated node discovery and network operations. |

---

## 3. Quick Start (5-Minute Demo)

Follow these steps to build your first noun-verb CLI.

### 1. Create a new Rust project
```bash
cargo new myapp && cd myapp
cargo add clap-noun-verb clap-noun-verb-macros serde
```

### 2. Write the CLI code in `src/main.rs`
```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// --- Business Logic / Domain Layer ---
// Pure functions, reusable and independent of the CLI.
fn get_services() -> Vec<String> {
    vec!["database".to_string(), "web-api".to_string(), "cache".to_string()]
}

fn restart_service(name: &str) -> bool {
    // Perform restart logic...
    true
}

// --- CLI / Presentation Layer ---

#[derive(Serialize, Debug)]
pub struct StatusOutput {
    pub services: Vec<String>,
    pub status: String,
}

#[derive(Serialize, Debug)]
pub struct RestartOutput {
    pub service: String,
    pub restarted: bool,
}

/// Show the status of all application services
#[verb("status", "services")] // Verb: status, Noun: services
fn cmd_status() -> Result<StatusOutput> {
    Ok(StatusOutput {
        services: get_services(),
        status: "healthy".to_string(),
    })
}

/// Restart a specific service
#[verb("restart", "services")] // Verb: restart, Noun: services
fn cmd_restart(
    /// Name of the service to restart
    name: String,
    /// Force restart immediately even if active [default: false]
    #[arg(long)]
    force: bool,
) -> Result<RestartOutput> {
    let success = restart_service(&name);
    Ok(RestartOutput {
        service: name,
        restarted: success,
    })
}

fn main() -> Result<()> {
    // Auto-discovers and routes all commands
    clap_noun_verb::run()
}
```

### 3. Run the CLI

Execute the status command:
```bash
$ cargo run -- services status
{"services":["database","web-api","cache"],"status":"healthy"}
```

Execute the restart command (passing positional argument and boolean flag):
```bash
$ cargo run -- services restart web-api --force
{"service":"web-api","restarted":true}
```

View the generated help interface:
```bash
$ cargo run -- --help
```

---

## 4. Core Concepts

### Nouns & Verbs
* **Nouns (Resources):** Represent domain entities or subsystems (e.g., `services`, `users`, `configs`). 
  * Nouns can be defined explicitly in the attribute macro: `#[verb("verb_name", "noun_name")]`.
  * Alternatively, nouns can be **auto-inferred** from the source file name (e.g., a file named `services.rs` will automatically group all its `#[verb]` commands under the `services` noun) and describe it using the module level doc comments (`//! ...`).
* **Verbs (Actions):** Represent operations performed on the noun (e.g., `status`, `restart`, `create`).
  * Declared using the `#[verb]` attribute on handler functions.
  * Verbs are automatically converted to subcommands under the corresponding noun. 
  * Verb names are auto-inferred from function names (e.g., `fn status()` -> verb name `"status"`, stripping redudant noun prefixes automatically) if not specified explicitly.

### Parameter and Flag Mapping
* Function parameter names map directly to CLI flags or arguments.
* Command-line flags are automatically normalized to idiomatic `kebab-case` (e.g., parameter `dry_run` maps to `--dry-run`).
* Positional arguments vs Option flags:
  * Regular parameters (like `name: String`) represent required positional arguments.
  * Parameters wrapped in `Option<T>` or annotated with `#[arg(long)]` represent optional CLI flags.
* Help texts are auto-extracted from the doc comments (`///`) placed on function parameters and the function itself.
* Validation tags can be added directly to parameters:
  * `#[validate(min = 10, max = 100)]`
  * `#[validate(min_length = 3, max_length = 50)]`

### Compile-Time Poka-Yoke (Error-Proofing) Guards
To enforce domain separation and prevent architectural leakage, the macro framework implements compile-time validation:
1. **CLI Layer Purity (Complexity limit ≤ 5):** The cyclomatic complexity of a `#[verb]` function is measured at compile-time. If it exceeds `5`, compilation fails. This prevents developers from writing complex business logic inside the CLI handler, forcing delegation to pure domain functions.
2. **Domain Independence (No CLI type contamination):** Handler functions are forbidden from accepting internal CLI/framework types (e.g., `ArgMatches`, `Command`, `VerbContext`, `VerbArgs`, or `HandlerInput`) as parameters.
3. **Serialization Contract:** The return type of any `#[verb]` function must implement `serde::Serialize` (typically returning `Result<T>` or `Option<T>` where `T: Serialize`), ensuring output formatting safety.

---

## 5. Introspection & Command Operations

`clap-noun-verb` includes built-in machinery designed for automation and integration with Large Language Models (LLMs).

### LLM Introspection (`--introspect`)
Passing the global `--introspect` flag immediately dumps a standard JSON Schema array of `ToolDefinition`s describing every registered command and its parameters:

```bash
$ myapp --introspect
```

The output format is directly compatible with LLM tool-calling schemas (e.g., OpenAI or Anthropic):

```json
[
  {
    "name": "services_status",
    "description": "Show the status of all application services",
    "parameters": {
      "type": "object",
      "properties": {},
      "required": []
    }
  },
  {
    "name": "services_restart",
    "description": "Restart a specific service",
    "parameters": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Name of the service to restart"
        },
        "force": {
          "type": "boolean",
          "description": "Force restart immediately even if active"
        }
      },
      "required": ["name"]
    }
  }
]
```

### In-Process Command Chaining (`++`)
Multiple commands can be chained in a single execution using the `++` separator. Execution runs sequentially, and the outputs of preceding commands can be reference-injected into subsequent commands using step-results notation: `@{step_index.json_path}` (1-based indexing).

```bash
$ myapp session login john_doe ++ session verify @{1.token}
```
In this example:
1. Step 1 executes `session login john_doe` and returns a JSON payload (e.g., `{"token": "xyz-99", "active": true}`).
2. Step 2 extracts `token` from Step 1's output (`@{1.token}`) and passes it as the argument to `session verify`.

### Stdin Redirection (`@-`)
To easily pipeline structured data or secrets without exposing them in shell history, use stdin redirection:
* **Raw Stdin Input (`@-`):** Pulls the entire raw stdin payload into an argument.
  ```bash
  echo "secret-api-token" | myapp session verify @-
  ```
* **Structured JSON Stdin Extraction (`@-::json_path`):** Parses the stdin as JSON and extracts a specific nested field.
  ```bash
  echo '{"auth": {"token": "my-secret-key"}}' | myapp session verify @-::auth.token
  ```

---

## 6. Verification

To verify that your CLI configuration, macros, and validations compile and run correctly, execute:

```bash
cargo test
```

This runs:
* **Unit Tests:** Verifies noun-verb registrations, auto-discovery slices, and parameter validation logic.
* **Doc Tests:** Verifies all code examples and quick starts in documentation compile and execute without errors.
* **Integration Tests:** Asserts pipeline operations, command chaining (`++`), stdin preprocessing (`@-`), and introspection capabilities.
