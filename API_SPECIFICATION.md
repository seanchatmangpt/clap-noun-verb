# API Specification: Graph-Universe Core Components

## Overview

This document specifies the public APIs of the graph-universe system's core components. All types, methods, and contracts are formally specified here.

---

## 1. KNHK API: Knowledge Graph Access

### 1.1 Capability Definition

```rust
pub struct Capability {
    pub id: String,                    // "storage.create"
    pub noun: String,                  // "storage"
    pub verb: String,                  // "create"
    pub signature: Signature,          // Input/output types
    pub effects: Vec<Effect>,          // MutateState, ReadOnly, etc.
    pub guards: Vec<Guard>,            // Constraints
    pub deprecated: Option<DeprecationInfo>,  // If deprecated
}

impl Capability {
    /// Check if this capability is available to agent
    pub fn is_available_to(&self, agent: &Agent) -> Result<(), AuthError>;

    /// Get the contract for this capability
    pub fn contract(&self) -> CapabilityContract;

    /// Verify that inputs match signature
    pub fn verify_inputs(&self, args: &[Value]) -> Result<(), TypeError>;

    /// Estimate resource usage (conservative upper bound)
    pub fn estimate_resources(&self) -> ResourceEstimate;
}

pub struct Signature {
    pub inputs: Vec<Parameter>,
    pub output: Type,
}

pub struct Parameter {
    pub name: String,
    pub type_: Type,
    pub required: bool,
    pub default: Option<Value>,
    pub constraints: Vec<Constraint>,
}

pub struct Guard {
    pub guard_type: String,    // "WriteQuota", "TimingBound", "AuthRequired"
    pub limit: String,          // "1MB", "100ns", "AdminRole"
    pub consequence: String,    // "reject", "queue", "error"
}
```

### 1.2 Type System

```rust
pub enum Type {
    Primitive(PrimitiveType),
    Custom(String),
    Generic { base: Box<Type>, params: Vec<Type> },
    Union(Vec<Type>),
}

pub enum PrimitiveType {
    String,
    Bytes,
    Int64,
    Uint64,
    Bool,
    Duration,
    Timestamp,
    AgentId,
    SessionId,
}

impl Type {
    /// Check if value matches this type
    pub fn matches(&self, value: &Value) -> bool;

    /// Get human-readable name
    pub fn display_name(&self) -> String;

    /// Get serialization size (or bound)
    pub fn estimated_size(&self) -> usize;
}
```

### 1.3 Schema Versioning

```rust
pub struct Schema {
    pub version: Version,           // "1.2.3"
    pub capabilities: Vec<Capability>,
    pub types: Vec<Type>,
    pub invariants: Vec<Invariant>,
    pub deprecations: Vec<DeprecationInfo>,
}

impl Schema {
    /// Load schema from KNHK
    pub fn load() -> Result<Self, SchemaError>;

    /// Check if capability exists
    pub fn has_capability(&self, id: &str) -> bool;

    /// Get capability by ID
    pub fn get_capability(&self, id: &str) -> Result<&Capability, NotFoundError>;

    /// Get version history
    pub fn history(&self) -> Vec<(Version, String, Timestamp)>;

    /// Check backwards compatibility
    pub fn is_compatible_with(&self, other: &Schema) -> CompatibilityLevel;
}
```

---

## 2. μ-Kernel API: Deterministic Execution

### 2.1 Session Kernel

```rust
pub struct CapabilityContext {
    pub session_id: SessionId,
    pub agent_id: AgentId,
    pub authority_level: AuthorityLevel,
    pub quota: QuotaBudget,
    pub clock: SystemClock,
    pub telemetry: TelemetryCollector,
}

impl CapabilityContext {
    /// Check quota before operation
    pub fn check_quota(&self, resource: &str, amount: usize) -> Result<(), QuotaError>;

    /// Record that an operation succeeded
    pub fn record_effect(&mut self, effect: Effect);

    /// Record a cryptographic receipt
    pub fn record_receipt(&mut self, result: &impl Hash) -> Result<(), ReceiptError>;

    /// Get deterministic clock for timing
    pub fn now(&self) -> Timestamp;

    /// Escalate authority (with type-state check)
    pub fn elevate_to(&mut self, level: AuthorityLevel) -> Result<(), AuthError>;
}

pub enum AuthorityLevel {
    Unauthenticated,
    Authenticated,
    Admin,
    System,
}

impl AuthorityLevel {
    /// Check if this level permits action
    pub fn can_perform(&self, action: &str) -> bool;

    /// Check if can escalate to next level
    pub fn can_escalate_to(&self, next: AuthorityLevel) -> bool;
}
```

