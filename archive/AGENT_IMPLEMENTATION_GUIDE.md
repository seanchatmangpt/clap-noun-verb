# Agent Implementation and Configuration Guide

This guide provides practical implementation details for instantiating and operating the 6 specialized agents in your Claude Code workflow.

---

## Quick Start: Agent Templates

### MacroReviewAgent Template

**When to invoke**: Any changes to `clap-noun-verb-macros/src/`

```bash
# For PR review on macro changes
claude code --agent macro_review --prompt "
Review the proc-macro changes in this PR:
- Validate token stream handling (syn/quote usage)
- Check compile-time error detection completeness
- Verify no runtime overhead from macro expansion
- Ensure generated code passes clippy deny rules
- Confirm distributed slice entries are collision-free
- Measure incremental build time regression

Reference ADL-002 (Proc-Macro Over Derive-Based Registration)
"
```

**Command checklist before approving**:
```bash
cargo expand --lib                    # Visualize macro expansion
cargo make check                      # Verify compilation
cargo make clippy                     # Check warnings
cargo test --lib clap_noun_verb_macros  # Run macro tests
time cargo make build                 # Measure compile time (target: <2s)
```

---

### TestOrchestratorAgent Template

**When to invoke**: Before merging any code, during release validation

```bash
# Full test matrix run
claude code --agent test_orchestrator --prompt "
Execute complete test validation:
1. Run quick tests (parallel): cargo make test
2. Run deterministic tests (serial): cargo make test-lib-deterministic
3. Run feature matrix (23 combinations): cargo make test-frontier-matrix
4. Identify any flaky tests (compare serial vs parallel results)
5. Verify all assertions are behavioral (grep for 'is_ok()')
6. Report total execution time (must be <1000ms)

Provide summary: PASS/FAIL with details on any failures
"
```

**Automated test checks**:
```bash
#!/bin/bash
set -e

echo "=== Quick Tests (Parallel) ==="
cargo make test

echo "=== Deterministic Tests (Serial) ==="
cargo make test-lib-deterministic

echo "=== Feature Matrix (23 combos) ==="
cargo make test-frontier-matrix

echo "=== All tests passed ==="
```

---

### ReleaseConductorAgent Template

**When to invoke**: At release time (coordinate with version bump)

```bash
# Pre-release validation
claude code --agent release_conductor --prompt "
Prepare and execute release workflow:

1. VALIDATION:
   - Verify versions match in clap-noun-verb-macros/Cargo.toml
     and clap-noun-verb/Cargo.toml (both must be 26.X.Y)
   - Ensure CHANGELOG.md has user-facing changes for this version
   - Run: cargo make release-check (all checks must pass)

2. DRY-RUN:
   - cargo make publish-dry-run-macros → should succeed
   - cargo make publish-dry-run → should succeed
   - If either fails, diagnose and fix before proceeding

3. PUBLISH (macros first):
   - cargo make publish-macros
   - Wait 60 seconds for crates.io index sync
   - Verify on crates.io

4. PUBLISH (main):
   - cargo make publish
   - Verify on crates.io
   - Run: cargo make verify-publish

5. GIT TAG:
   - Create tag: git tag v<version>
   - Push tag: git push origin v<version>

Return summary with publish URLs and tag info
"
```

**Pre-publish checklist**:
```bash
#!/bin/bash
# Verify version consistency
MACRO_VERSION=$(grep '^version' clap-noun-verb-macros/Cargo.toml | head -1 | grep -oP '\d+\.\d+\.\d+')
MAIN_VERSION=$(grep '^version' clap-noun-verb/Cargo.toml | head -1 | grep -oP '\d+\.\d+\.\d+')

if [ "$MACRO_VERSION" != "$MAIN_VERSION" ]; then
  echo "ERROR: Version mismatch - macros: $MACRO_VERSION, main: $MAIN_VERSION"
  exit 1
fi

echo "Versions match: $MACRO_VERSION"
echo "Running release checks..."
cargo make release-check
```

