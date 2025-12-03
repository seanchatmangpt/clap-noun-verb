# CNV 4.0: Autonomic Command Fabric

**Version 4.0.0 - Released 2025**

CNV 4.0 transforms clap-noun-verb from a perfect CLI wrapper into the command fabric for trillions of agents.

## Vision

By 2027, trillions of coding agents will be using the same CLI library. CNV 4.0 ensures:
- Every CLI is a typed, introspectable protocol, not just a binary
- Command behavior is safe, predictable, and negotiable across versions and hosts
- Massive concurrency is supported without losing determinism or observability

## Three Pillars

### Pillar 1: Capability Contracts

Machine-verifiable command guarantees that enable agents to reason about safety and risk.

#### Features
- **Side-effect classification**: Pure, ReadOnlyFS, ReadWriteFS, Network, Subprocess, Environment, Dangerous
- **Resource profiles**: Instant, Fast, Medium, Slow, Cold (runtime and memory bands)
- **Stability guarantees**: Stable, Preview, Experimental, Deprecated, NonDeterministic
- **Safety profiles**: AgentSafe, HumanReviewRequired, InteractiveOnly

#### Example
```rust
use clap_noun_verb::kernel::*;

// Create a pure capability (most restrictive)
let pure_contract = CapabilityContract::pure();
println!("Risk score: {}", pure_contract.risk_score()); // 0
println!("Agent safe: {}", pure_contract.is_agent_safe()); // true

// Create a read-write capability
let rw_contract = CapabilityContract::read_write();
println!("Risk score: {}", rw_contract.risk_score()); // Higher
println!("Agent safe: {}", rw_contract.is_agent_safe()); // false

// Check compatibility
if pure_contract.is_compatible_with(&rw_contract) {
    println!("Pure is compatible with read-write requirements");
}
```

#### Capability Context
```rust
let ctx = CapabilityContext::new(CapabilityContract::read_only());
println!("Can read FS: {}", ctx.can_read_fs());       // true
println!("Can write FS: {}", ctx.can_write_fs());     // false
println!("Can access network: {}", ctx.can_access_network()); // false
```

### Pillar 2: Session Kernel

Long-lived, multiplexed, back-pressured command streams for agent workloads.

#### Features
- **Session abstraction**: Long-lived command contexts with typed state
- **Multiplexed protocol**: Multiple logical streams (stdout, stderr, logs, metrics, control) over one pipe
- **Backpressure**: Cooperative flow control to prevent overwhelming downstream consumers
- **Cancellation**: Graceful command termination with cleanup
- **Session-scoped telemetry**: Per-session metrics and distributed tracing

#### Example
```rust
use clap_noun_verb::kernel::*;

// Create a session
let mut session = SessionBuilder::new()
    .config(SessionConfig::default())
    .profile(TelemetryProfile::default())
    .capability(CapabilityContract::pure())
    .build();

// Yield data frames
let frame = session.yield_data(
    StreamId::Stdout,
    serde_json::json!({"result": "success"}),
)?;

// Yield log frames
let log = session.yield_log("info", "Operation completed", None)?;

// Check metrics
let metrics = session.metrics();
println!("Frames sent: {}", metrics.frames_sent);
println!("Bytes sent: {}", metrics.bytes_sent);

// Handle cancellation
if session.is_cancelled() {
    // Cleanup and exit
}
```

#### Frame Protocol
Each frame contains:
- Session ID (UUID)
- Stream ID (stdout, stderr, logs, metrics, control)
- Sequence number (monotonic within stream)
- Timestamp (milliseconds since epoch)
- Payload (structured data)

### Pillar 3: Version Negotiation

Structured change management for evolving CLIs in a world of trillions of agents.

#### Features
- **Grammar Delta Model**: Structural diffs between grammar versions
- **Change Classification**: Automatic breaking/non-breaking/potentially-breaking detection
- **Version Negotiation**: Agent compatibility protocol
- **Capability-Aware Changes**: Track side-effect and safety changes

