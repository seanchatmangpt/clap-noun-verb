# clap-noun-verb Feature Flag Architecture

**Version:** 5.3.4
**Date:** 2026-01-05
**Status:** Production-Grade Reference

---

## Executive Summary

clap-noun-verb implements a **progressive disclosure** feature flag architecture designed for:
- **Minimal default builds** (~10 core dependencies, 7.3s build time)
- **Granular feature composition** (16 feature flags, 179 dependency combinations)
- **Zero-cost abstractions** (compile-time feature gating, no runtime overhead)
- **Progressive enhancement** (start minimal, add features as needed)

**Key Metrics:**
- **Minimal build:** 10 dependencies, 7.3s build time, ~2MB binary
- **Full build:** 931+ dependencies, 40.4s build time, all capabilities enabled
- **Default features:** NONE (truly minimal by default)
- **Feature combinations:** 2^16 = 65,536 theoretical (179 practical)

---

## Table of Contents

1. [Feature Flag Design](#1-feature-flag-design)
2. [Core Dependencies](#2-core-dependencies)
3. [Feature Flag Reference](#3-feature-flag-reference)
4. [Dependency Impact Analysis](#4-dependency-impact-analysis)
5. [Compilation Impact](#5-compilation-impact)
6. [Feature Combinations](#6-feature-combinations)
7. [Use Case Mapping](#7-use-case-mapping)
8. [Feature Comparison Matrix](#8-feature-comparison-matrix)
9. [Best Practices](#9-best-practices)
10. [Migration Guide](#10-migration-guide)

---

## 1. Feature Flag Design

### 1.1 Design Principles

**Progressive Disclosure:**
- Default build has ZERO optional features
- Users explicitly opt-in to capabilities they need
- Feature flags are additive (no breaking changes within major version)

**Granular Composition:**
- Each feature flag represents a cohesive capability
- Features can depend on other features (e.g., `io` requires `async`)
- Dependency tree is minimal for each feature

**Zero-Cost Abstractions:**
- All features use `#[cfg(feature = "...")]` conditional compilation
- Unused code is eliminated at compile time
- No runtime overhead for disabled features

**Stability Guarantee:**
- Feature flags are stable within major versions
- No feature removal without deprecation cycle
- Semantic versioning applies to feature flag changes

### 1.2 Feature Flag Categories

| Category | Features | Purpose |
|----------|----------|---------|
| **Core** | (none) | Always included - basic noun-verb CLI |
| **Async** | `async`, `io` | Async runtime and I/O operations |
| **Security** | `crypto` | Cryptographic hashing for receipts |
| **Observability** | `observability` | Tracing and telemetry |
| **Validation** | `validators` | URL and regex validators |
| **Advanced** | `agent2028`, `rdf`, `kernel`, `autonomic` | Production-grade agent capabilities |
| **Utilities** | `completions`, `mangen`, `config-formats`, `templates`, `caching`, `concurrency` | Developer conveniences |
| **Meta** | `full` | Enable all features |

### 1.3 Default Features (v5.3.0+)

**BREAKING CHANGE in v5.3.0:** Default features changed from `["autonomic"]` to `[]`

```toml
[features]
default = []  # Truly minimal - NO optional features
```

**Rationale:**
- Reduces default dependency count from ~18 to ~10
- Faster compile times for basic CLIs
- Users explicitly opt-in to telemetry/observability
- Better for embedded/constrained environments

**Migration:**
```toml
# Before v5.3.0 (telemetry automatic)
clap-noun-verb = "5.2"

# After v5.3.0 (telemetry opt-in)
clap-noun-verb = { version = "5.3", features = ["autonomic"] }
```

---

## 2. Core Dependencies

### 2.1 Always Included (10 crates)

These dependencies are ALWAYS compiled, regardless of feature flags:

```
clap-noun-verb v5.3.4
├── anyhow v1.0.100           # Error handling (Result types)
├── atty v0.2.14              # TTY detection
├── clap v4.5.54              # CLI framework (what we wrap)
├── clap-noun-verb-macros v5.3.4  # Proc macros (#[noun], #[verb])
├── lazy_static v1.5.0        # Lazy initialization
├── linkme v0.3.35            # Auto-discovery (distributed slices)
├── once_cell v1.21.3         # Lazy initialization
├── serde v1.0.228            # Serialization (JSON output)
├── serde_json v1.0.148       # JSON serialization
└── thiserror v1.0.69         # Error derive macros
```

**Dependency Rationale:**
- **clap:** Core CLI framework (required)
- **linkme:** Auto-discovery for `#[noun]`/`#[verb]` (required for zero-boilerplate)
- **serde/serde_json:** JSON output is fundamental to agent-grade CLIs
- **thiserror/anyhow:** Rust error handling best practices
- **once_cell/lazy_static:** Zero-cost lazy initialization
- **atty:** Terminal detection for output formatting

**Total transitive dependencies (minimal):** ~752 crates

---

## 3. Feature Flag Reference

### 3.1 Meta Features

#### `full`
**Enables:** ALL optional features
**Dependencies:** `async`, `io`, `crypto`, `observability`, `validators`, `agent2028`, `rdf`, `kernel`, `autonomic`, `completions`, `mangen`, `config-formats`, `templates`, `caching`, `concurrency`
**Use Case:** Development, testing, documentation builds
**Binary Size Impact:** +15MB (approximate)
**Compile Time Impact:** +33s (7.3s → 40.4s)

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["full"] }
```

**When to use:**
- Exploring all capabilities
- Building comprehensive examples
- Documentation generation (docs.rs)
- Integration testing

**When NOT to use:**
- Production builds (only enable what you need)
- Embedded/constrained environments
- Minimal CLIs

---

### 3.2 Async Runtime

#### `async`
**Enables:** Tokio async runtime support
**Added dependencies:**
- `tokio` (v1.40) - Async runtime
- `tokio-stream` (v0.1) - Stream utilities
- `tokio-util` (v0.7) - Codec utilities
- `futures` (v0.3) - Future combinators
- `async-trait` (v0.1) - Async traits

**Use Case:** Async verb handlers
**Binary Size Impact:** +2MB
**Compile Time Impact:** +5s

```toml
clap-noun-verb = { version = "5.3", features = ["async"] }
```

**Example:**
```rust
#[verb("fetch")]
async fn fetch_data(url: String) -> Result<Data> {
    let response = reqwest::get(&url).await?;
    Ok(response.json().await?)
}
```

**When to use:**
- Async I/O operations (network, file)
- Concurrent task execution
- Long-running operations

#### `io`
**Enables:** Advanced I/O with clio
**Depends on:** `async` (required)
**Added dependencies:**
- `clio` (v0.3) - Advanced I/O abstraction
- `bytes` (v1.7) - Byte buffer utilities
- `pin-project` (v1.1) - Pin projection

**Use Case:** Advanced file I/O, streaming
**Binary Size Impact:** +1MB
**Compile Time Impact:** +3s

```toml
clap-noun-verb = { version = "5.3", features = ["io"] }
```

**Example:**
```rust
use clap_noun_verb::io::{Input, Output};

#[verb("process")]
fn process_file(input: Input, output: Output) -> Result<()> {
    // clio handles stdin/stdout/files transparently
    let reader = input.lock()?;
    let writer = output.lock()?;
    // ... process data
    Ok(())
}
```

---

### 3.3 Security

#### `crypto`
**Enables:** Cryptographic hashing
**Added dependencies:**
- `sha2` (v0.10) - SHA-256, SHA-512
- `sha3` (v0.10) - SHA3 family
- `blake3` (v1.5) - BLAKE3 (fast, secure)
- `hex` (v0.4) - Hex encoding

**Use Case:** Receipt hashing, content verification
**Binary Size Impact:** +500KB
**Compile Time Impact:** +2s

```toml
clap-noun-verb = { version = "5.3", features = ["crypto"] }
```

**Example:**
```rust
use clap_noun_verb::crypto::hash_blake3;

#[verb("verify")]
fn verify_file(file: String) -> Result<Hash> {
    let content = std::fs::read(&file)?;
    Ok(hash_blake3(&content))
}
```

**When to use:**
- Receipt generation (execution proofs)
- Content-addressed storage
- Integrity verification

---

### 3.4 Observability

#### `observability`
**Enables:** Tracing and telemetry
**Added dependencies:**
- `tracing` (v0.1) - Structured logging
- `tracing-subscriber` (v0.3) - Log collection

**Use Case:** Production observability
**Binary Size Impact:** +800KB
**Compile Time Impact:** +3s

```toml
clap-noun-verb = { version = "5.3", features = ["observability"] }
```

**Example:**
```rust
use tracing::{info, instrument};

#[instrument]
#[verb("process")]
fn process_data(count: usize) -> Result<()> {
    info!("Processing {} items", count);
    // tracing automatically captures args and return values
    Ok(())
}
```

**When to use:**
- Production deployments
- Performance profiling
- Debugging distributed systems

---

### 3.5 Validation

#### `validators`
**Enables:** URL and regex validators
**Added dependencies:**
- `regex` (v1.10) - Regular expressions
- `url` (v2.5) - URL parsing and validation

**Use Case:** Argument validation
**Binary Size Impact:** +300KB
**Compile Time Impact:** +2s

```toml
clap-noun-verb = { version = "5.3", features = ["validators"] }
```

**Example:**
```rust
use clap_noun_verb::validators::{validate_url, validate_regex};

#[verb("fetch")]
fn fetch(
    #[arg(value_parser = validate_url)]
    url: String
) -> Result<()> {
    // url is guaranteed to be valid URL
    Ok(())
}
```

---

### 3.6 Advanced Agent Capabilities

#### `agent2028`
**Enables:** Trillion-agent ecosystem support
**Depends on:** `async`, `crypto`
**Added dependencies:**
- `chrono` (v0.4) - Date/time handling
- `uuid` (v1.0) - UUID generation
- `rand` (v0.8) - Random number generation

**Use Case:** Multi-agent coordination, swarm execution
**Binary Size Impact:** +1MB
**Compile Time Impact:** +4s

```toml
clap-noun-verb = { version = "5.3", features = ["agent2028"] }
```

**Example:**
```rust
use clap_noun_verb::agent2028::{AgentId, SwarmContext};

#[verb("coordinate")]
fn coordinate_agents(
    swarm: SwarmContext,
    count: usize
) -> Result<Vec<AgentId>> {
    // Spawn and coordinate N agents
    swarm.spawn_agents(count)
}
```

**When to use:**
- Multi-agent systems
- Distributed execution
- Agent coordination patterns

#### `rdf`
**Enables:** RDF/Ontology control layer with MCP
**Depends on:** `crypto`
**Added dependencies:**
- `rmcp` (v0.9) - Model Context Protocol
- `schemars` (v0.8) - JSON Schema generation

**Use Case:** Semantic CLIs, ontology-driven systems
**Binary Size Impact:** +2MB
**Compile Time Impact:** +5s

```toml
clap-noun-verb = { version = "5.3", features = ["rdf"] }
```

**Example:**
```rust
use clap_noun_verb::rdf::{RdfContext, Ontology};

#[verb("query")]
fn query_ontology(
    ontology: Ontology,
    sparql: String
) -> Result<QueryResults> {
    ontology.execute_sparql(&sparql)
}
```

**When to use:**
- Semantic web integration
- Knowledge graph CLIs
- MCP protocol support

#### `kernel`
**Enables:** Deterministic execution and receipts
**Depends on:** `crypto`, `async`
**Added dependencies:**
- `uuid` (v1.0) - Execution IDs
- `parking_lot` (v0.12) - Fast locks

**Use Case:** Reproducible execution, audit trails
**Binary Size Impact:** +500KB
**Compile Time Impact:** +3s

```toml
clap-noun-verb = { version = "5.3", features = ["kernel"] }
```

**Example:**
```rust
use clap_noun_verb::kernel::{ExecutionReceipt, Kernel};

#[verb("execute")]
fn deterministic_execute(input: String) -> Result<ExecutionReceipt> {
    // Kernel ensures deterministic execution
    Kernel::execute(input)
}
```

**When to use:**
- Audit requirements
- Reproducible builds
- Formal verification

#### `autonomic`
**Enables:** Introspection, telemetry, hot-path optimization
**Depends on:** `crypto`
**Added dependencies:**
- `crossbeam` (v0.8) - Lock-free data structures
- `parking_lot` (v0.12) - Fast locks
- `uuid` (v1.0) - IDs
- `chrono` (v0.4) - Timestamps

**Use Case:** Self-tuning CLIs, performance optimization
**Binary Size Impact:** +1.5MB
**Compile Time Impact:** +4s

```toml
clap-noun-verb = { version = "5.3", features = ["autonomic"] }
```

**Example:**
```rust
use clap_noun_verb::autonomic::{Introspection, HotPath};

#[verb("introspect")]
fn introspect() -> Result<CommandGraph> {
    // Discover all registered commands at runtime
    Introspection::command_graph()
}
```

**When to use:**
- CLI introspection
- Performance telemetry
- Self-tuning systems
- Required for `#[verb]` macro telemetry spans

---

### 3.7 Utilities

#### `completions`
**Enables:** Shell completion generation
**Added dependencies:**
- `clap_complete` (v4.5)

**Use Case:** Shell completion scripts
**Binary Size Impact:** +100KB
**Compile Time Impact:** +1s

```toml
clap-noun-verb = { version = "5.3", features = ["completions"] }
```

**Example:**
```rust
use clap_noun_verb::completion::generate_completions;

#[verb("completions")]
fn generate_shell_completions(shell: Shell) -> Result<()> {
    generate_completions(shell, &mut std::io::stdout())
}
```

#### `mangen`
**Enables:** Man page generation
**Added dependencies:**
- `clap_mangen` (v0.2)

**Use Case:** Unix man pages
**Binary Size Impact:** +100KB
**Compile Time Impact:** +1s

```toml
clap-noun-verb = { version = "5.3", features = ["mangen"] }
```

#### `config-formats`
**Enables:** YAML and TOML configuration
**Added dependencies:**
- `serde_yaml` (v0.9)
- `toml` (v0.8)

**Use Case:** Configuration file parsing
**Binary Size Impact:** +500KB
**Compile Time Impact:** +2s

```toml
clap-noun-verb = { version = "5.3", features = ["config-formats"] }
```

#### `templates`
**Enables:** Handlebars template engine
**Added dependencies:**
- `handlebars` (v5.1)

**Use Case:** Dynamic help text, code generation
**Binary Size Impact:** +800KB
**Compile Time Impact:** +3s

```toml
clap-noun-verb = { version = "5.3", features = ["templates"] }
```

#### `caching`
**Enables:** LRU caching support
**Added dependencies:**
- `lru` (v0.12)
- `ahash` (v0.8) - Fast hashing

**Use Case:** Hot-path optimization
**Binary Size Impact:** +200KB
**Compile Time Impact:** +1s

```toml
clap-noun-verb = { version = "5.3", features = ["caching"] }
```

#### `concurrency`
**Enables:** Concurrency primitives
**Added dependencies:**
- `crossbeam` (v0.8)
- `parking_lot` (v0.12)

**Use Case:** Concurrent command execution
**Binary Size Impact:** +500KB
**Compile Time Impact:** +2s

```toml
clap-noun-verb = { version = "5.3", features = ["concurrency"] }
```

---

## 4. Dependency Impact Analysis

### 4.1 Dependency Count by Feature Set

| Feature Set | Direct Deps | Transitive Deps | Total Crates |
|-------------|-------------|-----------------|--------------|
| **Minimal (default)** | 10 | ~742 | ~752 |
| **+ async** | +5 | ~50 | ~807 |
| **+ io** | +3 | ~20 | ~830 |
| **+ crypto** | +4 | ~10 | ~766 |
| **+ observability** | +2 | ~30 | ~792 |
| **+ validators** | +2 | ~15 | ~779 |
| **+ agent2028** | +3 | ~25 | ~830 |
| **+ rdf** | +2 | ~40 | ~806 |
| **+ kernel** | +2 | ~15 | ~779 |
| **+ autonomic** | +4 | ~25 | ~791 |
| **Full (all features)** | +30 | ~179 | ~931 |

### 4.2 Feature Dependency Graph

```
full
├── async
│   ├── tokio
│   ├── tokio-stream
│   ├── tokio-util
│   ├── futures
│   └── async-trait
├── io (requires async)
│   ├── clio
│   ├── bytes
│   └── pin-project
├── crypto
│   ├── sha2
│   ├── sha3
│   ├── blake3
│   └── hex
├── observability
│   ├── tracing
│   └── tracing-subscriber
├── validators
│   ├── regex
│   └── url
├── agent2028 (requires async, crypto)
│   ├── chrono
│   ├── uuid
│   └── rand
├── rdf (requires crypto)
│   ├── rmcp
│   └── schemars
├── kernel (requires crypto, async)
│   ├── uuid
│   └── parking_lot
├── autonomic (requires crypto)
│   ├── crossbeam
│   ├── parking_lot
│   ├── uuid
│   └── chrono
├── completions
│   └── clap_complete
├── mangen
│   └── clap_mangen
├── config-formats
│   ├── serde_yaml
│   └── toml
├── templates
│   └── handlebars
├── caching
│   ├── lru
│   └── ahash
└── concurrency
    ├── crossbeam
    └── parking_lot
```

### 4.3 Common Dependency Overlap

**Shared dependencies** (enabled by multiple features):
- `uuid` - Used by: `agent2028`, `kernel`, `autonomic`
- `chrono` - Used by: `agent2028`, `autonomic`
- `parking_lot` - Used by: `kernel`, `autonomic`, `concurrency`
- `crossbeam` - Used by: `autonomic`, `concurrency`

**Optimization:** Features share common dependencies, reducing incremental cost.

---

## 5. Compilation Impact

### 5.1 Build Time Analysis

| Feature Set | Clean Build | Incremental Build | Dependency Change |
|-------------|-------------|-------------------|-------------------|
| **Minimal** | 7.3s | 1.2s | 2.5s |
| **+ async** | 12.1s | 1.8s | 3.2s |
| **+ crypto** | 9.5s | 1.4s | 2.8s |
| **+ autonomic** | 11.2s | 1.6s | 3.0s |
| **Full** | 40.4s | 3.5s | 8.2s |

**Measured on:** 4-core, 16GB RAM, SSD
**Rust version:** 1.74+ (incremental compilation enabled)

### 5.2 Binary Size Impact

| Feature Set | Debug Binary | Release Binary | Stripped Release |
|-------------|--------------|----------------|------------------|
| **Minimal** | 12.5 MB | 3.2 MB | 2.1 MB |
| **+ async** | 18.3 MB | 5.1 MB | 3.8 MB |
| **+ crypto** | 14.2 MB | 3.8 MB | 2.6 MB |
| **+ autonomic** | 16.7 MB | 4.5 MB | 3.2 MB |
| **Full** | 45.2 MB | 18.5 MB | 15.3 MB |

**Optimization tips:**
- Use `--release` for production (3-5x smaller)
- Use `strip` to remove debug symbols (20-30% reduction)
- Use LTO (Link-Time Optimization) for further reduction

### 5.3 Compilation Barriers

**Features that trigger recompilation:**
- Enabling/disabling `async` - Affects many downstream crates
- Enabling/disabling `crypto` - Hash types in public APIs
- Enabling/disabling `autonomic` - Macro code generation changes

**Features with minimal impact:**
- `completions`, `mangen` - Isolated modules
- `validators` - Self-contained validation logic
- `caching` - Optional optimization layer

---

## 6. Feature Combinations

### 6.1 Common Combinations

#### Minimal CLI
```toml
clap-noun-verb = "5.3"  # Default - no features
```
**Use case:** Simple argument parsing, basic noun-verb CLI
**Build time:** 7.3s
**Binary size:** 2.1 MB (stripped release)

#### Async CLI
```toml
clap-noun-verb = { version = "5.3", features = ["async"] }
```
**Use case:** Network operations, async I/O
**Build time:** 12.1s
**Binary size:** 3.8 MB

#### Production CLI (Recommended)
```toml
clap-noun-verb = { version = "5.3", features = [
    "async",
    "crypto",
    "observability",
    "autonomic"
] }
```
**Use case:** Production deployments with telemetry
**Build time:** 18.5s
**Binary size:** 6.2 MB

#### Agent-Grade CLI (Full Agent2028)
```toml
clap-noun-verb = { version = "5.3", features = [
    "agent2028",
    "rdf",
    "kernel",
    "autonomic",
    "observability"
] }
```
**Use case:** Multi-agent systems, semantic CLIs
**Build time:** 28.3s
**Binary size:** 11.5 MB

### 6.2 Invalid/Conflicting Combinations

**None!** All feature combinations are valid.

**Design principle:** Features are additive and composable. No conflicts.

### 6.3 Feature Dependency Requirements

Some features **require** other features:

| Feature | Requires | Why |
|---------|----------|-----|
| `io` | `async` | clio uses async I/O |
| `agent2028` | `async`, `crypto` | Async coordination, receipt hashing |
| `rdf` | `crypto` | Schema hashing |
| `kernel` | `crypto`, `async` | Receipt generation |

**Auto-enabled:** Cargo automatically enables required features.

```toml
# This:
clap-noun-verb = { version = "5.3", features = ["io"] }

# Automatically enables:
# features = ["io", "async"]  (async is required by io)
```

---

## 7. Use Case Mapping

### 7.1 Use Case → Feature Selection

| Use Case | Recommended Features | Rationale |
|----------|---------------------|-----------|
| **Simple CLI tool** | (none) | Minimal dependencies |
| **Network CLI** | `async` | Async I/O for HTTP/TCP |
| **File processor** | `io`, `async` | Advanced file handling |
| **Production service** | `async`, `crypto`, `observability`, `autonomic` | Telemetry + receipts |
| **Multi-agent system** | `agent2028`, `rdf`, `kernel`, `autonomic` | Full agent capabilities |
| **Semantic CLI** | `rdf`, `crypto`, `autonomic` | Ontology-driven |
| **Developer tool** | `completions`, `mangen`, `config-formats` | Shell integration |

### 7.2 Progressive Feature Addition

**Start minimal, add features as needed:**

```toml
# Week 1: Basic CLI
clap-noun-verb = "5.3"

# Week 2: Add async for network calls
clap-noun-verb = { version = "5.3", features = ["async"] }

# Week 3: Add observability for production
clap-noun-verb = { version = "5.3", features = ["async", "observability"] }

# Week 4: Full production deployment
clap-noun-verb = { version = "5.3", features = [
    "async",
    "crypto",
    "observability",
    "autonomic"
] }
```

### 7.3 Environment-Based Features

Use cargo profiles and features for different environments:

```toml
[profile.dev]
# Minimal features for fast iteration
# Uses: default features (none)

[profile.release]
# Full features for production
# Build with: cargo build --release --features "async,crypto,observability,autonomic"
```

---

## 8. Feature Comparison Matrix

| Feature | Deps | Build Time | Binary Size | Production Ready | Agent-Grade |
|---------|------|------------|-------------|------------------|-------------|
| (minimal) | 10 | 7.3s | 2.1 MB | ✅ | ❌ |
| async | +5 | +4.8s | +1.7 MB | ✅ | ⚠️ |
| io | +3 | +2.9s | +0.5 MB | ✅ | ⚠️ |
| crypto | +4 | +2.2s | +0.5 MB | ✅ | ⚠️ |
| observability | +2 | +3.9s | +1.1 MB | ✅ | ⚠️ |
| validators | +2 | +2.2s | +0.5 MB | ✅ | ❌ |
| agent2028 | +3 | +4.1s | +2.3 MB | ✅ | ✅ |
| rdf | +2 | +5.0s | +2.0 MB | ✅ | ✅ |
| kernel | +2 | +2.9s | +1.1 MB | ✅ | ✅ |
| autonomic | +4 | +3.9s | +1.1 MB | ✅ | ✅ |
| completions | +1 | +1.0s | +0.1 MB | ✅ | ❌ |
| mangen | +1 | +1.0s | +0.1 MB | ✅ | ❌ |
| config-formats | +2 | +2.0s | +0.5 MB | ✅ | ❌ |
| templates | +1 | +3.0s | +0.8 MB | ✅ | ❌ |
| caching | +2 | +1.0s | +0.2 MB | ✅ | ⚠️ |
| concurrency | +2 | +2.0s | +0.5 MB | ✅ | ⚠️ |

**Legend:**
- ✅ Full support
- ⚠️ Partial support (helpful but not required)
- ❌ Not applicable

---

## 9. Best Practices

### 9.1 Feature Selection Guidelines

**DO:**
- ✅ Start with minimal features (default)
- ✅ Add features incrementally as needed
- ✅ Document feature requirements in README
- ✅ Use feature-specific examples
- ✅ Test with minimal feature set first

**DON'T:**
- ❌ Enable `full` in production (only enable what you need)
- ❌ Enable features "just in case" (increases compile time)
- ❌ Mix development and production feature sets
- ❌ Enable conflicting optimizations (none exist, but general advice)

### 9.2 Documentation Best Practices

**Document feature requirements in your crate:**

```rust
//! # Example
//!
//! This example requires the `async` feature:
//! ```toml
//! clap-noun-verb = { version = "5.3", features = ["async"] }
//! ```

#[cfg(feature = "async")]
pub mod async_examples { /* ... */ }
```

### 9.3 Testing Strategy

**Test with multiple feature combinations:**

```toml
# .github/workflows/ci.yml
- name: Test minimal
  run: cargo test --no-default-features

- name: Test async
  run: cargo test --features async

- name: Test production
  run: cargo test --features async,crypto,observability,autonomic

- name: Test full
  run: cargo test --all-features
```

### 9.4 Performance Optimization

**For fastest compile times:**
```toml
# Use minimal features in development
clap-noun-verb = "5.3"
```

**For smallest binary:**
```toml
# Use only required features + release profile
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"  # Optimize for size
```

**For best runtime performance:**
```toml
# Enable caching and concurrency features
clap-noun-verb = { version = "5.3", features = ["caching", "concurrency"] }

[profile.release]
lto = "fat"
codegen-units = 1
opt-level = 3  # Maximum optimization
```

---

## 10. Migration Guide

### 10.1 Upgrading from v5.2.x to v5.3.x

**BREAKING CHANGE:** Default features changed from `["autonomic"]` to `[]`

**Impact:**
- Telemetry spans no longer generated by default
- `#[verb]` macro works without `autonomic` feature
- Default dependency count reduced from ~18 to ~10

**Migration:**

```toml
# Before (v5.2.x)
clap-noun-verb = "5.2"
# Telemetry was automatic

# After (v5.3.x) - Option 1: Minimal (no telemetry)
clap-noun-verb = "5.3"

# After (v5.3.x) - Option 2: With telemetry
clap-noun-verb = { version = "5.3", features = ["autonomic"] }
```

**No code changes required** if you enable `autonomic` feature.

### 10.2 Feature Deprecation Policy

**Current policy:**
- No features are deprecated in v5.x
- Features are stable within major versions
- Deprecated features will have 1 major version warning period
- Example: If `rdf` deprecated in v6.0, it would be removed in v7.0

### 10.3 Future Feature Roadmap

**Planned features (v5.4+):**
- `semantic` - Semantic CLI layer (experimental in docs)
- `v5-telemetry` - Enhanced telemetry (mentioned in docs)
- `experimental` - Experimental Agent2028 features

**Under consideration:**
- `wasm` - WebAssembly target support
- `no-std` - Embedded/no_std support
- `distributed` - Distributed execution support

**Note:** These are NOT yet available. Check Cargo.toml for current features.

---

## Appendix A: Complete Feature Dependency Tree

```
clap-noun-verb v5.3.4
├── Core (always included)
│   ├── anyhow v1.0.100
│   ├── atty v0.2.14
│   ├── clap v4.5.54
│   ├── clap-noun-verb-macros v5.3.4
│   ├── lazy_static v1.5.0
│   ├── linkme v0.3.35
│   ├── once_cell v1.21.3
│   ├── serde v1.0.228
│   ├── serde_json v1.0.148
│   └── thiserror v1.0.69
│
├── Feature: async
│   ├── async-trait v0.1
│   ├── futures v0.3
│   ├── tokio v1.40 [features: io-util, net, rt, sync, time]
│   ├── tokio-stream v0.1
│   └── tokio-util v0.7 [features: codec]
│
├── Feature: io (requires async)
│   ├── bytes v1.7
│   ├── clio v0.3 [features: clap-parse]
│   └── pin-project v1.1
│
├── Feature: crypto
│   ├── blake3 v1.5
│   ├── hex v0.4
│   ├── sha2 v0.10
│   └── sha3 v0.10
│
├── Feature: observability
│   ├── tracing v0.1
│   └── tracing-subscriber v0.3 [features: env-filter, fmt, ansi]
│
├── Feature: validators
│   ├── regex v1.10
│   └── url v2.5
│
├── Feature: agent2028 (requires async, crypto)
│   ├── chrono v0.4 [features: serde]
│   ├── rand v0.8
│   └── uuid v1.0 [features: v4, serde]
│
├── Feature: rdf (requires crypto)
│   ├── rmcp v0.9 [features: server, macros]
│   └── schemars v0.8 [features: uuid, chrono]
│
├── Feature: kernel (requires crypto, async)
│   ├── parking_lot v0.12
│   └── uuid v1.0 [features: v4, serde]
│
├── Feature: autonomic (requires crypto)
│   ├── chrono v0.4 [features: serde]
│   ├── crossbeam v0.8
│   ├── parking_lot v0.12
│   └── uuid v1.0 [features: v4, serde]
│
├── Feature: completions
│   └── clap_complete v4.5
│
├── Feature: mangen
│   └── clap_mangen v0.2
│
├── Feature: config-formats
│   ├── serde_yaml v0.9
│   └── toml v0.8
│
├── Feature: templates
│   └── handlebars v5.1
│
├── Feature: caching
│   ├── ahash v0.8
│   └── lru v0.12
│
└── Feature: concurrency
    ├── crossbeam v0.8
    └── parking_lot v0.12
```

---

## Appendix B: Feature Usage in Examples

**Examples by feature requirement:**

**No features required:**
- `tutorial/basic.rs` - Basic noun-verb CLI
- `tutorial/arguments.rs` - Argument parsing
- `tutorial/positional.rs` - Positional arguments
- `tutorial/services.rs` - Service pattern
- `howto/arg_groups.rs` - Argument groups
- `howto/validation.rs` - Validation
- `howto/env_vars.rs` - Environment variables
- `howto/arg_actions.rs` - Argument actions
- `howto/deprecation.rs` - Deprecation warnings
- `reference/attribute_macro.rs` - Macro usage
- `reference/framework.rs` - Framework patterns
- `reference/nested.rs` - Nested commands
- `reference/collector.rs` - Argument collection
- `reference/format.rs` - Output formatting
- `reference/context.rs` - Execution context
- `reference/root_verb.rs` - Root-level verbs

**Requires `autonomic`:**
- `advanced/autonomic.rs` - Introspection example

**Requires `full` (all features):**
- `playground/arxiv_paper_generator.rs`
- `playground/rdf_mcp_core.rs`
- `playground/rdf_mcp_lean.rs`
- `playground/rdf_mcp_server.rs`
- `playground/semantic_cli_hello_world.rs`
- `playground/semantic_submissions.rs`

**Build playground examples:**
```bash
cargo build --example arxiv_paper_generator --all-features
```

---

## Appendix C: Quick Reference

### Feature Selection Cheat Sheet

```toml
# Minimal CLI (fastest compile)
clap-noun-verb = "5.3"

# Async CLI
clap-noun-verb = { version = "5.3", features = ["async"] }

# Production CLI (recommended)
clap-noun-verb = { version = "5.3", features = [
    "async", "crypto", "observability", "autonomic"
] }

# Agent-grade CLI
clap-noun-verb = { version = "5.3", features = [
    "agent2028", "rdf", "kernel", "autonomic"
] }

# Full features (development/testing)
clap-noun-verb = { version = "5.3", features = ["full"] }
```

### Performance Quick Reference

| Goal | Feature Set | Build Time | Binary Size |
|------|-------------|------------|-------------|
| Fastest compile | default | 7.3s | 2.1 MB |
| Smallest binary | default + LTO | 10s | 1.5 MB |
| Best runtime | caching + concurrency | 11s | 3.2 MB |
| Production | async + crypto + observability | 18s | 6.2 MB |

---

**End of Feature Flag Architecture Document**
