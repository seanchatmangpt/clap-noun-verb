# ggen Producer Needs — clap-noun-verb v26.6.1 Analysis

**Date:** 2026-06-01  
**Version:** 26.6.1 (Minimalist Refactor)  
**Research Scope:** Phase 2 ledgers (public API only)

---

## Executive Summary

**Can ggen manufacture CLIs from graph law using clap-noun-verb v26.6.1?**

**Answer:** YES, but four critical APIs are missing.

clap-noun-verb v26.6.1 has strong **code generation foundations** (working binary, validated macros, deterministic tests). However, it lacks **structured metadata export** and **proof infrastructure** needed for graph law manufacturing and validation.

---

## What Works Today

### 1. Code Emission (✅ Fully Supported)

The `clap-noun-verb-gen` binary demonstrates that ggen can:
- Parse specifications (TTL, YAML)
- Generate valid Rust code using `#[verb]` and `#[arg]` macros
- Output compilable `main.rs`, `lib.rs`, and command modules
- Verify compilation with `cargo check`

**Public APIs ggen can use:**
- `#[verb]` proc-macro for command registration
- `#[arg]` attribute for parameter metadata
- `Result<T>` where `T: Serialize` return type
- `CliBuilder` and `CommandRegistry` for runtime construction

---

### 2. Testing (✅ Fully Supported)

ggen can generate tests because:
- `CliBuilder::build_command()` exposes the clap `Command` tree
- `CommandRegistry::run_with_args(Vec<String>)` allows deterministic testing
- `VerbArgs` provides access to parsed arguments at test time
- Existing test patterns (AAA with behavior assertions) are well-established

**Example ggen can use:**
```rust
// Generated test
#[test]
fn test_service_status() {
    let output = cli.run_with_args(vec![
        "myapp".to_string(),
        "services".to_string(),
        "status".to_string(),
    ]);
    assert!(output.is_ok());
}
```

---

### 3. Help Text / Documentation (✅ Fully Supported)

The `#[verb]` macro parses Rust docstrings:
- Extracts `///` comments for help text
- Parses `# Arguments` section for parameter docs
- Recognizes metadata tags (`[default: X]`, `[env: Y]`)
- Auto-generates clap help with all metadata

**ggen can:**
- Embed docstrings in generated `#[verb]` code
- Let clap handle `--help` rendering automatically

---

### 4. Compilation & Validation (✅ Fully Supported)

Generated code compiles using only public APIs:
- Compile-time validation of return types (must be `Serialize`)
- Macro validation of argument attributes
- Helper functions in `clap_noun_verb::validators` for input validation

**ggen can:**
- Generate code that compiles out-of-the-box
- Trust macro-time validation for type safety
- Use `cargo check/test` for verification

---

## What's Missing (The Four Gaps)

### Gap #1: Command Metadata Export (CRITICAL)

**Problem:** ggen cannot query a CLI's structure at runtime or introspect generated code without custom parsing.

**Current State:**
```rust
// This exists:
pub fn command_structure(&self) -> HashMap<String, Vec<String>> {
    // Only returns noun -> [verb names]
}
```

**Needed:**
```rust
pub struct CommandMetadata {
    pub app_name: String,
    pub app_about: String,
    pub version: Option<String>,
    pub nouns: Vec<NounMetadata>,
}

pub struct NounMetadata {
    pub name: String,
    pub about: String,
    pub verbs: Vec<VerbMetadata>,
    pub sub_nouns: Vec<Box<dyn NounMetadata>>,
}

pub struct VerbMetadata {
    pub name: String,
    pub about: String,
    pub args: Vec<ArgMetadata>,
}

pub struct ArgMetadata {
    pub name: String,
    pub typ: TypeAnnotation,  // String, i32, f64, etc.
    pub required: bool,
    pub about: String,
    pub default: Option<String>,
    pub env_var: Option<String>,
    pub short: Option<char>,
    pub long: Option<String>,
    pub is_flag: bool,
    pub values: Vec<String>,  // For enums
}

// In CliBuilder:
pub fn export_schema(&self) -> Result<serde_json::Value> {
    serde_json::to_value(&self.command_metadata())
}
```

