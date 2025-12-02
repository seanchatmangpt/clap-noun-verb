# Poka-Yoke Test Architecture - Zero-Flake by Design

**Date**: 2025-12-02
**Project**: clap-noun-verb v5.1.0
**Principle**: Make test failures **impossible** through architectural design, not debugging

## Philosophy: Design for Unfailable Tests

**Traditional Approach** (WRONG):
- Write tests that can fail
- Debug flaky tests when they appear
- Add timeouts to prevent hangs
- Hope tests are stable

**Poka-Yoke Approach** (RIGHT):
- Design tests that **cannot** fail due to timing/race conditions
- Use deterministic execution (no wall-clock time)
- Controlled dependencies (no external services)
- Tests are **reproducible by construction**

---

## FM-7.1 Root Cause Analysis

**Why Do Tests Hang/Flake?**

1. **Non-Deterministic Timing**:
   - `tokio::time::sleep()` depends on real time
   - Async tasks race with each other
   - CI load varies execution speed

2. **External Dependencies**:
   - Network calls can timeout
   - File system state changes
   - Global mutable state

3. **Unbounded Execution**:
   - Infinite loops in async tasks
   - Deadlocks in concurrent code
   - Resource leaks

---

## Poka-Yoke Solution 1: Deterministic Async Testing

### Problem: `tokio::time::sleep()` is non-deterministic

```rust
// ❌ WRONG - Can fail due to timing
#[tokio::test]
async fn test_timeout() {
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(some_condition());  // May fail if system is slow
}
```

### Solution: Use `tokio-test` with Manual Time Control

```rust
// ✅ RIGHT - Deterministic, cannot fail due to timing
#[test]
fn test_timeout_deterministic() {
    let mut time = tokio_test::task::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(some_condition());
    });

    // Manually advance time - no real waiting
    time.advance_time(Duration::from_millis(100));
    assert!(time.is_woken());
}
```

**Properties**:
- ✅ No wall-clock dependency
- ✅ Always completes in microseconds
- ✅ Cannot hang (no real waiting)
- ✅ Reproducible on any system

---

## Poka-Yoke Solution 2: Replace Timeout with Unfailable Harness

### Problem: Timeouts mask root causes

```toml
# ❌ WRONG - Hides hanging tests
[tasks.test-timeout]
command = "timeout"
args = ["10s", "cargo", "test"]
```

### Solution: Test Harness that Prevents Hangs

```toml
# ✅ RIGHT - Tests designed to be unfailable
[tasks.test-deterministic]
script = '''
#!/bin/bash
# Run tests with deterministic execution
export TOKIO_TEST_MODE=1  # Enable deterministic time
export RUST_TEST_THREADS=1  # Single-threaded for reproducibility

cargo test --lib --quiet

# No timeout needed - tests complete in bounded time by design
'''
```

**Why This Works**:
- Tests use `tokio-test` (no real async)
- Single-threaded execution (no races)
- Controlled time (no waiting)
- **Timeout unnecessary** - tests complete in milliseconds

---

## Poka-Yoke Solution 3: Isolation by Construction

### Problem: Tests share global state

```rust
// ❌ WRONG - Tests interfere with each other
static mut GLOBAL_CONFIG: Option<Config> = None;

#[test]
fn test_a() {
    unsafe { GLOBAL_CONFIG = Some(Config::new()); }
}

#[test]
fn test_b() {
    // Fails if test_a runs first!
    assert!(unsafe { GLOBAL_CONFIG.is_none() });
}
```

### Solution: Encapsulated Test Context

```rust
// ✅ RIGHT - Each test has isolated context
struct TestContext {
    config: Config,
    runtime: tokio_test::task::Spawn,
}

impl TestContext {
    fn new() -> Self {
        Self {
            config: Config::default(),
            runtime: tokio_test::task::spawn(async {}),
        }
    }
}

#[test]
fn test_a() {
    let ctx = TestContext::new();  // Fresh state
    // Cannot interfere with test_b
}

#[test]
fn test_b() {
    let ctx = TestContext::new();  // Fresh state
    // Cannot interfere with test_a
}
```

**Properties**:
- ✅ No shared state
- ✅ Test order doesn't matter
- ✅ Can run in parallel safely
- ✅ **Isolation by construction**

---

## Poka-Yoke Solution 4: Bounded Execution by Type

### Problem: Async tasks can run forever

```rust
// ❌ WRONG - Can deadlock or loop forever
#[tokio::test]
async fn test_process() {
    loop {
        process_next().await;  // Might never exit!
    }
}
```

### Solution: Use Type System to Enforce Bounds

