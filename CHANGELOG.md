# Changelog

All notable changes to clap-noun-verb will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