**Impact:**
- **Without this:** ggen must parse `#[verb]` source code or introspect clap's internal Command structure
- **With this:** ggen gets structured data directly; enables documentation generation, test scaffolding, and graph traversal

**Classification:** `REQUIRES_PUBLIC_API_EXTENSION`

---

### Gap #2: Receipt/Proof-of-Execution Type (CRITICAL)

**Problem:** No canonical proof structure for graph law validation (Process Mining Chicago TDD).

**Current State:**
```rust
// telemetry.rs exists but doesn't define proofs
pub struct CliMetrics {
    pub schema_version: String,
    // ... timing info
}
```

**Needed:**
```rust
pub struct Receipt {
    pub id: String,                      // Unique ID (UUID or hash)
    pub timestamp: i64,                  // Unix ms
    pub command_path: Vec<String>,       // ["noun", "verb", ...]
    pub exit_code: i32,
    pub input_args: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub stderr: Option<String>,
}

pub trait VerbCommand {
    // Existing:
    fn run(&self, args: &VerbArgs) -> Result<()>;
    
    // New:
    fn emit_receipt(&self, args: &VerbArgs, output: &HandlerOutput) 
        -> Option<Receipt> {
        None  // Default: no receipt
    }
}

// Helper in error.rs:
pub fn generate_receipt_id() -> String {
    // UUID or content hash
}
```

**Impact:**
- **Without this:** ggen cannot generate proof events; graph law validators have no evidence trail
- **With this:** Every command execution can emit a provable event; enables conformance checking

**Classification:** `REQUIRES_PUBLIC_API_EXTENSION`

---

### Gap #3: Registry Serializability (IMPORTANT)

**Problem:** `CommandRegistry` cannot be serialized to JSON for export/documentation.

**Current State:**
```rust
pub struct CommandRegistry {
    nouns: HashMap<String, Box<dyn NounCommand>>,
    config: RegistryConfig,
    extensions: TypeMap,
    pub has_completions_subcommand: bool,
}
// No impl Serialize
```

**Needed:**
```rust
impl serde::Serialize for CommandRegistry {
    // Serialize config + command_metadata() result
}

impl CliBuilder {
    pub fn export_metadata(&self) -> Result<serde_json::Value> {
        serde_json::to_value(self.registry())
    }
}
```

**Impact:**
- **Without this:** ggen cannot export CLI structure as JSON for LLM tool calling or documentation
- **With this:** `--introspect` flag can return JSON Schema; ggen can generate docs automatically

**Classification:** `REQUIRES_PUBLIC_API_EXTENSION`

---

### Gap #4: --introspect Implementation (IMPORTANT)

**Problem:** Flag exists but is unimplemented; no routing to metadata export.

**Current State:**
```rust
// In registry.rs build_command():
cmd = cmd.arg(
    clap::Arg::new("introspect")
        .long("introspect")
        .action(clap::ArgAction::SetTrue)
        .global(true)
        .help("Introspect CLI capabilities as JSON Schema array for LLM tool-calling"),
);

// But: no handler in route() method
```

**Needed:**
```rust
pub fn route(&self, matches: &ArgMatches) -> Result<()> {
    // Check for introspect flag BEFORE normal routing
    if matches.get_flag("introspect") {
        let schema = self.export_metadata()?;
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(());
    }
    
    // ... normal routing continues
}
```

**Impact:**
- **Without this:** ggen cannot query CLI structure at runtime via CLI invocation
- **With this:** ggen can run: `mycli --introspect` to get JSON metadata

**Classification:** `DOC_ONLY` + `TEMPLATE_ONLY` (once metadata struct exists)

---

## Requirements Matrix

| Requirement | Q1 | Q2 | Q3 | Q4 | Q5 | Q6 | Q7 |
|---|---|---|---|---|---|---|---|
| **Data Struct for Surface** | ✅ | - | - | - | - | - | ❌ |
| **Emit Rust Code** | - | ✅ | - | - | - | - | ✅ |
| **Emit Tests** | - | - | ✅ | - | - | - | ✅ |
| **Emit Docs/Help** | - | - | - | ✅ | - | - | ✅ |
| **Metadata + Receipts** | - | - | - | - | ⚠️ | - | ❌ |
| **Compile w/ Stable APIs** | - | - | - | - | - | ✅ | ✅ |
| **API Complete** | ⚠️ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |

