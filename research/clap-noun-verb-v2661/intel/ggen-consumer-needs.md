# ggen Consumer Needs Analysis
## clap-noun-verb v26.6.1 Dependency Research

**Project**: mcpp-cli (ggen CLI implementation as playground)
**Date**: 2026-06-01
**Framework Version**: clap-noun-verb v26.6.1
**Analysis Scope**: CLI command registration, execution, output formatting, error handling

---

## Executive Summary

ggen (mcpp-cli in the playground) is a moderately complex CLI application built on clap-noun-verb. It consumes:
- **Macro API**: `#[verb]` for command registration (17 commands across 18 files)
- **Runtime API**: `Result<T>`, `NounVerbError`, `OutputFormat`, `format_output()`
- **Registry API**: Implicit auto-discovery via `linkme` distributed slices
- **Error Contract**: Expects `Result<()>` and `Result<SomeType>` where `SomeType: Serialize`
- **Output Contract**: JSON default, with YAML/Table fallback support

---

## Consumer Profile

### Project Metadata
- **Binary Name**: `mcpp`
- **Library Name**: `mcpp_cli`
- **Crate Root**: `/Users/sac/clap-noun-verb/playground/`
- **Edition**: 2021
- **Rust Version**: 1.74+

### Dependency Footprint
```toml
clap-noun-verb = { path = ".." }
clap-noun-verb-macros = { path = "../clap-noun-verb-macros" }
clap-noun-verb-utils = { path = "../utils" }
```

---

## API INVENTORY

### MACRO API

#### `#[verb]` Attribute
**Used in**: 17 command handlers across 18 files

**Pattern**:
```rust
#[verb("action")]
fn handler_name(arg1: Type, arg2: Option<Type>) -> Result<OutputType>
```

**Observed Usage**:
1. **Command Registration**: Each `#[verb]` creates a subcommand entry
2. **Argument Inference**: Arguments derived from function signature
3. **Return Type Contract**: Can return `Result<()>` or `Result<SomeType: Serialize>`

**Detailed Mapping**:
- `config.rs`: 3 verbs (get, set, show)
- `pack.rs`: 7 verbs (add, remove, list, show, verify, graph, update)
- `receipt.rs`: 4 verbs (emit, verify, sign, log)
- `capability.rs`: Multiple verbs (enable, disable, show)
- `meta.rs`, `papers.rs`, `policy.rs`: 1-2 verbs each
- `sync.rs`, `telco.rs`, `wizard.rs`: 1-2 verbs each
- `ontology.rs`, `spec.rs`, `registry.rs`, `accept.rs`: 1 verb each
- `thesis.rs`, `powl8.rs`: 1 verb each

**Macro Features Used**:
- `#[arg(index = N)]` — Positional arguments
- `#[arg(...)]` — Attribute-level configuration (not doc comments)
- Return type inference (auto-serialization)

---

### RUNTIME API

#### Core Types

**1. Error Type: `NounVerbError`**

Used in every command that needs error reporting.

```rust
// From playground/src/commands/config.rs:52
clap_noun_verb::NounVerbError::execution_error(e.to_string())

// From playground/src/commands/pack.rs:19
NounVerbError::ExecutionError { message: e }
```

**Observed Error Patterns**:
- `ExecutionError { message: String }` — Wrapping domain errors
- `.execution_error()` — Helper constructor
- All errors flow through `Result<T>` for propagation via `?` operator

**2. Result Type: `clap_noun_verb::Result<T>`**

Pattern: `type Result<T> = std::result::Result<T, NounVerbError>`

Used as:
- `Result<()>` — Commands with side effects (emit, delete, save)
- `Result<OutputType: Serialize>` — Commands returning data (list, show, verify)

**Example**:
```rust
fn add_pack(identifier: String, version: Option<String>, force: bool) -> Result<PackAddedOutput>
fn receipt_emit(target: Option<String>, agent: Option<String>) -> Result<serde_json::Value>
```

**3. Output Formatting: `OutputFormat` & `format_output()`**

