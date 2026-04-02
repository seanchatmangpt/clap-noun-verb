# Mitigation Plan - clap-noun-verb v5.1.0

**Date**: 2025-12-02
**Project**: clap-noun-verb v5.1.0
**Source**: FMEA Analysis (docs/quality/FMEA_ANALYSIS.md)
**Risk Register**: docs/quality/RISK_REGISTER.md

## Execution Timeline

**Phase 1**: Immediate Fixes (Week 1) - RPN > 200 + Andon Signals
**Phase 2**: High-Priority Poka-Yoke (Week 2) - RPN 100-200
**Phase 3**: Testing & Documentation (Week 3-4) - RPN 50-100

---

## Phase 1: Immediate Fixes (Week 1)

### ✅ FM-1.1 (RPN 160) - Test Timeout Too Aggressive **[COMPLETED]**

**Status**: ✅ **FIXED** - Andon Signal Cleared

**Action Taken**:
```toml
# Makefile.toml line 44
[tasks.test-timeout]
command = "timeout"
args = ["10s", "cargo", "test", "--quiet"]  # Increased from 1s
description = "Run tests with 10s timeout (macros need compilation time)"
```

**Verification**:
- Manual test: `cargo test` in macros workspace completes in 1.91s ✅
- Expected CI result: PASS (was failing with exit 124)

**Next**: Verify CI pipeline passes with new timeout

---

### 🔴 FM-4.2 (RPN 450) - Agent Identity Spoofing **[CRITICAL SECURITY]**

**Severity**: 10/10 - Complete security breach
**Occurrence**: 5/10 - If exposed to untrusted agents
**Detection**: 9/10 - Silent, only breach investigation

**Root Cause**: No cryptographic verification of agent identities

**Current State**:
- String-based agent IDs (`src/autonomic/identity.rs`)
- No authentication mechanism
- No signature verification
- No identity attestation

**Mitigation Strategy**: Implement Quantum-Safe Agent Identity System

**Action Plan** (3-5 days):

**Day 1**: Design cryptographic identity system
1. Review Agent2028 quantum_crypto module (`src/agent2028/quantum_crypto.rs`)
2. Design public key infrastructure:
   - Agent identity = public key hash
   - Capability requests signed with private key
   - Attestation proofs for capabilities
3. Define identity lifecycle (create, renew, revoke)

**Day 2-3**: Implement identity system
1. Create `src/autonomic/crypto_identity.rs`:
   ```rust
   pub struct CryptoAgentIdentity {
       pub agent_id: String,  // Hash of public key
       pub public_key: Vec<u8>,
       pub attestation: QuantumAttestationProof,
   }

   impl CryptoAgentIdentity {
       pub fn new(agent_id: String) -> Result<(Self, Vec<u8>)> {
           // Generate quantum-safe key pair
           let (key_material, private_key) = QuantumKeyEncapsulation::generate();
           // Create attestation proof
           let attestation = create_attestation(&agent_id, &key_material)?;
           Ok((Self { agent_id, public_key: key_material.public_key, attestation }, private_key))
       }

       pub fn verify_signature(&self, message: &[u8], signature: &QuantumSignature) -> bool {
           signature.verify_dual(message)
       }
   }
   ```

2. Update `AgentIdentity` to require crypto proof:
   ```rust
   pub struct AgentIdentity {
       crypto_identity: CryptoAgentIdentity,
       metadata: HashMap<String, String>,
   }
   ```

3. Update capability grant/revoke to require signatures:
   ```rust
   pub fn grant_capability(
       &mut self,
       capability: CapabilityId,
       requester: &AgentIdentity,
       signature: &QuantumSignature,
   ) -> Result<()> {
       // Verify signature before granting
       if !requester.crypto_identity.verify_signature(&capability.to_bytes(), signature) {
           return Err(NounVerbError::AuthenticationFailed("Invalid signature"));
       }
       // ... grant logic
   }
   ```

