# clap-noun-verb

**Machine-grade CLI framework for AI agents and autonomous systems**

[![Crates.io](https://img.shields.io/crates/v/clap-noun-verb)](https://crates.io/crates/clap-noun-verb)
[![Documentation](https://docs.rs/clap-noun-verb/badge.svg)](https://docs.rs/clap-noun-verb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

**Current Version**: "26.5.28" | [Changelog](CHANGELOG.md) | [API Reference](docs/reference/api-catalog.md)

> **Architecture First:** CLI is interface, not application. Separate domain logic from CLI.

---

## Documentation by Use Case

### 🎓 Learning from Scratch?
**Start:** [Domain Separation Architecture](docs/tutorial/02-domain-separation.md)
→ [Tutorial Series](docs/tutorial/README.md) — 6 progressive lessons (10 mins - 2 hours)

### 🔧 Solving a Problem?
**Start:** [How-To Guides](docs/howto/README.md)
→ [How-To Production Guides](docs/howto/production/deployment.md) — Deployment, monitoring, configuration, security

### 📖 Looking Up an API?
**Start:** [Reference: #[verb] Macro](docs/reference/api/verb-macro.md)
→ [API Reference](docs/reference/README.md) — Types, errors, CLI runner

### 🤔 Understanding Design?
**Start:** [Architecture Philosophy](docs/explanation/README.md)
→ Why domain separation, type-first thinking, agent-grade CLIs

---

## Architecture Principle

**The Golden Rule:** CLI validates, domain computes, integration connects.

```
┌─────────────┐
│   CLI Layer │  ← clap-noun-verb (this crate)
│  (thin, UI) │
└──────┬──────┘
       │
┌──────▼──────────┐
│ Integration     │  ← Glue code (minimal)
└──────┬──────────┘
       │
┌──────▼──────────┐
│  Domain Logic   │  ← Your business logic (pure, testable)
│  (pure, tested) │
└─────────────────┘
```

**Why this matters:**
- CLI layer is thin validation and routing
- Domain logic is pure Rust functions (testable, reusable)
- Integration glues CLI to domain
- Tests focus on domain, not CLI parsing

---

## Installation

```toml
[dependencies]
clap-noun-verb = "26.5.28"
```

For development: also add `clap-noun-verb-macros = "26.5.28"`

---

## 2-Minute Example

```rust
use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct CalcResult { result: i32 }

// Business logic (pure, testable)
fn add(x: i32, y: i32) -> i32 { x + y }

// CLI wrapper (thin, delegating)
//
// Noun "calc" is auto-detected from the filename (e.g., calc.rs)
#[verb("add")]
fn cmd_add(x: i32, y: i32) -> Result<CalcResult> {
    Ok(CalcResult { result: add(x, y) })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

**Usage:**
```bash
$ myapp calc add 2 3
{"result": 5}
```

**Key:** Delegate to pure domain logic immediately. CLI only validates.

---

## Doc Comment Tags

**Typer-like Doc Comment Syntax** for argument configuration:

```rust
/// # Arguments
/// * `format` - Output format [env: OUTPUT_FORMAT] [default: json]
/// * `json` - Export as JSON [group: format]
/// * `yaml` - Export as YAML [group: format]
/// * `output` - Output file [requires: format] [value_hint: FilePath]
#[verb("export")]
fn export(json: bool, yaml: bool, format: Option<String>, output: Option<String>) -> Result<Output> {
    // [group:] makes json and yaml mutually exclusive
    // [requires:] ensures output needs format
    // [env:] and [value_hint:] provide sensible defaults/hints
}
```

**New tags in v5.2.0:**
- `[group: name]` - Exclusive argument group
- `[requires: arg]` - Argument dependency
- `[conflicts: arg]` - Mutually exclusive arguments
- `[env: VAR]` - Read from environment
- `[default: value]` - Default value
- `[value_hint: type]` - Shell completion hint
- `[hide]` - Hide from help
- `[help_heading: name]` - Organize help output
- `[global]` - Propagate to subcommands
- `[exclusive]` - Can't combine with other args

See [API Catalog](docs/reference/api-catalog.md) for complete details.

---

## Advanced Features (v5.6+)

### ⛓️ In-Process Command Chaining (`++`)
Execute multiple commands sequentially within a single process run. Separate independent steps using `++`. Steps are executed in order, and results from preceding steps can be interpolated into subsequent step arguments using standard JSONPath-like notation `@{<step_index>.<key>}` (1-based index):
```bash
$ myapp session login john_doe ++ session verify @{1.token}
```

### 📥 Stdin Stream Extraction (`@-` and `@-::key`)
Inject standard input streams dynamically into command arguments:
- Use `@-` to read the entire stdin buffer as a raw string argument.
- Use `@-::json.path` to parse stdin as JSON and extract nested attributes.

```bash
# Read raw stdin string:
$ echo "my-secret-key" | myapp auth login --key @-

# Extract specific JSON key:
$ echo '{"user": {"token": "abc123xyz"}}' | myapp session verify @-::user.token
```

### 🐚 Dynamic Shell Completions Command
Generate dynamic tab-completions scripts for `bash`, `zsh`, `fish`, and `powershell`. Simply register the completions subcommand using the fluent builder:

```rust
fn main() -> Result<()> {
    clap_noun_verb::build_cli()
        .with_completions_subcommand()
        .run()
}
```
Users can then run:
```bash
$ myapp completions zsh > ~/.zsh/completion/_myapp
```

### 🤖 LLM Agent Introspection (`--introspect`)
Query capability metadata dynamically. Passing the global `--introspect` flag instructs the CLI to output all registered commands as a standard JSON Schema array of tools. This format is fully compatible with OpenAI, Anthropic, and Model Context Protocol (MCP) tool-calling specifications:
```bash
$ myapp --introspect
```

### 🚨 Autonomic Structured Errors (`--structured-errors`)
The `--structured-errors` (or `--autonomic`) global flag formats errors using a machine-readable JSON format matching the autonomic MAPE-K control loop pattern:
```bash
$ myapp calc add invalid_arg --structured-errors
```
Output:
```json
{
  "kind": "InvalidInput",
  "severity": "Error",
  "message": "Argument parsing failed: invalid digit found in string",
  "details": {
    "message": "invalid digit found in string"
  },
  "action_templates": []
}
```
If a command is misspelled, suggestion action templates are returned:
```json
{
  "kind": "CommandNotFound",
  "severity": "Error",
  "message": "Command 'cal' not found. Did you mean: calc?",
  "details": {
    "noun": "cal",
    "suggestion": ". Did you mean: calc?"
  },
  "action_templates": [
    {
      "suggested_command": "calc",
      "reason": "Suggested correction for misspelled command 'cal'"
    }
  ]
}
```

### 💬 Interactive REPL Mode
Start an interactive shell loop with auto-completions and persistent history. (Requires the `repl` feature flag).
To run:
```rust
#[cfg(feature = "repl")]
{
    let registry = clap_noun_verb::CommandRegistry::get();
    let repl = clap_noun_verb::Repl::new(registry.lock().unwrap().clone())
        .with_history_file(std::path::PathBuf::from(".myapp_history"));
    repl.run()?;
}
```
Inside the interactive REPL shell:
```bash
myapp> calc add 2 3
{"result": 5}
myapp> exit
```

For more details on completions, parameter chaining, stdin extraction, LLM introspection, and REPL mode, see the [Advanced Features Reference](docs/reference/api/advanced-features.md). For distributed tracing, metrics, and W3C traceparents, see the [Telemetry Reference](docs/reference/api/telemetry.md).

---

## Key Highlights

✅ **Type-Safe By Construction** - Compile-time validation of commands
✅ **Zero-Cost Abstractions** - Generics & macros, no runtime overhead
✅ **Domain-Separated** - Thin CLI layer + pure domain logic
✅ **Agent-Ready** - JSON output, introspection, MCP compatible
✅ **Production Tested** - 100% pass rate, comprehensive examples

---

## Contributing

Issues and PRs welcome: [github.com/seanchatmangpt/clap-noun-verb](https://github.com/seanchatmangpt/clap-noun-verb)

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
