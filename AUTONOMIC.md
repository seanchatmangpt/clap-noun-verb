# Autonomic CLI Layer

The Autonomic CLI Layer transforms clap-noun-verb into a **machine-grade interface** designed for agents, MAPE-K loops, orchestration systems, and autonomic controllers.

## Overview

Traditional CLIs are primarily human interfaces. The Autonomic CLI Layer makes commands:
- **Introspectable**: Discover capabilities, arguments, and metadata at runtime
- **Analyzable**: Reason about effects, risks, and constraints
- **Observable**: Generate structured receipts for audit and analysis
- **Constrained**: Enforce guards and budgets on execution

## Core Concepts

### 1. Introspection

Commands can describe themselves in machine-readable format:

```bash
# Discover all capabilities
myapp --capabilities

# Introspect all commands
myapp --introspect

# Introspect a specific noun
myapp --introspect-noun services

# Show command graph with dependencies
myapp --graph
```

**Example output (`--capabilities`)**:
```json
{
  "cli_version": "3.8.0",
  "schema_version": "1.0.0",
  "features": ["introspect", "capabilities", "effects", "planes", "guards", "receipts"],
  "app": {
    "name": "myapp",
    "version": "1.0.0",
    "about": "My application"
  }
}
```

### 2. Effect Metadata

Commands declare their effect profile:

```rust
use clap_noun_verb::autonomic::*;

impl AutonomicVerbCommand for StatusVerb {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::new()
            .with_effects(
                EffectMetadata::new(EffectType::ReadOnly)
                    .with_sensitivity(Sensitivity::Low)
            )
    }
}
```

**Effect Types**:
- `ReadOnly` - No mutations
- `MutateState` - Changes runtime state
- `MutateConfig` - Changes configuration
- `MutateOntology` - Changes schema/structure
- `MutateSecurity` - Changes security settings

**Sensitivity Levels**:
- `Low` - Minimal impact
- `Medium` - Moderate impact
- `High` - Significant impact
- `Critical` - Severe impact potential

### 3. Plane Interactions (O/Σ/Q/ΔΣ)

Commands declare how they interact with conceptual planes:

```rust
.with_planes(
    PlaneInteraction::new()
        .observe_read()      // O:read - Read observations
        .ontology_read()     // Σ:read - Read ontology
        .invariants_check()  // Q:check - Check invariants
        .overlays_emit()     // ΔΣ:emit - Emit overlays
)
```

**Planes**:
- **O (Observations)**: Runtime telemetry and monitoring
- **Σ (Ontology)**: Schema and type definitions
- **Q (Invariants)**: Guards and constraints
- **ΔΣ (Overlays)**: Proposed ontology changes

### 4. Guards & Budgets

Commands can declare resource budgets:

```rust
.with_guards(
    GuardConfig::new()
        .with_max_latency_ms(100)
        .with_max_memory_kb(1024)
        .with_max_cpu_ms(50)
)
```

Guards can be enforced at runtime:
```bash
myapp --enforce-guards services status
```

### 5. Execution Receipts

Commands can emit structured execution records:

```rust
let receipt = ExecutionReceipt::new("services status")
    .with_duration_ms(50)
    .with_guard(GuardResult::within_budget(50, 100))
    .with_planes(&plane_interaction);

println!("{}", receipt.to_json()?);
```

**Example receipt**:
```json
{
  "command": "services status",
  "timestamp": "2025-01-16T10:00:00Z",
  "duration_ms": 50,
  "guard": {
    "enforced": true,
    "latency_ms": 50,
    "max_latency_ms": 100,
    "status": "within_budget"
  },
  "planes": {
    "O": ["read"],
    "Σ": ["read"]
  },
  "correlation_id": "uuid-...",
  "success": true
}
```

### 6. Structured Errors

Uniform, machine-readable error format:

```rust
StructuredError::deadline_exceeded(100, 150)
```

**Example error**:
```json
{
  "error": {
    "kind": "DeadlineExceeded",
    "message": "Deadline 100ms exceeded, took 150ms",
    "details": {
      "deadline_ms": 100,
      "actual_ms": 150
    }
  }
}
```

**Error Kinds**:
- `InvalidInput` - Invalid arguments
- `PermissionDenied` - Authorization failure
- `InvariantBreach` - Constraint violation
- `DeadlineExceeded` - Timeout
- `GuardExceeded` - Budget exceeded
- `CommandNotFound` / `VerbNotFound` - Resolution failure
- `ExecutionError` - Runtime error
- `InternalError` - Internal failure

## Usage

### Basic Setup

