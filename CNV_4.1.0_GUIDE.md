# CNV 4.1.0: Advanced Framework Capabilities

**Version 4.1.0 - Next Generation Capability Framework (2025)**

This guide documents the five major feature areas introduced in CNV 4.1.0:
1. **Type-Level Capability Enforcement** - Compile-time safety guarantees
2. **Capability Introspection** - Runtime capability discovery (`--capabilities`, `--explain`)
3. **Session Streaming Protocol** - Server/client mode for long-lived streams
4. **Distributed Tracing** - Observability across agent deployments
5. **Schema Registry** - Immutable grammar versioning

---

## Part 1: Tutorials

### Tutorial 1: Getting Started with Type-Level Capability Enforcement

Type-level capability enforcement uses Rust's phantom type system to ensure at compile time that verbs can only be called in appropriate capability contexts.

#### 1.1 Understanding the Capability Hierarchy

```rust
use clap_noun_verb::kernel::*;

// Capability markers form a hierarchy
// ReadWriteFS ⊃ ReadOnlyFS ⊃ Pure
//
// This means:
// - A Pure operation can run in any context
// - A ReadOnlyFS operation can run in ReadOnlyFS or ReadWriteFS contexts
// - A ReadWriteFS operation ONLY runs in ReadWriteFS context

// Create a pure computation (no side effects)
let pure_verb = TypedVerb::<Pure, Safe>::new("compute".to_string());

// Create a read-only verb (accesses filesystem)
let readonly_verb = TypedVerb::<ReadOnlyFS, Safe>::new("read_config".to_string());

// Create a write verb (modifies state)
let write_verb = TypedVerb::<ReadWriteFS, Safe>::new("update_config".to_string());
```

#### 1.2 Creating Typed Contexts

```rust
// Create a pure execution context - only pure verbs can run here
let pure_context = TypedContext::<Pure>::new("session-1".to_string());

// Create a read-only context - pure and readonly verbs can run
let readonly_context = TypedContext::<ReadOnlyFS>::new("session-2".to_string());

// Create a write context - all verbs can run
let write_context = TypedContext::<ReadWriteFS>::new("session-3".to_string());
```

#### 1.3 Type-Safe Verb Execution

```rust
// This compiles and runs successfully
let pure_context = TypedContext::<Pure>::new("s1".to_string());
let verb = TypedVerb::<Pure, Safe>::new("count".to_string());
pure_context.execute(&verb, || {
    Ok("Operation successful")
})?;

// This is a COMPILE ERROR - ReadWriteFS verb in Pure context
// let write_verb = TypedVerb::<ReadWriteFS, Safe>::new("delete".to_string());
// pure_context.execute(&write_verb, || Ok("..."))?;  // ❌ Won't compile!
```

#### 1.4 Handling Unsafe (Review-Required) Verbs

Some operations require explicit human approval:

```rust
use clap_noun_verb::kernel::*;

// An operation that requires approval
let unsafe_verb = TypedVerb::<ReadWriteFS, UnsafeMeta>::new("dangerous_delete".to_string());

let context = TypedContext::<ReadWriteFS>::new("session".to_string());

// Create an approval token with 5-minute TTL
let approval = ApprovalToken::new(
    vec!["read-write-fs".to_string()],
    300,  // 300 seconds = 5 minutes
    "User approved deletion of config".to_string(),
);

// Execute only with explicit approval
context.execute_unsafe(&unsafe_verb, approval, || {
    // Dangerous operation runs only with token
    Ok("Deleted successfully")
})?;
```

---

### Tutorial 2: Getting Started with Capability Introspection

Capability introspection allows end users and agents to discover what a CLI can do.

#### 2.1 Building a Capability Registry

