# Migration Guide: v4.0 → v5.0

## Overview

v5.0 introduces a **machine-centric capability system** while maintaining **full backward compatibility** with v4 human-friendly CLI features. This is a major release that adds a powerful autonomic layer for AI agents, MCP servers, and autonomous systems without breaking existing v4 functionality.

## TL;DR - Do I Need to Migrate?

**For CLI Users (Human Interaction):**
- ✅ **No changes required** - All v4 CLI features work exactly as before
- ✅ Just update your dependency version to `5.0.0`
- ✅ Your existing commands, arguments, and workflows are unchanged

**For Machine/Agent Integration:**
- 🆕 **New v5.0 features now available** for AI agents and autonomous systems
- 🔧 **Telemetry API changes** required if you use direct telemetry calls
- 🔍 Use new introspection API for capability discovery

## Quick Migration Steps

### Step 1: Update Dependencies

In your `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "5.0.0"          # Updated from 4.0.2
clap-noun-verb-macros = "5.0.0"   # Updated from 4.0.2
```

### Step 2: Update Your Code (If Using Telemetry)

**If you DON'T use telemetry directly** - Skip to Step 3.

**If you DO use telemetry** - Update to TelemetryManager:

**Before (v4):**
```rust
// Old direct telemetry access
use clap_noun_verb::telemetry::*;

let span = create_span("my_operation");
record_event("operation_complete");
```

**After (v5):**
```rust
// New TelemetryManager facade
use clap_noun_verb::telemetry::TelemetryManager;

let manager = TelemetryManager::instance();
let span = manager.create_span("my_operation", trace_id)?;
manager.record_event("operation_complete")?;
```

### Step 3: Run Tests

Verify your CLI works correctly:

```bash
cargo make test
```

### Step 4: Done!

Your v4 code now runs on v5 with access to new machine features.

---

## Breaking Changes in Detail

### 1. Telemetry API (TelemetryManager Facade)

**What Changed:**
Direct access to telemetry functions is replaced with a centralized `TelemetryManager` facade for better control and coordination.

**Who's Affected:**
Code that directly calls telemetry functions like `create_span()`, `record_event()`, or accesses telemetry state.

**Migration:**

| v4 (Old) | v5 (New) |
|----------|----------|
| `create_span("op")` | `TelemetryManager::instance().create_span("op", trace_id)?` |
| `record_event("evt")` | `TelemetryManager::instance().record_event("evt")?` |
| `get_telemetry_state()` | `TelemetryManager::instance().get_state()` |

**Example Migration:**

```rust
// v4 Code
use clap_noun_verb::telemetry::*;

fn process_command() -> Result<()> {
    let span = create_span("process");
    // ... do work ...
    record_event("processed");
    Ok(())
}
```

```rust
// v5 Code
use clap_noun_verb::telemetry::TelemetryManager;

fn process_command(trace_id: &str) -> Result<()> {
    let manager = TelemetryManager::instance();
    let span = manager.create_span("process", trace_id)?;
    // ... do work ...
    manager.record_event("processed")?;
    Ok(())
}
```

### 2. Span API (Requires trace_id)

**What Changed:**
All span creation now requires a `trace_id` parameter for distributed tracing support.

**Who's Affected:**
Code that creates spans for telemetry or tracing.

**Migration:**

```rust
// v4 - Span without trace_id
let span = create_span("my_operation");

// v5 - Span requires trace_id
use uuid::Uuid;
let trace_id = Uuid::new_v4().to_string();
let span = manager.create_span("my_operation", &trace_id)?;
```

**Why This Change:**
Enables distributed tracing across agent boundaries and multi-system coordination, essential for trillion-agent ecosystems.

### 3. Dispatcher (Automatic - No User Action)

**What Changed:**
A new routing layer automatically directs requests to either:
- **v4 path**: Human-friendly CLI features
- **v5 path**: Machine-centric introspection and autonomic operations

**Who's Affected:**
No one - this is handled internally by the framework.

**What You Get:**
- CLI commands work exactly as before (v4 path)
- Machine introspection calls use new v5 path automatically
- Zero overhead for CLI operations

---

## New v5.0 Features for Machine Integration

v5.0 adds powerful machine-centric capabilities for AI agents, MCP servers, and autonomous systems:

### 1. Introspection API

Query available capabilities at runtime:

```rust
use clap_noun_verb::v5::introspection::CapabilityRegistry;

let registry = CapabilityRegistry::new();
let capabilities = registry.list_capabilities()?;

for cap in capabilities {
    println!("Capability: {}", cap.name);
    println!("  Effects: {:?}", cap.effects);
    println!("  Inputs: {:?}", cap.inputs);
}
```

**Use Cases:**
- AI agents discovering available commands
- Dynamic workflow generation
- Capability-based access control

### 2. Formal Effects Declaration

Machine-readable side-effect specifications:

```rust
use clap_noun_verb::v5::effects::Effect;

#[verb("deploy")]
#[effects(Effect::FileWrite, Effect::NetworkIO)]
fn deploy_service(config: String) -> Result<DeployOutput> {
    // Effects are verified and tracked
    Ok(deploy(config))
}
```