### 2.2 Quota Enforcement

```rust
pub struct QuotaBudget {
    pub cpu_cycles: u64,
    pub memory_bytes: u64,
    pub time_ns: u64,
    pub io_operations: u64,

    pub cpu_used: u64,
    pub memory_used: u64,
    pub time_used: u64,
    pub io_used: u64,
}

impl QuotaBudget {
    /// Reserve resources (pre-check)
    pub fn reserve(&mut self, cpu: u64, memory: u64) -> Result<Reservation, QuotaError>;

    /// Record actual usage
    pub fn record_usage(&mut self, usage: &Usage) -> Result<(), QuotaError>;

    /// Check if within budget
    pub fn is_within_budget(&self) -> bool;

    /// Get remaining budget
    pub fn remaining(&self) -> Usage;
}

pub struct Reservation {
    pub reservation_id: Uuid,
    pub cpu_reserved: u64,
    pub memory_reserved: u64,
}

impl Drop for Reservation {
    fn drop(&mut self) { /* Automatically release on drop */ }
}
```

### 2.3 Deterministic Execution Guarantees

```rust
pub struct DeterministicExecution {
    pub request_hash: Hash,
    pub execution_hash: Hash,
    pub timing_ns: u64,
    pub input_seed: u64,
}

impl DeterministicExecution {
    /// Verify execution was deterministic
    /// Same input_hash & seed → same execution_hash
    pub fn verify(&self, other: &DeterministicExecution) -> Result<(), DeterminismError>;

    /// Replay execution with same inputs
    pub fn replay(&self) -> Result<DeterministicExecution, ReplayError>;

    /// Prove determinism (check matches hash)
    pub fn prove_hash(&self, code_hash: &Hash) -> Result<ProofToken, ProofError>;
}
```

### 2.4 Timing Physics

```rust
pub const CHATMAN_CONSTANT_NS: u64 = 100;  // τ <= 100 nanoseconds

pub struct TimingBound {
    pub operation: String,
    pub max_nanoseconds: u64,
    pub percentile: f64,  // 0.99 = p99
}

pub struct TimingProof {
    pub operation: String,
    pub median_ns: f64,
    pub p99_ns: f64,
    pub max_ns: u64,
    pub violation_count: u64,
    pub violation_rate_ppm: f64,
}

impl TimingProof {
    /// Check if timing bound is met
    pub fn satisfies(&self, bound: &TimingBound) -> bool {
        self.p99_ns <= bound.max_nanoseconds as f64
    }

    /// Get safety margin (how close to bound)
    pub fn safety_margin(&self, bound: &TimingBound) -> f64 {
        (bound.max_nanoseconds as f64 - self.p99_ns) / (bound.max_nanoseconds as f64)
    }
}
```

---

## 3. ggen API: Projection Engine

### 3.1 Ontology Input

```rust
pub struct ProjectionInput {
    pub schema: Schema,
    pub invariants: Vec<Invariant>,
    pub profile: ProjectionProfile,
}

impl ProjectionInput {
    /// Validate that input is complete and consistent
    pub fn validate(&self) -> Result<(), ValidationError>;

    /// Check that all capabilities have timing bounds
    pub fn validate_timing_bounds(&self) -> Result<(), ValidationError>;

    /// Check that invariants don't conflict
    pub fn validate_invariants(&self) -> Result<(), ConflictError>;
}

pub enum ProjectionProfile {
    RustNative {
        async_support: bool,
        test_framework: TestFramework,
        optimization: OptimizationLevel,
    },
    CloudFunctions {
        runtime: CloudRuntime,
        timeout_ms: u32,
    },
    PythonStub {
        min_coverage: f64,
    },
}

pub enum OptimizationLevel {
    Debug,      // No optimization, maximum debugging
    Balanced,   // Normal Rust optimizations
    Aggressive, // Maximum optimization for speed
}
```

### 3.2 Code Generation