```rust
use clap_noun_verb::autonomic::*;
use clap_noun_verb::{noun, CommandRegistry, VerbCommand};

// Define verb with autonomic metadata
struct StatusVerb;

impl VerbCommand for StatusVerb {
    fn name(&self) -> &'static str { "status" }
    fn about(&self) -> &'static str { "Show status" }
    fn run(&self, _args: &VerbArgs) -> Result<()> {
        // Implementation
        Ok(())
    }
}

impl AutonomicVerbCommand for StatusVerb {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::new()
            .with_effects(EffectMetadata::new(EffectType::ReadOnly))
            .with_planes(PlaneInteraction::new().observe_read())
            .with_guards(GuardConfig::new().with_max_latency_ms(100))
            .with_output_type("ServiceStatus")
    }
}

// Build registry
let registry = CommandRegistry::new()
    .name("myapp")
    .about("My application")
    .register_noun(noun!("services", "Manage services", [StatusVerb]));

// Create autonomic CLI
let app_metadata = AppMetadata::new("myapp")
    .with_version("1.0.0")
    .with_about("My application");
let autonomic = AutonomicCli::new(registry, "3.8.0", app_metadata);

// Use autonomic features
println!("{}", autonomic.capabilities_json()?);
println!("{}", autonomic.introspect_json()?);
```

### Command Graph

Visualize command dependencies:

```rust
let graph = CommandGraph::new()
    .add_node(
        GraphNode::new("services.status")
            .with_effect("read_only")
            .with_metadata("sensitivity", "low")
    )
    .add_node(
        GraphNode::new("services.restart")
            .with_effect("mutate_state")
    )
    .add_edge(
        GraphEdge::new("services.restart", "services.status", "precondition")
    );

println!("{}", graph.to_json()?);
```

## Integration with MAPE-K Loops

The Autonomic CLI Layer is designed for MAPE-K (Monitor-Analyze-Plan-Execute-Knowledge) loops:

1. **Monitor**: Use introspection to discover available commands
2. **Analyze**: Check effect metadata and guard budgets
3. **Plan**: Build execution plans based on dependencies and preconditions
4. **Execute**: Run commands with deadline enforcement
5. **Knowledge**: Collect receipts for learning and adaptation

Example workflow:
```bash
# 1. Discover capabilities
myapp --introspect > commands.json

# 2. Analyze dependencies
myapp --graph > graph.json

# 3. Execute with monitoring
myapp --autonomic --deadline-ms 200 services status

# 4. Collect receipt
myapp --receipt-only services status > receipt.json
```

## Future Milestones

### Milestone 3: Autonomic Mode (Planned)

```bash
# Autonomic invocation with hard deadline
myapp --autonomic services restart --deadline-ms 200

# Receipt-only mode
myapp --receipt-only services status

# Strict guard enforcement
myapp --enforce-guards services restart
```

### Milestone 4: Command Graph Extensions (Planned)

- Precondition declarations via `#[verb(pre = "services status")]`
- Relation types: `precondition`, `may_follow`, `mutates_same_resource`
- Graph filtering: `--graph --filter effects=mutate_ontology`

### Milestone 5: Domain Namespacing (Planned)

```bash
# Group commands by domain
myapp --introspect-domain ops
```

## Schema Versioning

The autonomic layer uses semantic versioning:
- **Schema Version**: `1.0.0` (current)
- **CLI Version**: From your application

All introspection responses include both versions for compatibility checking.

## Examples

See `examples/autonomic_example.rs` for a complete working example.

Run it:
```bash
cargo run --example autonomic_example -- --capabilities
cargo run --example autonomic_example -- --introspect
cargo run --example autonomic_example -- --graph
```

## Architecture

The autonomic layer is organized as:
- `src/autonomic/mod.rs` - Module exports and constants
- `src/autonomic/cli.rs` - CLI integration
- `src/autonomic/introspection.rs` - Introspection data structures
- `src/autonomic/effects.rs` - Effect and sensitivity metadata
- `src/autonomic/planes.rs` - O/Σ/Q/ΔΣ plane definitions
- `src/autonomic/guards.rs` - Budget and constraint declarations
- `src/autonomic/receipts.rs` - Execution receipt generation
- `src/autonomic/errors.rs` - Structured error model

## Design Principles

1. **Machine-First, Human-Compatible**: JSON is primary, human text is layered on top
2. **CLI as Contract**: Commands are introspectable contracts, not scripts
3. **Deterministic & Inspectable**: Same inputs → same outputs within documented guards
4. **Invariants First**: Guards and constraints are part of the command definition
5. **Zero-Cost When Unused**: Autonomic features are opt-in and have minimal overhead

## License

Same as clap-noun-verb: MIT OR Apache-2.0
