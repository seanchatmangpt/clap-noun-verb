# clap-noun-verb

**Machine-grade CLI framework for AI agents and autonomous systems**

[![Crates.io](https://img.shields.io/crates/v/clap-noun-verb)](https://crates.io/crates/clap-noun-verb)
[![Documentation](https://docs.rs/clap-noun-verb/badge.svg)](https://docs.rs/clap-noun-verb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE.md)

**Current Version**: v5.2.0 | [Changelog](CHANGELOG.md) | [Phase 2 Features](docs/phase2-analysis.md)

> **Architecture First:** CLI is interface, not application. Separate domain logic from CLI.

---

## Documentation by Use Case

### 🎓 Learning from Scratch?
**Start:** [Your First CLI in 5 Minutes](docs/tutorial/01-your-first-cli.md)
→ [Tutorial Series](docs/tutorial/) — 10 progressive lessons (5 mins - 3 hours)

### 🔧 Solving a Problem?
**Start:** [How-To Production Guides](docs/howto/production/)
→ Deployment, monitoring, configuration, security

### 📖 Looking Up an API?
**Start:** [Reference: #[verb] Macro](docs/reference/api/verb-macro.md)
→ [All Reference Docs](docs/reference/api/) — Types, errors, CLI runner

### 🤔 Understanding Design?
**Start:** [Architecture Philosophy](docs/explanation/)
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
clap-noun-verb = "5.2"
```

For development: also add `clap-noun-verb-macros = "5.2"`

---

## 2-Minute Example

```rust
use clap_noun_verb_macros::{noun, verb};
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct CalcResult { result: i32 }

// Business logic (pure, testable)
fn add(x: i32, y: i32) -> i32 { x + y }

// CLI wrapper (thin, delegating)
#[noun("calc", "Calculator")]
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

## v5.2.0 Features (Phase 2)

**Typer-like Doc Comment Syntax** for argument relationships:

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

See [Phase 2 Analysis](docs/phase2-analysis.md) for complete details.

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