---

### ArchitectureGuardian Template

**When to invoke**: On any significant code changes, quarterly review

```bash
# Architecture compliance check
claude code --agent architecture_guardian --prompt "
Review changes against Architecture Decision Log:

1. ADL-001 (Noun-Verb Pattern):
   - All new commands follow 'noun verb' pattern
   - grep: find any flat commands like 'myapp <action>'

2. ADL-002 (Proc-Macro):
   - Any new macros use distributed slices
   - Check for proper use of linkme

3. ADL-003 (JSON Output):
   - New verbs return Serializable types
   - Verify via grep for non-Serialize verb returns

4. ADL-004 (Async):
   - Async logic in async_verb.rs or async feature
   - Review verb signatures for proper async usage

5. ADL-005 (No Panics):
   - Zero unwrap/expect/panic in production code
   - Run: cargo make clippy (should pass deny rules)

6. ADL-006 (Feature Gating):
   - Experimental features properly #[cfg(...)] gated
   - Confirm frontier features in Cargo.toml match code

7. ADL-007 (Minimalist Core):
   - Only 2 optional modules in src/: async_verb.rs, federation/
   - Review for any new core modules

8. ADL-008 (Distributed Slices):
   - Verb registration uses linkme::distributed_slice!
   - Check CommandRegistry implementation

9. ADL-009 (SLOs):
   - Measure: incremental build time, binary size
   - Should stay: <=2s compile, <=10MB binary

10. ADL-010 (Trait Design):
    - Core traits are sync and dyn-compatible
    - No async methods in NounCommand, VerbCommand

Report: PASS/FAIL for each ADL with specific violations found
"
```

**Quick ADL validation**:
```bash
#!/bin/bash

echo "=== ADL-005: No panics in production ==="
cargo make clippy  # Should pass all deny rules

echo "=== ADL-009: SLO Check ==="
cargo make slo-check

echo "=== ADL-006: Feature gating ==="
cargo make check-frontier-all
cargo make check-frontier-minimal

echo "=== ADL-010: Trait safety ==="
cargo make build  # Ensures dyn-compatible traits compile
```

---

### PerformanceAnalystAgent Template

**When to invoke**: Before merging performance-critical changes, during release validation

```bash
# Performance analysis
claude code --agent performance_analyst --prompt "
Analyze performance metrics:

1. COMPILE TIME:
   - Measure incremental build (must be <=2s):
     touch src/lib.rs && time cargo make build
   - Compare against baseline (currently 0.66s)
   - If >5% regression, identify slow crate

2. BINARY SIZE:
   - Release build size (must be <=10MB):
     cargo make build-release && ls -lh target/release/clap-noun-verb-gen
   - Current size: 2.2MB (well under target)

3. BENCHMARKS:
   - Run: cargo make bench
   - Compare against baseline: cargo make bench-compare
   - Any regressions >10% require justification

4. DEPENDENCY ANALYSIS:
   - Check for feature bloat: cargo tree --duplicates
   - Identify heavy dependencies

5. MEMORY PROFILING:
   - Review code for large stack allocations
   - Check for unnecessary clones in hot paths

Report: Current metrics, any regressions, optimization recommendations
"
```

**Performance validation script**:
```bash
#!/bin/bash
set -e

echo "=== Baseline Measurements ==="
echo "Compiling (incremental)..."
touch src/lib.rs
START=$(date +%s%N)
cargo make build > /dev/null 2>&1
END=$(date +%s%N)
COMPILE_TIME_MS=$(( (END - START) / 1000000 ))
echo "Compile time: ${COMPILE_TIME_MS}ms (target: <=2000ms)"

echo "Binary size (release)..."
cargo make build-release > /dev/null 2>&1
SIZE=$(du -h target/release/clap-noun-verb-gen | cut -f1)
SIZE_MB=$(stat -f%z target/release/clap-noun-verb-gen | awk '{print int($1/1048576)}')
echo "Size: $SIZE ($SIZE_MB MB, target: <=10MB)"

echo ""
echo "=== Running Benchmarks ==="
cargo make bench-baseline
cargo make bench-compare

echo ""
echo "=== Dependency Check ==="
cargo tree --duplicates | head -20 || echo "No duplicates"
```

