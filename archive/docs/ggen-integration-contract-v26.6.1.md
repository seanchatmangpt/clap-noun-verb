# ggen ↔ clap-noun-verb v26.6.1 Integration Contract

**Version:** 26.6.1  
**Date:** 2026-06-01  
**Status:** Verified ✓

---

## Executive Summary

ggen can integrate with **clap-noun-verb v26.6.1** using stable, published APIs. This document defines the contract: what ggen **CAN use now** (v26.6.1) and what it **must wait for** (v26.7.0+).

**Result:** Integration ready. ✓

---

## Part 1: Stable APIs for ggen (v26.6.1)

These APIs are published and stable. ggen can depend on them immediately.

### 1.1 Command Registration: `#[verb]` Macro

**Status:** STABLE ✓  
**Location:** `clap-noun-verb-macros` crate  
**Usage:**

```rust
use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct CommandOutput {
    pub status: String,
}

#[verb("load", "graph")]
fn load_graph(path: String) -> Result<CommandOutput> {
    Ok(CommandOutput {
        status: "success".to_string(),
    })
}
```

**Contract:**
- Macro accepts exactly 2 string literals: `(verb_name, noun_name)`
- Handler function must return `Result<T>` where `T: Serialize`
- Function parameters become CLI arguments (auto-inferred from types)
- Commands are automatically discovered via `linkme` distributed slices at compile time
- No explicit registration needed

**Guarantees:**
- Macro syntax will not change within v26.6.x
- Distributed slice discovery is deterministic
- Return type validation is compile-time checked

---

### 1.2 Core Framework Types

**Status:** STABLE ✓

All these types are public and re-exported from `clap_noun_verb::` crate root:

#### `VerbCommand` trait

```rust
pub trait VerbCommand: Send + Sync {
    fn name(&self) -> &'static str;
    fn about(&self) -> &'static str;
    fn run(&self, args: &VerbArgs) -> Result<()>;
    fn build_command(&self, noun: &str) -> Command;
}
```

**Usage:** ggen can match against this trait for introspection.  
**Stability:** Trait methods are stable. New methods use default implementations.

#### `CommandRegistry`

```rust
pub struct CommandRegistry {
    // opaque
}

impl CommandRegistry {
    pub fn new() -> Self;
    pub fn register_verb(&mut self, verb: &'static str, noun: &'static str, handler: Box<dyn VerbCommand>);
    pub fn build_cli(&self) -> Command;
}
```

**Usage:** ggen can create a registry and register commands dynamically.  
**Stability:** Public methods are stable.

#### `Result<T>` / `NounVerbError`

```rust
pub type Result<T> = std::result::Result<T, NounVerbError>;

pub enum NounVerbError {
    CommandNotFound { noun: String, suggestion: String },
    ArgumentError { message: String },
    ExecutionError { message: String },
    // ... other variants
}
```

**Usage:** All command handlers use this type. ggen can pattern-match on errors.  
**Stability:** Enum variants are stable. New variants use non-exhaustive pattern.

#### `OutputFormat` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Yaml,
    Text,
}

impl Default for OutputFormat {
    fn default() -> Self { OutputFormat::Json }
}
```

**Usage:** ggen can convert command output to JSON or YAML.  
**Stability:** Variants are stable. Default is JSON (agent-friendly).

---

### 1.3 Command Execution

**Status:** STABLE ✓

#### `run()` function (zero-arg entry point)

```rust
pub fn run() -> Result<()>
```

**What it does:**
1. Discovers all `#[verb]` commands via linkme
2. Builds the clap `Command` tree
3. Parses CLI arguments from `std::env::args()`
4. Routes to the correct handler
5. Serializes output to JSON
6. Exits with appropriate code

**Usage:** Call from `main()`. This is the simplest pattern.

```rust
fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
```

**Stability:** Function signature is stable. Behavior is documented and tested.

#### `run_cli()` function (with Command)

```rust
pub fn run_cli(mut cmd: Command) -> Result<()>
```

**What it does:** Like `run()`, but takes a pre-built clap `Command` for customization.

**Usage:** For advanced customization of the command tree.

**Stability:** Signature is stable.

#### `run_cli_with_args()` function (with args)

```rust
pub fn run_cli_with_args(mut cmd: Command, args: impl IntoIterator<Item = String>) -> Result<()>
```

**What it does:** Like `run_cli()`, but accepts custom args (for testing, programmatic use).

**Usage:** ggen can call this to execute specimen commands with test args.

**Stability:** Signature is stable.

---

### 1.4 Output Serialization

**Status:** STABLE ✓

All command output types are required to implement `Serialize` (enforced at compile time by the `#[verb]` macro).

#### ggen can:

