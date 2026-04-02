# clap-noun-verb Advanced Features Quick Reference

**Generated:** 2026-01-05
**Source:** [Complete Guide](ADVANCED_FEATURES_GUIDE.md)

---

## Table of Contents (Quick Navigation)

1. [Feature Overview Matrix](#feature-overview-matrix)
2. [Decision Tree](#decision-tree)
3. [Progressive Mastery Path](#progressive-mastery-path)
4. [Feature Combinations](#feature-combinations)
5. [Performance Impact](#performance-impact)
6. [Troubleshooting Checklist](#troubleshooting-checklist)

---

## Feature Overview Matrix

| Feature | Complexity | Performance Impact | Dependencies | Use When |
|---------|-----------|-------------------|--------------|----------|
| **Procedural Macros** | ⭐ Beginner | Zero-cost | Default | Always |
| **Type Inference** | ⭐ Beginner | Zero-cost | Default | Always |
| **Auto-Discovery** | ⭐ Beginner | Zero-cost | Default | Modular CLIs |
| **Async Support** | ⭐⭐ Intermediate | <5% | +5 deps | I/O operations |
| **Autonomic Layer** | ⭐⭐ Intermediate | <5% | +4 deps | AI agents |
| **Feature Flags** | ⭐⭐ Intermediate | Zero-cost | Default | Minimal builds |
| **Validators** | ⭐⭐ Intermediate | <1% | +2 deps | Input validation |
| **Completions** | ⭐⭐ Intermediate | Zero-cost | +1 dep | UX enhancement |
| **Observability** | ⭐⭐ Intermediate | <2% | +2 deps | Production |
| **Kernel Features** | ⭐⭐⭐ Expert | <10% | +3 deps | Determinism |
| **Agent2028** | ⭐⭐⭐ Expert | <15% | +6 deps | Distributed |
| **RDF/SPARQL** | ⭐⭐⭐ Expert | 15-20% | +4 deps | Semantic |
| **Advanced I/O** | ⭐⭐⭐ Expert | <5% | +3 deps | Streaming |
| **Crypto Receipts** | ⭐⭐⭐ Expert | <8% | +3 deps | Audit trails |

---

## Decision Tree

### "Which features should I use?"

```
START
  │
  ├─ Building simple CLI tool?
  │  └─ YES → Use: default (10 deps, 8s compile, 2.1 MB)
  │
  ├─ Need I/O operations (API calls, DB)?
  │  └─ YES → Add: async (+5 deps, 12s compile, 3.4 MB)
  │
  ├─ Building for AI agents?
  │  └─ YES → Add: autonomic + crypto (+8 deps, 15s compile, 4.2 MB)
  │
  ├─ Need production monitoring?
  │  └─ YES → Add: observability (+2 deps, 17s compile, 4.5 MB)
  │
  ├─ Audit compliance required?
  │  └─ YES → Add: kernel (+3 deps, 20s compile, 5.2 MB)
  │
  ├─ Building distributed system?
  │  └─ YES → Add: agent2028 (+6 deps, 30s compile, 6.8 MB)
  │
  ├─ Need semantic discovery?
  │  └─ YES → Add: rdf (+4 deps, 35s compile, 7.5 MB)
  │
  └─ Want everything?
     └─ YES → Use: full (55 deps, 45s compile, 8.5 MB)
```

---

## Progressive Mastery Path

### Week 1: Foundations (Beginner)
- **Day 1-2:** Procedural macros (`#[verb]`, `#[noun]`)
- **Day 3-4:** Type inference & auto-discovery
- **Day 5-7:** Build CLI with 10+ commands

### Week 2-3: Intermediate Features
- **Day 8-10:** Async operations (`#[async_verb]`)
- **Day 11-13:** Autonomic layer (introspection, effects, receipts)
- **Day 14-17:** Observability (tracing, metrics)
- **Day 18-21:** Build async health checker

### Week 4-6: Expert Features
- **Day 22-25:** Kernel features (deterministic execution)
- **Day 26-30:** Agent2028 (delegation, policy, certificates)
- **Day 31-35:** RDF/SPARQL (semantic ontology)
- **Day 36-42:** Build distributed agent system

---

## Feature Combinations

### Lightweight CLI (Development Tools)
```toml
[dependencies]
clap-noun-verb = "5.3"  # Just 10 core dependencies
```
**Result:** 8s compile, 2.1 MB binary

---

### Agent-Ready CLI (AI Systems)
```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["autonomic", "async", "crypto"] }
```
**Result:** 20s compile, 4.5 MB binary
**Use for:** AI agents, MAPE-K loops, autonomous systems

---

### Production CLI (Observability Required)
```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = [
    "autonomic",
    "async",
    "observability",
    "validators"
] }
```
**Result:** 22s compile, 4.8 MB binary
**Use for:** Production tools, DevOps, monitoring

---

### Semantic CLI (Ontology-Driven)
```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["rdf", "autonomic", "agent2028"] }
```
**Result:** 35s compile, 7.5 MB binary
**Use for:** Knowledge graphs, MCP integration, semantic systems

---

### Distributed Agent System (Full-Featured)
```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = [
    "agent2028",
    "kernel",
    "async",
    "crypto",
    "observability"
] }
```
**Result:** 38s compile, 7.8 MB binary
**Use for:** Multi-agent coordination, Byzantine systems, distributed execution

---

### All Features (Comprehensive)
```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["full"] }
```
**Result:** 45s compile, 8.5 MB binary
**Use for:** Research, prototyping, maximum capabilities

---

## Performance Impact

### Compile Time Comparison

| Configuration | Dependencies | Compile Time | Incremental |
|--------------|--------------|--------------|-------------|
| Default | 10 | 8s | 2s |
| + async | 15 | 12s | 3s |
| + autonomic | 14 | 11s | 3s |
| + agent2028 | 20 | 18s | 5s |
| + rdf | 24 | 25s | 6s |
| Full | 55 | 45s | 10s |

### Runtime Performance

| Feature | Overhead | Latency | Memory |
|---------|----------|---------|--------|
| Procedural Macros | 0% | 0ms | 0 KB |
| Type Inference | 0% | 0ms | 0 KB |
| Async (Tokio) | 2-5% | +1ms | +2KB/task |
| Autonomic | 3-5% | +1-5ms | +200KB |
| Observability | 1-2% | <1ms | +100KB |
| Kernel | 8-10% | +5-10ms | +1KB/receipt |
| Agent2028 | 12-15% | +10-20ms | +5KB/agent |
| RDF/SPARQL | 15-20% | +5-50ms | +10MB |
| Crypto | 5-8% | +1-10ms | +200B/receipt |

### Binary Size

| Configuration | Binary Size |
|--------------|------------|
| Debug build | 45 MB |
| Release (default) | 8.5 MB |
| Release + strip | 4.2 MB |
| Release + strip + opt-level=z | 2.8 MB |
| Minimal features | 2.1 MB |

---

## Troubleshooting Checklist

### Command Not Found
- [ ] Module included in `main.rs`?
- [ ] `#[verb]` macro used?
- [ ] Macro imported (`use clap_noun_verb_macros::verb;`)?

### Async Runtime Not Found
- [ ] `#[tokio::main]` on main function?
- [ ] Feature `async` enabled in Cargo.toml?
- [ ] Using `run_async()` instead of `run()`?

### Type Inference Failed
- [ ] Using concrete types (not complex generics)?
- [ ] Parameter types implement `FromStr`?
- [ ] Return type implements `Serialize`?

### Feature Not Available
- [ ] Feature enabled in Cargo.toml?
- [ ] Correct feature name (check [Cargo.toml](../Cargo.toml))?
- [ ] Rebuild after adding feature (`cargo clean && cargo build`)?

### Slow Compile Times
- [ ] Using minimal features (not `full`)?
- [ ] Incremental compilation enabled?
- [ ] Using `sccache` for caching?
- [ ] Cargo workspace for domain separation?

### Slow Runtime
- [ ] Hot-path optimized (use `Lazy` static)?
- [ ] Caching enabled for repeated operations?
- [ ] Using async concurrency for I/O?
- [ ] Profiled with `cargo flamegraph`?

### Large Binary Size
- [ ] Release build with `strip = true`?
- [ ] Using `opt-level = "z"`?
- [ ] Minimal features enabled?
- [ ] Checked with `cargo bloat`?

---

## Quick Start Examples

### Basic CLI (Beginner)

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct GreetingResult {
    message: String,
}

#[verb("greet", "users")]
fn greet(name: String) -> Result<GreetingResult, Box<dyn std::error::Error>> {
    Ok(GreetingResult {
        message: format!("Hello, {}!", name),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    clap_noun_verb::run()
}
```

---

### Async CLI (Intermediate)

```rust
use clap_noun_verb_macros::async_verb;

#[async_verb(help = "Fetch data from API")]
async fn fetch(url: String) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    let response = reqwest::get(&url).await?.json().await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    clap_noun_verb::run_async().await
}
```

---

### Agent-Ready CLI (Intermediate)

```rust
#[verb(
    help = "Deploy with audit trail",
    effects = ["writes_state", "network_call"],
    sensitivity = "high"
)]
fn deploy(app: String) -> Result<DeploymentReceipt, Box<dyn std::error::Error>> {
    let deployment = crate::domain::deployments::deploy(&app)?;

    let receipt = Receipt::new()
        .with_operation("deploy")
        .with_input(&app)
        .with_output(&deployment)
        .sign()?;

    Ok(DeploymentReceipt {
        deployment_id: deployment.id,
        receipt_hash: receipt.hash(),
    })
}
```

---

### Distributed System (Expert)

```rust
#[async_verb(
    help = "Distributed execution with delegation",
    effects = ["writes_state"],
    sensitivity = "critical",
    kernel = "deterministic"
)]
async fn distributed_deploy(
    manifest: String,
    token_id: String,
    ctx: KernelContext,
) -> Result<DistributedReceipt, Box<dyn std::error::Error>> {
    // Verify delegation
    let token = ctx.delegation_registry().get(&token_id)?;
    token.verify_chain()?;

    // Execute with certificate
    let cert = Certificate::unchecked(invocation)
        .policy_check(ctx.policy_engine())?
        .capability_check(&token)?;

    let deployment = execute_distributed(&manifest, &cert).await?;

    // Generate receipt
    let receipt = ctx.create_receipt()
        .with_certificate(&cert)
        .sign()?;

    ctx.ledger().append(&receipt)?;

    Ok(DistributedReceipt {
        deployment_id: deployment.id,
        receipt_hash: receipt.hash(),
    })
}
```

---

## When to Use Each Feature

| If you need... | Use feature... | Complexity | Impact |
|----------------|---------------|-----------|--------|
| Basic CLI | `default` | ⭐ | Zero |
| API calls, DB queries | `async` | ⭐⭐ | Low |
| AI agent integration | `autonomic` | ⭐⭐ | Low |
| Production monitoring | `observability` | ⭐⭐ | Low |
| Input validation | `validators` | ⭐⭐ | Minimal |
| Shell completions | `completions` | ⭐⭐ | Zero |
| Audit compliance | `kernel` + `crypto` | ⭐⭐⭐ | Medium |
| Multi-agent systems | `agent2028` | ⭐⭐⭐ | Medium |
| Semantic discovery | `rdf` | ⭐⭐⭐ | High |
| Stream processing | `io` | ⭐⭐⭐ | Low |

---

## Common Anti-Patterns

### ❌ Feature Bloat
```toml
# DON'T: Use full without understanding cost
features = ["full"]  # 55 deps, 45s compile
```

### ✅ Minimal Features
```toml
# DO: Use only what you need
features = ["async", "autonomic"]  # 20 deps, 18s compile
```

---

### ❌ Blocking Async
```rust
// DON'T: Block async runtime
#[async_verb]
async fn bad() -> Result<(), Error> {
    std::thread::sleep(Duration::from_secs(10));  // Blocks!
}
```

### ✅ Async Best Practices
```rust
// DO: Use spawn_blocking
#[async_verb]
async fn good() -> Result<(), Error> {
    tokio::task::spawn_blocking(|| {
        expensive_work()
    }).await?;
    Ok(())
}
```

---

### ❌ Over-Engineering
```rust
// DON'T: RDF for simple CLI
features = ["rdf"]  // Overkill for 3 commands
```

### ✅ Right-Sizing
```rust
// DO: Match complexity to needs
features = []  // Simple CLI = simple features
```

---

## Resources

- **Complete Guide:** [ADVANCED_FEATURES_GUIDE.md](ADVANCED_FEATURES_GUIDE.md)
- **Tutorial Series:** [Tutorial 01-10](tutorial/)
- **How-To Guides:** [How-To Production](howto/production/)
- **API Reference:** [Reference Docs](reference/)
- **Architecture:** [Architecture V5](ARCHITECTURE_V5_COMPLETE.md)

---

## Quick Links

- [GitHub Repository](https://github.com/seanchatmangpt/clap-noun-verb)
- [Crates.io](https://crates.io/crates/clap-noun-verb)
- [Documentation](https://docs.rs/clap-noun-verb)
- [Issue Tracker](https://github.com/seanchatmangpt/clap-noun-verb/issues)

---

**Last Updated:** 2026-01-05
**Version:** 5.3.4