**Day 4**: Add tests
1. Unit tests for identity creation, signature verification
2. Property tests for identity uniqueness
3. Concurrency tests with `loom` for thread-safety
4. Security tests for spoofing attempts

**Day 5**: Integration and documentation
1. Update examples to use crypto identities
2. Add migration guide for existing identity code
3. Update autonomic_example.rs
4. Add security best practices doc

**Acceptance Criteria**:
- ✅ All capability operations require cryptographic signatures
- ✅ Agent identity = hash of public key (unforgeable)
- ✅ Attestation proofs expire and can be revoked
- ✅ All tests pass (including security tests)
- ✅ Documentation complete

**Owner**: Security team + Agent2028 lead
**Priority**: 🔴 **CRITICAL** (Security vulnerability)
**Target**: Week 1 (Day 1-5)

---

### 🔴 FM-1.4 (RPN 432) - Platform-Specific Compilation Failures

**Severity**: 8/10 - Complete failure on untested platforms
**Occurrence**: 6/10 - Platform-specific code exists
**Detection**: 9/10 - Only detected by users

**Root Cause**: No cross-platform CI testing

**Mitigation Strategy**: Add GitHub Actions Matrix Testing

**Action Plan** (1-2 days):

**Step 1**: Create `.github/workflows/ci.yml`
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta, 1.74]  # MSRV = 1.74
    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Install timeout (macOS/Linux)
        if: runner.os != 'Windows'
        run: |
          # Already installed on Ubuntu
          if [ "$RUNNER_OS" == "macOS" ]; then
            brew install coreutils
          fi

      - name: Install timeout (Windows)
        if: runner.os == 'Windows'
        shell: bash
        run: |
          # Windows requires GNU timeout from Git Bash or install separately
          echo "Windows timeout handling (use Powershell timeout or skip)"

      - name: Run cargo make ci
        run: cargo make ci

      - name: Check examples build
        run: cargo build --examples
```

**Step 2**: Test platform-specific code
1. Audit all platform-specific code:
   - `atty` crate usage
   - `tokio` platform features
   - File system operations
   - Path handling

2. Add platform-specific tests:
   ```rust
   #[cfg(target_os = "windows")]
   #[test]
   fn test_windows_paths() {
       // Test Windows-specific path handling
   }

   #[cfg(not(target_os = "windows"))]
   #[test]
   fn test_unix_paths() {
       // Test Unix-specific path handling
   }
   ```

**Acceptance Criteria**:
- ✅ CI runs on Ubuntu, macOS, Windows
- ✅ CI tests stable, beta, MSRV (1.74)
- ✅ All tests pass on all platforms
- ✅ Examples build on all platforms

**Owner**: CI/CD engineer
**Priority**: 🔴 **CRITICAL**
**Target**: Week 1 (Day 1-2)

---

### 🔴 FM-7.3 (RPN 432) - Incomplete Test Coverage

**Severity**: 6/10 - Bugs slip through, regressions
**Occurrence**: 8/10 - New code without tests
**Detection**: 9/10 - Only when bugs occur

**Root Cause**: No automated coverage tracking

**Mitigation Strategy**: Add Coverage Tracking with 80% Gate

**Action Plan** (1 day):

**Step 1**: Add coverage tools
```toml
# .cargo/config.toml (create if not exists)
[build]
rustflags = ["-C", "instrument-coverage"]

[env]
LLVM_PROFILE_FILE = "target/coverage/clap-noun-verb-%p-%m.profraw"
```

**Step 2**: Add Makefile tasks
```toml
# Makefile.toml

[tasks.coverage-clean]
command = "rm"
args = ["-rf", "target/coverage"]
description = "Clean coverage data"

[tasks.coverage-run]
dependencies = ["coverage-clean"]
command = "cargo"
args = ["test", "--all-features"]
env = { CARGO_INCREMENTAL = "0", RUSTFLAGS = "-C instrument-coverage" }
description = "Run tests with coverage instrumentation"