Used in 5 files:
- `config.rs` — Lines 14, 40-60 (frequent)
- `meta.rs` — Format enumeration/introspection
- `papers.rs` — Result formatting
- And others

**Pattern**:
```rust
use clap_noun_verb::{OutputFormat, format_output};

let fmt = format.as_deref()
    .and_then(|s| OutputFormat::from_str(s).ok())
    .unwrap_or(OutputFormat::JsonPretty);

let output = format_output(&data, fmt)?;
println!("{}", output);
```

**Supported Formats**:
- `json` — Compact JSON
- `json-pretty` — Default (pretty JSON)
- `yaml` — YAML output
- `table` — ASCII table
- `plain` — Key:value pairs
- `tsv` — Tab-separated

---

### REGISTRY & DISCOVERY API

**No explicit API usage**: ggen relies entirely on:
1. Auto-discovery via `linkme` (implicit in `#[verb]` macro)
2. Bulk initialization in `main.rs`:
   ```rust
   fn main() -> Result<()> {
       mcpp_cli::init();
       clap_noun_verb::run()
   }
   ```

**Observed Pattern**:
- `mcpp_cli::init()` → Forces linker to include all `#[verb]` entries
- `clap_noun_verb::run()` → Auto-discovers and runs CLI

**No Custom Registry Manipulation**: ggen does NOT use:
- `CommandRegistry::get()`
- `CommandRegistry::register()`
- Manual verb registration
- Custom routing

---

### TYPE EXPORTS

**From `clap_noun_verb`**:
```rust
pub use clap::{Arg, ArgAction, ArgMatches, Command};
pub use cli::run;
```

**Used in playground**:
- `clap::ArgMatches` — In `config_loader.rs:49` (creating empty matches)
- `clap::Command` — In builder patterns (rare, mostly implicit)

---

## OUTPUT CONTRACT

### Return Type Serialization

All `Result<T>` where `T: Serialize` are automatically converted to JSON.

**Observed Patterns**:

**Pattern 1: Structured Output Type**
```rust
#[derive(Serialize)]
pub struct PackAddedOutput {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub installed_at: String,
}

#[verb("add")]
fn add_pack(...) -> Result<PackAddedOutput> {
    Ok(PackAddedOutput { ... })
}
```

**Pattern 2: Direct JSON**
```rust
#[verb("emit")]
fn receipt_emit(...) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "schema": "chatmangpt.sr.result.v1",
        "command": "sr.receipt.emit",
        ...
    }))
}
```

**Pattern 3: Empty Result**
```rust
#[verb("set")]
fn set_config(...) -> Result<()> {
    eprintln!("⚠️  DEPRECATED: ...");
    Ok(())
}
```

---

## INTEGRATION PATTERNS

### Configuration Integration
**File**: `config.rs`

Uses `clap_noun_verb_utils::adapters::LayeredConfigAdapter<Config>`:
```rust
let adapter = LayeredConfigAdapter::<Config>::new(
    Some(std::path::PathBuf::from("ggen.toml")),
    Some("GGEN_".to_string()),
);
let empty_matches = clap_noun_verb::ArgMatches::default();
let config = adapter.resolve(&empty_matches)?;
```

**Key Insight**: Requires `ArgMatches::default()` constructor for non-CLI config scenarios.

### Error Wrapping Pattern
**Files**: `config.rs:52`, `pack.rs:19`

```rust
// Two styles observed:
NounVerbError::execution_error(e.to_string())
NounVerbError::ExecutionError { message: e }
```

---

## FEATURE USAGE

**Features NOT used** in ggen:
- `async` — All verbs are synchronous
- `io` — No `clio::Input`/`clio::Output` types
- `crypto` — Receipt signing uses `ed25519-dalek` directly
- `rdf` — RDF modules referenced but not in critical path
- `full` — Selective feature use

**Features implicitly used**:
- Default features (error handling, format, registry)

---

## TESTING PATTERNS

**File**: `tests/ggen_integration.rs`