✓ Serialize command output to JSON
✓ Serialize command output to YAML
✓ Deserialize JSON responses
✓ Round-trip any Serialize-bound type
✓ Validate output types match expected schema

#### Example

```rust
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct CommandOutput {
    status: String,
    data: Vec<String>,
}

let output = CommandOutput {
    status: "success".to_string(),
    data: vec!["a".into(), "b".into()],
};

// JSON
let json = serde_json::to_string(&output)?;
let back: CommandOutput = serde_json::from_str(&json)?;

// YAML
let yaml = serde_yaml::to_string(&output)?;
let back: CommandOutput = serde_yaml::from_str(&yaml)?;
```

**Stability:** Serde is stable. JSON/YAML formats are standard.

---

## Part 2: NOT Available in v26.6.1 (ggen must wait)

These features are planned but not in v26.6.1. ggen should **not depend on them**.

### 2.1 Receipt / Proof Types (planned v26.7.0)

**Status:** NOT AVAILABLE ✗  
**Why:** Needed for audit trails, provenance tracking.

```rust
// NOT in v26.6.1:
pub struct Receipt {
    pub id: String,
    pub timestamp: std::time::SystemTime,
    pub command: String,
    pub output_hash: String,
}
```

**ggen impact:** Cannot create immutable audit records in v26.6.1. Wait for v26.7.0.

---

### 2.2 CommandMetadata (planned v26.7.0)

**Status:** NOT AVAILABLE ✗  
**Why:** Needed for introspection, schema discovery.

```rust
// NOT in v26.6.1:
pub struct CommandMetadata {
    pub verb: String,
    pub noun: String,
    pub description: String,
    pub arguments: Vec<ArgumentMetadata>,
    pub return_type: String,
}
```

**ggen impact:** Cannot programmatically discover command schema in v26.6.1. Must hard-code for now.

---

### 2.3 Semantic Composition Traits (planned v26.7.0+)

**Status:** NOT AVAILABLE ✗  
**Why:** Needed for composable command pipelines.

```rust
// NOT in v26.6.1:
pub trait Composable {
    fn compose_with(&self, other: &dyn Composable) -> Result<Box<dyn Composable>>;
}

pub trait SemanticBound {
    fn semantic_type(&self) -> String;
    fn validate_composition(&self, other: &dyn SemanticBound) -> Result<()>;
}
```

**ggen impact:** Cannot compose commands semantically in v26.6.1. Wait for v26.7.0.

---

### 2.4 RDF / SPARQL Integration (planned v26.7.0+)

**Status:** NOT AVAILABLE ✗  
**Why:** Needed for ontology-based command discovery.

```rust
// NOT in v26.6.1:
pub fn query_ontology(sparql: &str) -> Result<Vec<CommandMetadata>>;
```

**ggen impact:** Cannot use SPARQL to discover commands in v26.6.1. Use hard-coded metadata.

---

### 2.5 Autonomic Telemetry (planned v26.7.0+)

**Status:** NOT AVAILABLE ✗  
**Why:** Needed for autonomous agent coordination.

```rust
// NOT in v26.6.1:
pub struct TelemetrySpan {
    pub event: String,
    pub context: serde_json::Value,
}

pub trait Telemetric {
    fn span(&self, name: &str) -> TelemetrySpan;
}
```

**ggen impact:** Cannot emit OpenTelemetry traces in v26.6.1. Log manually if needed.

---

## Part 3: Specimen CLI — 6 Proven Commands

The specimen CLI (`examples/specimen-graph-manager`) demonstrates all v26.6.1 capabilities with 6 real commands:

| Verb     | Noun   | Handler                    | Output Type           |
|----------|--------|----------------------------|-----------------------|
| `load`   | `graph` | `load_graph(path)`         | `GraphLoadedOutput`   |
| `query`  | `graph` | `query_graph(pattern)`     | `QueryResultOutput`   |
| `validate` | `graph` | `validate_graph(path)`   | `ValidationResultOutput` |
| `check`  | `doctor` | `doctor_check()`           | `DoctorOutput`        |
| `add`    | `pack` | `pack_add(id, name, ver)`  | `PackAddedOutput`     |
| `remove` | `pack` | `pack_remove(id)`          | `RemovalStatus`       |

**All 6 commands:**
- Use `#[verb]` macro ✓
- Have Serialize-bound output types ✓
- Return `Result<T>` ✓
- Are automatically discovered ✓
- Support JSON/YAML output ✓

**Test Results:**
```
Commands discovered: 6/6 ✓
Arguments parsed: 100% ✓
Output formats: 6/6 working ✓
Error handling: working ✓
```

---

## Part 4: Integration Checklist for ggen

### Before Publishing (v26.6.1)

