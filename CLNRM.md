# clnrm: Cleanroom Hermetic Testing Framework

## What is clnrm?

**clnrm** (cleanroom) is the **hermetic testing framework** for the graph-universe system. It ensures that:

- **Tests are isolated**: No external service calls (database, API, filesystem, network)
- **Tests are deterministic**: Same input → same output every time
- **Tests are fast**: No I/O blocking, no network latency
- **Tests are reproducible**: Failures are always reproducible

A **hermetic test** is one where everything needed for the test is explicitly provided and controlled. There are no hidden dependencies on external services or non-deterministic behavior.

---

## Why Hermetic Testing?

### The Problem with Non-Hermetic Tests

Traditional tests often have hidden dependencies:

```rust
#[test]
fn test_storage_create() {
    // ❌ Non-hermetic: depends on external services
    let db = Database::connect("localhost:5432")?;  // Network call
    let result = storage::create("key", "value", db)?;
    assert!(result.is_ok());
}
```

If the database is down, the test fails (even though the code is correct). If the network is slow, the test is slow. If another test modifies the database, this test might fail.

### The Solution: Hermetic Tests

```rust
#[test]
fn test_storage_create_hermetic() {
    // ✓ Hermetic: all dependencies are injected
    let container = HermeticContainer::new();
    let mock_db = container.mock_service("database");
    mock_db.expect_write("key", "value");

    let result = storage::create("key", "value", mock_db)?;

    assert!(result.is_ok());
    assert_eq!(container.external_call_count(), 0);  // Verify no real network calls
}
```

Benefits:
- **Fast**: No network latency (milliseconds instead of seconds)
- **Reliable**: No external failures can cause test failures
- **Isolated**: Other tests don't interfere
- **Reproducible**: Same code always produces same result

---

## Architecture

### HermeticContainer

The `HermeticContainer` is the test sandbox:

```rust
pub struct HermeticContainer {
    id: Uuid,
    mocks: Arc<Mutex<MockServices>>,
    spans: Arc<Mutex<Vec<RecordedSpan>>>,
    external_calls: Arc<AtomicUsize>,
    quota: Arc<Mutex<QuotaBudget>>,
    clock: Arc<Mutex<DeterministicClock>>,
    traced_execution: Arc<AtomicBool>,
}
```

**Components:**

1. **MockServices** — In-memory stubs for all external dependencies (database, API, filesystem)
2. **Span Recorder** — Records OpenTelemetry spans for tracing
3. **External Call Tracker** — Counts calls to external services (should be zero)
4. **Quota Budget** — Enforces resource limits (CPU, memory, time, syscalls)
5. **Deterministic Clock** — Controlled time advancement
6. **Traced Execution Flag** — Marks when execution is being traced

### Workflow

```
1. Create container
   container = HermeticContainer::new()

2. Set expectations
   container.expect_external_calls(0)  // Must be hermetic
   container.mock_service("database").expect_read("key")

3. Run test code
   result = some_function(&container)

4. Verify hermetic
   container.verify_hermetic()?  // Verify no external calls

5. Check constraints
   assert!(container.verify_determinism(&|| ...)?);  // Same output twice
   assert_eq!(container.quota().cpu_used, 42_000);  // Quota tracking
```

---

## Features

### 1. Hermetic Execution

```rust
let container = HermeticContainer::new();
container.expect_external_calls(0);  // Default: must be isolated

// Run code
let result = my_function(&container)?;

// Verify
container.verify_hermetic()?;  // Fails if any external calls made
```

### 2. Mock Services

```rust
let container = HermeticContainer::new();

// Mock a database service
let db = container.mock_service("database");
db.mock_response(200, r#"{"status": "ok"}"#);

// Code calls mocked service
let result = db.query("SELECT ...")?;

// Get next response from mock (LIFO queue)
let response = db.pop_response();
```

### 3. Span Graph Validation

```rust
let container = HermeticContainer::new();

// Run code that generates spans
my_function(&container)?;

// Validate span graph
let spans = container.recorded_spans();
assert!(spans.iter().all(|s| s.is_local_span()));

// Extract span timing
for span in spans {
    println!("Span: {} took {} ns", span.name, span.duration_ns);
}
```

### 4. Quota Enforcement

```rust
let container = HermeticContainer::new();
{
    let mut quota = container.quota();
    quota.cpu_cycles = 1_000_000;      // 1M cycles
    quota.memory_bytes = 10 * 1024 * 1024;  // 10MB
    quota.time_ns = 10_000_000;        // 10ms
}

// Code must stay within budget
let result = my_function(&container)?;

// Verify quota usage
{
    let quota = container.quota();
    println!("CPU: {} / {}", quota.cpu_used, quota.cpu_cycles);
    assert!(quota.is_within_budget());
}
```

### 5. Deterministic Execution

```rust
let container = HermeticContainer::new();

// Verify that same input produces same output
container.verify_determinism(|| {
    my_function(&container).map(|r| r.hash())
})?;

// If code is non-deterministic, error is raised
// This catches timing-dependent bugs, randomization bugs, etc.
```

### 6. Controlled Clock

```rust
let container = HermeticContainer::new();
{
    let mut clock = container.clock();
    println!("Start time: {}", clock.now());
    clock.advance(1_000_000);  // Advance 1ms
    println!("After advance: {}", clock.now());
}
```