```rust
use clap_noun_verb::kernel::*;

let mut registry = CapabilityRegistry::new();

// Register a simple capability
registry.register(CapabilityInfo {
    id: "list".to_string(),
    name: "List Items".to_string(),
    description: "List all available items".to_string(),
    side_effects: vec![SideEffect::ReadOnlyFS],
    resource_profile: ResourceProfile::Fast,
    stability: StabilityGuarantee::Stable,
    safety: SafetyProfile::AgentSafe,
    agent_safe: true,
    requires_approval: vec![],
});

// Register a more complex capability
registry.register(CapabilityInfo {
    id: "delete".to_string(),
    name: "Delete Item".to_string(),
    description: "Permanently delete an item".to_string(),
    side_effects: vec![SideEffect::ReadWriteFS, SideEffect::Dangerous],
    resource_profile: ResourceProfile::Medium,
    stability: StabilityGuarantee::Stable,
    safety: SafetyProfile::HumanReviewRequired,
    agent_safe: false,
    requires_approval: vec!["admin".to_string()],
});
```

#### 2.2 Using the Introspection Handler

```rust
use clap_noun_verb::kernel::*;

let handler = IntrospectionHandler::new(registry);

// Handle `--capabilities` command
let output = handler.list_capabilities();
println!("{} total capabilities", output.total_capabilities);
println!("{} are agent-safe", output.agent_safe_count);

// Handle `--explain <capability>` command
let explanation = handler.explain_capability("delete")?;
println!("Capability: {}", explanation.capability.name);
println!("Risk level: {}/100", explanation.risk_level);
for implication in explanation.implications {
    println!("  - {}", implication);
}
```

#### 2.3 Filtering Capabilities by Properties

```rust
// Find all capabilities that are safe for agents
let agent_safe = registry.agent_safe_capabilities();
println!("Agent-safe: {:?}", agent_safe.iter().map(|c| &c.name).collect::<Vec<_>>());

// Find all capabilities requiring approval
let restricted = registry.capabilities_requiring_approval();
println!("Restricted: {:?}", restricted.iter().map(|c| &c.name).collect::<Vec<_>>());

// Find capabilities with specific side effects
let filesystem_ops = registry.find_by_side_effect(&SideEffect::ReadWriteFS);
println!("Filesystem operations: {}", filesystem_ops.len());
```

---

### Tutorial 3: Getting Started with Session Streaming

Session streaming enables long-lived command execution with multiplexing and backpressure.

#### 3.1 Creating a Streaming Session (Server Side)

```rust
use clap_noun_verb::kernel::*;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;

    rt.block_on(async {
        let handler = ServerStreamingHandler::new();

        // Create a streaming session
        let (session, sink) = handler.create_session(
            "session-abc123".to_string(),
            "process_items".to_string(),
            vec!["--count".to_string(), "1000".to_string()],
            100,  // Buffer up to 100 frames
        ).await?;

        println!("Session started: {}", session.id);

        // Send frames through the sink
        for i in 0..10 {
            let frame = StreamFrame::Data {
                session_id: session.id.clone(),
                sequence: i,
                payload: serde_json::json!({ "item": i, "status": "processed" }),
            };

            sink.send(frame).await?;
        }

        // Finish the session
        let done = StreamFrame::Done {
            session_id: session.id.clone(),
            exit_code: 0,
        };
        sink.send(done).await?;

        Ok(())
    })
}
```

#### 3.2 Consuming Stream on Client Side

```rust
use clap_noun_verb::kernel::*;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;

    rt.block_on(async {
        // Receiver from previous example
        let mut client = ClientStreamingHandler::new(
            "session-abc123".to_string(),
            rx,
        );

        // Consume all frames until completion
        let output = client.consume_all().await?;

        println!("Received {} data frames", output.data.len());
        println!("Exit code: {}", output.exit_code);
        println!("CPU usage: {}μs", output.cpu_us);
    })
}
```

#### 3.3 Handling Backpressure

```rust
// Frame sink has bounded queue - backpressure prevents memory overflow
let (sink, rx) = FrameSink::new(100);  // Max 100 buffered frames

// try_send fails immediately if buffer full (non-blocking)
match sink.try_send(frame) {
    Ok(()) => println!("Frame queued"),
    Err(BackpressureError { message }) => {
        eprintln!("Backpressure: {}", message);
        // Handle by pausing, waiting, or dropping frame
    }
}

// send is async and will wait if buffer is full
sink.send(frame).await?;  // Waits for space, then sends
```

---

### Tutorial 4: Getting Started with Distributed Tracing

