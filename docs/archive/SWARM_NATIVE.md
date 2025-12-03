# Swarm-Native CLI Design (2027 Vision)

**clap-noun-verb** now provides a comprehensive swarm-native CLI layer designed for a future where trillions of AI agents use CLIs as their primary action interface.

## Table of Contents

1. [Overview](#overview)
2. [Core Capabilities](#core-capabilities)
3. [Quick Start](#quick-start)
4. [Feature Guide](#feature-guide)
5. [Examples](#examples)
6. [Migration Guide](#migration-guide)

## Overview

When trillions of agents share one CLI ecosystem, traditional CLI design patterns break down. Swarm-native CLIs must address:

- **Protocol Stability**: Capability IDs that survive renames
- **Effect Control**: Declarative safety and resource requirements
- **Resource Contention**: QoS hints and priority classes
- **Global Observability**: Deterministic receipts for every invocation
- **Ontology Drift**: Versioned capabilities and migration paths

clap-noun-verb solves these systematically with the **Autonomic CLI Layer**.

## Core Capabilities

### 1. Machine-First Contracts & Introspection

**Problem**: Help text doesn't scale to trillions of agents.

**Solution**: Every CLI exports a structured ontology with stable IDs:

```rust
use clap_noun_verb::autonomic::*;

// Verbs get stable capability IDs automatically
let verb_meta = VerbMetadata::with_noun_context("deploy", "services", "Deploy a service")
    .with_capability_id(CapabilityId::from_path("services.deploy"));

// Agents bind to IDs, not strings
let capability_id = CapabilityId::from_path("services.deploy");
```

**Key types:**
- `CapabilityId` - Stable, opaque identifiers for nouns/verbs
- `CapabilityVersion` - Track capability versions independently of binary versions
- `CapabilityChangelog` - Machine-readable change records
- `ChangeType` - Classify changes: Addition, Extension, Breaking, Deprecation, Removal

### 2. Effect & Safety Declarations

**Problem**: Uncontrolled verbs become systemic risk at swarm scale.

**Solution**: Declare effects, isolation requirements, and data sensitivity up front:

```rust
use clap_noun_verb::autonomic::*;

CommandMetadata::new()
    .with_effects(
        EffectMetadata::new(EffectType::MutateState)
            .with_sensitivity(Sensitivity::High)
            .with_isolation(IsolationRequirement::Isolated)
            .with_data_sensitivity(DataSensitivityTag::Pii)
            .supports_dry_run()
    )
```

**Effect Types:**
- `ReadOnly` - No mutations
- `MutateState` - Mutates application state
- `MutateConfig` - Mutates configuration
- `MutateOntology` - Mutates schema/types
- `MutateSecurity` - Mutates security settings
- `NetworkAccess` - External network access
- `StorageWrite` - Persistent storage writes
- `Privileged` - Requires elevated permissions

**Isolation Requirements:**
- `Shared` - Can run in shared process
- `Isolated` - Should run in isolated process
- `Sandboxed` - Must run in sandbox
- `Containerized` - Requires containerization

**Data Sensitivity Tags:**
- `Pii` - Personally identifiable information
- `Financial` - Financial records
- `HealthData` - Health information
- `Secrets` - Cryptographic keys
- `Credentials` - Authentication credentials

### 3. Deterministic Receipts & Traceability

**Problem**: Logs are noise without structure and verifiability.

**Solution**: Every invocation produces a structured receipt:

```rust
use clap_noun_verb::autonomic::*;

let receipt = ExecutionReceipt::new("services.deploy")
    .with_duration_ms(450)
    .with_guard(GuardResult::within_budget(450, 5000))
    .with_result_hash(ExecutionReceipt::compute_hash(&result).unwrap())
    .with_correlation_id("saga-12345");

println!("{}", receipt.to_json()?);
```

**Receipt Contents:**
- Invocation ID and correlation ID
- Noun/verb ID and version
- Arguments (normalized, optionally redacted)
- Effect profile and guard results
- Execution outcome and timings
- Result hash for verification

### 4. Swarm-Scale Capability Graph

**Problem**: Agents need to navigate and compose thousands of CLIs as one unified capability surface.

**Solution**: Input/output schemas enable automatic composition:

```rust
use clap_noun_verb::autonomic::*;

// Define what this command consumes and produces
let composition = CompositionMetadata::new(input_schema, output_schema)
    .consumes(Resource::new("configuration", "service-manifest"))
    .produces(Resource::new("deployment", "deployment-record"));

// Check if outputs can pipe to inputs
if cmd1.composition.can_pipe_to(&cmd2.composition) {
    // Safe to compose
}
```

**Composition Features:**
- `InputSchema` / `OutputSchema` - Type schemas for I/O
- `TypeSchema` - Primitive, array, object, reference, union types
- `Resource` - Typed resources consumed/produced
- `EquivalenceClass` - Group equivalent capabilities across CLIs

### 5. Multi-Agent Identity & Tenancy

**Problem**: Trillions of agents means overlapping roles, permissions, and resource classes.

**Solution**: First-class agent and tenant identity:

```rust
use clap_noun_verb::autonomic::*;

let agent = AgentIdentity::new("agent-123", "code-assistant")
    .with_version("1.0.0");

let tenant = TenantIdentity::new("project-456")
    .with_environment("production");

let context = InvocationContext::new(agent, tenant)
    .with_policy(PolicyContext::new("default-policy"))
    .with_qos(QoSHints::new()
        .with_priority(PriorityClass::High)
        .must_succeed());
```

**Key Types:**
- `AgentIdentity` - Who is executing (agent ID, type, version)
- `TenantIdentity` - What context (tenant ID, organization, environment)
- `PolicyContext` - What rules apply (policy ID, enforcement mode)
- `QoSHints` - Priority, deadline, latency category, importance
- `PriorityClass` - BestEffort, Normal, High, Critical

### 6. Agent-Friendly Interaction Modes

**Problem**: Command-line tools were designed for humans typing, not swarms.

**Solution**: Sessions, streaming, and incremental receipts:

```rust
use clap_noun_verb::autonomic::*;

// Create a long-lived session
let session = SessionContext::generate()
    .with_noun_scope("services")
    .with_timeout(3600);

// Emit streaming events
let event = StreamEvent::new(1, StreamEventType::Progress)
    .with_data(serde_json::json!({"percent": 50}))
    .with_session(session.session_id.clone());

// Emit incremental receipts for milestones
let receipt = IncrementalReceipt::new("receipt-001", 1, "Stage 1 complete")
    .with_progress(25)
    .with_session(session.session_id);
```

**Key Types:**
- `SessionContext` - Stateful session with noun scope
- `StreamEvent` - Events emitted over time (Started, Progress, PartialResult, Log, Warning, Error, Completed, Cancelled)
- `IncrementalReceipt` - Milestone receipts for long operations
- `SessionManager` - Track and manage active sessions

### 7. Ontology-Aware Evolution

**Problem**: CLIs must evolve without breaking orchestrators.

**Solution**: Built-in patterns for safe evolution:

```rust
use clap_noun_verb::autonomic::*;

// Mark capabilities as deprecated
let deprecation = DeprecationInfo::new("2025-01-01")
    .will_be_removed("2026-01-01")
    .replaced_by(CapabilityId::from_path("new.verb"))
    .with_migration_guide("Use new.verb with --flag");

let version = CapabilityVersion::new("1.0.0", capability_id)
    .deprecated(deprecation);

// Track changes in a changelog
let changelog = CapabilityChangelog::new("3.0.0")
    .add_change(
        CapabilityChange::new(
            capability_id,
            ChangeType::Breaking,
            "3.0.0",
            "Changed argument format"
        )
    );
```

**Change Types:**
- `Addition` - New capability (safe)
- `Extension` - Extended with optional fields (safe)
- `Breaking` - Breaking change (requires migration)
- `Deprecation` - Deprecated but functional
- `Removal` - Capability removed

### 8. Instrumentation & Governance Hooks

**Problem**: Operators must govern trillions of invocations without touching every CLI.

**Solution**: Pluggable policy engines:

```rust
use clap_noun_verb::autonomic::*;

// Define policies declaratively
let policy_engine = RuleBasedPolicyEngine::new("production-policy")
    .add_rule(
        PolicyRule::new("deny-privileged", "Deny privileged operations")
            .with_condition(PolicyCondition::EffectType {
                effect: "privileged".to_string(),
            })
            .with_action(PolicyAction::Deny {
                reason: "Privileged operations not allowed".to_string(),
            })
    );

// Evaluate policies before execution
let request = PolicyRequest::new(context, "services", "restart", args, effects);
let result = policy_engine.evaluate(&request)?;

match result.decision {
    PolicyDecision::Allow => { /* execute */ }
    PolicyDecision::Deny { reason, .. } => { /* reject */ }
    PolicyDecision::Rewrite { new_args } => { /* transform and execute */ }
    PolicyDecision::Redirect { noun, verb, args } => { /* route to alternative */ }
}
```

**Policy Features:**
- `PolicyEngine` trait - Pluggable policy evaluation
- `PolicyRule` - Declarative rule definition
- `PolicyCondition` - Match on effect type, sensitivity, agent type, tenant, command pattern
- `PolicyAction` - Allow, Deny, RequireApproval
- `PolicyDecision` - Allow, Deny, Rewrite, Redirect

## Quick Start

### 1. Add Dependencies

```toml
[dependencies]
clap-noun-verb = "3.7.1"
clap-noun-verb-macros = "3.7.1"
```

### 2. Create a Swarm-Native Verb

```rust
use clap_noun_verb::autonomic::*;
use clap_noun_verb::{Result, VerbArgs, VerbCommand};

struct StatusVerb;

impl VerbCommand for StatusVerb {
    fn name(&self) -> &'static str { "status" }
    fn about(&self) -> &'static str { "Show service status" }

    fn run(&self, _args: &VerbArgs) -> Result<()> {
        // Your logic here
        Ok(())
    }
}

impl AutonomicVerbCommand for StatusVerb {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::new()
            .with_effects(
                EffectMetadata::new(EffectType::ReadOnly)
                    .with_sensitivity(Sensitivity::Low)
            )
            .with_guards(GuardConfig::new().with_max_latency_ms(100))
    }
}
```

### 3. Enable Introspection

```rust
use clap_noun_verb::autonomic::*;
use clap_noun_verb::{CommandRegistry, noun};

let registry = CommandRegistry::new()
    .name("myapp")
    .version("1.0.0")
    .register_noun(noun!("services", "Manage services", [StatusVerb]));

let autonomic = AutonomicCli::new(
    registry,
    "1.0.0",
    AppMetadata::new("myapp").with_version("1.0.0")
);

// Export capabilities for agents
let capabilities = autonomic.capabilities();
println!("{}", serde_json::to_string_pretty(&capabilities)?);

// Export full introspection
let introspection = autonomic.introspect();
println!("{}", serde_json::to_string_pretty(&introspection)?);
```

## Feature Guide

### Stable IDs

Agents bind to capability IDs, not human-readable names:

```rust
// Automatic ID generation from path
let id = CapabilityId::from_path("services.deploy");

// Versioned IDs
let versioned_id = CapabilityId::from_path_versioned("services.deploy", "2.0.0");

// Manual IDs (for stability across renames)
let manual_id = CapabilityId::new("cap_a1b2c3d4e5f6");
```

### Effect Declarations

```rust
// Read-only, low-risk operation
EffectMetadata::new(EffectType::ReadOnly)
    .with_sensitivity(Sensitivity::Low)
    .with_isolation(IsolationRequirement::Shared)

// High-risk mutation with data sensitivity
EffectMetadata::new(EffectType::MutateState)
    .with_sensitivity(Sensitivity::High)
    .with_data_sensitivity(DataSensitivityTag::Pii)
    .with_isolation(IsolationRequirement::Sandboxed)
    .supports_dry_run()

// Network access with medium sensitivity
EffectMetadata::new(EffectType::NetworkAccess)
    .with_sensitivity(Sensitivity::Medium)
    .with_required_role("api-consumer")
```

### Composition Schemas

```rust
// Define input schema
let input_schema = InputSchema::new()
    .with_required("service", TypeSchema::string())
    .with_optional("timeout", TypeSchema::number());

// Define output schema
let output_schema = OutputSchema::new(TypeSchema::reference("DeploymentResult"));

// Create composition metadata
let composition = CompositionMetadata::new(input_schema, output_schema)
    .consumes(Resource::new("config", "service-manifest"))
    .produces(Resource::new("deployment", "deployment-record"));
```

### Policy Enforcement

```rust
// Create a policy engine
let engine = RuleBasedPolicyEngine::new("prod-policy")
    .add_rule(
        PolicyRule::new("require-approval-for-high-sensitivity", "High sensitivity ops need approval")
            .with_condition(PolicyCondition::Sensitivity {
                min_level: "high".to_string(),
            })
            .with_action(PolicyAction::RequireApproval {
                approver: "security-team".to_string(),
            })
    );

// Evaluate before execution
let decision = engine.evaluate(&request)?.decision;
```

### Streaming & Sessions

```rust
// Create a session
let mut session = SessionContext::generate()
    .with_noun_scope("deployments")
    .with_timeout(3600);

// Emit streaming events
impl StreamProducer for MyCommand {
    fn emit(&mut self, event: StreamEvent) -> Result<(), std::io::Error> {
        println!("{}", serde_json::to_string(&event)?);
        Ok(())
    }
}

// Emit progress
cmd.emit_progress(1, 25, "Downloading artifacts")?;
cmd.emit_partial_result(2, serde_json::json!({"downloaded": 5}))?;
cmd.emit_log(3, "info", "Deployment started")?;
```

## Examples

See the `examples/` directory:

- `swarm_native_2027.rs` - Comprehensive demonstration of all swarm-native features
- `autonomic_example.rs` - Basic autonomic CLI layer usage

Run examples:

```bash
# Show capabilities
cargo run --example swarm_native_2027 -- --capabilities

# Show introspection
cargo run --example swarm_native_2027 -- --introspect

# Show changelog
cargo run --example swarm_native_2027 -- --changelog

# Execute with agent context
cargo run --example swarm_native_2027 -- --agent-id agent-123 --tenant-id proj-456 services status

# Dry-run with policy evaluation
cargo run --example swarm_native_2027 -- --dry-run services deploy myapp v2.0.0
```

## Migration Guide

### From Basic Autonomic CLI to Swarm-Native

1. **Add Stable IDs:**
   ```rust
   // Before
   VerbMetadata::new("deploy", "Deploy service")

   // After
   VerbMetadata::with_noun_context("deploy", "services", "Deploy service")
       .with_capability_id(CapabilityId::from_path("services.deploy"))
   ```

2. **Extend Effect Metadata:**
   ```rust
   // Before
   EffectMetadata::new(EffectType::MutateState)

   // After
   EffectMetadata::new(EffectType::MutateState)
       .with_isolation(IsolationRequirement::Isolated)
       .with_data_sensitivity(DataSensitivityTag::Pii)
       .supports_dry_run()
   ```

3. **Add Composition Metadata:**
   ```rust
   CommandMetadata::new()
       .with_composition(
           CompositionMetadata::new(input_schema, output_schema)
               .consumes(Resource::new("config", "manifest"))
               .produces(Resource::new("deployment", "record"))
       )
   ```

4. **Wrap with Invocation Context:**
   ```rust
   let context = InvocationContext::new(agent, tenant)
       .with_policy(PolicyContext::new("default"))
       .with_qos(QoSHints::critical());
   ```

5. **Add Policy Evaluation:**
   ```rust
   let engine = RuleBasedPolicyEngine::new("policy");
   let result = engine.evaluate(&PolicyRequest::new(context, noun, verb, args, effects))?;
   ```

## Schema Version

Current schema version: **2.0.0**

Supported features:
- introspect
- capabilities
- effects
- planes
- guards
- receipts
- errors
- stable_ids (new in 2.0.0)
- versioning (new in 2.0.0)
- tenancy (new in 2.0.0)
- policy (new in 2.0.0)
- composition (new in 2.0.0)
- streaming (new in 2.0.0)
- sessions (new in 2.0.0)

## Success Criteria (2027)

By 2027, a mature swarm-native CLI deployment should achieve:

1. **Adoption**: Majority of swarm-facing CLIs export machine-readable contracts, effect profiles, and receipts.

2. **Safety**: No policy-bypassing "mystery verbs" in production—every verb declares its effect class, and orchestrators enforce allow/deny centrally.

3. **Stability**: Capability ontology changes are visible and schedulable—orchestrators plan migrations before deprecations take effect.

4. **Scale**: CLIs handle sustained, automated invocations at agent scale without collapsing under governance, observability, or protocol drift.

---

**License**: MIT OR Apache-2.0

For more information, see the [main README](README.md) and [API documentation](https://docs.rs/clap-noun-verb).