---

### DocMaintainerAgent Template

**When to invoke**: On documentation changes, before releasing, quarterly review

```bash
# Documentation validation
claude code --agent doc_maintainer --prompt "
Validate documentation completeness and accuracy:

1. VERSION SYNCHRONIZATION:
   - CLAUDE.md version == Cargo.toml version
   - Feature list in docs == Cargo.toml [features]
   - Critical rules match lint configuration

2. EXAMPLE CODE:
   - All examples in CLAUDE.md recipes compile
   - Example commands execute successfully
   - Example code follows AAA testing pattern

3. DOC TESTS:
   - Run: cargo test --doc (must pass)
   - Verify /// examples in code comments work

4. EXAMPLES BUILD:
   - cargo make build-examples (must succeed)
   - All examples in examples/ directory

5. ADL ACCURACY:
   - ADL entries (1-10) reflect current code
   - Code comments reference relevant ADLs
   - No outdated or aspirational ADLs

6. TROUBLESHOOTING:
   - Common issues documented
   - Solutions tested and verified
   - Error messages match actual behavior

7. DEAD LINKS:
   - Check file references are correct
   - Cross-section links (e.g., 'see ADL-005') are valid

Report: PASS/FAIL with specific documentation issues
"
```

**Doc validation script**:
```bash
#!/bin/bash
set -e

echo "=== Version Check ==="
CLAUDE_VERSION=$(grep '^Version' /dev/null 2>&1 || grep -A5 'version 26' CLAUDE.md | head -1)
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1)
echo "Cargo.toml: $CARGO_VERSION"
echo "CLAUDE.md: $CLAUDE_VERSION (should match)"

echo ""
echo "=== Example Compilation ==="
cargo make build-examples

echo ""
echo "=== Doc Tests ==="
cargo test --doc

echo ""
echo "=== Lint Rules vs Docs ==="
echo "Documented panic prevention rule:"
grep -A2 'NEVER.*unwrap' CLAUDE.md | head -3
echo "Actual lint configuration:"
grep -A5 'unwrap_used' Cargo.toml | head -5
```

---

## Integration with CI/CD Pipelines

### GitHub Actions Integration

```yaml
# .github/workflows/agent-validation.yml
name: Specialized Agent Validation

on: [pull_request, push]

jobs:
  macro-review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run MacroReviewAgent checks
        run: |
          cargo expand --lib
          cargo make clippy
          cargo test --lib clap_noun_verb_macros

  test-orchestration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run TestOrchestratorAgent checks
        run: |
          cargo make test-lib-deterministic
          cargo make test-feature-combinations
          timeout 60 cargo make test || exit 1

  architecture-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run ArchitectureGuardian checks
        run: |
          cargo make clippy
          cargo make slo-check
          cargo make check-frontier-all

  performance-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run PerformanceAnalystAgent checks
        run: |
          cargo make bench-baseline
          cargo make slo-check

  doc-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run DocMaintainerAgent checks
        run: |
          cargo make build-examples
          cargo test --doc
```

---

## Agent Communication Patterns

### Serial Workflow (Merge → Release)

```
Developer makes changes
  ↓
MacroReviewAgent (if macros changed)
  ↓
ArchitectureGuardian
  ↓
TestOrchestratorAgent (full matrix)
  ↓
PerformanceAnalystAgent (if perf-critical)
  ↓
DocMaintainerAgent (if docs changed)
  ↓
All checks pass → Merge
  ↓
ReleaseConductorAgent (if release)
  ↓
Published
```

### Parallel Workflow (Quick Checks)