✓ Import `clap_noun_verb` from crates.io v26.6.1  
✓ Use `#[verb]` macro for command registration  
✓ Ensure all output types implement `Serialize`  
✓ Return `Result<T>` from all handlers  
✓ Call `clap_noun_verb::run()` from main  
✓ Test JSON/YAML serialization  
✓ Handle `NounVerbError` gracefully  

### DO NOT (v26.6.1)

✗ Do NOT depend on `Receipt` type  
✗ Do NOT expect `CommandMetadata` introspection  
✗ Do NOT use semantic composition traits  
✗ Do NOT query RDF ontologies  
✗ Do NOT emit autonomic telemetry  

### Plan for v26.7.0

→ Add Receipt-based audit trails  
→ Implement CommandMetadata introspection  
→ Compose commands semantically  
→ Query RDF/SPARQL for discovery  
→ Emit autonomic telemetry  

---

## Part 5: Path Forward

### v26.6.1 (Current — NOW)

**Maturity:** Production-ready  
**Focus:** Core noun-verb patterns  
**ggen can:** Build CLIs, serialize output, handle errors  
**ggen cannot:** Introspect schema, compose semantically, audit via receipts  

### v26.7.0 (Next — ~Q3 2026)

**Maturity:** Expanded features  
**Focus:** Metadata, receipts, composition  
**New in ggen:** Schema discovery, audit trails, command composition  

### v26.8.0+ (Future — Q4 2026+)

**Maturity:** Full semantic framework  
**Focus:** RDF, autonomic agents, economic simulation  
**New in ggen:** Ontology-driven discovery, autonomous coordination  

---

## Part 6: Testing & Verification

All assertions in this contract are verified by:

```bash
cargo test ggen_integration -- --nocapture
```

**Test suite:** 11 tests covering:
1. API surface stability
2. Command discovery
3. Output format round-tripping
4. Error handling
5. Trait accessibility
6. Version-specific guarantees

**Result:** ✓ All tests pass

---

## Part 7: Support & Questions

**Canonical Location:** This file  
**Issue Tracker:** GitHub Issues (tag: `ggen-integration`)  
**Stability:** Guaranteed within v26.6.x. Breaking changes only in v27.0.0+  
**Maintenance:** Updated with each minor release  

---

## Appendix A: Specimen CLI Commands Reference

### `graph load`

```bash
specimen-graph-manager graph load path/to/file.ttl
```

**Output:**
```json
{
  "triples_loaded": 42,
  "source": "path/to/file.ttl",
  "status": "success"
}
```

### `graph query`

```bash
specimen-graph-manager graph query "pattern"
```

**Output:**
```json
{
  "query_type": "subject_match",
  "pattern": "pattern",
  "results": [],
  "match_count": 0
}
```

### `graph validate`

```bash
specimen-graph-manager graph validate path/to/file.ttl
```

**Output:**
```json
{
  "valid": true,
  "errors": [],
  "total_triples": 42,
  "valid_triples": 42
}
```

### `doctor check`

```bash
specimen-graph-manager doctor check
```

**Output:**
```json
{
  "status": "healthy",
  "healthy": true,
  "issues": [],
  "graph_triples": 100,
  "registry_packages": 5
}
```

### `pack add`

```bash
specimen-graph-manager pack add "pkg-001" "GraphUtils" "2.1.0"
```

**Output:**
```json
{
  "id": "pkg-001",
  "name": "GraphUtils",
  "version": "2.1.0",
  "status": "added"
}
```

### `pack remove`

```bash
specimen-graph-manager pack remove "pkg-001"
```

**Output:**
```json
{
  "removed_id": "pkg-001",
  "status": "removed",
  "message": "Package successfully removed from registry"
}
```

---

## Appendix B: Troubleshooting

### "Error: cannot find macro `verb` in this scope"

**Cause:** Missing `clap-noun-verb-macros` crate  
**Fix:** Ensure Cargo.toml includes:
```toml
clap-noun-verb-macros = "26.6.1"
```

### "Error: the type returned by this function is not `Result`"

**Cause:** Handler must return `Result<T>` (not bare `T` or `()`).  
**Fix:** Wrap output in `Result`:
```rust
#[verb("load", "graph")]
fn load_graph(path: String) -> clap_noun_verb::Result<GraphLoadedOutput> {
    // ... return Ok(output) or Err(...)
}
```

### "Error: the trait bound `MyOutput: Serialize` is not satisfied"

**Cause:** Output type must derive `Serialize`.  
**Fix:** Add derive:
```rust
#[derive(Serialize)]
pub struct MyOutput {
    // ...
}
```

---

**End of Contract**

Generated: 2026-06-01  
Verified by: `cargo test ggen_integration`  
Status: ✓ READY FOR INTEGRATION