Uses `clap_noun_verb::Result` for test error handling:
```rust
fn test_sync_dry_run() -> Result<()> {
    let temp_dir = create_test_workspace()?;
    // ... Act & Assert
    Ok(())
}
```

No direct registry API testing — all tests via domain logic.

---

## CLASSIFICATION SUMMARY

### REQUIRED_RUNTIME_API (CRITICAL)
- ✅ `clap_noun_verb::Result<T>` — Central error propagation
- ✅ `clap_noun_verb::NounVerbError` — Error construction (`execution_error()`, `ExecutionError`)
- ✅ `clap_noun_verb::OutputFormat` — Format selection
- ✅ `clap_noun_verb::format_output()` — Output serialization
- ✅ `clap::ArgMatches` (re-exported) — Used in adapter patterns
- ✅ `clap_noun_verb::run()` — Main CLI entry point

### REQUIRED_MACRO_API (CRITICAL)
- ✅ `#[clap_noun_verb_macros::verb]` — 17 command registrations
- ✅ `#[arg(index = N)]` — Positional argument binding

### REQUIRED_TRAIT_API
- ✅ `serde::Serialize` — All output types must implement
- ✅ Implicit `std::str::FromStr` — For `OutputFormat` parsing

### REQUIRED_OUTPUT_CONTRACT
- ✅ JSON serialization (default, auto)
- ✅ YAML fallback support
- ✅ Table/Plain format support
- ✅ `serde_json::Value` compatibility

### REQUIRED_REGISTRY_CONTRACT
- ✅ Implicit `linkme` auto-discovery
- ✅ `mcpp_cli::init()` → linker forcing
- ✅ `clap_noun_verb::run()` → bulk discovery + execution

### HISTORICAL_USAGE
- `#[noun]` — NOT observed in ggen (deprecated in v26.6.1)
- Custom `CommandRegistry` API — NOT used (all auto-discovery)
- Manual routing — NOT used (implicit in `#[verb]`)

### MIGRATION_REQUIRED
None observed. ggen is **forward-compatible** with current API.

### NO_LONGER_USED
- Doc comment argument tags — Code uses inline `#[arg(...)]` attributes, not doc string parsing

---

## API STABILITY REQUIREMENTS

### Backward Compatibility Must Preserve
1. **Macro signatures**: `#[verb("action")]`, `#[arg(index = N)]`
2. **Error constructors**: `NounVerbError::execution_error()`, `ExecutionError { message }`
3. **Result type alias**: `type Result<T> = std::result::Result<T, NounVerbError>`
4. **OutputFormat enum**: All variants (Json, JsonPretty, Yaml, Table, Plain, Tsv, Quiet)
5. **format_output function**: `fn(value: &S, fmt: OutputFormat) -> Result<String>`
6. **run() function**: No-arg entry point, returns `Result<()>`
7. **ArgMatches re-export**: Via `pub use clap::ArgMatches`

### Safe to Change (Internal)
- Implementation of `format_output()` (new formatters, optimizations)
- Internal error types (as long as public API remains)
- Registry mechanics (as long as `#[verb]` + `run()` work unchanged)

---

## Performance & Scale

**Command Count**: 17 verbs across 18 files
**Compilation Time**: Not benchmarked in this analysis
**Runtime Characteristics**:
- Synchronous handlers (no async)
- JSON serialization on every output
- Single-threaded execution per command

---

## Conclusion

ggen is a **well-structured, mature consumer** of clap-noun-verb's core APIs. It:
1. Uses `#[verb]` exclusively for command registration
2. Returns `Result<T: Serialize>` for all handlers
3. Relies on implicit auto-discovery (no custom registry code)
4. Integrates output formatting via `OutputFormat::from_str()` + `format_output()`
5. Wraps domain errors with `NounVerbError::execution_error()`

**Risk Assessment**: MINIMAL
- No private API usage
- No unusual error handling patterns
- No custom macro extensions
- Straightforward integration with utils layer

**Recommendation**: v26.6.1 → v27.0.0+ migration should be safe as long as the 7 API stability requirements above are preserved.