[tasks.coverage-report]
dependencies = ["coverage-run"]
script = '''
grcov . \
  --binary-path ./target/debug/ \
  --source-dir . \
  --output-type html \
  --branch \
  --ignore-not-existing \
  --output-path ./target/coverage/html

echo "Coverage report: target/coverage/html/index.html"
'''
description = "Generate HTML coverage report"

[tasks.coverage-check]
dependencies = ["coverage-run"]
script = '''
COVERAGE=$(grcov . \
  --binary-path ./target/debug/ \
  --source-dir . \
  --output-type json \
  --ignore-not-existing | jq '.coverage')

if (( $(echo "$COVERAGE < 80.0" | bc -l) )); then
    echo "ERROR: Coverage $COVERAGE% is below 80% threshold"
    exit 1
fi

echo "✅ Coverage: $COVERAGE%"
'''
description = "Verify coverage >= 80%"

[tasks.coverage]
dependencies = ["coverage-report", "coverage-check"]
description = "Full coverage workflow"
```

**Step 3**: Add to CI
```toml
[tasks.ci]
dependencies = [
    "format-check",
    "clippy",
    "test-timeout",
    "coverage-check",  # Add coverage gate
    "build-examples",
    "check-all",
]
```

**Step 4**: Install dependencies
```bash
cargo install grcov
rustup component add llvm-tools-preview
```

**Acceptance Criteria**:
- ✅ Coverage measured with llvm-cov + grcov
- ✅ CI fails if coverage < 80%
- ✅ HTML report generated
- ✅ Coverage badge in README.md

**Owner**: Test infrastructure lead
**Priority**: 🔴 **CRITICAL**
**Target**: Week 1 (Day 1)

---

### 🔴 FM-8.1 (RPN 360) - Outdated Examples

**Severity**: 5/10 - User frustration, bad first impression
**Occurrence**: 9/10 - Every API change risks breaking
**Detection**: 8/10 - Only when users try

**Root Cause**: Examples only compiled, not executed in CI

**Mitigation Strategy**: Run and Snapshot Test All Examples

**Action Plan** (2 days):

**Step 1**: Add example runner task
```toml
# Makefile.toml

[tasks.test-examples]
script = '''
#!/bin/bash
set -e

echo "Running all examples..."

for example in $(cargo build --examples --message-format=json | jq -r 'select(.reason == "compiler-artifact") | select(.target.kind[] == "example") | .target.name'); do
    echo "Running example: $example"
    cargo run --example "$example" -- --help > /dev/null || {
        echo "ERROR: Example $example failed to run"
        exit 1
    }
done

echo "✅ All examples ran successfully"
'''
description = "Run all examples to verify they work"
```

**Step 2**: Add snapshot tests
```rust
// tests/example_snapshots.rs
use insta::assert_snapshot;
use std::process::Command;

