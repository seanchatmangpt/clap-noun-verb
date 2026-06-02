# CLAP-NOUN-VERB V26.6.2 COMMAND SURFACE

**Date:** 2026-06-02  
**Version:** 26.6.1 (published as v26.6.2)  

---

## Core CLI Architecture

The clap-noun-verb framework provides **noun-verb command patterns** via declarative macros:

- **`#[noun]` macro** (deprecated no-op) marks domain objects
- **`#[verb]` macro** generates command handlers registered via `linkme` distributed slices
- **`CommandRegistry`** auto-discovers all registered verbs at startup
- **`CommandRouter`** dispatches parsed arguments to handler functions
- **`CliBuilder`** constructs clap `Command` tree from registry

---

## Public Command Projection

### 1. Command Discovery via clap-noun-verb-gen

**Signature:**
```bash
clap-noun-verb-gen --manifest-path <path> --output <dir> [--format json|rust]
```

**Purpose:** Generate CLIs from structured specifications (noun-verb templates)

**Example:**
```bash
clap-noun-verb-gen --manifest-path Cargo.toml --output src/generated --format rust
```

**Output Format:** Rust module exports or JSON spec document

---

### 2. Verb Registration & Discovery

**Macro Signature:**
```rust
#[verb(
    noun = "resources",
    name = "list",
    about = "List all resources"
)]
async fn list_resources(#[arg] format: String) -> Result<HandlerOutput> { }
```

**Generated Behavior:**
- Automatically registers handler in distributed slice
- Supports required/optional arguments via `#[arg]` macro
- Auto-detects `clio::Input`/`clio::Output` types for I/O
- Return type must implement `serde::Serialize` (compile-time validated)

**Example with I/O:**
```rust
#[verb(noun = "config", name = "apply")]
fn apply_config(
    #[arg] input: clio::Input,
    #[arg] output: clio::Output
) -> Result<HandlerOutput<ApplyResult>> { }
```

---

### 3. Handler Input/Output Bridge

**HandlerInput Type:**
```rust
pub struct HandlerInput {
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub stdin: Option<Box<dyn std::io::Read>>,
}
```

**HandlerOutput Type (default JSON):**
```rust
pub struct HandlerOutput<T: Serialize> {
    pub data: T,
    pub exit_code: u32,
    pub metadata: Option<CommandMetadata>,
}
```

**Example:**
```rust
#[verb(noun = "services", name = "status")]
fn status(#[arg] service_id: String) -> Result<HandlerOutput<ServiceStatus>> {
    let status = ServiceStatus { running: true, uptime_ms: 1234 };
    Ok(HandlerOutput::new(status, 0))
}
```

---

### 4. Async Verb Support (async feature)

**Signature:**
```rust
#[verb(noun = "tasks", name = "execute")]
async fn execute_task(#[arg] task_id: String) -> Result<HandlerOutput<TaskResult>> {
    let result = fetch_task(task_id).await?;
    Ok(HandlerOutput::new(result, 0))
}
```

**Runtime:** Runtime must be initialized (tokio recommended)

---

### 5. RDF Composition (rdf feature)

**SPARQL Query Integration:**
```rust
#[verb(noun = "graph", name = "query")]
fn query_graph(
    #[arg] sparql: String,
    #[arg] format: String
) -> Result<HandlerOutput<QueryResult>> {
    // SPARQL queries against RDF store
}
```

**Supported Formats:** ntriples, turtle, jsonld

---

### 6. Autonomic Policy Routing (autonomic feature)

**Suggest-Mode Policies:**
```rust
#[verb(noun = "pipeline", name = "suggest")]
fn suggest_pipeline() -> Result<HandlerOutput<PolicyRecommendation>> {
    // Evaluates compile status, test results, fmt/clippy violations
    // Returns: merge-ready | needs-review | requires-fixes
}
```

**Signals Monitored:**
- Compile status (success/failure)
- Test results (passed/failed)
- Code quality (fmt, clippy violations)
- Benchmark regressions

---

### 7. Code Generation (wizard feature - LLM Integration)

**Signature:**
```rust
#[verb(noun = "code", name = "generate")]
fn generate_code(
    #[arg] spec: String,
    #[arg] language: String
) -> Result<HandlerOutput<GeneratedCode>> {
    // Uses rust-genai multi-provider LLM support
}
```

**Supported Providers:** OpenAI, Anthropic, Ollama (pluggable)

---

### 8. ggen Manufacturing Pipeline

**Command:**
```bash
ggen sync --manifest-path Cargo.toml
```

**Stages:**
1. Spec extraction (ontology → JSON spec)
2. Validation (conformance check)
3. Trait generation (template → Rust code)
4. Documentation (template → Markdown)
5. Test generation (template → Rust tests)

**Output Artifacts:**
- `src/generated/cli_spec.json` — Command specifications
- `src/generated/verbs/{noun}/{verb}.rs` — Generated handlers
- `docs/generated/commands/{noun}/{verb}.md` — Auto-docs
- `tests/generated/{noun}_{verb}_test.rs` — Generated tests
- `receipts/ggen/{timestamp}.jsonld` — Artifact receipt (SHA256 hash chain)

---

## Trait Design Rules