```rust
pub struct CodeGenerator {
    profile: ProjectionProfile,
    templates: TemplateEngine,
}

impl CodeGenerator {
    /// Generate Rust implementation for capability
    pub fn generate_capability(
        &self,
        cap: &Capability,
    ) -> Result<GeneratedCode, GenerationError>;

    /// Generate test suite
    pub fn generate_tests(
        &self,
        cap: &Capability,
    ) -> Result<GeneratedTestCode, GenerationError>;

    /// Generate documentation
    pub fn generate_docs(
        &self,
        cap: &Capability,
    ) -> Result<GeneratedDocs, GenerationError>;

    /// Verify generated code compiles
    pub fn verify_compiles(
        &self,
        code: &GeneratedCode,
    ) -> Result<(), CompilationError>;
}

pub struct GeneratedCode {
    pub code: String,
    pub imports: Vec<String>,
    pub file_path: PathBuf,
    pub generated_timestamp: Timestamp,
    pub schema_hash: Hash,
}

impl GeneratedCode {
    /// Add @generated markers
    pub fn mark_generated(&mut self);

    /// Get hash of generated code
    pub fn code_hash(&self) -> Hash;

    /// Verify code matches schema hash
    pub fn verify_matches_schema(&self, schema_hash: &Hash) -> Result<(), MismatchError>;
}
```

### 3.3 Projection Profiles

```rust
pub struct TemplateContext {
    pub capability: Capability,
    pub variables: HashMap<String, String>,
}

pub enum TemplateEngine {
    Handlebars(HandlebarsEngine),
    Askama(AskamaEngine),
}

impl TemplateEngine {
    /// Render template with context
    pub fn render(
        &self,
        template_name: &str,
        context: &TemplateContext,
    ) -> Result<String, TemplateError>;

    /// Register custom template
    pub fn register_template(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<(), TemplateError>;
}
```

---

## 4. nomrg API: Conflict-Free Composition

### 4.1 Overlay Definition

```rust
pub struct Delta {
    pub id: String,
    pub timestamp: Timestamp,
    pub proposed_by: AgentId,
    pub changes: Vec<Change>,
    pub justification: String,
}

pub enum Change {
    AddCapability { cap: Capability },
    RemoveCapability { cap_id: String },
    ModifyCapability { cap_id: String, modifications: Vec<Modification> },
    AddInvariant { inv: Invariant },
    ModifyInvariant { inv_id: String, mods: Vec<Modification> },
}

pub enum Modification {
    AddParameter { param: Parameter },
    RemoveParameter { param_name: String },
    ChangeEffect { new_effect: Effect },
    ChangeGuard { old: Guard, new: Guard },
}

impl Delta {
    /// Verify that ΔΣ is valid (doesn't violate invariants)
    pub fn verify(&self, base: &Schema, invariants: &[Invariant]) -> Result<(), VerificationError>;

    /// Apply to schema
    pub fn apply(&self, schema: &mut Schema) -> Result<(), ApplicationError>;

    /// Compose with another delta
    pub fn compose(&self, other: &Delta) -> Result<Delta, CompositionError>;
}
```

### 4.2 Overlay Algebra

```rust
impl Delta {
    /// Check commutativity: ΔA ⊕ ΔB = ΔB ⊕ ΔA
    pub fn commutes_with(&self, other: &Delta) -> Result<bool, CompositionError>;

    /// Check associativity: (ΔA ⊕ ΔB) ⊕ ΔC = ΔA ⊕ (ΔB ⊕ ΔC)
    pub fn associates_with(
        &self,
        other1: &Delta,
        other2: &Delta,
    ) -> Result<bool, CompositionError>;

    /// Verify composition produces valid schema
    pub fn composition_is_valid(
        &self,
        other: &Delta,
        base: &Schema,
    ) -> Result<Schema, ValidationError>;
}

pub struct CompositionProof {
    pub commutative: bool,
    pub associative: bool,
    pub conflict_free: bool,
    pub proof_token: String,  // Can be verified cryptographically
}

impl CompositionProof {
    /// Verify proof (check against published public key)
    pub fn verify(&self) -> Result<(), ProofError>;
}
```

---

## 5. CTT API: Verification Pipeline

### 5.1 Phase Definition