Distributed tracing enables observability across agent deployments using W3C standards.

#### 4.1 Creating and Propagating Trace Context

```rust
use clap_noun_verb::kernel::*;

// Create root trace context (entry point to system)
let trace_ctx = TraceContext::new();
println!("Trace ID: {}", trace_ctx.trace_id);
println!("W3C Header: {}", trace_ctx.to_trace_context_header());

// Share with downstream services via HTTP header
let header_value = trace_ctx.to_trace_context_header();
// client.get("http://api.example.com").header("traceparent", header_value)

// When receiving in another service, reconstruct:
let received_ctx = TraceContext::from_trace_context_header(
    "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
)?;
println!("Reconstructed trace: {}", received_ctx.trace_id);

// Create child span for this service
let child_ctx = received_ctx.child_span();
println!("Child span: {}", child_ctx.span_id);
```

#### 4.2 Creating and Recording Spans

```rust
use clap_noun_verb::kernel::*;

let ctx = TraceContext::new();
let mut span = Span::new("database_query".to_string(), &ctx);

// Record attributes
span.add_attribute("db.name".to_string(), SpanAttribute::String("users".to_string()));
span.add_attribute("db.rows_affected".to_string(), SpanAttribute::Number(42.0));

// Record events
span.add_event(SpanEvent {
    name: "connection_opened".to_string(),
    timestamp_ns: current_time_ns(),
    attributes: Default::default(),
});

// Mark completion status
if let Err(e) = perform_query() {
    span.set_error(format!("Query failed: {}", e));
} else {
    span.set_ok();
}

span.end();

let duration_ms = span.duration_ms();
println!("Operation took: {:?}ms", duration_ms);
```

#### 4.3 Using Sampling Strategies

```rust
use clap_noun_verb::kernel::*;
use std::sync::Arc;

// Development: sample everything (AlwaysSampler)
let sampler: Arc<dyn SamplingStrategy> = Arc::new(AlwaysSampler);

// Production: sample 1% of requests (ProbabilisticSampler)
let sampler: Arc<dyn SamplingStrategy> = Arc::new(ProbabilisticSampler { probability: 0.01 });

let exporter = Arc::new(InMemoryExporter::new());
let provider = TracingProvider::new(sampler, exporter);

let ctx = TraceContext::new();
if provider.should_sample(&ctx) {
    // Create and record span
    let span = Span::new("sampled_operation".to_string(), &ctx);
    provider.start_span(span);
}
```

---

### Tutorial 5: Getting Started with Schema Registry

Schema registry provides immutable versioning for grammar evolution.

#### 5.1 Registering Schema Versions

```rust
use clap_noun_verb::kernel::*;

let registry = SchemaRegistry::new();

// Create schema entry for v1.0.0
let schema_v1 = SchemaEntry::new(
    SchemaVersion::new(1, 0, 0),
    r#"{
        "nouns": [
            {"name": "services", "verbs": ["status", "logs"]}
        ]
    }"#.to_string(),
    "alice@example.com".to_string(),
    "Initial schema release".to_string(),
);

registry.register_schema(schema_v1)?;

// Create schema entry for v1.1.0
let schema_v1_1 = SchemaEntry::new(
    SchemaVersion::new(1, 1, 0),
    r#"{
        "nouns": [
            {"name": "services", "verbs": ["status", "logs", "metrics"]}
        ]
    }"#.to_string(),
    "bob@example.com".to_string(),
    "Added metrics verb".to_string(),
);

registry.register_schema(schema_v1_1)?;

// List all registered versions
for version in registry.list_versions() {
    println!("Schema v{}", version.to_string());
}
```

#### 5.2 Checking Compatibility

```rust
let v1_0_0 = SchemaVersion::new(1, 0, 0);
let v1_1_0 = SchemaVersion::new(1, 1, 0);
let v2_0_0 = SchemaVersion::new(2, 0, 0);

// Same major version = compatible
let compat = registry.check_compatibility(&v1_0_0, &v1_1_0);
assert_eq!(compat, Some(CompatibilityType::FullyCompatible));

// Different major = breaking
let compat = registry.check_compatibility(&v1_0_0, &v2_0_0);
assert_eq!(compat, Some(CompatibilityType::Breaking));
```