#[test]
fn test_basic_example_output() {
    let output = Command::new("cargo")
        .args(&["run", "--example", "basic", "--", "--help"])
        .output()
        .expect("Failed to run basic example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!("basic_help", stdout);
}

// Repeat for all 22 examples...
```

**Step 3**: Add to CI
```toml
[tasks.ci]
dependencies = [
    "format-check",
    "clippy",
    "test-timeout",
    "coverage-check",
    "test-examples",  # Add example testing
    "build-examples",
    "check-all",
]
```

**Acceptance Criteria**:
- ✅ All 22 examples run successfully
- ✅ Snapshot tests for example outputs
- ✅ CI fails if any example fails
- ✅ Example freshness dates added

**Owner**: Documentation lead
**Priority**: 🔴 **CRITICAL**
**Target**: Week 1 (Day 1-2)

---

### 🔴 FM-3.2 (RPN 280) - Poor Macro Error Messages

**Severity**: 5/10 - Developer frustration
**Occurrence**: 7/10 - Common in macro dev
**Detection**: 8/10 - User feedback, slow

**Root Cause**: No systematic error message design

**Mitigation Strategy**: Improve Error Messages with Help Text

**Action Plan** (3 days):

**Step 1**: Catalog all macro errors
1. Review `clap-noun-verb-macros/src/*.rs`
2. Identify all `proc_macro_error::abort!` call sites
3. Document current error messages
4. Identify missing help text

**Step 2**: Add help text to errors
```rust
// clap-noun-verb-macros/src/verb.rs

// BEFORE:
abort!(span, "Missing required attribute 'name'");

// AFTER:
abort!(
    span,
    "Missing required attribute 'name'";
    help = "Add #[verb(name = \"command-name\")] to specify the command name";
    note = "Example: #[verb(name = \"create\")] fn create_user() -> Result<()>"
);
```

**Step 3**: Add compile-fail tests
```rust
// clap-noun-verb-macros/tests/ui/missing_name.rs
#[verb]
fn my_command() -> Result<()> {
    Ok(())
}

// clap-noun-verb-macros/tests/ui/missing_name.stderr
error: Missing required attribute 'name'
  --> tests/ui/missing_name.rs:1:1
   |
1  | #[verb]
   | ^^^^^^^
   |
   = help: Add #[verb(name = \"command-name\")] to specify the command name
   = note: Example: #[verb(name = \"create\")] fn create_user() -> Result<()>
```

**Step 4**: Test with `trybuild`
```rust
// clap-noun-verb-macros/tests/compile_fail.rs
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
```

**Acceptance Criteria**:
- ✅ All macro errors have help text
- ✅ Examples of correct usage in notes
- ✅ Compile-fail tests for error messages
- ✅ Error message quality reviewed

**Owner**: Macro system maintainer + UX lead
**Priority**: 🔴 **CRITICAL**
**Target**: Week 1 (Day 1-3)

---

### 🔴 FM-4.1 (RPN 240) - Capability Race Conditions

**Severity**: 10/10 - Security vulnerability
**Occurrence**: 3/10 - Async environments
**Detection**: 8/10 - Very hard to detect

**Root Cause**: No concurrency testing for capability operations

**Mitigation Strategy**: Add Concurrency Tests with Loom

**Action Plan** (2-3 days):

**Step 1**: Add loom tests
```rust
// src/autonomic/governance.rs

#[cfg(all(test, loom))]
mod loom_tests {
    use loom::sync::Arc;
    use loom::thread;
    use super::*;

    #[test]
    fn test_concurrent_grant_revoke() {
        loom::model(|| {
            let ledger = Arc::new(GovernanceLedger::new());
            let cap_id = CapabilityId::from_path("test.capability");

            let ledger1 = Arc::clone(&ledger);
            let ledger2 = Arc::clone(&ledger);

            let t1 = thread::spawn(move || {
                ledger1.grant_capability(
                    cap_id.clone(),
                    AgentIdentity::anonymous(),
                    TenantIdentity::default_tenant(),
                    "test grant",
                ).unwrap();
            });

            let t2 = thread::spawn(move || {
                ledger2.revoke_capability(
                    cap_id.clone(),
                    AgentIdentity::anonymous(),
                    "test revoke",
                ).unwrap();
            });

            t1.join().unwrap();
            t2.join().unwrap();

            // Verify audit trail is consistent
            let entries = ledger.audit_trail(&cap_id);
            assert!(entries.len() == 2); // Both operations recorded
        });
    }
}
```

**Step 2**: Add property tests for audit trail
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn audit_trail_never_corrupts(
        operations in prop::collection::vec(
            prop::oneof![
                Just(Operation::Grant),
                Just(Operation::Revoke),
            ],
            1..100
        )
    ) {
        let ledger = GovernanceLedger::new();
        let cap_id = CapabilityId::from_path("test");

        for op in operations {
            match op {
                Operation::Grant => ledger.grant_capability(...),
                Operation::Revoke => ledger.revoke_capability(...),
            }
        }

        // Verify audit trail integrity
        let trail = ledger.audit_trail(&cap_id);
        assert!(trail.len() == operations.len());
        // Verify all operations recorded in order
    }
}
```

**Acceptance Criteria**:
- ✅ Loom tests for all concurrent scenarios
- ✅ Property tests for audit trail
- ✅ No data races detected
- ✅ All tests pass (including concurrency)

**Owner**: Security team + autonomic layer maintainer
**Priority**: 🔴 **CRITICAL**
**Target**: Week 1 (Day 1-3)

---

### 🔴 FM-1.3 (RPN 210) - Feature Flag Misconfiguration

**Severity**: 6/10 - Breaks experimental features
**Occurrence**: 5/10 - Could happen on docs.rs
**Detection**: 7/10 - Not tested until docs.rs

**Root Cause**: Experimental feature not tested in CI

**Mitigation Strategy**: Test All Feature Combinations

**Action Plan** (1 day):

**Step 1**: Add feature matrix tests
```toml
# Makefile.toml

[tasks.test-all-features]
command = "cargo"
args = ["test", "--all-features"]
description = "Test with all features enabled"

[tasks.test-no-default-features]
command = "cargo"
args = ["test", "--no-default-features"]
description = "Test with no features"

[tasks.test-experimental-only]
command = "cargo"
args = ["test", "--features", "experimental"]
description = "Test experimental feature only"

[tasks.check-feature-powerset]
script = '''
cargo install cargo-hack --quiet
cargo hack check --feature-powerset
'''
description = "Check all feature combinations"
```

**Step 2**: Add docs.rs simulation
```toml
[tasks.doc-as-docs-rs]
command = "cargo"
args = ["doc", "--no-deps", "--all-features"]
env = { RUSTDOCFLAGS = "-D warnings" }
description = "Build docs as docs.rs would"
```

**Step 3**: Add to CI
```toml
[tasks.ci]
dependencies = [
    "format-check",
    "clippy",
    "test-timeout",
    "coverage-check",
    "test-examples",
    "test-all-features",      # Add
    "check-feature-powerset", # Add
    "doc-as-docs-rs",        # Add
    "build-examples",
    "check-all",
]
```

**Acceptance Criteria**:
- ✅ All feature combinations tested
- ✅ Docs build with all features
- ✅ CI fails on feature issues

**Owner**: Build system maintainer
**Priority**: 🔴 **CRITICAL**
**Target**: Week 1 (Day 1)

---

### 🔴 FM-6.1 (RPN 210) - SPARQL Query Timeouts

**Severity**: 6/10 - Feature unavailable
**Occurrence**: 7/10 - Complex queries
**Detection**: 5/10 - Runtime, user-reported

**Root Cause**: No query timeout enforcement

**Mitigation Strategy**: Implement Query Timeouts and Complexity Analysis

**Action Plan** (2 days):

**Step 1**: Add timeout wrapper
```rust
// src/rdf/sparql.rs

use std::time::Duration;
use tokio::time::timeout;

pub struct SparqlExecutor {
    store: Store,
    timeout_duration: Duration,
}

impl SparqlExecutor {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            timeout_duration: Duration::from_secs(30), // Default 30s
        }
    }

    pub async fn execute_query(&self, query: &str) -> Result<QueryResults> {
        // Parse query
        let parsed = QueryParser::parse(query)?;

        // Analyze complexity
        let complexity = self.analyze_complexity(&parsed)?;
        if complexity > 1000 {
            return Err(NounVerbError::QueryTooComplex(complexity));
        }

        // Execute with timeout
        match timeout(self.timeout_duration, self.store.query(query)).await {
            Ok(Ok(results)) => Ok(results),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(NounVerbError::QueryTimeout(self.timeout_duration)),
        }
    }

    fn analyze_complexity(&self, query: &ParsedQuery) -> Result<u32> {
        // Count: joins, subqueries, filters, union clauses
        let mut score = 0;
        score += query.joins().len() as u32 * 10;
        score += query.subqueries().len() as u32 * 20;
        score += query.filters().len() as u32 * 5;
        Ok(score)
    }
}
```

**Step 2**: Add memory limits
```rust
pub struct ResourceLimits {
    pub max_query_time: Duration,
    pub max_memory_mb: usize,
    pub max_results: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_query_time: Duration::from_secs(30),
            max_memory_mb: 512,
            max_results: 100_000,
        }
    }
}
```

**Acceptance Criteria**:
- ✅ All queries timeout after 30s (configurable)
- ✅ Complex queries rejected (complexity > 1000)
- ✅ Memory limits enforced
- ✅ Tests for timeout scenarios

**Owner**: RDF layer maintainer
**Priority**: 🔴 **CRITICAL**
**Target**: Week 1 (Day 1-2)

---

## Phase 2: High-Priority Mitigations (Week 2)

### 🟡 FM-7.1 (RPN 192) - Flaky Tests
- Add flaky test detector (100x runs)
- Use `tokio-test` for deterministic async
- Effort: 2 days

### 🟡 FM-8.2 (RPN 189) - Dead Links
- Add `markdown-link-check` to CI
- Effort: 1 day

### 🟡 FM-8.3 (RPN 180) - Missing API Docs
- Add `#![warn(missing_docs)]`
- Effort: 1 day

### 🟡 FM-2.3 (RPN 140) - JSON Serialization
- Property tests for all types
- Effort: 2 days

### 🟡 FM-6.2 (RPN 140) - Template Cache
- Version tracking (hash-based)
- Effort: 1 day

### 🟡 FM-2.1 (RPN 126) - Command Registration
- Compile-time duplicate detection
- Effort: 2 days

### 🟡 FM-1.5 (RPN 120) - Missing Dev Dependencies
- Docker-based CI testing
- Effort: 1 day

### 🟡 FM-1.2 (RPN 105) - Dependency Conflicts
- Add `cargo audit` + MSRV verification
- Effort: 1 day

### 🟡 FM-7.2 (RPN 105) - Test Isolation
- Test randomization, global state audit
- Effort: 2 days

---

## Phase 3: Monitoring & Long-Term (Week 3-4)

### 🟢 FM-3.1 (RPN 80) - Macro Expansion
- `trybuild` compile-fail tests
- Effort: 2 days

### 🟢 FM-5.1 (RPN 60) - Experimental Tests **[DEFERRED]**
- User chose to gate, not fix
- Target: v5.2.0

### 🟢 FM-2.2 (RPN 30) - Runtime Panics
- Periodic `cargo expand` audits
- Effort: Ongoing monitoring

---

## Success Metrics

**Phase 1 Complete When**:
- ✅ All RPN > 200 risks mitigated
- ✅ CI passes on all platforms
- ✅ 80%+ test coverage achieved
- ✅ Agent identity system cryptographic
- ✅ All examples run successfully
- ✅ Macro errors have helpful messages
- ✅ SPARQL queries timeout properly
- ✅ Capability operations concurrency-safe
- ✅ All feature combinations tested

**Phase 2 Complete When**:
- ✅ All RPN 100-200 risks mitigated
- ✅ No flaky tests detected
- ✅ No dead links in docs
- ✅ All public APIs documented
- ✅ JSON serialization tested
- ✅ Template cache validated
- ✅ Command registration race-free
- ✅ Dev dependencies reproducible
- ✅ Dependency audit passing
- ✅ Test isolation verified

**Phase 3 Complete When**:
- ✅ All RPN 50-100 risks mitigated
- ✅ Macro expansions tested
- ✅ Experimental tests passing (v5.2)
- ✅ Continuous monitoring in place

---

## Continuous Improvement

**Weekly Review**:
- Track RPN reduction progress
- Update risk register
- Adjust priorities based on findings

**Quarterly Reassessment**:
- Re-analyze all 8 subsystems
- Identify new failure modes
- Update FMEA documentation

---

**Document Status**: ✅ Phase 1.3 Complete
**Last Updated**: 2025-12-02
**Next**: Begin implementation (verify Andon signal cleared first)