```rust
pub struct VerificationPhase {
    pub phase_number: u8,
    pub name: String,
    pub description: String,
    pub tests: Vec<TestCase>,
    pub required_passes: u32,
    pub depends_on: Vec<u8>,  // Previous phases
}

pub struct TestCase {
    pub id: String,
    pub name: String,
    pub test_fn: Box<dyn Fn() -> TestResult>,
    pub phase: u8,
    pub timeout_ms: u32,
    pub property_based: bool,  // Uses proptest
    pub deterministic: bool,   // Must be deterministic
}

pub enum TestResult {
    Pass { duration_ns: u64, metrics: HashMap<String, f64> },
    Fail { reason: String, expected: String, actual: String },
    Timeout { timeout_ms: u32 },
    Error { error: String },
}

impl TestResult {
    /// Check if result is passing
    pub fn is_pass(&self) -> bool;

    /// Get timing of execution
    pub fn duration(&self) -> Option<u64>;
}
```

### 5.2 Verification Pipeline Execution

```rust
pub struct VerificationPipeline {
    phases: Vec<VerificationPhase>,
    results: Vec<PhaseResult>,
}

pub struct PhaseResult {
    pub phase: u8,
    pub passed: u32,
    pub failed: u32,
    pub duration_ms: u32,
    pub coverage_percent: f64,
}

impl VerificationPipeline {
    /// Run entire pipeline (all 13 phases)
    pub fn run_all(&mut self) -> Result<PipelineReport, PipelineError>;

    /// Run specific phase
    pub fn run_phase(&mut self, phase: u8) -> Result<PhaseResult, PhaseError>;

    /// Get cumulative results
    pub fn results(&self) -> &[PhaseResult];

    /// Check all phases passed
    pub fn all_passed(&self) -> bool;

    /// Get coverage report
    pub fn coverage_report(&self) -> CoverageReport;
}

pub struct PipelineReport {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub total_duration_ms: u32,
    pub overall_coverage: f64,
    pub phases: Vec<PhaseResult>,
    pub verdict: Verdict,
}

pub enum Verdict {
    AllPass,
    SomeFail { first_failure_phase: u8, reason: String },
    Timeout { phase: u8 },
}
```

---

## 6. clnrm API: Hermetic Testing

### 6.1 Container Management

```rust
pub struct HermeticContainer {
    id: Uuid,
    mocks: Arc<Mutex<MockServices>>,
    quota: Arc<Mutex<QuotaBudget>>,
    clock: Arc<Mutex<DeterministicClock>>,
    spans: Arc<Mutex<Vec<RecordedSpan>>>,
}

impl HermeticContainer {
    /// Create new isolated container
    pub fn new() -> Self;

    /// Set allowed external calls (default: 0)
    pub fn expect_external_calls(&mut self, count: usize);

    /// Get mock service
    pub fn mock_service(&mut self, name: &str) -> &mut MockService;

    /// Verify hermeticity: no external calls made
    pub fn verify_hermetic(&self) -> Result<(), HermeticityError>;

    /// Verify determinism
    pub fn verify_determinism<F>(&self, f: F) -> Result<(), DeterminismError>
    where
        F: Fn() -> Result<String>;
}

pub struct MockService {
    responses: Vec<MockResponse>,
}

impl MockService {
    /// Queue a response (LIFO)
    pub fn expect(&mut self, response: MockResponse);

    /// Get next response
    pub fn next_response(&mut self) -> Option<MockResponse>;

    /// Verify all expected calls were made
    pub fn verify_all_calls_made(&self) -> Result<(), VerificationError>;
}
```

### 6.2 Quota and Clock

```rust
pub struct QuotaBudget {
    pub cpu_cycles: u64,
    pub memory_bytes: u64,
    pub time_ns: u64,
    pub syscalls: u64,
}

impl QuotaBudget {
    /// Enforce quota (fails if exceeded)
    pub fn enforce(&mut self, usage: &Usage) -> Result<(), QuotaError>;

    /// Get remaining budget
    pub fn remaining(&self) -> Usage;
}

pub struct DeterministicClock {
    current_time_ns: u64,
}

impl DeterministicClock {
    /// Advance time by nanoseconds
    pub fn advance(&mut self, ns: u64);

    /// Get current time
    pub fn now(&self) -> u64;

    /// Set absolute time
    pub fn set(&mut self, ns: u64);
}
```