```
Code pushed
  ├─ MacroReviewAgent (parallel)
  ├─ TestOrchestratorAgent (parallel)
  ├─ ArchitectureGuardian (parallel)
  └─ DocMaintainerAgent (parallel)
      ↓
All pass → PerformanceAnalystAgent
      ↓
Release candidate ready
```

---

## Troubleshooting Agent Issues

### Agent Hangs

**Issue**: TestOrchestratorAgent timeout on test-frontier-matrix

**Solution**:
```bash
# Run matrix manually to identify slow test
cargo test --features "frontier-semantic" --quiet
# If hangs, check for infinite loops or network I/O in tests
```

### Agent Finds False Positives

**Issue**: DocMaintainerAgent reports stale version when docs intentionally reference "latest"

**Solution**: Use agent context to clarify expectations
```
"Check version accuracy ONLY for dated entries like 'Version 26.6.1'.
Allow references to 'latest version' or 'current main' without version numbers."
```

### Agent Output Unclear

**Issue**: ArchitectureGuardian reports ADL violations without specific line numbers

**Solution**: Request detailed output
```
"Report each ADL violation with:
- Specific file path
- Line number or code snippet
- Relevant ADL section
- Suggested fix"
```

---

## Customization and Extension

### Adding Phase-Specific Performance Checks

```bash
# Extend PerformanceAnalystAgent for phase-specific benchmarks
cargo make bench-phase1  # Foundation benchmarks
cargo make bench-phase2  # RDF/Semantic benchmarks
cargo make bench-phase3  # Optimization & ML benchmarks
cargo make bench-phase4  # Advanced features benchmarks
```

### Custom Test Scenarios

```bash
# Extend TestOrchestratorAgent for adversarial testing
cargo test --all-features -- --test-threads=1 --nocapture
# Run with environment variable fuzzing
RUST_BACKTRACE=1 cargo test
```

### Andon Signal Protocol (Stop-the-Line)

```bash
# ArchitectureGuardian can invoke Andon checks for critical issues
cargo make andon-check
# This runs:
# - Compiler error detection
# - Compiler warning detection
# - Test failure detection
# - Clippy warning detection
# - Code formatting detection
```

---

## Success Dashboards

### Quick Status Check

```bash
#!/bin/bash
echo "=== Agent Status Dashboard ==="

echo "[MacroReviewAgent] Macro compilation..."
cargo make check > /dev/null && echo "✅ PASS" || echo "❌ FAIL"

echo "[TestOrchestratorAgent] Quick test suite..."
timeout 30 cargo make test > /dev/null && echo "✅ PASS" || echo "❌ FAIL"

echo "[ArchitectureGuardian] SLO validation..."
cargo make slo-check > /dev/null && echo "✅ PASS" || echo "❌ FAIL"

echo "[PerformanceAnalystAgent] Size check..."
SIZE=$(du -m target/release/clap-noun-verb-gen | cut -f1)
[ "$SIZE" -lt 10 ] && echo "✅ PASS ($SIZE MB)" || echo "❌ FAIL ($SIZE MB)"

echo "[DocMaintainerAgent] Doc tests..."
cargo test --doc > /dev/null && echo "✅ PASS" || echo "❌ FAIL"

echo ""
echo "=== Release Ready? ==="
cargo make release-check > /dev/null && echo "✅ READY" || echo "❌ NOT READY"
```

---

## Quick Reference: Agent to Task Mapping

| Agent | Primary Task | Makefile Command | Time Budget |
|-------|--------------|------------------|------------|
| **MacroReviewAgent** | Macro validation | `cargo make clippy` | 5-10s |
| **TestOrchestratorAgent** | Test matrix | `cargo make test-frontier-matrix` | 30-45s |
| **ReleaseConductorAgent** | Publishing | `cargo make publish-all` | 2-5m |
| **ArchitectureGuardian** | ADL compliance | `cargo make lint` | 10-15s |
| **PerformanceAnalystAgent** | Benchmarking | `cargo make bench-compare` | 1-2m |
| **DocMaintainerAgent** | Documentation | `cargo test --doc` | 5-10s |