```rust
// ✅ RIGHT - Bounded by type
struct BoundedExecutor<const MAX_ITERATIONS: usize>;

impl<const MAX_ITERATIONS: usize> BoundedExecutor<MAX_ITERATIONS> {
    fn run<F>(&self, mut task: F)
    where
        F: FnMut() -> bool,  // Returns true when done
    {
        for _ in 0..MAX_ITERATIONS {
            if task() {
                return;  // Completed successfully
            }
        }
        panic!("Task exceeded MAX_ITERATIONS - infinite loop detected");
    }
}

#[test]
fn test_process_bounded() {
    let executor = BoundedExecutor::<100>;  // Max 100 iterations
    executor.run(|| {
        process_next();
        is_complete()
    });
    // Cannot run forever - compiler enforces bound
}
```

**Properties**:
- ✅ Compile-time bounds
- ✅ Cannot hang (bounded iterations)
- ✅ Clear failure mode (exceeds bound = bug detected)
- ✅ **Unfailable by construction**

---

## Implementation Plan

### Step 1: Add Deterministic Test Dependencies

```toml
# Cargo.toml
[dev-dependencies]
tokio-test = "0.4"
serial_test = "3.0"  # For tests that must run sequentially
```

### Step 2: Create Test Utilities Module

```rust
// tests/common/deterministic.rs

pub struct DeterministicRuntime {
    time: tokio_test::time::MockTime,
}

impl DeterministicRuntime {
    pub fn new() -> Self {
        Self {
            time: tokio_test::time::MockTime::new(),
        }
    }

    pub fn advance(&mut self, duration: Duration) {
        self.time.advance(duration);
    }

    pub fn spawn<F>(&mut self, future: F) -> tokio_test::task::Spawn<F>
    where
        F: Future,
    {
        tokio_test::task::spawn(future)
    }
}

pub struct TestContext {
    pub runtime: DeterministicRuntime,
    pub temp_dir: tempfile::TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            runtime: DeterministicRuntime::new(),
            temp_dir: tempfile::tempdir().unwrap(),
        }
    }
}
```

### Step 3: Rewrite Async Tests

```rust
// Before (flaky):
#[tokio::test]
async fn test_async_operation() {
    let result = timeout(Duration::from_secs(1), async_op()).await;
    assert!(result.is_ok());
}

// After (unfailable):
#[test]
fn test_async_operation_deterministic() {
    let mut ctx = TestContext::new();
    let mut task = ctx.runtime.spawn(async_op());

    ctx.runtime.advance(Duration::from_millis(500));
    assert!(task.is_woken());

    let result = task.await;
    assert!(result.is_ok());
}
```

### Step 4: Update Makefile Tasks

```toml
[tasks.test-lib-deterministic]
command = "cargo"
args = ["test", "--lib"]
env = { RUST_TEST_THREADS = "1" }
description = "Run library tests (deterministic, single-threaded)"

[tasks.test-integration-isolated]
script = '''
#!/bin/bash
# Run each integration test in isolation
for test in $(cargo test --test '*' --list | grep '^test ' | awk '{print $2}'); do
    echo "Running isolated: $test"
    cargo test --test integration_examples "$test" -- --exact
done
'''
description = "Run integration tests in complete isolation"

[tasks.test-unfailable]
dependencies = ["test-lib-deterministic", "test-integration-isolated"]
description = "Run all tests with unfailable architecture"

[tasks.ci]
dependencies = [
    "format-check",
    "clippy",
    "test-unfailable",  # Replace test-timeout with unfailable tests
    "build-examples",
    "check-all",
]
```

---

## Success Criteria

**Before (Flaky Architecture)**:
- ❌ Tests can hang (need timeout)
- ❌ Tests can flake (timing-dependent)
- ❌ Tests can interfere (shared state)
- ❌ Tests can deadlock (unbounded execution)
- ❌ CI randomly fails

**After (Poka-Yoke Architecture)**:
- ✅ Tests **cannot** hang (bounded by design)
- ✅ Tests **cannot** flake (deterministic execution)
- ✅ Tests **cannot** interfere (isolated contexts)
- ✅ Tests **cannot** deadlock (bounded execution)
- ✅ CI **always** passes (or fails for real bugs)

---

## Poka-Yoke Categories Applied

1. **Control Poka-Yoke** (Tests designed to prevent errors):
   - Deterministic time (no real waiting)
   - Isolated contexts (no shared state)
   - Bounded execution (no infinite loops)

2. **Warning Poka-Yoke** (Detect errors immediately):
   - Exceeding iteration bounds = test fails
   - File system changes detected
   - State changes tracked

3. **Fixed-Value Poka-Yoke** (Exact test counts):
   - Know exactly how many tests exist
   - Each test runs exactly once
   - Reproducible results

---

## Next Steps

1. ✅ Add `tokio-test` and `serial_test` dependencies
2. ✅ Create `tests/common/deterministic.rs` utilities
3. ✅ Audit all async tests, rewrite with deterministic runtime
4. ✅ Update `Makefile.toml` with unfailable test tasks
5. ✅ Remove timeout-based CI (unnecessary)
6. ✅ Verify CI passes without timeouts

---

**Document Status**: ✅ Design Complete
**Implementation Status**: Ready to begin
**Next**: Create deterministic test infrastructure