#### Example
```rust
use clap_noun_verb::kernel::*;

// Create two grammar versions
let v1 = GrammarModel::new("my-app").with_version("1.0.0");
let v2 = GrammarModel::new("my-app").with_version("2.0.0");

// Compute delta
let delta = GrammarDelta::compute(&v1, &v2)?;
println!("Severity: {:?}", delta.severity);
println!("Breaking changes: {}", delta.has_breaking_changes());

// Print change summary
println!("{}", delta.summary());

// Version negotiation
let mut negotiator = VersionNegotiator::new(v2);
negotiator.add_history("1.0.0".to_string(), v1);

let request = NegotiationRequest {
    known_version: "1.0.0".to_string(),
    required_capabilities: None,
    compatibility_level: CompatibilityLevel::Strict,
};

let response = negotiator.negotiate(&request)?;
if !response.compatible {
    for warning in response.warnings {
        eprintln!("Warning: {}", warning);
    }
}
```

## Test Harness Enhancements

CNV 4.0 extends the test harness with capability and compatibility enforcement:

```rust
use clap_noun_verb::kernel::*;

let harness = TestHarness::new()?;

// Validate capability contracts
let report = harness.validate();
assert!(report.is_valid());

// Get agent-safe commands
let safe_commands = harness.agent_safe_commands();
println!("Agent-safe commands: {}", safe_commands.len());

// Get commands by capability
let read_only = harness.commands_by_capability(CapabilityClass::ReadOnlyFS);

// Generate capability report
let cap_report = harness.capability_report();
println!("Agent-safe percentage: {:.1}%", cap_report.agent_safe_percentage());
println!("Coverage: {:.1}%", cap_report.coverage_percentage());

// Assert no breaking changes
harness.assert_no_breaking_changes(&old_grammar)?;
```

## Grammar Model Extensions

The grammar model now includes capability metadata:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarVerb {
    pub name: String,
    pub noun: String,
    pub help: Option<String>,
    pub arguments: Vec<GrammarArgument>,
    pub deprecated: bool,
    /// CNV 4.0: Capability contract
    pub capability: Option<CapabilityContract>,
    // ... other fields
}
```

## Migration Guide

### From CNV 3.x to 4.0

1. **Add capability contracts to your verbs** (optional but recommended):
```rust
// Before (CNV 3.x)
fn my_verb() -> Result<()> {
    // ...
}

// After (CNV 4.0)
fn my_verb() -> Result<()> {
    // Same implementation, but add metadata:
    // #[capability(ReadOnlyFS, Fast, Stable, AgentSafe)]
}
```

2. **Use session mode for long-running operations** (optional):
```rust
// Traditional single-invocation mode still works
fn my_verb(args: &Args) -> Result<()> {
    // ...
}

// New session mode for agents
fn my_verb_session(session: &mut SessionHandle) -> Result<()> {
    for item in items {
        session.check_cancellation()?;
        let result = process(item);
        session.yield_data(StreamId::Stdout, result)?;
    }
    Ok(())
}
```

3. **Track grammar versions for compatibility checking**:
```rust
// Store grammar snapshots for each release
let grammar = Grammar::extract_with_name("my-app")?;
let snapshot = grammar.to_json()?;
// Save snapshot to version control
```

## Breaking Changes

None. CNV 4.0 is fully backward compatible with CNV 3.x. All new features are opt-in.

## Performance

- Zero overhead for single-invocation mode (traditional CLI usage)
- Session mode adds minimal overhead (~5μs per frame)
- Grammar introspection is computed on-demand
- Version negotiation is O(n) where n is number of changed commands

## Examples

See `examples/cnv4_example.rs` for a comprehensive demonstration of all three pillars.

Run the example:
```bash
cargo run --example cnv4_example
```

## Documentation

Full API documentation is available at [docs.rs/clap-noun-verb](https://docs.rs/clap-noun-verb).

Key modules:
- `clap_noun_verb::kernel::capability` - Capability contracts
- `clap_noun_verb::kernel::session` - Session kernel
- `clap_noun_verb::kernel::version` - Version negotiation
- `clap_noun_verb::kernel::test_harness` - Enhanced test harness

## Roadmap

Future enhancements for CNV 4.x:
- Type-level capability enforcement (compile-time checks)
- Capability introspection commands (`--capabilities`, `--explain`)
- Session streaming protocol (server/client mode)
- Distributed tracing integration
- Schema registry for grammar versions

## License

MIT OR Apache-2.0

## Credits

CNV 4.0: Autonomic Command Fabric designed for the 2027 world of trillions of agents.