---

## 7. DFLSS API: Autonomous Optimization

### 7.1 Optimization Workflow

```rust
pub struct OptimizationObjective {
    pub id: String,
    pub metric: String,
    pub baseline: f64,
    pub target: f64,
    pub success_criteria: SuccessCriteria,
}

pub struct SuccessCriteria {
    pub min_improvement: f64,      // e.g., 0.20 for 20%
    pub max_regression: f64,        // e.g., 0.05 for 5%
    pub safety_constraint: String,  // e.g., "no_breaking_changes"
}

pub struct DFLSSOptimizer {
    objectives: Vec<OptimizationObjective>,
}

impl DFLSSOptimizer {
    /// Define objective
    pub fn add_objective(&mut self, obj: OptimizationObjective);

    /// Measure baseline from Γ (receipt graph)
    pub fn measure(&self, obj: &OptimizationObjective) -> Result<Measurement>;

    /// Explore candidates
    pub fn explore(&self, measurement: &Measurement) -> Result<Vec<Candidate>>;

    /// Design optimal solution
    pub fn design(&self, candidates: &[Candidate]) -> Result<Design>;

    /// Implement and deploy
    pub fn implement(&self, design: &Design) -> Result<DeploymentStatus>;

    /// Verify improvements
    pub fn verify(
        &self,
        baseline: &Measurement,
        observed: &Measurement,
        criteria: &SuccessCriteria,
    ) -> Result<VerificationResult>;
}
```

### 7.2 Deployment Phases

```rust
pub enum DeploymentPhase {
    Canary { traffic_pct: u8 },
    EarlyAdopters { traffic_pct: u8 },
    Majority { traffic_pct: u8 },
    Full,
}

pub struct DeploymentStatus {
    pub phase: DeploymentPhase,
    pub phase_start_time: Timestamp,
    pub observed_metrics: Measurement,
    pub verdict: Option<DeploymentVerdict>,
}

pub enum DeploymentVerdict {
    Pass { improvement_pct: f64 },
    Fail { reason: String },
    Rollback { reason: String },
}

impl DeploymentStatus {
    /// Check if should proceed to next phase
    pub fn should_proceed(&self) -> bool;

    /// Should rollback
    pub fn should_rollback(&self) -> bool;
}
```

---

## 8. AHI API: Autonomic Governance

### 8.1 Policy Engine

```rust
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
    ledger: Vec<PolicyDecision>,
}

pub struct PolicyRule {
    pub id: String,
    pub condition: String,     // e.g., "error_rate > 50ppm"
    pub action: PolicyAction,
    pub severity: Severity,
}

pub enum PolicyAction {
    Reject,
    Alert,
    Mitigate,
    Escalate,
}

pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct PolicyDecision {
    pub rule_id: String,
    pub timestamp: Timestamp,
    pub decision: String,
    pub reason: String,
}

impl PolicyEngine {
    /// Check if operation is allowed
    pub fn authorize(&self, op: &Operation) -> Result<PolicyDecision, AuthorizationError>;

    /// Apply policy rule
    pub fn apply_rule(&mut self, rule_id: &str, op: &Operation) -> Result<PolicyDecision>;

    /// Get decision history
    pub fn history(&self) -> &[PolicyDecision];
}
```

### 8.2 Receipt Graph Integration

```rust
pub struct ReceiptGraph {
    entries: Vec<Receipt>,
    causal_links: HashMap<Uuid, Uuid>,  // child → parent
}

impl ReceiptGraph {
    /// Query operations matching criteria
    pub fn query(&self, filter: &QueryFilter) -> Vec<Receipt>;

    /// Get causal chain for receipt
    pub fn causal_chain(&self, receipt_id: &Uuid) -> Vec<Receipt>;

    /// Aggregate metrics (percentiles, means)
    pub fn aggregate_metrics(
        &self,
        metric: &str,
        time_window: Duration,
    ) -> Result<AggregatedMetrics>;

    /// Detect anomalies
    pub fn detect_anomalies(&self) -> Vec<Anomaly>;
}

pub struct Anomaly {
    pub metric: String,
    pub baseline: f64,
    pub observed: f64,
    pub deviation_pct: f64,
    pub severity: Severity,
}
```

---

## 9. CNV API: Agent Interface