Legend:
- ✅ = ALREADY_SUPPORTED
- ⚠️ = PARTIALLY_SUPPORTED or TEMPLATE_ONLY
- ❌ = REQUIRES_PUBLIC_API_EXTENSION

---

## What ggen Must Do Today (Workarounds)

Since the four APIs don't exist yet, ggen can:

1. **For Metadata Export:**
   - Call `clap-noun-verb-gen` as a subprocess (if ggen is Rust)
   - Or parse `#[verb]` source code using syn + proc-macro-like logic
   - Or introspect clap's Command tree via reflection (fragile)

2. **For Receipts:**
   - Define its own `Receipt` struct in ggen
   - Emit receipts to a side-channel (file, event log)
   - Integrate manually via middleware

3. **For --introspect:**
   - Not available; ggen must call help subcommands instead
   - Parse `--help` output (brittle)

4. **For Serialization:**
   - Implement custom serialization using `command_structure()` as a base
   - Manually walk the noun/verb trees

---

## Recommended Implementation Path

### Phase 1: v26.7.0 (Introspection API)

```toml
# In Cargo.toml
[dependencies]
uuid = { version = "1.0", features = ["v4", "serde"] }
```

**Additions to `src/`:**

1. **New: `src/metadata.rs`**
```rust
pub struct CommandMetadata { ... }  // Full hierarchy
pub struct ArgMetadata { ... }      // Argument details
pub enum TypeAnnotation { String, Int, Float, Bool, Path, Json }
```

2. **In `src/registry.rs`:**
```rust
impl CommandRegistry {
    pub fn metadata(&self) -> CommandMetadata { ... }
    pub fn export_schema(&self) -> Result<serde_json::Value> { ... }
}
```

3. **In `src/verb.rs`:**
```rust
pub trait VerbCommand {
    fn emit_receipt(&self, args: &VerbArgs, output: &HandlerOutput) 
        -> Option<Receipt> { None }
}
```

4. **In `src/error.rs` or new `src/proof.rs`:**
```rust
pub struct Receipt { ... }
pub fn generate_receipt_id() -> String { ... }
```

5. **In `src/registry.rs` route() method:**
```rust
if matches.get_flag("introspect") {
    let schema = self.export_metadata()?;
    println!("{}", serde_json::to_string_pretty(&schema)?);
    return Ok(());
}
```

---

### Phase 2: v26.8.0 (Graph Law Integration)

- Telemetry integration with Receipt emission
- Event log export (for Process Mining Chicago TDD)
- SPARQL query support for proof conformance

---

## Summary for ggen Authors

**To manufacture CLIs from graph law with clap-noun-verb v26.6.1:**

| Component | Status | Workaround | Target Version |
|-----------|--------|-----------|-----------------|
| Code Generation | ✅ Ready | None needed | 26.6.1 |
| Testing | ✅ Ready | None needed | 26.6.1 |
| Documentation | ✅ Ready | None needed | 26.6.1 |
| Metadata Export | ❌ Missing | Parse source or subprocess | 26.7.0 |
| Receipt/Proofs | ❌ Missing | Implement custom Receipt | 26.7.0 |
| Graph Conformance | ❌ Missing | Manual validation | 26.8.0 |

**Critical Blocker:** Without metadata export and receipts, ggen cannot:
- Introspect a CLI at runtime
- Generate accurate test fixtures
- Emit proofs for graph law validators
- Support LLM tool calling

**Recommendation:** Target v26.7.0 for metadata + receipts. This unblocks all higher-level ggen features.

---

## References

- **clap-noun-verb:** v26.6.1 [repository](https://github.com/seanchatmangpt/clap-noun-verb)
- **Phase 2 Ledgers:** Public API surface analysis
- **ggen:** Graph law manufacturing engine (separate project)
- **Process Mining Chicago TDD:** Doctrine requiring proof infrastructure