**Use Cases:**
- Formal verification of operations
- Security auditing
- Effect-based authorization

### 3. Cryptographic Receipts

Proof of execution with blake3 hashing:

```rust
use clap_noun_verb::v5::receipts::Receipt;

let receipt = Receipt::for_operation("deploy", &inputs, &outputs)?;
println!("Receipt hash: {}", receipt.hash());
println!("Timestamp: {}", receipt.timestamp());
```

**Use Cases:**
- Audit trails
- Non-repudiation
- Compliance tracking

### 4. Delegation Chains

Agent-to-agent authorization:

```rust
use clap_noun_verb::v5::delegation::DelegationChain;

let chain = DelegationChain::new()
    .delegate_from("agent_a")
    .to("agent_b")
    .with_capability("deploy")
    .sign(private_key)?;

if chain.verify()? {
    // Execute delegated operation
}
```

**Use Cases:**
- Multi-agent workflows
- Hierarchical authorization
- Zero-trust architectures

### 5. RDF/Ontology Layer

Semantic capability management:

```rust
use clap_noun_verb::v5::rdf::CapabilityOntology;

let ontology = CapabilityOntology::load()?;
let related = ontology.find_related_capabilities("deploy")?;
```

**Use Cases:**
- Semantic search across capabilities
- Intelligent capability recommendation
- Knowledge graph integration

---

## Step-by-Step Migration Examples

### Example 1: Basic CLI Application (No Changes)

**v4 Code:**
```rust
// services.rs
use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

#[verb]
fn show_status() -> Result<Status> {
    Ok(Status {
        services: vec!["api".to_string()],
        healthy: true,
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

**v5 Migration:**
```rust
// No changes needed! Update Cargo.toml to 5.0.0 and you're done.
```

### Example 2: CLI with Telemetry

**v4 Code:**
```rust
use clap_noun_verb::telemetry::*;

#[verb("deploy")]
fn deploy_service(config: String) -> Result<DeployOutput> {
    let span = create_span("deploy_service");

    // Deploy logic
    let result = do_deploy(&config)?;

    record_event("deployment_complete");
    Ok(result)
}
```

**v5 Migration:**
```rust
use clap_noun_verb::telemetry::TelemetryManager;
use uuid::Uuid;

#[verb("deploy")]
fn deploy_service(config: String) -> Result<DeployOutput> {
    let trace_id = Uuid::new_v4().to_string();
    let manager = TelemetryManager::instance();
    let span = manager.create_span("deploy_service", &trace_id)?;

    // Deploy logic
    let result = do_deploy(&config)?;

    manager.record_event("deployment_complete")?;
    Ok(result)
}
```

### Example 3: Adding Machine Integration

**v4 Code:**
```rust
// Just a CLI application
```

**v5 Enhancement:**
```rust
use clap_noun_verb::v5::introspection::CapabilityRegistry;

// Your existing CLI code works unchanged

// NEW: Add machine introspection endpoint
#[verb("introspect")]
fn list_capabilities() -> Result<CapabilityList> {
    let registry = CapabilityRegistry::new();
    let capabilities = registry.list_capabilities()?;
    Ok(CapabilityList { capabilities })
}

// Now machines can discover your CLI's capabilities!
```

---

## Troubleshooting

### "Cannot find TelemetryManager"

**Problem:** Compilation error about missing TelemetryManager.

**Solution:**
```bash
cargo update -p clap-noun-verb
cargo clean
cargo build
```

### "trace_id parameter required"

**Problem:** Span creation requires trace_id.

**Solution:**
```rust
// Add trace_id generation
use uuid::Uuid;
let trace_id = Uuid::new_v4().to_string();
let span = manager.create_span("operation", &trace_id)?;
```

### "Old telemetry functions not found"

**Problem:** Direct telemetry calls no longer work.

**Solution:**
Replace with TelemetryManager facade:
```rust
// Old
create_span("op");

// New
TelemetryManager::instance().create_span("op", trace_id)?;
```

---

## Testing Your Migration

### Run Full Test Suite

```bash
cargo make test
```

### Test CLI Functionality

```bash
# Your existing commands should work unchanged
cargo run -- <noun> <verb> --args

# Example
cargo run -- services status
```

### Verify Telemetry (If Used)

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- <command>

# Check for telemetry warnings
```

---

## Getting Help

- **Documentation**: See `docs/V5_MACHINE_API.md` for v5 machine features
- **Examples**: Check `examples/v5_*` for working code
- **Issues**: Report problems at [GitHub Issues](https://github.com/ruvnet/clap-noun-verb/issues)
- **Discussions**: Ask questions at [GitHub Discussions](https://github.com/ruvnet/clap-noun-verb/discussions)

---

## Summary Checklist

- [ ] Update `Cargo.toml` to version `5.0.0`
- [ ] If using telemetry: Migrate to `TelemetryManager` facade
- [ ] If creating spans: Add `trace_id` parameter
- [ ] Run `cargo make test` to verify
- [ ] Review new v5 features for machine integration opportunities
- [ ] Update documentation to mention v5 compatibility

---

**Congratulations!** You've successfully migrated to clap-noun-verb v5.0. Your CLI works exactly as before, with powerful new machine-centric capabilities available when you need them.