### NounCommand Trait
```rust
pub trait NounCommand: Clone + Send + Sync {
    fn noun_name(&self) -> &'static str;
    fn noun_about(&self) -> &'static str;
    fn execute(&self, input: HandlerInput) -> Result<HandlerOutput>;
}
```

### VerbCommand Trait
```rust
pub trait VerbCommand: Clone + Send + Sync {
    fn verb_name(&self) -> &'static str;
    fn verb_about(&self) -> &'static str;
    fn execute(&self, input: HandlerInput) -> Result<HandlerOutput>;
}
```

**Design Constraints:**
- ✓ All traits are `dyn` compatible
- ✓ No async methods in traits (use `async_verb.rs` module instead)
- ✓ Return type `&'static str` (no allocations)
- ✓ Sync trait methods (async handled separately)

---

## Output Format

### Default: JSON
```json
{
  "data": { "running": true, "uptime_ms": 1234 },
  "exit_code": 0,
  "metadata": {
    "command": "services.status",
    "duration_ms": 45,
    "timestamp": "2026-06-02T12:34:56Z"
  }
}
```

### With Errors
```json
{
  "error": "Service not found",
  "exit_code": 1,
  "code": "SERVICE_NOT_FOUND"
}
```

---

## Argument Binding

### Required Argument
```rust
#[verb]
fn cmd(#[arg] name: String) -> Result<HandlerOutput> { }
```

### Optional Argument
```rust
#[verb]
fn cmd(#[arg] #[arg(required = false)] name: Option<String>) -> Result<HandlerOutput> { }
```

### Multiple Argument Types
```rust
#[verb]
fn cmd(
    #[arg] count: i32,
    #[arg] force: bool,
    #[arg] input: clio::Input,
) -> Result<HandlerOutput> { }
```

### Environment Variable Binding
```rust
#[verb]
fn cmd(#[arg(env = "MY_VAR")] token: String) -> Result<HandlerOutput> { }
```

---

## Error Handling

### NounVerbError Types
```rust
pub enum NounVerbError {
    ArgumentError(String),           // Invalid argument
    CommandNotFound(String),         // Verb not registered
    IOError(std::io::Error),         // File I/O failures
    SerializationError(String),      // JSON serialization
    HandlerError(String),            // Handler execution failure
}
```

### Best Practices
- ✓ Always return `Result<T>` (no `unwrap()`, `expect()`, `panic!()`)
- ✓ Use `map_err()` to convert error types
- ✓ Use `?` operator for early returns
- ✓ Test error paths (AAA pattern: Arrange, Act, Assert)

---

## Logging

### Library Code
```rust
use log::{debug, info, warn, error};

#[verb]
fn cmd() -> Result<HandlerOutput> {
    info!("Command starting");
    debug!("Processing args: {:?}", args);
    warn!("Potential issue");
    Err(NounVerbError::handler_error("Operation failed".to_string()))
}
```

### PROHIBITED in Library
```rust
// ❌ NEVER in src/ — only in bin/, build.rs, tests
println!("Debug output");
eprintln!("Error output");
```

---

## Compilation Guarantees

### Macro Validation (Compile-Time)
- ✓ Return type must be `Serialize`
- ✓ No duplicate verb names within noun
- ✓ Function must be `pub`
- ✓ Complexity checks (prevents massive macros)

### Clippy Denials
- ✗ `unwrap_used`, `expect_used`, `panic`, `unimplemented`, `todo`, `exit`
- ✗ All Clippy warnings are treated as errors (`-D warnings`)

---

## Feature Matrix

| Feature | Default | Async | RDF | Autonomic | Wizard | Full |
|---------|---------|-------|-----|-----------|--------|------|
| Core macros | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Async verbs | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ |
| I/O detection | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| RDF queries | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ |
| ggen mfg. | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Autonomic routing | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ |
| LLM integration | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ |
| Plugins | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |

---

## Quick Start Example

### Define a Noun-Verb Command
```rust
use clap_noun_verb::{verb, HandlerInput, HandlerOutput, NounVerbError, Result};
use serde::Serialize;

#[derive(Serialize)]
pub struct GreetingResult {
    message: String,
}

#[verb(
    noun = "greet",
    name = "user",
    about = "Greet a user by name"
)]
pub fn greet_user(
    #[arg] name: String,
) -> Result<HandlerOutput<GreetingResult>> {
    Ok(HandlerOutput::new(
        GreetingResult {
            message: format!("Hello, {}!", name),
        },
        0,
    ))
}
```

### Use the CLI
```bash
$ myapp greet user --name Alice
{
  "data": { "message": "Hello, Alice!" },
  "exit_code": 0,
  "metadata": { ... }
}
```

---

## Publishing

### Macros Crate First
```bash
cargo make publish-macros
# → Publishes clap-noun-verb-macros to crates.io
```

### Main Crate Second
```bash
cargo make publish
# → Publishes clap-noun-verb to crates.io
```

**Dependency Constraint:** Main crate depends on macros crate, so macros must publish first.

---

## See Also

- **Macro Documentation:** `/Users/sac/clap-noun-verb/clap-noun-verb-macros/src/lib.rs`
- **Core Traits:** `src/noun.rs`, `src/verb.rs`
- **CLI Entry Point:** `src/cli/mod.rs`
- **Router Implementation:** `src/router.rs`
- **Examples:** `examples/` directory
- **Tests:** `tests/` directory (33 passing tests)