#### 5.3 Defining Migration Paths

```rust
// Define migration rules
let rule_1_0_to_1_1 = EvolutionRule {
    from_version: SchemaVersion::new(1, 0, 0),
    to_version: SchemaVersion::new(1, 1, 0),
    compatibility: CompatibilityType::FullyCompatible,
    migration_script: None,
    breaking_changes: vec![],
};

registry.add_evolution_rule(rule_1_0_to_1_1)?;

// Get migration path from v1.0.0 to v1.1.0
let path = registry.get_migration_path(
    &SchemaVersion::new(1, 0, 0),
    &SchemaVersion::new(1, 1, 0),
);

for rule in path.unwrap() {
    println!("Migrate: {} -> {}", rule.from_version.to_string(), rule.to_version.to_string());
}
```

---

## Part 2: How-To Guides

### How-To: Restrict a CLI to Agent-Safe Verbs Only

```rust
use clap_noun_verb::kernel::*;

// Get all agent-safe capabilities
let registry = build_capability_registry();  // Your registry builder
let handler = IntrospectionHandler::new(registry);
let output = handler.list_capabilities();

// Filter to agent-safe only
let agent_safe: Vec<_> = output.capabilities
    .iter()
    .filter(|cap| cap.agent_safe)
    .collect();

println!("Agent-safe verbs:");
for cap in agent_safe {
    println!("  {} - {}", cap.name, cap.description);
}
```

### How-To: Enforce Capability Hierarchy in Your CLI

```rust
// Define your capability zones
fn pure_zone() -> TypedContext<Pure> {
    TypedContext::new("zone-compute".to_string())
}

fn readonly_zone() -> TypedContext<ReadOnlyFS> {
    TypedContext::new("zone-read".to_string())
}

fn admin_zone() -> TypedContext<ReadWriteFS> {
    TypedContext::new("zone-admin".to_string())
}

// Callers specify which zone they want
fn run_user_provided_verb(zone: &str, verb_name: &str) -> Result<(), String> {
    match zone {
        "compute" => {
            let zone = pure_zone();
            let verb = TypedVerb::<Pure, Safe>::new(verb_name.to_string());
            zone.execute(&verb, || {
                // User's pure verb runs here
                Ok(())
            })
        }
        "read" => {
            let zone = readonly_zone();
            let verb = TypedVerb::<ReadOnlyFS, Safe>::new(verb_name.to_string());
            zone.execute(&verb, || {
                // User's readonly verb runs here
                Ok(())
            })
        }
        _ => Err("Unknown zone".to_string()),
    }
}
```

### How-To: Monitor Long-Running Commands with Streaming

```rust
use clap_noun_verb::kernel::*;

async fn run_with_monitoring(
    command: String,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let handler = ServerStreamingHandler::new();
    let (session, sink) = handler.create_session(
        "monitoring-session".to_string(),
        command,
        args,
        100,
    ).await?;

    // Spawn monitoring task
    let session_id = session.id.clone();
    let handler_clone = handler.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            if let Ok(Some(session)) = handler_clone.get_session(&session_id).await.ok_or(()) {
                if session.is_cancelled {
                    println!("Command was cancelled");
                    break;
                }
                println!("Progress: {} frames, {} bytes", session.frame_count, session.byte_count);
            }
        }
    });

    Ok(())
}
```

### How-To: Trace API Calls Across Microservices

```rust
use clap_noun_verb::kernel::*;
use std::sync::Arc;

// Setup tracing infrastructure
let sampler = Arc::new(AlwaysSampler);
let exporter = Arc::new(InMemoryExporter::new());
let provider = TracingProvider::new(sampler, exporter);

// In your HTTP handler:
fn handle_request(traceparent: Option<&str>) -> Result<String, String> {
    let ctx = if let Some(header) = traceparent {
        TraceContext::from_trace_context_header(header)?
    } else {
        TraceContext::new()
    };

    // Create span for this operation
    let mut span = Span::new("process_request".to_string(), &ctx);
    span.add_attribute("http.method".to_string(), SpanAttribute::String("GET".to_string()));

    // Do work...
    let result = do_work();

    span.set_ok();
    provider.start_span(span);

    Ok(result)
}
```