---

## Integration with CTT (Chicago TDD Tools)

clnrm is the **verification layer** for CTT 12-phase system:

- **Phase 1-2**: Zero-copy SIMD, Attestation → clnrm verifies no external calls
- **Phase 3-4**: Type-state, Lock-free → clnrm checks quota behavior
- **Phase 5-6**: Deterministic exec, Quota enforcement → clnrm validates both

Each phase can have hermetic tests powered by clnrm.

---

## Current Status

### Implemented (✓)

- `HermeticContainer` — Basic sandbox
- `MockServices` — Service mocking
- `RecordedSpan` — Span recording
- `QuotaBudget` — Quota tracking
- `DeterministicClock` — Controlled time
- Unit tests for core functionality

### TODO (Implementation Roadmap)

**Phase 1: Core Framework (2-3 weeks)**
- [ ] Integrate with OpenTelemetry (real span recording)
- [ ] Add Weaver configuration parser
- [ ] Implement filesystem mocking
- [ ] Create test harness macros (#[hermetic_test])

**Phase 2: Advanced Features (2-3 weeks)**
- [ ] Network call interception
- [ ] Process isolation (containers/VMs for true hermeticity)
- [ ] Syscall tracking
- [ ] Memory profiling

**Phase 3: Integration (1-2 weeks)**
- [ ] Integrate with CTT verification pipeline
- [ ] Add GitHub Actions CI checks
- [ ] Documentation and examples

**Phase 4: Production Hardening (1-2 weeks)**
- [ ] Performance optimization
- [ ] Error handling improvements
- [ ] Logging and debugging support

---

## Example: Hermetic Test Suite

```rust
use clap_noun_verb::kernel::clnrm::*;

mod storage_tests {
    use super::*;

    #[test]
    fn storage_create_is_hermetic() -> Result<()> {
        let container = HermeticContainer::new();
        container.expect_external_calls(0);

        // Run command
        let result = execute_command(
            "storage", "create",
            &["key=test-key", "value=test-value"],
            &container
        )?;

        // Verify
        container.verify_hermetic()?;
        assert!(result.success);
        assert_eq!(container.external_call_count(), 0);

        // Check span graph
        let spans = container.recorded_spans();
        assert!(spans.iter().all(|s| s.is_local_span()));
        assert!(spans.iter().any(|s| s.name.contains("create")));

        Ok(())
    }

    #[test]
    fn storage_create_respects_quota() -> Result<()> {
        let container = HermeticContainer::new();
        {
            let mut quota = container.quota();
            quota.memory_bytes = 1024;  // 1KB limit
        }

        // Try to create large value (should respect quota)
        let result = execute_command(
            "storage", "create",
            &["key=test", "value=<<1MB of data>>"],
            &container
        );

        // Should fail or truncate due to quota
        let quota = container.quota();
        assert!(quota.is_within_budget());

        Ok(())
    }

    #[test]
    fn storage_create_is_deterministic() -> Result<()> {
        let container = HermeticContainer::new();

        container.verify_determinism(|| {
            execute_command("storage", "create", &["key=x", "value=y"], &container)
                .map(|r| r.hash())
        })?;

        Ok(())
    }
}
```

---

## API Reference

### HermeticContainer

```rust
impl HermeticContainer {
    pub fn new() -> Self
    pub fn expect_external_calls(&mut self, count: usize)
    pub fn external_call_count(&self) -> usize
    pub fn record_span(&self, span: RecordedSpan)
    pub fn recorded_spans(&self) -> Vec<RecordedSpan>
    pub fn record_external_call(&self)
    pub fn quota(&self) -> MutexGuard<QuotaBudget>
    pub fn verify_hermetic(&self) -> Result<(), String>
    pub fn verify_determinism<F>(&self, f: F) -> Result<(), String>
}
```

### MockServices

```rust
impl MockServices {
    pub fn new() -> Self
    pub fn mock(&mut self, service: &str, response: MockResponse)
    pub fn get_response(&mut self, service: &str) -> Option<MockResponse>
}
```

### RecordedSpan

```rust
impl RecordedSpan {
    pub fn is_local_span(&self) -> bool
    pub fn is_external_call(&self) -> bool
}
```

### QuotaBudget

```rust
impl QuotaBudget {
    pub fn is_within_budget(&self) -> bool
    pub fn record_cpu(&mut self, cycles: u64) -> Result<(), String>
    pub fn record_memory(&mut self, bytes: u64) -> Result<(), String>
}
```

---

## Philosophy

clnrm embodies the graph-universe philosophy:

- **Determinism** (μ-kernel property): Tests produce identical results
- **Hermeticity** (isolation property): No external dependencies
- **Auditability** (receipt property): Every action recorded in span graph
- **Verification** (CTT property): Phase-by-phase validation

By making all tests hermetic, we prove that:
- Code is deterministic (A = μ(O))
- Code respects quotas and constraints (Q is enforced)
- Code leaves auditable traces (Γ is complete)

---

## References

- **PHILOSOPHY.md** — Graph-universe thesis
- **MU_KERNEL.md** — Deterministic execution
- **CTT** — Chicago TDD Tools (12-phase verification)
- **src/kernel/clnrm.rs** — Implementation