### 9.1 Command Invocation

```rust
pub struct Command {
    pub noun: String,
    pub verb: String,
    pub args: HashMap<String, Value>,
}

pub struct CommandResult {
    pub status: CommandStatus,
    pub output: Value,
    pub receipt: ExecutionReceipt,
}

pub enum CommandStatus {
    Success,
    Failed { reason: String },
    Timeout { timeout_ms: u32 },
    QuotaExceeded { resource: String, used: u64, limit: u64 },
}

pub struct CNVRuntime {
    context: CapabilityContext,
}

impl CNVRuntime {
    /// Execute command
    pub fn execute(&mut self, cmd: Command) -> Result<CommandResult>;

    /// Get capabilities (for introspection)
    pub fn capabilities(&self) -> Vec<CapabilityInfo>;

    /// Get capability graph
    pub fn capability_graph(&self) -> &Graph;

    /// Check if agent can invoke capability
    pub fn can_invoke(&self, cap_id: &str) -> Result<bool>;
}
```

---

## 10. Type System & Error Handling

### 10.1 Result Types

```rust
pub type ApiResult<T> = Result<T, ApiError>;

pub enum ApiError {
    NotFound { what: String, id: String },
    InvalidInput { field: String, reason: String },
    Authorization { required: String, have: String },
    Timeout { resource: String, timeout_ms: u32 },
    ConflictError { reason: String },
    InternalError { message: String },
}

impl ApiError {
    /// Get HTTP status code
    pub fn http_status(&self) -> u16;

    /// Get user-friendly message
    pub fn user_message(&self) -> String;

    /// Get detailed diagnostic
    pub fn diagnostic(&self) -> String;
}
```

### 10.2 Hash & Proof Types

```rust
pub type Hash = [u8; 32];  // SHA256

pub struct ProofToken {
    pub content: Vec<u8>,
    pub signature: ed25519::Signature,
}

impl ProofToken {
    /// Verify proof (check signature)
    pub fn verify(&self, public_key: &ed25519::PublicKey) -> Result<(), ProofError>;

    /// Get human-readable representation
    pub fn to_string_hex(&self) -> String;
}
```

---

## Summary: API Contract Guarantees

All APIs guarantee:

| Property | Meaning | Enforcement |
|----------|---------|-------------|
| **Determinism** | Same input → same output | All operations are pure functions |
| **Idempotency** | Can retry without side effects | Requests are deduplicated |
| **Atomicity** | All-or-nothing execution | Transactions roll back fully on error |
| **Ordering** | Causal ordering preserved | Receipt chain proves ordering |
| **Immutability** | Results cannot change | Once returned, results are permanent |
| **Timing Bounds** | Operations bounded to τ | Enforced by μ-kernel (<100ns) |
| **Quota Safety** | Resources cannot exceed allocation | Pre-check + post-check enforcement |
| **Auditability** | Everything is logged | Every receipt in Γ (receipt graph) |

---

## Appendix: Common Patterns

### Pattern 1: Check-then-Act

```rust
// Always pre-check before execution
context.check_quota("memory", 1024)?;  // Returns Err if quota exceeded
perform_operation()?;
context.record_usage(Usage { memory: 1024, ..Default::default() })?;
```

### Pattern 2: Hermetic Testing

```rust
let container = HermeticContainer::new();
container.expect_external_calls(0);  // Must be hermetic

// Run code
let result = my_capability(&container)?;

// Verify
container.verify_hermetic()?;
assert_eq!(container.external_call_count(), 0);
```

### Pattern 3: Determinism Verification

```rust
// Verify same input gives same output
container.verify_determinism(|| {
    let result1 = capability(args.clone())?;
    let result2 = capability(args.clone())?;
    assert_eq!(result1.hash(), result2.hash());
    Ok(())
})?;
```

### Pattern 4: DFLSS Optimization

```rust
let mut optimizer = DFLSSOptimizer::new();
optimizer.add_objective(OptimizationObjective {
    metric: "p99_latency",
    baseline: 150.0,
    target: 120.0,
    success_criteria: SuccessCriteria {
        min_improvement: 0.20,
        ..Default::default()
    },
});

let measurement = optimizer.measure(&objective)?;
let candidates = optimizer.explore(&measurement)?;
let design = optimizer.design(&candidates)?;
```