### How-To: Verify Grammar Compatibility Before Deployment

```rust
use clap_noun_verb::kernel::*;

fn check_deployment_compatibility(
    current_version: &str,
    new_version: &str,
    registry: &SchemaRegistry,
) -> Result<bool, String> {
    let from = parse_version(current_version)?;
    let to = parse_version(new_version)?;

    match registry.check_compatibility(&from, &to) {
        Some(CompatibilityType::FullyCompatible) => {
            println!("✓ Fully compatible - safe to deploy");
            Ok(true)
        }
        Some(CompatibilityType::RequiresMigration) => {
            println!("⚠ Requires migration - check migration script");
            Ok(false)
        }
        Some(CompatibilityType::Breaking) => {
            println!("✗ Breaking change - coordination required");
            Ok(false)
        }
        _ => Err("Compatibility unknown".to_string()),
    }
}
```

---

## Part 3: Explanations

### Explanation: Why Type-Level Capability Enforcement?

Traditional capability systems work at runtime: you check permissions, then execute. If you check wrong, or forget to check, you get a security bug.

Type-level enforcement moves the check to the compiler:

```rust
// Traditional (runtime check)
fn execute_verb(context: &RuntimeContext, verb: &str) {
    if context.can_write_fs() {  // ← Runtime check (can forget!)
        do_write(verb);
    }
}

// Type-level (compiler enforces)
fn execute_verb<C: CapabilityMarker>(context: &TypedContext<C>, verb: &TypedVerb<C>) {
    // Compiler ensures C matches - no runtime check needed!
}
```

