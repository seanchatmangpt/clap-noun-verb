# clap-noun-verb Advanced Features Guide

**Version:** 5.3.4
**Last Updated:** 2026-01-05
**Purpose:** Comprehensive guide to advanced capabilities for building production-grade, agent-ready CLIs

---

## Executive Summary

This guide synthesizes all advanced features of clap-noun-verb into a progressive learning path. From procedural macros to trillion-agent ecosystems, each feature is organized by complexity level with practical examples, performance implications, and integration strategies.

**Who this guide is for:**
- Developers moving beyond basic CLI patterns
- Teams building agent-grade autonomous systems
- Architects designing distributed CLI frameworks
- Performance engineers optimizing CLI applications

---

## Table of Contents

1. [Feature Overview Matrix](#1-feature-overview-matrix)
2. [Hierarchical Feature Taxonomy](#2-hierarchical-feature-taxonomy)
3. [Decision Matrix](#3-decision-matrix)
4. [Progressive Mastery Path](#4-progressive-mastery-path)
5. [Feature Deep Dives](#5-feature-deep-dives)
6. [Integration Patterns](#6-integration-patterns)
7. [Architecture Patterns](#7-architecture-patterns)
8. [Performance Tuning Guide](#8-performance-tuning-guide)
9. [Safety Considerations](#9-safety-considerations)
10. [Troubleshooting Guide](#10-troubleshooting-guide)
11. [Future Directions](#11-future-directions)

---

## 1. Feature Overview Matrix

### Core Advanced Features

| Feature | Complexity | Performance Impact | Use Cases | Cargo Feature |
|---------|-----------|-------------------|-----------|--------------|
| **Procedural Macros** | Beginner | Zero-cost | All CLIs | `default` |
| **Type Inference** | Beginner | Zero-cost | Ergonomic APIs | `default` |
| **Auto-Discovery** | Beginner | Zero-cost | Modular CLIs | `default` |
| **Async Support** | Intermediate | Low overhead | I/O-bound ops | `async` |
| **Autonomic Layer** | Intermediate | <5% overhead | Agent systems | `autonomic` |
| **Feature Flags** | Intermediate | Zero-cost | Minimal builds | `default` |
| **Validators** | Intermediate | Low overhead | Input validation | `validators` |
| **Shell Completions** | Intermediate | Zero-cost | UX enhancement | `completions` |
| **Observability** | Intermediate | <2% overhead | Production monitoring | `observability` |
| **Kernel Features** | Expert | <10% overhead | Determinism | `kernel` |
| **Agent2028** | Expert | <15% overhead | Distributed agents | `agent2028` |
| **RDF/SPARQL** | Expert | Moderate overhead | Semantic systems | `rdf` |
| **Advanced I/O** | Expert | Low overhead | Stream processing | `io` |
| **Crypto Receipts** | Expert | <8% overhead | Audit trails | `crypto` |

**Performance Impact Legend:**
- **Zero-cost:** Compile-time only, no runtime overhead
- **Low overhead:** <5% performance impact
- **Moderate overhead:** 5-15% performance impact
- **High overhead:** >15% performance impact (requires justification)

---

## 2. Hierarchical Feature Taxonomy

### 2.1 Beginner Advanced Features

**Goal:** Master type-first thinking and zero-cost abstractions

#### 2.1.1 Procedural Macro System

**What it is:** Compile-time code generation for command registration

**Why it matters:**
- Zero boilerplate command registration
- Type-safe argument inference
- Automatic help text generation
- Compile-time validation

**Code Example:**

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct GreetingResult {
    message: String,
    timestamp: String,
}

/// Greet a user by name
#[verb("greet", "users")]
fn greet_user(
    #[arg(help = "User's name")] name: String,
    #[arg(long, help = "Greeting language", default_value = "en")]
    lang: String,
) -> Result<GreetingResult, Box<dyn std::error::Error>> {
    let message = match lang.as_str() {
        "es" => format!("¡Hola, {}!", name),
        "fr" => format!("Bonjour, {}!", name),
        _ => format!("Hello, {}!", name),
    };

    Ok(GreetingResult {
        message,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
```

**Key Capabilities:**
- `#[verb]` - Auto-register command
- `#[arg]` - Customize argument behavior
- Type inference from function signature
- Docstring extraction for help text

**Performance Characteristics:**
- Compile time: +0.5-2s (one-time cost)
- Runtime: Zero overhead (monomorphization)
- Binary size: +10-50KB per command

**When to use:**
- All CLI commands (this is the foundation)
- Want zero-boilerplate registration
- Need type-safe argument parsing

**Anti-patterns:**
- Don't use for runtime-dynamic commands
- Don't over-complicate with macros (keep logic in domain layer)

---

#### 2.1.2 Type Inference System

**What it is:** Automatic argument type detection from function signatures

**Why it matters:**
- No manual type specifications
- Compiler enforces correctness
- Self-documenting APIs

**Code Example:**

```rust
// Types automatically inferred from function signature
#[verb]
fn calculate(
    x: i32,           // ← Inferred as integer argument
    y: i32,           // ← Inferred as integer argument
    operation: String, // ← Inferred as string argument
    #[arg(long)]
    precision: Option<u8>, // ← Inferred as optional argument
) -> Result<CalculationResult, Box<dyn std::error::Error>> {
    // Implementation
}
```

**Supported Types:**
- Primitives: `String`, `i32`, `u32`, `f64`, `bool`
- Options: `Option<T>` for optional arguments
- Vectors: `Vec<String>` for multi-value arguments
- Custom: Any type implementing `FromStr` or `clap::ValueEnum`

**Performance Characteristics:**
- Zero runtime overhead (monomorphization)
- Compile-time type checking
- No reflection or dynamic dispatch

---

#### 2.1.3 Auto-Discovery with Linkme

**What it is:** Automatic command discovery at compile time using distributed slices

**Why it matters:**
- No manual registration code
- Modular command organization
- Compile-time guarantee of command availability

**How it works:**

```
┌─────────────────────────────────────────┐
│   Compile Time (linkme)                  │
│                                          │
│  #[verb] → COMMANDS slice registration   │
│  #[noun] → NOUNS slice registration      │
│                                          │
│  All slices merged at link time          │
└───────────────┬─────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────┐
│   Runtime (clap_noun_verb::run())        │
│                                          │
│  Iterate COMMANDS slice                  │
│  Build clap Command tree                 │
│  Execute matched command                 │
└─────────────────────────────────────────┘
```

**Code Example:**

```rust
// commands/users.rs
#[verb("list", "users")]
fn list_users() -> Result<UserList, Box<dyn std::error::Error>> {
    // Automatically discovered at compile time
}

// commands/services.rs
#[verb("start", "services")]
fn start_service(name: String) -> Result<ServiceInfo, Box<dyn std::error::Error>> {
    // Also automatically discovered
}

// main.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    clap_noun_verb::run() // ← All commands auto-discovered
}
```

**Performance Characteristics:**
- Compile time: No overhead (linkme is zero-cost)
- Runtime: O(n) command lookup (n = total commands)
- Binary size: +0KB (metadata only)

**When to use:**
- Multi-module CLIs
- Plugin architectures
- Large command sets

---

### 2.2 Intermediate Advanced Features

**Goal:** Add async, observability, and agent-readiness

#### 2.2.1 Async Support (Tokio Integration)

**What it is:** First-class async/await support for I/O-bound operations

**Why it matters:**
- Concurrent network requests
- Non-blocking database queries
- Efficient resource utilization
- Scalable I/O operations

**Code Example:**

```rust
use clap_noun_verb_macros::async_verb;
use tokio::time::{timeout, Duration};

#[async_verb(
    help = "Fetch data from multiple APIs concurrently",
    effects = ["network_call"],
    sensitivity = "low"
)]
async fn fetch_all(
    #[arg(help = "Comma-separated API URLs")]
    urls: String,
) -> Result<MultiSourceResult, Box<dyn std::error::Error>> {
    let url_list: Vec<&str> = urls.split(',').collect();

    // Launch concurrent requests
    let futures: Vec<_> = url_list
        .iter()
        .map(|url| async move {
            timeout(
                Duration::from_secs(5),
                reqwest::get(*url)
            )
            .await?
            .json::<ApiResponse>()
            .await
        })
        .collect();

    // Wait for all to complete
    let results = futures::future::try_join_all(futures).await?;

    Ok(MultiSourceResult {
        sources: results.len(),
        data: results,
    })
}
```

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["async"] }
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
```

**Performance Characteristics:**
- Runtime overhead: 2-5% (tokio scheduler)
- Concurrency: Up to 1000s of concurrent tasks
- Memory: ~2KB per task
- Latency: Near-optimal for I/O-bound workloads

**When to use:**
- Network requests (APIs, databases)
- File I/O operations
- Multiple concurrent operations
- Long-running background tasks

**Anti-patterns:**
- Don't use for CPU-bound operations (use rayon instead)
- Don't mix blocking and async code without proper bridging
- Don't spawn unbounded tasks (use semaphores)

---

#### 2.2.2 Autonomic CLI Layer

**What it is:** Machine-grade introspection for AI agents and autonomous systems

**Why it matters:**
- AI agents can discover capabilities
- Effect metadata enables safety analysis
- Execution receipts provide audit trails
- Guards enforce resource constraints

**Code Example:**

```rust
#[verb(
    help = "Deploy application to production",
    effects = ["writes_state", "network_call", "expensive"],
    sensitivity = "critical",
    guards = ["budget_check", "approval_required"]
)]
fn deploy_production(
    #[arg(help = "Application name")]
    app: String,

    #[arg(help = "Deployment region")]
    region: String,
) -> Result<DeploymentReceipt, Box<dyn std::error::Error>> {
    // Domain logic
    let deployment = crate::domain::deployments::deploy(&app, &region)?;

    // Generate cryptographic receipt
    let receipt = Receipt::new()
        .with_operation("deploy")
        .with_effects(&["writes_state", "network_call"])
        .with_result(&deployment)
        .sign()?;

    Ok(DeploymentReceipt {
        deployment_id: deployment.id,
        receipt_hash: receipt.hash(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
```

**Introspection API:**

```bash
# Discover all capabilities
$ myapp --capabilities
{
  "capabilities": [
    {
      "noun": "deployments",
      "verb": "deploy-production",
      "effects": ["writes_state", "network_call", "expensive"],
      "sensitivity": "critical",
      "guards": ["budget_check", "approval_required"],
      "arguments": [...]
    }
  ]
}

# Deep inspection of specific command
$ myapp deployments deploy-production --introspect
{
  "command": "deployments deploy-production",
  "effects": ["writes_state", "network_call", "expensive"],
  "sensitivity": "critical",
  "risk_assessment": {
    "destructiveness": "high",
    "reversibility": "difficult",
    "blast_radius": "production"
  },
  "required_approvals": ["engineering_lead", "sre_team"]
}
```

**Effect Types:**

| Effect | Meaning | Agent Behavior |
|--------|---------|----------------|
| `reads_state` | Reads system state | Safe for automation |
| `writes_state` | Modifies state | Requires review |
| `network_call` | External communication | Monitor for failures |
| `destructive` | Irreversible changes | Human approval required |
| `expensive` | High resource usage | Budget check required |

**Sensitivity Levels:**

| Level | Automation | Approval | Examples |
|-------|-----------|----------|----------|
| `low` | Fully automated | None | List resources |
| `medium` | Semi-automated | Review recommended | Restart service |
| `high` | Manual approval | Required | Deploy to production |
| `critical` | Human-in-loop | Multi-party | Drop database |

**Performance Characteristics:**
- Runtime overhead: 3-5% (telemetry spans)
- Binary size: +200KB (autonomic runtime)
- Latency: +1-5ms per command

**When to use:**
- AI agent integration
- MAPE-K loops
- Audit compliance requirements
- Safety-critical systems

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["autonomic", "crypto"] }
```

---

#### 2.2.3 Feature Flag Architecture

**What it is:** Granular control over compiled features for minimal dependency burden

**Why it matters:**
- Minimal default build (10 dependencies)
- Pay only for what you use
- Fast compile times
- Small binary sizes

**Feature Catalog:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = [
    # Choose only what you need:

    # Core (always available)
    # - macros, type inference, auto-discovery

    # Optional features:
    "async",           # Tokio async runtime (+5 deps)
    "autonomic",       # Introspection & telemetry (+4 deps)
    "agent2028",       # Trillion-agent ecosystem (+6 deps)
    "kernel",          # Deterministic execution (+3 deps)
    "rdf",             # RDF/SPARQL ontology (+4 deps)
    "crypto",          # Cryptographic receipts (+3 deps)
    "io",              # Advanced I/O with clio (+3 deps)
    "observability",   # Tracing & metrics (+2 deps)
    "validators",      # URL/regex validation (+2 deps)
    "completions",     # Shell completions (+1 dep)
    "mangen",          # Man page generation (+1 dep)
    "config-formats",  # YAML/TOML configs (+2 deps)
    "templates",       # Handlebars templates (+1 dep)
    "caching",         # LRU caching (+2 deps)
    "concurrency",     # Crossbeam primitives (+2 deps)

    # Convenience:
    "full",            # All features (+45 deps total)
] }
```

**Dependency Comparison:**

| Configuration | Dependencies | Compile Time | Binary Size |
|--------------|--------------|--------------|-------------|
| Default | 10 | 8s | 2.1 MB |
| + async | 15 | 12s | 3.4 MB |
| + autonomic | 14 | 11s | 2.8 MB |
| + agent2028 | 20 | 18s | 4.2 MB |
| Full | 55 | 45s | 8.5 MB |

**Strategic Feature Combinations:**

```toml
# Lightweight CLI (development tools)
features = []  # Just 10 core dependencies

# Agent-ready CLI (AI systems)
features = ["autonomic", "async", "crypto"]

# Production CLI (observability required)
features = ["autonomic", "async", "observability", "validators"]

# Semantic CLI (ontology-driven)
features = ["rdf", "autonomic", "agent2028"]

# All features (comprehensive systems)
features = ["full"]
```

**Performance Characteristics:**
- Compile time: Linear with feature count
- Runtime: No overhead (features are compile-time)
- Binary size: ~150-500KB per feature

**When to use:**
- Minimizing compile times
- Reducing binary size
- Embedded systems
- Fast CI/CD pipelines

---

#### 2.2.4 Validators

**What it is:** Built-in validation for URLs, regexes, and custom patterns

**Why it matters:**
- Type-safe input validation
- Better error messages
- Fail-fast at argument parsing

**Code Example:**

```rust
use clap_noun_verb::validators::{validate_url, validate_regex};

#[verb(help = "Fetch from URL")]
fn fetch(
    #[arg(help = "API URL", validator = validate_url)]
    url: String,

    #[arg(help = "Filter pattern", validator = validate_regex)]
    pattern: String,
) -> Result<FetchResult, Box<dyn std::error::Error>> {
    // url is guaranteed to be valid URL
    // pattern is guaranteed to be valid regex
}
```

**Built-in Validators:**

```rust
// URL validation
validate_url("https://api.example.com") // ✅ Valid
validate_url("not-a-url")               // ❌ Error

// Regex validation
validate_regex(r"^\d+$")                // ✅ Valid
validate_regex(r"[unclosed")            // ❌ Error

// Custom validators
fn validate_port(s: &str) -> Result<(), String> {
    let port: u16 = s.parse()
        .map_err(|_| format!("Invalid port: {}", s))?;

    if port < 1024 {
        return Err("Port must be >= 1024".to_string());
    }

    Ok(())
}

#[verb]
fn connect(
    #[arg(validator = validate_port)]
    port: String,
) -> Result<Connection, Box<dyn std::error::Error>> {
    // port is guaranteed to be valid and >= 1024
}
```

**Performance Characteristics:**
- Runtime overhead: <1ms per validation
- No heap allocations for simple validators
- Error messages pre-allocated

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["validators"] }
```

---

#### 2.2.5 Shell Completions

**What it is:** Auto-generate shell completion scripts for Bash, Zsh, Fish, PowerShell

**Why it matters:**
- Enhanced user experience
- Discoverability of commands
- Reduced typos and errors

**Code Example:**

```rust
use clap_noun_verb::completion::{generate_completion, Shell};

#[verb(help = "Generate shell completions")]
fn generate_completions(
    #[arg(help = "Shell type (bash, zsh, fish, powershell)")]
    shell: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let shell = match shell.as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => return Err("Invalid shell".into()),
    };

    let mut cmd = build_cli();
    print_completion(&mut cmd, shell, "myapp")?;

    Ok(())
}
```

**Usage:**

```bash
# Generate Bash completions
$ myapp completions generate --shell bash > /etc/bash_completion.d/myapp

# Now you can tab-complete:
$ myapp <TAB>
deployments  services  users  config

$ myapp deployments <TAB>
deploy  list  delete  status
```

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["completions"] }
```

---

#### 2.2.6 Observability (Tracing & Metrics)

**What it is:** Built-in tracing and metrics for production monitoring

**Why it matters:**
- Performance profiling
- Debug production issues
- Monitor command execution
- SLO compliance tracking

**Code Example:**

```rust
use tracing::{info, warn, instrument};

#[instrument(skip(database_url))]
#[async_verb(help = "Sync data from external source")]
async fn sync_data(
    #[arg(env = "DATABASE_URL")]
    database_url: String,

    #[arg(help = "Source API URL")]
    source: String,
) -> Result<SyncResult, Box<dyn std::error::Error>> {
    info!("Starting data sync from {}", source);

    // Fetch from source (automatically traced)
    let data = fetch_data(&source).await?;
    info!(records = data.len(), "Fetched data");

    // Write to database (automatically traced)
    let written = write_to_db(&database_url, &data).await?;
    info!(written, "Wrote to database");

    Ok(SyncResult {
        fetched: data.len(),
        written,
    })
}
```

**Tracing Output:**

```
2026-01-05T12:00:00.123Z INFO sync_data: Starting data sync from https://api.example.com
2026-01-05T12:00:01.456Z INFO sync_data: Fetched data records=1000
2026-01-05T12:00:02.789Z INFO sync_data: Wrote to database written=1000
```

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["observability"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Performance Characteristics:**
- Runtime overhead: 1-2% (with subscriber)
- Zero overhead if no subscriber installed
- Asynchronous log writing

---

### 2.3 Expert Advanced Features

**Goal:** Master deterministic execution, distributed agents, and semantic systems

#### 2.3.1 Kernel Features (Deterministic Execution)

**What it is:** Deterministic command execution with cryptographic receipts and replay capability

**Why it matters:**
- Reproducible builds and execution
- Audit compliance (SOC2, HIPAA)
- Byzantine fault tolerance
- State machine verification

**Architecture:**

```
┌─────────────────────────────────────────────────────────┐
│                 KERNEL EXECUTION PIPELINE                │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Input → Policy Check → Guard Check → Execute →         │
│          Certificate    Budget       Domain             │
│                                      Logic              │
│                                        │                │
│                                        ▼                │
│                                   Receipt               │
│                                   Generation            │
│                                        │                │
│                                        ▼                │
│                               Governance                │
│                               Ledger                    │
└─────────────────────────────────────────────────────────┘
```

**Code Example:**

```rust
use clap_noun_verb::kernel::{Receipt, KernelContext};

#[verb(
    help = "Execute with deterministic receipt",
    effects = ["writes_state"],
    sensitivity = "high",
    kernel = "deterministic"  // ← Enable kernel mode
)]
fn deploy_with_receipt(
    #[arg(help = "Application name")]
    app: String,

    ctx: KernelContext,  // ← Kernel context injected
) -> Result<DeploymentReceipt, Box<dyn std::error::Error>> {
    // Execute domain logic
    let deployment = crate::domain::deployments::deploy(&app)?;

    // Generate deterministic receipt
    let receipt = ctx.create_receipt()
        .with_input(&app)
        .with_output(&deployment)
        .with_effects(&["writes_state"])
        .sign()?;

    // Append to governance ledger
    ctx.ledger().append(&receipt)?;

    Ok(DeploymentReceipt {
        deployment_id: deployment.id,
        receipt_hash: receipt.hash(),
        timestamp: receipt.timestamp(),
        signature: receipt.signature(),
    })
}
```

**Receipt Structure:**

```rust
pub struct Receipt {
    // Unique receipt ID
    pub id: ReceiptId,

    // Command executed
    pub command: String,

    // Input arguments (hashed)
    pub input_hash: Hash,

    // Output result (hashed)
    pub output_hash: Hash,

    // Effects applied
    pub effects: Vec<Effect>,

    // Timestamp (deterministic)
    pub timestamp: SystemTime,

    // Cryptographic signature
    pub signature: Signature,
}
```

**Governance Ledger:**

```rust
// Query execution history
let receipts = ctx.ledger().query()
    .command("deploy")
    .time_range(start, end)
    .execute()?;

// Verify receipt chain
let valid = ctx.ledger().verify_chain()?;

// Replay execution from receipt
let result = ctx.replay_receipt(&receipt_id)?;
```

**Performance Characteristics:**
- Runtime overhead: 8-10% (hashing + signing)
- Memory: ~1KB per receipt
- Disk: ~500 bytes per ledger entry
- Verification: O(n) for n receipts

**When to use:**
- Audit compliance requirements
- Non-repudiation of actions
- Reproducible execution
- Byzantine fault tolerance

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["kernel", "crypto"] }
```

---

#### 2.3.2 Agent2028 Ecosystem (Trillion-Agent Coordination)

**What it is:** Distributed agent coordination with delegation, policy enforcement, and certificate chains

**Why it matters:**
- Scale to millions/billions of agents
- Delegation of authority
- Policy-based access control
- Distributed execution

**Architecture:**

```
┌─────────────────────────────────────────────────────────┐
│                    AGENT2028 LAYERS                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │          POLICY ENGINE (Authorization)          │    │
│  └────────────────────────────────────────────────┘    │
│                         ▲                               │
│  ┌────────────────────────────────────────────────┐    │
│  │       DELEGATION SYSTEM (Identity Chain)        │    │
│  └────────────────────────────────────────────────┘    │
│                         ▲                               │
│  ┌────────────────────────────────────────────────┐    │
│  │     CERTIFICATE SYSTEM (Proof-Carrying)         │    │
│  └────────────────────────────────────────────────┘    │
│                         ▲                               │
│  ┌────────────────────────────────────────────────┐    │
│  │        PLANE INTERACTIONS (Metadata)            │    │
│  │  O: Observations  Σ: Ontology  Q: Invariants   │    │
│  └────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Delegation System:**

```rust
use clap_noun_verb::agent2028::{DelegationToken, DelegationRegistry};

// Create delegation token
let token = DelegationToken::new()
    .delegator("root-agent")
    .delegate("sub-agent-123")
    .capabilities(&["deployments:deploy", "services:restart"])
    .temporal_constraint(
        SystemTime::now(),
        SystemTime::now() + Duration::from_secs(3600)  // 1 hour
    )
    .sign()?;

// Register token
let registry = DelegationRegistry::global();
registry.register(token)?;

// Verify delegation
let valid = registry.verify(&token_id, "deployments:deploy")?;
```

**Policy Engine:**

```rust
use clap_noun_verb::agent2028::{PolicyEngine, Policy, Decision};

// Define policy
let policy = Policy::new()
    .rule("production-deploys", |request| {
        if request.capability == "deployments:deploy" &&
           request.target_env == "production" {
            // Require human approval
            Decision::Deny("Human approval required for production".into())
        } else {
            Decision::Allow
        }
    });

// Register policy
let engine = PolicyEngine::global();
engine.add_policy(policy)?;

// Evaluate request
let decision = engine.evaluate(&request)?;
```

**Certificate Chains:**

```rust
use clap_noun_verb::agent2028::Certificate;

// Create proof-carrying invocation
let cert = Certificate::unchecked(invocation)
    .policy_check(&policy_engine)?  // State transition: Unchecked → PolicyChecked
    .capability_check(&delegation)?  // State transition: PolicyChecked → CapabilityChecked
    .schema_check(&schema_hash)?;    // State transition: CapabilityChecked → SchemaChecked

// Execute with certificate
let result = execute_certified(&cert)?;

// Sign final certificate
let signed_cert = cert.sign()?;
```

**Plane Interactions (Metadata Tracking):**

```rust
// O Plane - Observations (telemetry)
ctx.plane_o().emit_observation("deployment_started", metadata);

// Σ Plane - Ontology (schema)
ctx.plane_sigma().read_schema("Deployment");

// Q Plane - Invariants (guards)
ctx.plane_q().check_invariant("budget_available", &budget);

// ΔΣ Plane - Overlays (migrations)
ctx.plane_delta_sigma().propose_migration("v2-schema");
```

**Performance Characteristics:**
- Runtime overhead: 12-15% (policy + delegation)
- Memory: ~5KB per agent
- Latency: +10-20ms per request (policy evaluation)

**When to use:**
- Multi-agent systems
- Distributed CLI frameworks
- Policy-driven access control
- Trillion-agent ecosystems

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["agent2028", "kernel", "crypto"] }
```

---

#### 2.3.3 RDF/SPARQL (Semantic Ontology Layer)

**What it is:** RDF-based ontology control with SPARQL queries for semantic CLI systems

**Why it matters:**
- Semantic command discovery
- Ontology-driven validation
- Knowledge graph integration
- MCP (Model Context Protocol) compatibility

**Architecture:**

```
┌─────────────────────────────────────────────────────────┐
│                  RDF/SPARQL LAYER                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │        SPARQL QUERY ENGINE (oxigraph)           │    │
│  └────────────────────────────────────────────────┘    │
│                         ▲                               │
│  ┌────────────────────────────────────────────────┐    │
│  │          RDF ONTOLOGY (Command Graph)           │    │
│  │  Nouns → Verbs → Effects → Constraints          │    │
│  └────────────────────────────────────────────────┘    │
│                         ▲                               │
│  ┌────────────────────────────────────────────────┐    │
│  │           MCP SERVER (Tool Integration)         │    │
│  └────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**RDF Command Representation:**

```turtle
@prefix cnv: <http://clap-noun-verb.org/ontology#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:DeployCommand a cnv:Command ;
    cnv:noun :Deployments ;
    cnv:verb :Deploy ;
    cnv:hasEffect :WritesState, :NetworkCall ;
    cnv:sensitivity "critical"^^xsd:string ;
    cnv:requiresCapability "deployments:deploy" .

:Deployments a cnv:Noun ;
    cnv:description "Application deployment resources" .

:Deploy a cnv:Verb ;
    cnv:description "Deploy application to environment" ;
    cnv:hasArgument [
        cnv:name "app" ;
        cnv:type "String" ;
        cnv:required true
    ] .
```

**SPARQL Queries:**

```rust
use clap_noun_verb::rdf::{RdfStore, SparqlQuery};

// Query all high-sensitivity commands
let query = SparqlQuery::new(r#"
    PREFIX cnv: <http://clap-noun-verb.org/ontology#>

    SELECT ?command ?effect
    WHERE {
        ?command a cnv:Command ;
                 cnv:sensitivity "critical" ;
                 cnv:hasEffect ?effect .
    }
"#);

let results = store.execute(&query)?;

// Discover commands by effect
let safe_commands = store.query(r#"
    SELECT ?command
    WHERE {
        ?command a cnv:Command ;
                 cnv:hasEffect cnv:ReadsState .
        FILTER NOT EXISTS {
            ?command cnv:hasEffect cnv:WritesState .
        }
    }
"#)?;
```

**MCP Integration:**

```rust
use clap_noun_verb::rdf::McpServer;

// Expose CLI as MCP tools
let mcp = McpServer::new()
    .with_ontology(&rdf_store)
    .with_capabilities(&capabilities)
    .serve()?;

// AI agents can now discover and execute commands via MCP
```

**Performance Characteristics:**
- Runtime overhead: 15-20% (RDF parsing + SPARQL)
- Memory: ~10MB for typical ontology
- Query latency: 5-50ms per SPARQL query

**When to use:**
- Semantic command discovery
- Knowledge graph integration
- MCP/AI agent systems
- Complex relationship modeling

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["rdf", "agent2028"] }
oxigraph = "0.5"  # SPARQL engine
```

---

#### 2.3.4 Advanced I/O (Stream Processing)

**What it is:** High-performance I/O with clio integration for streaming data

**Why it matters:**
- Process large files without loading into memory
- Pipe-friendly CLIs
- Efficient data transformation
- Unix philosophy compatibility

**Code Example:**

```rust
use clap_noun_verb::io::{ClioInput, ClioOutput};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[async_verb(help = "Transform JSON stream")]
async fn transform(
    #[arg(help = "Input file or stdin", default = "-")]
    input: ClioInput,

    #[arg(help = "Output file or stdout", default = "-")]
    output: ClioOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = input.async_buf_reader().await?;
    let mut writer = output.async_writer().await?;

    // Stream line-by-line
    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        // Parse JSON
        let data: serde_json::Value = serde_json::from_str(&line)?;

        // Transform
        let transformed = transform_data(data);

        // Write
        writer.write_all(serde_json::to_string(&transformed)?.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        line.clear();
    }

    writer.flush().await?;
    Ok(())
}
```

**Usage:**

```bash
# Read from stdin, write to stdout
$ cat data.json | myapp transform | gzip > output.json.gz

# Read from file, write to file
$ myapp transform --input large-file.json --output processed.json

# Read from URL (clio supports HTTP)
$ myapp transform --input https://api.example.com/data
```

**Performance Characteristics:**
- Memory: O(1) - constant memory regardless of file size
- Throughput: Up to 1GB/s on modern hardware
- Latency: Near-zero buffering

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["io", "async"] }
```

---

#### 2.3.5 Cryptographic Receipts

**What it is:** Tamper-evident execution receipts using SHA-256, SHA-3, and BLAKE3

**Why it matters:**
- Non-repudiation of actions
- Audit trail integrity
- Compliance (SOC2, HIPAA, PCI-DSS)
- Byzantine fault tolerance

**Code Example:**

```rust
use clap_noun_verb::crypto::{Receipt, HashAlgorithm};
use blake3::Hasher;

#[verb(
    help = "Execute with cryptographic receipt",
    effects = ["writes_state"],
    sensitivity = "high"
)]
fn execute_with_receipt(
    #[arg(help = "Operation")]
    operation: String,
) -> Result<OperationReceipt, Box<dyn std::error::Error>> {
    // Execute operation
    let result = crate::domain::execute(&operation)?;

    // Create receipt
    let receipt = Receipt::builder()
        .algorithm(HashAlgorithm::Blake3)  // Fastest
        .command("execute")
        .input(&operation)
        .output(&result)
        .effects(&["writes_state"])
        .timestamp(SystemTime::now())
        .build()?;

    // Sign receipt
    let signed = receipt.sign_with_key(&private_key)?;

    Ok(OperationReceipt {
        result,
        receipt_hash: signed.hash().to_hex(),
        signature: signed.signature().to_hex(),
    })
}
```

**Hash Algorithm Comparison:**

| Algorithm | Speed | Security | Output Size | Use Case |
|-----------|-------|----------|-------------|----------|
| SHA-256 | 300 MB/s | High | 32 bytes | General purpose |
| SHA-3 | 150 MB/s | Very High | 32 bytes | Maximum security |
| BLAKE3 | 3 GB/s | High | 32 bytes | Performance critical |

**Receipt Verification:**

```rust
// Verify receipt integrity
let valid = receipt.verify()?;

// Verify signature
let authentic = receipt.verify_signature(&public_key)?;

// Reconstruct receipt from command
let reconstructed = Receipt::from_execution(&command, &result)?;
assert_eq!(receipt.hash(), reconstructed.hash());
```

**Performance Characteristics:**
- Hashing overhead: 1-5ms per receipt (BLAKE3)
- Signature overhead: 5-10ms per receipt (Ed25519)
- Memory: ~200 bytes per receipt

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["crypto"] }
```

---

## 3. Decision Matrix

### 3.1 Feature Selection by Use Case

| Use Case | Required Features | Optional Features | Avoid |
|----------|------------------|-------------------|-------|
| **Simple CLI Tool** | `default` | `completions` | `agent2028`, `rdf` |
| **API Client** | `default`, `async` | `validators`, `observability` | `kernel`, `agent2028` |
| **DevOps Tool** | `default`, `async`, `autonomic` | `crypto`, `observability` | `rdf` |
| **AI Agent Integration** | `autonomic`, `async` | `agent2028`, `crypto` | - |
| **Distributed System** | `agent2028`, `kernel`, `async` | `rdf`, `crypto`, `observability` | - |
| **Semantic CLI** | `rdf`, `autonomic` | `agent2028`, `mcp` | - |
| **Audit-Critical** | `kernel`, `crypto`, `autonomic` | `observability`, `agent2028` | - |

### 3.2 Performance vs Feature Trade-offs

**High Performance, Low Feature:**
```toml
# Minimal build - 10 dependencies, 8s compile, 2.1 MB binary
features = []
```

**Balanced Performance & Features:**
```toml
# Agent-ready - 25 dependencies, 20s compile, 4.5 MB binary
features = ["autonomic", "async", "crypto", "validators"]
```

**Maximum Features, Moderate Performance:**
```toml
# Full featured - 55 dependencies, 45s compile, 8.5 MB binary
features = ["full"]
```

### 3.3 Complexity vs Benefit Analysis

| Feature | Complexity | Benefit | ROI | Recommendation |
|---------|-----------|---------|-----|----------------|
| Procedural Macros | Low | Very High | ⭐⭐⭐⭐⭐ | Always use |
| Type Inference | Low | High | ⭐⭐⭐⭐⭐ | Always use |
| Auto-Discovery | Low | High | ⭐⭐⭐⭐⭐ | Always use |
| Async Support | Medium | High | ⭐⭐⭐⭐ | Use for I/O |
| Autonomic Layer | Medium | Very High | ⭐⭐⭐⭐⭐ | Use for agents |
| Feature Flags | Low | High | ⭐⭐⭐⭐ | Always use |
| Validators | Low | Medium | ⭐⭐⭐⭐ | Use for input validation |
| Completions | Low | Medium | ⭐⭐⭐ | Use for UX |
| Observability | Medium | High | ⭐⭐⭐⭐ | Use for production |
| Kernel Features | High | Very High | ⭐⭐⭐⭐ | Use for audit |
| Agent2028 | Very High | Very High | ⭐⭐⭐⭐ | Use for distributed |
| RDF/SPARQL | Very High | High | ⭐⭐⭐ | Use for semantic |
| Advanced I/O | Medium | High | ⭐⭐⭐⭐ | Use for streams |
| Crypto Receipts | Medium | Very High | ⭐⭐⭐⭐⭐ | Use for audit |

---

## 4. Progressive Mastery Path

### Week 1: Foundations (Beginner)

**Goal:** Master type-first thinking and zero-cost abstractions

**Day 1-2:** Procedural Macros
- [ ] Learn `#[verb]` and `#[noun]` syntax
- [ ] Understand auto-discovery with linkme
- [ ] Build first command with type inference

**Day 3-4:** Type Inference & Auto-Discovery
- [ ] Master argument type mapping
- [ ] Implement multi-module CLI
- [ ] Use auto-discovery for plugin architecture

**Day 5-7:** Practice Project
- [ ] Build CLI with 10+ commands
- [ ] Use domain separation pattern
- [ ] Write Chicago TDD tests

**Resources:**
- [Tutorial 01: Your First CLI](tutorial/01-your-first-cli.md)
- [Tutorial 02: Domain Separation](tutorial/02-domain-separation.md)
- [Tutorial 03: Adding Commands](tutorial/03-adding-commands.md)

---

### Week 2-3: Intermediate Features

**Goal:** Add async, observability, and agent-readiness

**Day 8-10:** Async Operations
- [ ] Learn `#[async_verb]` macro
- [ ] Implement concurrent API calls
- [ ] Add timeout and cancellation

**Day 11-13:** Autonomic Layer
- [ ] Enable introspection API
- [ ] Add effect metadata
- [ ] Generate execution receipts

**Day 14-17:** Observability
- [ ] Add tracing spans
- [ ] Implement metrics
- [ ] Build monitoring dashboard

**Day 18-21:** Practice Project
- [ ] Build async health checker
- [ ] Add autonomic metadata
- [ ] Implement observability

**Resources:**
- [Tutorial 06: Autonomic Features](tutorial/06-autonomic-features.md)
- [Tutorial 07: Async Operations](tutorial/07-async-operations.md)
- [How-To: Production Monitoring](howto/production/monitoring.md)

---

### Week 4-6: Expert Features

**Goal:** Master deterministic execution, distributed agents, semantic systems

**Day 22-25:** Kernel Features
- [ ] Implement deterministic execution
- [ ] Generate cryptographic receipts
- [ ] Build governance ledger

**Day 26-30:** Agent2028 Ecosystem
- [ ] Implement delegation system
- [ ] Build policy engine
- [ ] Create certificate chains

**Day 31-35:** RDF/SPARQL
- [ ] Model CLI as RDF ontology
- [ ] Write SPARQL queries
- [ ] Integrate with MCP

**Day 36-42:** Capstone Project
- [ ] Build distributed agent system
- [ ] Implement full audit trail
- [ ] Deploy to production

**Resources:**
- [ARCHITECTURE_V5_COMPLETE.md](ARCHITECTURE_V5_COMPLETE.md)
- [Agent2028 Documentation](explanation/autonomic/agent2028.md)
- [RDF Integration Guide](rdf-v5-architecture.md)

---

## 5. Feature Deep Dives

### 5.1 Procedural Macro Implementation

**How it works internally:**

```rust
// User writes:
#[verb("deploy", "services")]
fn deploy_service(name: String) -> Result<DeploymentInfo, Error> {
    // ...
}

// Macro generates:
#[distributed_slice(VERBS)]
static DEPLOY_SERVICE: VerbCommand = VerbCommand {
    noun: "services",
    verb: "deploy",
    handler: |args| {
        let name = args.get("name")?;
        Box::pin(deploy_service(name))
    },
    metadata: VerbMetadata {
        help: "Deploy service",
        args: vec![
            ArgSpec {
                name: "name",
                ty: "String",
                required: true,
            }
        ],
    },
};
```

**Type Inference Algorithm:**

```
1. Parse function signature
2. Extract parameter types
3. Map types to clap argument types:
   - String → Arg::value_name("STRING")
   - i32/u32/f64 → Arg::value_name("NUMBER")
   - bool → Arg::action(ArgAction::SetTrue)
   - Option<T> → Arg::required(false)
   - Vec<T> → Arg::num_args(1..)
4. Generate clap Command with inferred args
5. Register in VERBS slice
```

**Compile-time Validation:**

```rust
// Macro validates at compile time:
#[verb("deploy")]
fn invalid(unknown_type: CustomType) -> Result<(), Error> {
    // ❌ Compile error: CustomType doesn't implement FromStr
}
```

---

### 5.2 Auto-Discovery Mechanism

**Linkme Distributed Slices:**

```rust
// In clap-noun-verb crate:
#[distributed_slice]
pub static VERBS: [VerbCommand];

// In user crates:
#[verb]  // ← Macro appends to VERBS slice
fn my_command() -> Result<Output, Error> { }

// At runtime:
fn main() {
    // VERBS slice contains all commands from all crates
    for verb in VERBS {
        println!("Found command: {} {}", verb.noun, verb.verb);
    }
}
```

**Link-time Merging:**

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Crate A     │     │  Crate B     │     │  Crate C     │
│  VERBS: [v1] │     │  VERBS: [v2] │     │  VERBS: [v3] │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       └────────────────────┴────────────────────┘
                            │
                            ▼
                  ┌──────────────────┐
                  │   Final Binary   │
                  │ VERBS: [v1,v2,v3]│
                  └──────────────────┘
```

---

### 5.3 Async Runtime Integration

**Tokio Runtime Lifecycle:**

```rust
// Without async:
fn main() -> Result<(), Error> {
    clap_noun_verb::run()  // Synchronous execution
}

// With async:
#[tokio::main]
async fn main() -> Result<(), Error> {
    clap_noun_verb::run_async().await  // Async execution
}

// Internally:
pub async fn run_async() -> Result<(), Error> {
    let cmd = build_cli();
    let matches = cmd.get_matches();

    // Dispatch to handler
    if is_async_verb(&matches) {
        // Execute async handler
        handler.call_async(matches).await?;
    } else {
        // Bridge to sync handler
        tokio::task::spawn_blocking(|| {
            handler.call_sync(matches)
        }).await??;
    }

    Ok(())
}
```

**Async/Sync Bridging:**

```rust
// Async calling sync (blocking)
#[async_verb]
async fn async_handler() -> Result<(), Error> {
    // Offload blocking work
    let result = tokio::task::spawn_blocking(|| {
        expensive_cpu_work()
    }).await?;

    Ok(())
}

// Sync calling async (requires runtime)
#[verb]
fn sync_handler() -> Result<(), Error> {
    // Create runtime if needed
    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(async {
        async_operation().await
    })?;

    Ok(())
}
```

---

### 5.4 Autonomic Introspection

**Capabilities Discovery:**

```rust
// Implementation of --capabilities
pub fn generate_capabilities() -> CapabilitiesReport {
    let mut capabilities = Vec::new();

    for verb in VERBS.iter() {
        capabilities.push(Capability {
            noun: verb.noun.to_string(),
            verb: verb.verb.to_string(),
            effects: verb.metadata.effects.clone(),
            sensitivity: verb.metadata.sensitivity.clone(),
            arguments: verb.metadata.args.iter().map(|arg| {
                ArgumentSpec {
                    name: arg.name.to_string(),
                    ty: arg.ty.to_string(),
                    required: arg.required,
                    default: arg.default.clone(),
                }
            }).collect(),
        });
    }

    CapabilitiesReport { capabilities }
}
```

**Effect Tracking:**

```rust
// Effect types
pub enum Effect {
    ReadsState,     // Reads system state
    WritesState,    // Modifies state
    NetworkCall,    // External network
    Destructive,    // Irreversible
    Expensive,      // High resource usage
}

// Effect metadata extraction
#[verb(effects = ["writes_state", "network_call"])]
fn deploy() -> Result<(), Error> {
    // Macro extracts effects at compile time
}

// Runtime effect verification
let effects = command.metadata.effects;
for effect in effects {
    match effect {
        Effect::Destructive => {
            // Require confirmation
            if !confirm {
                return Err("Confirmation required".into());
            }
        }
        Effect::Expensive => {
            // Check budget
            if !budget.check(estimated_cost) {
                return Err("Insufficient budget".into());
            }
        }
        _ => {}
    }
}
```

---

### 5.5 Kernel Deterministic Execution

**Receipt Generation Algorithm:**

```rust
pub fn generate_receipt(
    command: &str,
    input: &impl Serialize,
    output: &impl Serialize,
    effects: &[Effect],
) -> Result<Receipt, Error> {
    // 1. Hash input
    let mut hasher = blake3::Hasher::new();
    hasher.update(serde_json::to_vec(input)?.as_slice());
    let input_hash = hasher.finalize();

    // 2. Hash output
    let mut hasher = blake3::Hasher::new();
    hasher.update(serde_json::to_vec(output)?.as_slice());
    let output_hash = hasher.finalize();

    // 3. Create receipt
    let receipt = Receipt {
        id: ReceiptId::new(),
        command: command.to_string(),
        input_hash: input_hash.into(),
        output_hash: output_hash.into(),
        effects: effects.to_vec(),
        timestamp: SystemTime::now(),
        signature: None,  // To be signed
    };

    Ok(receipt)
}
```

**Governance Ledger:**

```rust
pub struct GovernanceLedger {
    entries: Vec<LedgerEntry>,
    index: HashMap<ReceiptId, usize>,
}

impl GovernanceLedger {
    // Append receipt (append-only)
    pub fn append(&mut self, receipt: Receipt) -> Result<(), Error> {
        let entry = LedgerEntry {
            receipt,
            prev_hash: self.latest_hash(),
            timestamp: SystemTime::now(),
        };

        self.index.insert(entry.receipt.id, self.entries.len());
        self.entries.push(entry);

        Ok(())
    }

    // Verify ledger integrity
    pub fn verify(&self) -> Result<bool, Error> {
        for i in 1..self.entries.len() {
            let prev_hash = self.entries[i - 1].hash();
            if self.entries[i].prev_hash != prev_hash {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // Replay execution from receipt
    pub fn replay(&self, receipt_id: &ReceiptId) -> Result<ReplayResult, Error> {
        let entry = self.get(receipt_id)?;

        // Reconstruct execution
        let command = parse_command(&entry.receipt.command)?;
        let input = deserialize_input(&entry.receipt.input_hash)?;

        // Execute command with same input
        let output = execute_command(&command, &input)?;

        // Verify output matches
        let output_hash = hash_output(&output);
        if output_hash != entry.receipt.output_hash {
            return Err("Replay mismatch".into());
        }

        Ok(ReplayResult { output })
    }
}
```

---

### 5.6 Agent2028 Delegation System

**Delegation Token Lifecycle:**

```rust
pub struct DelegationToken {
    pub id: TokenId,
    pub delegator: AgentId,
    pub delegate: AgentId,
    pub capabilities: Vec<Capability>,
    pub temporal: TemporalConstraint,
    pub parent: Option<TokenId>,  // For sub-delegation
    pub depth: u32,               // Delegation chain depth
}

impl DelegationToken {
    // Create root token
    pub fn root(delegator: AgentId, delegate: AgentId) -> Self {
        Self {
            id: TokenId::new(),
            delegator,
            delegate,
            capabilities: vec![],
            temporal: TemporalConstraint::default(),
            parent: None,
            depth: 0,
        }
    }

    // Sub-delegate (create child token)
    pub fn sub_delegate(&self, to: AgentId) -> Result<Self, Error> {
        if self.depth >= MAX_DELEGATION_DEPTH {
            return Err("Max delegation depth exceeded".into());
        }

        Ok(Self {
            id: TokenId::new(),
            delegator: self.delegate,
            delegate: to,
            capabilities: self.capabilities.clone(),  // Inherit capabilities
            temporal: self.temporal.clone(),
            parent: Some(self.id),
            depth: self.depth + 1,
        })
    }

    // Verify delegation chain
    pub fn verify_chain(&self, registry: &DelegationRegistry) -> Result<bool, Error> {
        // Check temporal constraint
        if !self.temporal.is_valid() {
            return Ok(false);
        }

        // Check parent chain
        if let Some(parent_id) = &self.parent {
            let parent = registry.get(parent_id)?;
            if !parent.verify_chain(registry)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
```

**Policy Engine:**

```rust
pub struct PolicyEngine {
    policies: Vec<Policy>,
}

pub enum Decision {
    Allow,
    Deny(String),
    Rewrite(Request),
    Redirect(AgentId),
}

impl PolicyEngine {
    pub fn evaluate(&self, request: &Request) -> Result<Decision, Error> {
        // Evaluate all policies in order
        for policy in &self.policies {
            match policy.evaluate(request)? {
                Decision::Allow => continue,
                decision => return Ok(decision),
            }
        }

        // Default deny
        Ok(Decision::Deny("No policy allows this request".into()))
    }
}

// Example policy
let production_policy = Policy::new("production-deploys")
    .rule(|req| {
        if req.capability.starts_with("deployments:deploy") &&
           req.context.get("environment") == Some("production") {
            // Require multi-party approval
            if req.approvals.len() < 2 {
                Decision::Deny("Requires 2 approvals for production".into())
            } else {
                Decision::Allow
            }
        } else {
            Decision::Allow
        }
    });
```

---

### 5.7 RDF/SPARQL Implementation

**RDF Ontology Structure:**

```turtle
@prefix cnv: <http://clap-noun-verb.org/ontology#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Noun class
cnv:Noun a rdfs:Class ;
    rdfs:label "CLI Noun" ;
    rdfs:comment "A resource category in the CLI" .

# Verb class
cnv:Verb a rdfs:Class ;
    rdfs:label "CLI Verb" ;
    rdfs:comment "An action that can be performed" .

# Command class
cnv:Command a rdfs:Class ;
    rdfs:label "CLI Command" ;
    rdfs:comment "A noun-verb combination" .

# Properties
cnv:hasNoun a rdf:Property ;
    rdfs:domain cnv:Command ;
    rdfs:range cnv:Noun .

cnv:hasVerb a rdf:Property ;
    rdfs:domain cnv:Command ;
    rdfs:range cnv:Verb .

cnv:hasEffect a rdf:Property ;
    rdfs:domain cnv:Command ;
    rdfs:range cnv:Effect .

cnv:sensitivity a rdf:Property ;
    rdfs:domain cnv:Command ;
    rdfs:range xsd:string .

# Effect types
cnv:ReadsState a cnv:Effect .
cnv:WritesState a cnv:Effect .
cnv:NetworkCall a cnv:Effect .
cnv:Destructive a cnv:Effect .
```

**SPARQL Query Examples:**

```rust
use oxigraph::{Store, SparqlQuery};

// Initialize RDF store
let store = Store::new()?;

// Load ontology
store.load_from_reader(
    RdfFormat::Turtle,
    include_str!("ontology.ttl").as_bytes()
)?;

// Query 1: Find all read-only commands
let query = SparqlQuery::parse(r#"
    PREFIX cnv: <http://clap-noun-verb.org/ontology#>

    SELECT ?command ?noun ?verb
    WHERE {
        ?command a cnv:Command ;
                 cnv:hasNoun ?noun ;
                 cnv:hasVerb ?verb ;
                 cnv:hasEffect cnv:ReadsState .

        FILTER NOT EXISTS {
            ?command cnv:hasEffect cnv:WritesState .
        }
    }
"#, None)?;

let results = store.query(query)?;

// Query 2: Find commands by capability
let query = SparqlQuery::parse(r#"
    PREFIX cnv: <http://clap-noun-verb.org/ontology#>

    SELECT ?command
    WHERE {
        ?command cnv:requiresCapability "deployments:deploy" .
    }
"#, None)?;

// Query 3: Discover related commands
let query = SparqlQuery::parse(r#"
    PREFIX cnv: <http://clap-noun-verb.org/ontology#>

    SELECT ?related ?relationship
    WHERE {
        cnv:DeployCommand cnv:hasNoun ?noun .
        ?related cnv:hasNoun ?noun ;
                 cnv:hasVerb ?verb .

        BIND(CONCAT("same noun: ", STR(?noun)) AS ?relationship)
    }
"#, None)?;
```

---

## 6. Integration Patterns

### 6.1 Feature Combinations

#### Pattern 1: Agent-Ready CLI

**Features:** `autonomic` + `async` + `crypto`

**Use case:** AI agents, MAPE-K loops, autonomous systems

```rust
use clap_noun_verb_macros::async_verb;

#[async_verb(
    help = "AI-ready deployment",
    effects = ["writes_state", "network_call"],
    sensitivity = "high"
)]
async fn deploy(
    #[arg(help = "Application name")]
    app: String,
) -> Result<DeploymentReceipt, Box<dyn std::error::Error>> {
    // Execute with async
    let deployment = deploy_async(&app).await?;

    // Generate receipt
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

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["autonomic", "async", "crypto"] }
```

---

#### Pattern 2: Production-Grade CLI

**Features:** `async` + `observability` + `validators` + `autonomic`

**Use case:** Production tools, DevOps, monitoring systems

```rust
use tracing::{info, instrument};

#[instrument]
#[async_verb(
    help = "Production-ready health check",
    effects = ["network_call"],
    sensitivity = "low"
)]
async fn health_check(
    #[arg(validator = validate_url)]
    endpoint: String,
) -> Result<HealthReport, Box<dyn std::error::Error>> {
    info!("Checking health of {}", endpoint);

    let response = reqwest::get(&endpoint).await?;
    let healthy = response.status().is_success();

    info!(healthy, "Health check complete");

    Ok(HealthReport {
        endpoint,
        healthy,
        timestamp: chrono::Utc::now(),
    })
}
```

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["async", "observability", "validators", "autonomic"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

#### Pattern 3: Distributed Agent System

**Features:** `agent2028` + `kernel` + `async` + `crypto`

**Use case:** Multi-agent coordination, distributed execution, Byzantine systems

```rust
use clap_noun_verb::agent2028::{DelegationToken, PolicyEngine};
use clap_noun_verb::kernel::KernelContext;

#[async_verb(
    help = "Distributed execution with delegation",
    effects = ["writes_state"],
    sensitivity = "critical",
    kernel = "deterministic"
)]
async fn distributed_deploy(
    #[arg(help = "Deployment manifest")]
    manifest: String,

    #[arg(help = "Delegation token ID")]
    token_id: String,

    ctx: KernelContext,
) -> Result<DistributedReceipt, Box<dyn std::error::Error>> {
    // Verify delegation
    let token = ctx.delegation_registry().get(&token_id)?;
    token.verify_chain()?;

    // Evaluate policy
    let decision = ctx.policy_engine().evaluate(&request)?;
    if !matches!(decision, Decision::Allow) {
        return Err("Policy denied execution".into());
    }

    // Execute with certificate
    let cert = Certificate::unchecked(invocation)
        .policy_check(ctx.policy_engine())?
        .capability_check(&token)?
        .schema_check(&schema)?;

    // Execute deployment
    let deployment = execute_distributed(&manifest, &cert).await?;

    // Generate receipt
    let receipt = ctx.create_receipt()
        .with_input(&manifest)
        .with_output(&deployment)
        .with_certificate(&cert)
        .sign()?;

    // Append to ledger
    ctx.ledger().append(&receipt)?;

    Ok(DistributedReceipt {
        deployment_id: deployment.id,
        receipt_hash: receipt.hash(),
        certificate_id: cert.id(),
    })
}
```

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["agent2028", "kernel", "async", "crypto"] }
```

---

#### Pattern 4: Semantic CLI

**Features:** `rdf` + `autonomic` + `agent2028`

**Use case:** Knowledge graph integration, ontology-driven CLIs, MCP systems

```rust
use clap_noun_verb::rdf::{RdfStore, SparqlQuery};

#[async_verb(
    help = "Semantic command discovery",
    effects = ["reads_state"],
    sensitivity = "low"
)]
async fn discover_commands(
    #[arg(help = "Effect filter (e.g., reads_state)")]
    effect_filter: Option<String>,
) -> Result<CommandDiscovery, Box<dyn std::error::Error>> {
    let store = RdfStore::global();

    // Build SPARQL query
    let query = if let Some(effect) = effect_filter {
        format!(r#"
            PREFIX cnv: <http://clap-noun-verb.org/ontology#>

            SELECT ?command ?noun ?verb ?description
            WHERE {{
                ?command a cnv:Command ;
                         cnv:hasNoun ?noun ;
                         cnv:hasVerb ?verb ;
                         cnv:description ?description ;
                         cnv:hasEffect cnv:{} .
            }}
        "#, effect)
    } else {
        r#"
            PREFIX cnv: <http://clap-noun-verb.org/ontology#>

            SELECT ?command ?noun ?verb ?description
            WHERE {
                ?command a cnv:Command ;
                         cnv:hasNoun ?noun ;
                         cnv:hasVerb ?verb ;
                         cnv:description ?description .
            }
        "#.to_string()
    };

    let results = store.execute(&SparqlQuery::parse(&query)?)?;

    Ok(CommandDiscovery {
        commands: results.into_iter().map(|row| {
            CommandInfo {
                noun: row.get("noun")?.to_string(),
                verb: row.get("verb")?.to_string(),
                description: row.get("description")?.to_string(),
            }
        }).collect(),
    })
}
```

**Dependencies:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["rdf", "autonomic", "agent2028"] }
oxigraph = "0.5"
```

---

### 6.2 Anti-Patterns to Avoid

#### Anti-Pattern 1: Feature Bloat

**Problem:** Adding all features without understanding cost

```toml
# ❌ BAD: Blindly using "full"
[dependencies]
clap-noun-verb = { version = "5.3", features = ["full"] }
# Result: 55 dependencies, 45s compile, 8.5 MB binary
```

**Solution:** Use only what you need

```toml
# ✅ GOOD: Minimal + specific features
[dependencies]
clap-noun-verb = { version = "5.3", features = ["async", "validators"] }
# Result: 15 dependencies, 12s compile, 3.4 MB binary
```

---

#### Anti-Pattern 2: Mixing Async and Sync Incorrectly

**Problem:** Blocking async runtime with sync code

```rust
// ❌ BAD: Blocking in async
#[async_verb]
async fn bad_async() -> Result<(), Error> {
    // This blocks the entire tokio runtime!
    std::thread::sleep(Duration::from_secs(10));
    Ok(())
}
```

**Solution:** Use spawn_blocking for CPU-bound work

```rust
// ✅ GOOD: Offload blocking work
#[async_verb]
async fn good_async() -> Result<(), Error> {
    tokio::task::spawn_blocking(|| {
        // CPU-intensive work here
        expensive_computation()
    }).await?;
    Ok(())
}
```

---

#### Anti-Pattern 3: Over-Engineering with RDF

**Problem:** Using RDF for simple CLIs

```rust
// ❌ BAD: RDF for 3 commands
[dependencies]
clap-noun-verb = { version = "5.3", features = ["rdf"] }

#[verb]  // Simple command doesn't need RDF
fn simple_command() -> Result<(), Error> { }
```

**Solution:** Use RDF only for semantic requirements

```rust
// ✅ GOOD: RDF for complex ontology
[dependencies]
clap-noun-verb = { version = "5.3", features = ["rdf"] }

// When you have 100+ commands and need:
// - Semantic discovery
// - Relationship modeling
// - SPARQL queries
```

---

## 7. Architecture Patterns

### 7.1 Layered Architecture

**Domain Separation Pattern:**

```
┌─────────────────────────────────────────────────────────┐
│                    CLI LAYER (Thin)                      │
│  - Argument parsing                                      │
│  - Validation                                            │
│  - JSON serialization                                    │
│  - Introspection                                         │
└───────────────────────────┬─────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│              INTEGRATION LAYER (Glue)                    │
│  - Type conversions                                      │
│  - Error mapping                                         │
│  - Configuration loading                                 │
└───────────────────────────┬─────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│              DOMAIN LAYER (Pure Logic)                   │
│  - Business rules                                        │
│  - Core algorithms                                       │
│  - No CLI dependencies                                   │
│  - 100% testable                                         │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**

```rust
// domain/deployments.rs (Pure business logic)
pub fn deploy(app: &str, region: &str) -> Result<Deployment, DomainError> {
    // Pure logic, no CLI dependencies
    validate_app_name(app)?;
    validate_region(region)?;

    let deployment = Deployment {
        id: generate_id(),
        app: app.to_string(),
        region: region.to_string(),
        status: DeploymentStatus::InProgress,
    };

    Ok(deployment)
}

// integration/deployments.rs (Glue)
use crate::domain;

pub fn deploy_from_cli(
    app: String,
    region: String,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    // Type conversion
    let deployment = domain::deployments::deploy(&app, &region)
        .map_err(|e| format!("Deployment failed: {}", e))?;

    // Convert to CLI result type
    Ok(DeploymentResult {
        deployment_id: deployment.id,
        status: deployment.status.to_string(),
    })
}

// commands/deployments.rs (CLI layer)
#[verb("deploy", "deployments")]
fn deploy(
    #[arg(help = "Application name")]
    app: String,

    #[arg(help = "AWS region")]
    region: String,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    // Delegate to integration layer
    crate::integration::deployments::deploy_from_cli(app, region)
}
```

**Benefits:**
- Domain logic is CLI-agnostic
- Easy to test (pure functions)
- Can reuse domain in other contexts (web API, library)
- Clear separation of concerns

---

### 7.2 Plugin Architecture

**Dynamic Command Loading:**

```rust
// plugin_api/mod.rs
pub trait CommandPlugin: Send + Sync {
    fn noun(&self) -> &str;
    fn verb(&self) -> &str;
    fn execute(&self, args: PluginArgs) -> Result<PluginOutput, PluginError>;
}

// plugins/deployment_plugin.rs
pub struct DeploymentPlugin;

impl CommandPlugin for DeploymentPlugin {
    fn noun(&self) -> &str { "deployments" }
    fn verb(&self) -> &str { "deploy" }

    fn execute(&self, args: PluginArgs) -> Result<PluginOutput, PluginError> {
        let app = args.get("app")?;
        let region = args.get("region")?;

        // Execute deployment
        let result = crate::domain::deployments::deploy(app, region)?;

        Ok(PluginOutput::from_json(serde_json::to_value(&result)?))
    }
}

// Register plugin
#[verb("deploy", "deployments")]
fn deploy(args: PluginArgs) -> Result<PluginOutput, PluginError> {
    let plugin = DeploymentPlugin;
    plugin.execute(args)
}
```

---

### 7.3 Event-Driven Architecture

**Command as Events:**

```rust
use tokio::sync::mpsc;

pub enum CommandEvent {
    DeployStarted { app: String, region: String },
    DeployCompleted { deployment_id: String },
    DeployFailed { error: String },
}

#[async_verb]
async fn deploy(
    app: String,
    region: String,
    event_tx: mpsc::Sender<CommandEvent>,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    // Emit start event
    event_tx.send(CommandEvent::DeployStarted {
        app: app.clone(),
        region: region.clone(),
    }).await?;

    // Execute deployment
    match crate::domain::deployments::deploy(&app, &region) {
        Ok(deployment) => {
            // Emit completion event
            event_tx.send(CommandEvent::DeployCompleted {
                deployment_id: deployment.id.clone(),
            }).await?;

            Ok(DeploymentResult { deployment_id: deployment.id })
        }
        Err(e) => {
            // Emit failure event
            event_tx.send(CommandEvent::DeployFailed {
                error: e.to_string(),
            }).await?;

            Err(e.into())
        }
    }
}
```

---

## 8. Performance Tuning Guide

### 8.1 Compile Time Optimization

**Problem:** Slow compile times with many features

**Solutions:**

1. **Minimize Features:**

```toml
# Instead of:
features = ["full"]  # 45s compile

# Use:
features = ["async", "autonomic"]  # 15s compile
```

2. **Use Cargo Workspaces:**

```toml
[workspace]
members = [
    "cli",       # Thin CLI layer
    "domain",    # Pure business logic (fast recompile)
    "integration"  # Glue code
]
```

3. **Incremental Compilation:**

```bash
# Enable incremental compilation
export CARGO_INCREMENTAL=1

# Parallel codegen units
export CARGO_BUILD_JOBS=8
```

4. **Caching with sccache:**

```bash
# Install sccache
cargo install sccache

# Configure
export RUSTC_WRAPPER=sccache

# Check stats
sccache --show-stats
```

**Benchmark:**

| Strategy | First Compile | Incremental | Clean |
|----------|--------------|-------------|-------|
| No optimization | 45s | 20s | 45s |
| Minimal features | 15s | 5s | 15s |
| Workspaces | 30s | 3s | 30s |
| sccache | 45s | 1s | 5s |

---

### 8.2 Runtime Performance Optimization

**Problem:** Slow command execution

**Solutions:**

1. **Hot-Path Optimization:**

```rust
use once_cell::sync::Lazy;

// ❌ BAD: Re-parse regex every time
#[verb]
fn validate(input: String) -> Result<(), Error> {
    let regex = Regex::new(r"^\d+$")?;  // Expensive!
    if !regex.is_match(&input) {
        return Err("Invalid input".into());
    }
    Ok(())
}

// ✅ GOOD: Parse once, reuse forever
static REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+$").unwrap()
});

#[verb]
fn validate(input: String) -> Result<(), Error> {
    if !REGEX.is_match(&input) {
        return Err("Invalid input".into());
    }
    Ok(())
}
```

2. **Caching:**

```rust
use lru::LruCache;
use parking_lot::Mutex;

static CACHE: Lazy<Mutex<LruCache<String, ApiResponse>>> = Lazy::new(|| {
    Mutex::new(LruCache::new(100))
});

#[async_verb]
async fn fetch_cached(url: String) -> Result<ApiResponse, Error> {
    // Check cache
    if let Some(cached) = CACHE.lock().get(&url) {
        return Ok(cached.clone());
    }

    // Fetch from API
    let response = reqwest::get(&url).await?.json().await?;

    // Cache result
    CACHE.lock().put(url, response.clone());

    Ok(response)
}
```

3. **Async Concurrency:**

```rust
// ❌ BAD: Sequential (slow)
#[async_verb]
async fn fetch_all_sequential(urls: Vec<String>) -> Result<Vec<Data>, Error> {
    let mut results = Vec::new();
    for url in urls {
        results.push(reqwest::get(&url).await?.json().await?);
    }
    Ok(results)
}

// ✅ GOOD: Concurrent (fast)
#[async_verb]
async fn fetch_all_concurrent(urls: Vec<String>) -> Result<Vec<Data>, Error> {
    let futures: Vec<_> = urls.iter()
        .map(|url| reqwest::get(url))
        .collect();

    let responses = futures::future::try_join_all(futures).await?;

    let futures: Vec<_> = responses.into_iter()
        .map(|r| r.json())
        .collect();

    futures::future::try_join_all(futures).await
}
```

**Benchmark:**

| Strategy | Sequential | Concurrent | Speedup |
|----------|-----------|-----------|---------|
| 10 API calls | 5000ms | 500ms | 10x |
| 100 API calls | 50000ms | 2000ms | 25x |

---

### 8.3 Memory Optimization

**Problem:** High memory usage

**Solutions:**

1. **Stream Processing:**

```rust
// ❌ BAD: Load entire file into memory
#[verb]
fn process_file(path: String) -> Result<Stats, Error> {
    let content = std::fs::read_to_string(&path)?;  // Loads all!
    let lines: Vec<&str> = content.lines().collect();
    Ok(Stats { line_count: lines.len() })
}

// ✅ GOOD: Stream line-by-line
#[async_verb]
async fn process_file_stream(path: String) -> Result<Stats, Error> {
    let file = tokio::fs::File::open(&path).await?;
    let reader = tokio::io::BufReader::new(file);

    let mut lines = reader.lines();
    let mut count = 0;

    while let Some(_line) = lines.next_line().await? {
        count += 1;
    }

    Ok(Stats { line_count: count })
}
```

2. **Avoid Cloning:**

```rust
// ❌ BAD: Unnecessary clones
#[verb]
fn process(data: Vec<String>) -> Result<ProcessedData, Error> {
    let copy1 = data.clone();  // Clone 1
    let copy2 = data.clone();  // Clone 2

    Ok(ProcessedData {
        original: data,  // Clone 3
        processed: copy1,
        cached: copy2,
    })
}

// ✅ GOOD: Use references
#[verb]
fn process(data: Vec<String>) -> Result<ProcessedData, Error> {
    Ok(ProcessedData {
        original: data,  // Moved (no clone)
        processed: process_internal(&data),  // Borrow
        cached: cache_internal(&data),  // Borrow
    })
}
```

---

### 8.4 Binary Size Optimization

**Problem:** Large binary size

**Solutions:**

1. **Strip Debug Symbols:**

```toml
[profile.release]
strip = true  # Remove debug symbols
opt-level = "z"  # Optimize for size
lto = true  # Link-time optimization
codegen-units = 1  # Better optimization
```

2. **Minimize Features:**

```toml
# Minimal build
[dependencies]
clap-noun-verb = { version = "5.3", default-features = false }
```

3. **Use `cargo-bloat`:**

```bash
# Install
cargo install cargo-bloat

# Analyze binary size
cargo bloat --release -n 20

# Output:
#  File  .text     Size Crate
#  0.5%   2.1%  34.2KiB clap
#  0.4%   1.7%  27.8KiB serde_json
#  0.3%   1.2%  19.5KiB regex
```

**Benchmark:**

| Configuration | Binary Size |
|--------------|------------|
| Debug build | 45 MB |
| Release (default) | 8.5 MB |
| Release + strip | 4.2 MB |
| Release + strip + opt-level=z | 2.8 MB |
| Minimal features | 2.1 MB |

---

## 9. Safety Considerations

### 9.1 Type Safety

**Compile-Time Guarantees:**

```rust
// ✅ Type system prevents invalid states
pub struct Unchecked;
pub struct PolicyChecked;
pub struct CapabilityChecked;

pub struct Certificate<S> {
    invocation: Invocation,
    _state: PhantomData<S>,
}

impl Certificate<Unchecked> {
    pub fn policy_check(self, engine: &PolicyEngine)
        -> Result<Certificate<PolicyChecked>, Error> {
        // Can only transition from Unchecked to PolicyChecked
        engine.evaluate(&self.invocation)?;
        Ok(Certificate {
            invocation: self.invocation,
            _state: PhantomData,
        })
    }
}

impl Certificate<PolicyChecked> {
    pub fn capability_check(self, token: &DelegationToken)
        -> Result<Certificate<CapabilityChecked>, Error> {
        // Can only transition from PolicyChecked to CapabilityChecked
        token.verify_capability(&self.invocation.capability)?;
        Ok(Certificate {
            invocation: self.invocation,
            _state: PhantomData,
        })
    }
}

// ❌ Compile error: Can't skip policy check
let cert: Certificate<Unchecked> = /* ... */;
cert.capability_check(token)?;  // ERROR: wrong state
```

---

### 9.2 Error Handling

**Never Panic in Production:**

```rust
// ❌ BAD: Panics on error
#[verb]
fn bad_handler(input: String) -> Result<Output, Error> {
    let parsed = input.parse::<i32>().unwrap();  // PANIC!
    Ok(Output { value: parsed })
}

// ✅ GOOD: Proper error propagation
#[verb]
fn good_handler(input: String) -> Result<Output, Error> {
    let parsed = input.parse::<i32>()
        .map_err(|e| format!("Invalid input: {}", e))?;
    Ok(Output { value: parsed })
}
```

**Custom Error Types:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
```

---

### 9.3 Security Considerations

**Input Validation:**

```rust
// ❌ BAD: No validation
#[verb]
fn execute_command(cmd: String) -> Result<(), Error> {
    std::process::Command::new(&cmd).spawn()?;  // Shell injection!
    Ok(())
}

// ✅ GOOD: Whitelist validation
#[verb]
fn execute_command(cmd: String) -> Result<(), Error> {
    let allowed = ["ls", "cat", "echo"];
    if !allowed.contains(&cmd.as_str()) {
        return Err("Command not allowed".into());
    }

    std::process::Command::new(&cmd).spawn()?;
    Ok(())
}
```

**Secret Management:**

```rust
// ❌ BAD: Secrets in logs
#[verb]
fn connect(password: String) -> Result<(), Error> {
    info!("Connecting with password: {}", password);  // LEAKED!
    Ok(())
}

// ✅ GOOD: Redacted logging
#[verb]
fn connect(password: String) -> Result<(), Error> {
    info!("Connecting with password: [REDACTED]");
    // Use password internally
    Ok(())
}
```

---

## 10. Troubleshooting Guide

### 10.1 Common Issues

#### Issue 1: Command Not Found

**Symptom:**
```bash
$ myapp users list
error: The subcommand 'users' wasn't recognized
```

**Causes:**
1. Command not registered in VERBS slice
2. Macro not imported
3. Module not included in main.rs

**Solution:**

```rust
// Ensure module is included
mod commands;

// Ensure command uses #[verb]
#[verb("list", "users")]
fn list_users() -> Result<UserList, Error> { }

// Ensure macro is in scope
use clap_noun_verb_macros::verb;
```

---

#### Issue 2: Async Runtime Not Found

**Symptom:**
```
error: no async runtime found
```

**Cause:** Using `#[async_verb]` without tokio runtime

**Solution:**

```rust
// Add tokio main
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    clap_noun_verb::run_async().await
}
```

---

#### Issue 3: Type Inference Failed

**Symptom:**
```
error: cannot infer type for `T`
```

**Cause:** Complex generic type in function signature

**Solution:**

```rust
// ❌ BAD: Complex generic
#[verb]
fn complex<T: Serialize>(data: T) -> Result<Output, Error> { }

// ✅ GOOD: Concrete type
#[verb]
fn simple(data: String) -> Result<Output, Error> { }
```

---

#### Issue 4: Feature Not Available

**Symptom:**
```
error: cannot find module `autonomic`
```

**Cause:** Feature not enabled in Cargo.toml

**Solution:**

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["autonomic"] }
```

---

### 10.2 Performance Debugging

**Use `tracing` for profiling:**

```rust
use tracing::{info_span, instrument};

#[instrument]
#[async_verb]
async fn slow_command() -> Result<(), Error> {
    {
        let _span = info_span!("database_query").entered();
        // Database work
    }

    {
        let _span = info_span!("api_call").entered();
        // API call
    }

    Ok(())
}

// Output:
// database_query: 500ms
// api_call: 200ms
// Total: 700ms
```

**Use `cargo-flamegraph`:**

```bash
# Install
cargo install flamegraph

# Profile
cargo flamegraph --bin myapp -- users list

# Open flamegraph.svg in browser
```

---

## 11. Future Directions

### 11.1 Roadmap (2026-2027)

**Q1 2026:**
- [ ] WASM compilation support
- [ ] Distributed execution primitives
- [ ] Enhanced RDF ontology

**Q2 2026:**
- [ ] Formal verification with Kani
- [ ] Byzantine fault tolerance
- [ ] Multi-agent consensus

**Q3 2026:**
- [ ] Zero-knowledge proofs for receipts
- [ ] Homomorphic execution
- [ ] Quantum-resistant cryptography

**Q4 2026:**
- [ ] Trillion-agent scalability
- [ ] Self-modifying CLI systems
- [ ] Neuromorphic command routing

---

### 11.2 Experimental Features

**Feature: Hot-Reloading Commands**

```rust
// Reload commands without restart
#[verb(hot_reload = true)]
fn dynamic_command() -> Result<(), Error> {
    // Command can be updated at runtime
}
```

**Feature: Neural Command Routing**

```rust
// AI-powered command suggestion
$ myapp "deploy my app to production"
# AI interprets: deployments deploy --app myapp --env production
```

---

### 11.3 Research Directions

1. **Formal Verification:**
   - Prove command correctness with Kani
   - Verify delegation chains mathematically
   - Ensure receipt integrity cryptographically

2. **Distributed Consensus:**
   - Multi-agent voting on commands
   - Byzantine-resilient execution
   - Federated policy enforcement

3. **Self-Optimization:**
   - Learn from execution patterns
   - Auto-tune performance
   - Adaptive caching strategies

---

## Summary

This guide covered:

✅ **Feature Overview** - 14 advanced features from beginner to expert
✅ **Decision Matrix** - When to use each feature
✅ **Progressive Path** - 6-week learning sequence
✅ **Deep Dives** - Internal implementations
✅ **Integration Patterns** - Combining features effectively
✅ **Architecture** - Layered, plugin, event-driven patterns
✅ **Performance** - Compile-time, runtime, memory optimization
✅ **Safety** - Type safety, error handling, security
✅ **Troubleshooting** - Common issues and solutions
✅ **Future** - Roadmap and research directions

**Next Steps:**
1. Choose features based on your use case
2. Follow progressive mastery path
3. Build production systems
4. Contribute to ecosystem

**Resources:**
- [Tutorial Series](tutorial/README.md)
- [How-To Guides](howto/README.md)
- [Reference Documentation](reference/README.md)
- [Architecture Docs](ARCHITECTURE_V5_COMPLETE.md)

---

**Questions? Issues?**
- GitHub: https://github.com/seanchatmangpt/clap-noun-verb/issues
- Documentation: https://docs.rs/clap-noun-verb

**Happy Building! 🚀**