The compiler uses the type parameter `C` to ensure that:
1. Only verbs with compatible capabilities can be called
2. Capability escalation is explicit (must call an upgrade method)
3. The hierarchy is respected (you can't call a ReadWriteFS verb in a Pure context)

This is the same idea as Rust's ownership system: prove properties at compile time rather than checking at runtime.

### Explanation: Why Capability Introspection?

Agents and users need to discover "what can I do?" without reading code.

Introspection answers:
- What verbs are available? (`--capabilities`)
- What does a specific verb do? (`--explain <verb>`)
- What's safe for my agent? (filter by `agent_safe`)
- What do I need approval for? (filter by `requires_approval`)

This enables:
1. **Agent autonomy** - Agents can discover capabilities and make decisions
2. **Human oversight** - Administrators see what needs approval
3. **Policy enforcement** - "Only use agent-safe verbs in this zone"

### Explanation: Why Session Streaming?

Traditional CLI execution: invoke → run → exit → return result

This breaks down at scale:
- For trillion agents, you need batching
- For long-running commands, you need progress reporting
- For concurrent commands, you need multiplexing

Session streaming allows:
1. **Long-lived** - One session = many operations
2. **Multiplexed** - Many logical streams over one pipe (stdout/stderr/logs/metrics)
3. **Backpressured** - Bounded queues prevent overflow
4. **Cancellable** - Stop a long-running operation

### Explanation: Why Distributed Tracing?

With trillion agents, debugging "why did command X fail?" requires:
1. Correlating logs across services
2. Measuring latency at each step
3. Tracking context through async operations
4. Understanding dependencies

W3C Trace Context does this by:
1. Adding headers to propagate context
2. Using trace IDs to correlate logs
3. Using span IDs to track individual operations
4. Using baggage to pass values through the call chain

### Explanation: Why Schema Registry?

Grammar (the set of available verbs/args) evolves. You need to track:
1. **What changed?** (GrammarDelta)
2. **Is it compatible?** (CompatibilityType)
3. **How do agents adapt?** (EvolutionRule + migration_script)

Schema registry with merkle trees provides:
1. **Immutability** - Once registered, version is locked
2. **Integrity** - Hash verification prevents corruption
3. **Auditability** - Full history of changes + authors
4. **Compatibility checking** - Automated compatibility analysis

---

## Part 4: Reference

### Type-Level Capability Enforcement API Reference

```rust
// Capability markers
pub struct Pure;                      // No side effects
pub struct ReadOnlyFS;               // Read filesystem only
pub struct ReadWriteFS;              // Read and write filesystem
pub struct Network;                  // Network access
pub struct Subprocess;               // Can spawn processes
pub struct Environment;              // Can modify environment
pub struct Dangerous;                // Dangerous operations

// Safety markers
pub struct Safe;                     // Agent-safe (no review needed)
pub struct UnsafeMeta;              // Requires human review

// Core types
pub struct TypedVerb<C: CapabilityMarker, S: SafetyMarker> { ... }
pub struct TypedContext<C: CapabilityMarker> { ... }
pub struct ApprovalToken { ... }

// Key methods
impl<C: CapabilityMarker> TypedContext<C> {
    pub fn new(session_id: String) -> Self
    pub fn execute<F, R>(&self, verb: &TypedVerb<C, Safe>, handler: F) -> Result<R, String>
    pub fn execute_unsafe<F, R>(&self, verb: &TypedVerb<C, UnsafeMeta>, token: ApprovalToken, handler: F) -> Result<R, String>
}

impl ApprovalToken {
    pub fn new(capabilities: Vec<String>, ttl_seconds: u64, reason: String) -> Self
    pub fn is_valid(&self) -> bool
}
```

### Capability Introspection API Reference

```rust
pub struct CapabilityInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub side_effects: Vec<SideEffect>,
    pub resource_profile: ResourceProfile,
    pub stability: StabilityGuarantee,
    pub safety: SafetyProfile,
    pub agent_safe: bool,
    pub requires_approval: Vec<String>,
}

pub struct IntrospectionHandler { ... }

impl IntrospectionHandler {
    pub fn new(registry: CapabilityRegistry) -> Self
    pub fn list_capabilities(&self) -> CapabilitiesOutput
    pub fn explain_capability(&self, id: &str) -> Result<ExplanationOutput, String>
}

pub struct CapabilityRegistry { ... }

impl CapabilityRegistry {
    pub fn new() -> Self
    pub fn register(&mut self, capability: CapabilityInfo)
    pub fn get(&self, id: &str) -> Option<Arc<CapabilityInfo>>
    pub fn list_all(&self) -> Vec<Arc<CapabilityInfo>>
    pub fn agent_safe_capabilities(&self) -> Vec<Arc<CapabilityInfo>>
    pub fn find_by_side_effect(&self, effect: &SideEffect) -> Vec<Arc<CapabilityInfo>>
}
```

### Session Streaming API Reference

```rust
pub enum StreamFrame {
    Data { session_id: String, sequence: u64, payload: serde_json::Value },
    Log { session_id: String, level: LogLevel, message: String, timestamp_ns: u64 },
    Metrics { session_id: String, cpu_us: u64, memory_bytes: u64, io_read_bytes: u64, io_write_bytes: u64 },
    Control { session_id: String, action: ControlAction },
    Error { session_id: String, code: u32, message: String },
    Done { session_id: String, exit_code: i32 },
}

pub struct FrameSink { ... }

impl FrameSink {
    pub async fn send(&self, frame: StreamFrame) -> Result<(), BackpressureError>
    pub fn try_send(&self, frame: StreamFrame) -> Result<(), BackpressureError>
    pub async fn pending_frames(&self) -> usize
}

pub struct ServerStreamingHandler { ... }

impl ServerStreamingHandler {
    pub async fn create_session(&self, id: String, command: String, args: Vec<String>, buffer_size: usize) -> Result<(StreamingSession, FrameSink), String>
    pub async fn get_session(&self, id: &str) -> Option<StreamingSession>
    pub async fn cancel_session(&self, id: &str, reason: Option<String>) -> Result<(), String>
    pub async fn list_sessions(&self) -> Vec<StreamingSession>
    pub async fn close_session(&self, id: &str)
}

pub struct ClientStreamingHandler { ... }

impl ClientStreamingHandler {
    pub fn new(session_id: String, rx: tokio::sync::mpsc::Receiver<StreamFrame>) -> Self
    pub async fn next_frame(&mut self) -> Option<StreamFrame>
    pub async fn consume_all(&mut self) -> Result<CollectedOutput, String>
}
```

### Distributed Tracing API Reference

```rust
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub trace_flags: TraceFlags,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    pub fn new() -> Self
    pub fn child_span(&self) -> Self
    pub fn to_trace_context_header(&self) -> String
    pub fn from_trace_context_header(header: &str) -> Result<Self, String>
    pub fn add_baggage(&mut self, key: String, value: String)
}

pub struct Span { ... }

impl Span {
    pub fn new(name: String, context: &TraceContext) -> Self
    pub fn add_attribute(&mut self, key: String, value: SpanAttribute)
    pub fn add_event(&mut self, event: SpanEvent)
    pub fn set_ok(&mut self)
    pub fn set_error(&mut self, message: String)
    pub fn end(&mut self)
    pub fn duration_ms(&self) -> Option<u64>
}

pub trait SamplingStrategy: Send + Sync {
    fn should_sample(&self, context: &TraceContext) -> bool;
}

pub struct TracingProvider { ... }

impl TracingProvider {
    pub fn new(sampler: Arc<dyn SamplingStrategy>, exporter: Arc<dyn SpanExporter>) -> Self
    pub fn should_sample(&self, context: &TraceContext) -> bool
    pub fn start_span(&self, span: Span)
    pub fn end_span(&self, span_id: &str)
    pub fn flush(&self) -> Result<(), String>
}
```

### Schema Registry API Reference

```rust
pub struct SchemaVersion { ... }

impl SchemaVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self
    pub fn to_string(&self) -> String
    pub fn is_compatible_with(&self, other: &Self) -> bool
}

pub struct SchemaEntry { ... }

impl SchemaEntry {
    pub fn new(version: SchemaVersion, schema_json: String, author: String, changelog: String) -> Self
    pub fn verify(&self) -> bool
}

pub struct SchemaRegistry { ... }

impl SchemaRegistry {
    pub fn new() -> Self
    pub fn register_schema(&self, entry: SchemaEntry) -> Result<(), String>
    pub fn get_schema(&self, version: &SchemaVersion) -> Option<SchemaEntry>
    pub fn list_versions(&self) -> Vec<SchemaVersion>
    pub fn add_evolution_rule(&self, rule: EvolutionRule) -> Result<(), String>
    pub fn check_compatibility(&self, from: &SchemaVersion, to: &SchemaVersion) -> Option<CompatibilityType>
    pub fn get_migration_path(&self, from: &SchemaVersion, to: &SchemaVersion) -> Option<Vec<EvolutionRule>>
    pub fn verify_schema_integrity(&self, version: &SchemaVersion) -> bool
    pub fn merkle_root(&self) -> Option<String>
}
```

---

## Summary: When to Use Each Feature

| Feature | Use When | Benefits |
|---------|----------|----------|
| **Type-Level Enforcement** | Building security-critical CLIs | Compile-time safety, zero runtime overhead |
| **Introspection** | Building agent-friendly CLIs | Machines can discover capabilities |
| **Streaming** | Long-running or high-throughput ops | Multiplexing, backpressure, progress reporting |
| **Distributed Tracing** | Multi-service deployments | Observability across agent swarms |
| **Schema Registry** | Versioning for active services | Safe grammar evolution, auditability |

---

## Migration Guide

### From CNV 4.0 to CNV 4.1

No breaking changes. All 4.0 code continues to work. v4.1 features are opt-in:

1. **Add type-level enforcement**: Wrap verbs in `TypedVerb<C, S>`
2. **Add introspection**: Create `CapabilityRegistry`, expose via `--capabilities`
3. **Add streaming**: Wrap long-running ops in `ServerStreamingHandler`
4. **Add tracing**: Use `TraceContext` to track requests
5. **Add schema registry**: Register schema versions as you release

---

## See Also

- **CNV_4.0_RELEASE.md** - Three pillars (Capability Contracts, Session Kernel, Version Negotiation)
- **src/kernel/** - All implementation code with tests
- **examples/** - Working examples for each feature
