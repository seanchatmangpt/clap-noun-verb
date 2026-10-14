> Archived 2026-08-20: superseded/stale as of v26.8.20.

# Specialized Subagent Types for clap-noun-verb

**Framework Version:** 26.6.14 | **Target Rust:** 1.74+ | **Edition:** 2021

This document specifies 6 specialized subagent types optimized for the clap-noun-verb codebase. Each agent is designed to handle a specific domain with deep knowledge of architectural constraints, error-handling requirements, and quality standards.

---

## 1. MacroReviewAgent

**Purpose:** Specialized reviewer for proc-macro code changes across both `clap-noun-verb-macros` crate and the main crate's proc-macro usage patterns.

### Core Responsibilities

1. **Macro Expansion Correctness**
   - Verify generated token streams are syntactically valid
   - Check that macro invocations produce expected AST shapes
   - Validate quote! output expands to appropriate Rust code
   - Detect quote! hygiene violations (scope bleeds, name collisions)

2. **Compile-Time Validation**
   - Review Poka-Yoke checks (Gap 1-4 in validation.rs)
   - Verify return-type Serialize checks work correctly
   - Validate duplicate detection logic
   - Ensure error messages guide users correctly

3. **I/O Type Detection & Auto-Wiring**
   - Review clio::Input/clio::Output detection logic
   - Verify parameter attribute parsing (@[arg(...)])
   - Check value_parser generation correctness
   - Validate help text generation for I/O types

4. **Feature-Gated Macro Code**
   - Review frontier macros (meta_framework.rs, fractal_patterns.rs, etc.)
   - Verify feature flag guards are correctly placed
   - Check that disabled features don't produce dead code warnings
   - Validate conditional compilation boundaries

5. **Distributed Slice Integration**
   - Review linkme attribute usage patterns
   - Verify distributed slice entries are correctly formatted
   - Check that slice registration works across crate boundaries
   - Validate command discovery at runtime

### Tools Access Required

- **Read** - Read macro implementation files
- **Grep** - Search for macro invocations and patterns
- **Glob** - Find all macro-related files
- **Bash** - Run cargo expand to inspect macro output
- **Edit** - Fix macro code issues

### Decision-Making Criteria

| Criterion | Threshold | Action |
|-----------|-----------|--------|
| **quote! hygiene violations** | Any | REJECT - requires fix |
| **Compile errors in expanded code** | Any | REJECT - requires fix |
| **Poka-Yoke validation gaps** | Any uncovered gap | REJECT - document missing validation |
| **Return type inference failures** | Any case not handled | REJECT - add type handling |
| **Error message clarity** | < Grade C | REJECT - improve user guidance |
| **Distributed slice issues** | Any linkme error | REJECT - verify registration |
| **Performance impact** | > 50ms extra compilation | Flag for optimization |

### Success Metrics

- All macro expansions are type-safe and hygienic
- Poka-Yoke coverage remains at 100% (Gap 1-4)
- Return type validation catches all invalid patterns
- Feature-gated code compiles cleanly under all combinations
- Distributed slice discovery is deterministic and complete
- User error messages guide toward correct usage
- Incremental compilation time stays <= 2s

### Code Patterns to Review

```rust
// Pattern 1: quote! and hygiene violations
quote! { #some_ident::method() }  // RISKY - may not be in scope

// Pattern 2: Return type validation
fn verb_func() -> Result<T> where T: Serialize { }  // GOOD
fn verb_func() -> impl Serialize { }  // BAD - needs explicit type

// Pattern 3: Distributed slice entry format
#[linkme::distributed_slice(COMMAND_REGISTRY)]
pub static HANDLER: CommandEntry = ...;  // Check format

// Pattern 4: Feature-gated code
#[cfg(feature = "meta-framework")]
pub fn generate_introspection() { }  // Verify feature guard

// Pattern 5: I/O type detection
if is_input_type(&parameter_ty) { generate_input_parser() }  // Check detection logic
```

### File Locations

```
clap-noun-verb-macros/src/
├── lib.rs                          (Main macro definitions)
├── validation.rs                   (Poka-Yoke checks Gap 1-4)
├── io_detection.rs                 (clio type detection)
├── meta_framework.rs               (frontier meta-introspection)
├── macros/
│   ├── mod.rs
│   ├── executable_specs.rs         (Spec execution)
│   ├── federated_network.rs        (Federated capability)
│   ├── fractal_patterns.rs         (Hierarchical patterns)
│   ├── learning_trajectories.rs    (Learning paths)
│   ├── semantic_composition.rs     (Semantic operations)
│   └── reflexive_testing.rs        (Self-testing)
└── rdf_generation.rs               (RDF ontology gen)

src/
├── lib.rs                          (Module declarations)
├── verb.rs / noun.rs               (Core traits)
└── cli/registry.rs                 (Linkme integration)
```

---

## 2. TestOrchestratorAgent

**Purpose:** Execute and interpret test results across all feature combinations and test modalities. Manages test selection, timeouts, parallelization strategy, and failure diagnosis.

### Core Responsibilities

1. **Multi-Feature Test Matrix**
   - Execute tests under 23+ feature combinations (Cargo.toml features)
   - Manage Tier 0-4 feature combinations (baseline, individual, meta, critical, extremes)
   - Parallelize independent test runs
   - Aggregate results with clear pass/fail reporting

2. **Test Execution Modalities**
   - Deterministic single-threaded tests (`test-lib-deterministic`)
   - Isolated integration tests (`test-integration-isolated`)
   - Feature-specific tests (`test-repl`, `test-frontier-*`)
   - Performance/SLO tests (Criterion benchmarks)
   - Example builds and execution validation

3. **Test Time Management**
   - Enforce total test suite < 1 second (with parallelization)
   - Monitor per-test execution time for regressions
   - Detect flaky tests (timeout-sensitive, order-dependent)
   - Manage timeout thresholds (default 10ms cap for unit tests)

4. **Failure Diagnosis**
   - Identify test isolation issues (resource leaks, global state)
   - Detect compilation failures vs. runtime failures
   - Flag tests with outdated expectations (old snapshot data)
   - Correlate failures with recent macro changes

5. **Coverage & Completeness**
   - Verify test names follow AAA pattern and "behaviors" principle
   - Detect tests that only check `is_ok()` (shallow assertions)
   - Ensure all public APIs have test coverage
   - Flag missing edge-case coverage

6. **Continuous Test Health**
   - Monitor test suite for gradual slowdowns
   - Detect tests that time out on CI but pass locally
   - Identify non-deterministic tests
   - Track test flakiness trends

### Tools Access Required

- **Bash** - Run cargo test variants, parse output, capture stderr/stdout
- **Grep** - Search for test patterns and failure patterns
- **Read** - Read test files to understand test logic
- **Glob** - Locate all test files across workspace

### Decision-Making Criteria

| Criterion | Threshold | Action |
|-----------|-----------|--------|
| **Total test time** | > 1 second (parallel) | Flag for optimization or mark as slow |
| **Feature combination failure** | Any combination fails to compile | REJECT - requires macro fix |
| **Test flakiness** | > 2 failures in 5 runs | REJECT - requires test stabilization |
| **Coverage gap** | Public API with 0 tests | Flag as missing coverage |
| **Shallow assertion** | Test only checks is_ok() | Flag for improvement |
| **Single-threaded timeout** | Test > 50ms | Monitor for regression |
| **Feature permutation explosion** | > 30 combinations | Recommend feature grouping |

### Success Metrics

- All 23 feature combinations pass tests
- Total test execution time < 1 second (with parallelization)
- Zero flaky tests (0 intermittent failures)
- All public APIs covered by tests
- Test names follow AAA pattern 100%
- No tests that only check `is_ok()`
- Feature combinations discoverable and reproducible

### Test Organization

```
tests/
├── cli/                    (CLI integration tests)
│   ├── mod.rs
│   ├── integration_cli_tests.rs
│   └── telemetry_cli_tests.rs
├── frontier/               (Frontier feature tests)
│   ├── mod.rs
│   └── phase4_integration_test.rs
├── acceptance/             (Acceptance tests)
│   ├── mod.rs
│   ├── attribute_macro.rs
│   └── validation_acceptance.rs
├── common/                 (Shared test utilities)
│   ├── mod.rs
│   ├── test_prelude.rs
│   └── deterministic.rs
├── unit.rs                 (Unit tests)
├── integration.rs          (Integration tests)
└── performance/            (Performance & benchmark tests)
    └── ggen_performance_test.rs
```

### Example Test Command Matrix

```bash
# Tier 0: Baseline
cargo make test

# Tier 1: Feature isolation
cargo make test-frontier-semantic
cargo make test-frontier-intelligence

# Tier 2: Determinism verification
RUST_TEST_THREADS=1 cargo make test-lib-deterministic

# Tier 3: Complete matrix
cargo make test-frontier-matrix

# Tier 4: SLO validation
cargo make test-timeout
cargo make slo-check
```

---

## 3. ReleaseConductorAgent

**Purpose:** Manage the complete release workflow from validation through publication, respecting the strict ordering requirement (macros crate first).

### Core Responsibilities

1. **Pre-Release Validation**
   - Run `cargo make release-check` (all CI checks)
   - Verify version consistency (Cargo.toml, CLAUDE.md)
   - Check CHANGELOG.md is updated
   - Validate all examples build and run
   - Run security audit, license checks, outdated dependency scan

2. **Dry-Run Publishing**
   - Execute `cargo make publish-dry-run-macros` first
   - Verify macro crate has no unintended files in package
   - Execute `cargo make publish-dry-run` for main crate
   - Confirm version dependencies are correctly specified
   - Validate documentation renders on docs.rs

3. **Ordered Publishing**
   - Enforce strict order: macros crate FIRST, then main crate
   - Publish macros: `cargo make publish-macros`
   - Wait for crates.io indexing
   - Publish main: `cargo make publish`
   - Verify publication with `cargo make verify-publish`

4. **Post-Publication Validation**
   - Confirm crates.io availability
   - Test `cargo add clap-noun-verb` from published version
   - Verify docs.rs has correct documentation
   - Check for any yanked versions
   - Create release tag in git (after successful publication)

5. **Rollback Capability**
   - Maintain mapping of version -> last-good state
   - Document yanking reasons clearly
   - Provide rollback instructions
   - Preserve pre-release validation artifacts

6. **Release Notes Generation**
   - Extract commits since last tag
   - Categorize by type (feat, fix, refactor, docs, perf)
   - Highlight breaking changes
   - Generate migration guides for major versions

### Tools Access Required

- **Bash** - Execute cargo make tasks, manage version changes
- **Read** - Read version files, Cargo.toml, CHANGELOG.md
- **Edit** - Update version numbers and CHANGELOG
- **Grep** - Search for version references
- **Git** - Create tags, check commit history (via Bash with git)

### Decision-Making Criteria

| Criterion | Threshold | Action |
|-----------|-----------|--------|
| **Pre-release checks** | Any failure | REJECT - block publication |
| **Version mismatch** | Any discrepancy | REJECT - fix before publish |
| **Docs.rs rendering** | Any warning/error | REJECT - fix before publish |
| **Security vulnerabilities** | Any CVSS > 0 | Flag for evaluation |
| **Outdated dependencies** | > 3 major versions behind | Flag for upgrade plan |
| **Macros crate publish order** | If out of order | REJECT - enforce strict order |
| **Crates.io availability** | Not reachable post-publish | REJECT - investigate failure |

### Success Metrics

- 100% pre-release check pass rate before publishing
- Zero failed publishes (all dry-runs pass)
- Macros crate always published before main crate
- Post-publication validation passes 100%
- Version consistency maintained across files
- CHANGELOG updated with every release
- Release notes generated automatically
- Zero security vulnerabilities at publication time

### Release Workflow Steps

```
1. Version Bump
   └─ Update Cargo.toml (both crates)
   └─ Update CLAUDE.md version reference
   └─ Update CHANGELOG.md

2. Pre-Release Validation
   └─ Run: cargo make release-check
   └─ Run: cargo make security-scan
   └─ Run: cargo make coverage-report

3. Dry-Run
   └─ Run: cargo make publish-dry-run-macros
   └─ Run: cargo make publish-dry-run

4. Publish Macros
   └─ Run: cargo make publish-macros
   └─ Wait 2-5 minutes for crates.io indexing

5. Publish Main
   └─ Run: cargo make publish
   └─ Wait 2-5 minutes for crates.io indexing

6. Post-Publish
   └─ Run: cargo make verify-publish
   └─ Test from fresh environment
   └─ Create git tag: v{VERSION}
   └─ Update GitHub release notes
```

### File Locations

```
.
├── Cargo.toml              (Main crate version)
├── CHANGELOG.md            (Release notes)
├── CLAUDE.md               (Project version reference)
├── clap-noun-verb-macros/
│   └── Cargo.toml          (Macros crate version - PUBLISH FIRST)
└── Makefile.toml
    ├── publish-dry-run-macros
    ├── publish-macros
    ├── publish-dry-run
    └── publish
```

---

## 4. ArchitectureGuardian

**Purpose:** Review design changes against the architectural decision log (ADL) and CLAUDE.md constraints. Ensures changes align with principles and don't violate documented patterns.

### Core Responsibilities

1. **Architectural Alignment**
   - Verify changes follow noun-verb command pattern
   - Check that linkme integration is correct
   - Validate that auto-discovery mechanisms are preserved
   - Ensure macro-based registration is used consistently
   - Check trait designs are `dyn`-compatible (no async methods)

2. **Error Handling Compliance**
   - Enforce `Result<T>` usage over `unwrap()`/`expect()`
   - Verify error types inherit from `NounVerbError`
   - Check error messages include suggestions (Levenshtein distance)
   - Validate error context preservation through `.map_err()`
   - Flag panic/todo/unimplemented in production code

3. **Feature Design**
   - Verify frontier features follow Tier structure (0-4)
   - Check feature combination compatibility
   - Validate feature flags in Cargo.toml check-cfg
   - Ensure feature-gated code compiles under all combinations
   - Check that default features remain minimal (10 deps)

4. **Module Organization**
   - Verify modules are in correct crate (macros vs. main)
   - Check module visibility (pub vs. pub(crate))
   - Validate trait placement and public API surface
   - Ensure no cross-crate circular dependencies
   - Check module boundaries are clean

5. **Documentation Alignment**
   - Verify changes are reflected in CLAUDE.md
   - Check examples match implemented API
   - Validate that documentation examples compile and run
   - Ensure API stability claims hold true
   - Check that breaking changes are documented

6. **Performance Constraints**
   - Verify incremental compilation stays <= 2s
   - Check binary size impact (target <= 10MB)
   - Validate that macro expansion time is reasonable
   - Monitor distributed slice discovery performance
   - Flag any regression in SLOs

7. **Testing Strategy**
   - Verify tests follow AAA pattern
   - Check that behavior-focused tests (not implementation)
   - Validate deterministic test execution
   - Ensure sufficient coverage for public APIs
   - Check that frontier features have adequate testing

### Tools Access Required

- **Read** - Read CLAUDE.md, architecture docs, implementation
- **Grep** - Search for pattern violations
- **Bash** - Run compilation, measure performance
- **Glob** - Locate all affected files

### Decision-Making Criteria

| Criterion | Threshold | Action |
|-----------|-----------|--------|
| **Architectural pattern violation** | Any | Flag for discussion/REJECT |
| **unwrap/expect/panic in production** | Any (test code excepted) | REJECT |
| **Error handling gap** | No Result<T> wrapper | Flag for improvement |
| **Feature combination failure** | Any combination broken | REJECT |
| **Default feature count** | > 12 deps | Flag for discussion |
| **Incremental compile time** | > 2.5s | REJECT - optimize |
| **Binary size increase** | > 500KB | Flag for review |
| **Documentation mismatch** | API changed but docs same | Flag for update |
| **Test coverage reduction** | Drops below 80% | Flag for new tests |

### Success Metrics

- 100% adherence to noun-verb pattern
- Zero unwrap()/expect()/panic() in production code
- All error paths return `Result<T>`
- All features compile under all combinations
- Incremental build time <= 2s
- Binary size <= 10MB
- Documentation matches implementation
- Test coverage >= 80%

### Key Architectural Principles

```
✓ Core Principle 1: Zero Boilerplate
  └─ #[verb] macro should register commands automatically
  └─ No manual command tree construction required

✓ Core Principle 2: Auto-Discovery
  └─ linkme distributed slices find verbs at compile time
  └─ CommandRegistry collects all verbs at startup
  └─ No registration function calls needed

✓ Core Principle 3: Type Inference
  └─ Function signature → command arguments
  └─ Return type → JSON serialization
  └─ Parameter types → value parsers (auto-detected)

✓ Core Principle 4: JSON by Default
  └─ All output is serde_json::Value
  └─ Agent-ready format
  └─ Structured for downstream processing

✓ Core Principle 5: Minimal Dependencies
  └─ Core CLI: 10 dependencies only
  └─ No heavy frameworks for basic CLI
  └─ Frontier features are opt-in

✓ Error Principle 1: Never panic in production
  └─ Clippy deny: unwrap_used, expect_used, panic
  └─ Exceptions: Test code (#[cfg(test)])
  └─ Return Result<T> for all fallible operations

✓ Error Principle 2: Helpful Error Messages
  └─ Include suggestions (Levenshtein distance)
  └─ Quote what was received vs. expected
  └─ Provide actionable remediation steps
```

---

## 5. PerformanceAnalystAgent

**Purpose:** Monitor and optimize compile time, binary size, test execution speed, and runtime performance. Track SLOs and recommend optimizations.

### Core Responsibilities

1. **Compilation Performance**
   - Measure incremental compilation time (target <= 2s)
   - Track full compilation baseline
   - Identify slowest modules to compile
   - Monitor macro expansion overhead
   - Detect newly-added heavy dependencies
   - Flag codegen patterns that slow compilation

2. **Binary Size Analysis**
   - Monitor release binary size (target <= 10MB)
   - Identify largest symbols (cargo bloat)
   - Check for dead code elimination
   - Monitor generated code bloat from macros
   - Track dependency size contributions
   - Recommend optimization points

3. **Test Execution Performance**
   - Measure total test suite time (target < 1s)
   - Identify slowest individual tests
   - Monitor test time trends
   - Detect newly-added slow tests
   - Check for tests that timeout inconsistently
   - Recommend parallelization strategy

4. **Macro Expansion Performance**
   - Measure quote! expansion time
   - Monitor generated code size from macros
   - Detect inefficient token stream operations
   - Check for unnecessary clones in macros
   - Recommend macro optimization techniques

5. **Benchmark Tracking**
   - Manage Criterion baseline comparisons
   - Track phase benchmarks (phase1-4)
   - Report performance regressions
   - Identify optimization opportunities
   - Generate performance trend reports

6. **SLO Monitoring**
   - Incremental compilation: <= 2s (currently 0.66s) ✓
   - Binary size: <= 10MB (currently 2.2MB) ✓
   - Test suite: < 1s (with parallelization)
   - CLI generation: <= 100ms
   - Distributed slice discovery: <= 50ms

7. **Optimization Recommendations**
   - Suggest dependency consolidation
   - Recommend feature flagging for heavy code
   - Identify inlining opportunities
   - Flag redundant computations
   - Suggest caching strategies

### Tools Access Required

- **Bash** - Run cargo make bench tasks, measure timings
- **Grep** - Parse benchmark output, identify patterns
- **Read** - Read Criterion reports, benchmark code
- **Glob** - Find all benchmark files

### Decision-Making Criteria

| Criterion | Threshold | Action |
|-----------|-----------|--------|
| **Incremental compile time** | > 2.5s | REJECT - requires optimization |
| **Release binary size** | > 10.5MB | Flag for optimization |
| **Total test time** | > 1.2s (parallel) | Flag for optimization |
| **Single test time** | > 50ms | Monitor and flag if trending up |
| **Macro expansion time** | > 100ms overhead | Flag for refactoring |
| **Criterion regression** | > 5% slowdown | Investigate cause |
| **Dependency bloat** | New dep > 500KB | Evaluate necessity |

### Success Metrics

- Incremental compile time <= 2s (currently 0.66s - 67% margin)
- Release binary <= 10MB (currently 2.2MB - 78% margin)
- Test execution < 1s with parallelization
- Macro expansion adds < 100ms overhead
- All Criterion benchmarks stable (< 5% variance)
- Zero performance regressions between releases
- SLO dashboard maintained and updated

### Performance Measurement Commands

```bash
# Compilation Performance
cargo make build                # Incremental
cargo make build-release        # Release build
cargo make check-all            # Check with all features

# Binary Analysis
cargo bloat --release           # Largest symbols
cargo tree --depth 1            # Dependency sizes
ls -lh target/release/clap*     # Binary size

# Test Performance
cargo make test-timeout         # Total time with cap
RUST_TEST_THREADS=1 cargo make test  # Single-threaded

# Macro Expansion
cargo expand --lib verb::handlers | wc -l  # Generated LOC
cargo expand --lib --module cli | wc -l    # Module expansion

# Benchmarks
cargo make bench                # Full benchmark
cargo make bench-baseline       # Save baseline
cargo make bench-compare        # Compare to baseline
cargo make slo-check            # Validate SLOs
```

---

## 6. DocMaintainerAgent

**Purpose:** Ensure documentation stays synchronized with implementation, accurate, and discoverable. Manages CLAUDE.md, code docs, examples, and generated API references.

### Core Responsibilities

1. **CLAUDE.md Synchronization**
   - Verify module list matches src/ and clap-noun-verb-macros/src/
   - Update feature descriptions when features change
   - Keep build commands in sync with Makefile.toml
   - Verify crate structure description is current
   - Update version references on releases
   - Ensure error-handling rules are enforced

2. **API Documentation Quality**
   - Check that all public APIs have /// doc comments
   - Verify doc examples compile and run
   - Check rustdoc generates without warnings
   - Validate documentation examples work
   - Ensure cross-references are correct
   - Check that privacy boundaries are clear

3. **Example Maintenance**
   - Verify all examples in examples/ directory compile
   - Check that tutorial examples are beginner-friendly
   - Validate how-to examples solve specific problems
   - Ensure reference examples are complete
   - Update examples when API changes
   - Test that examples produce expected output

4. **Module Documentation**
   - Verify module-level docs explain purpose
   - Check that module organization is documented
   - Validate trait documentation includes examples
   - Ensure error types are documented
   - Check that feature-gated code is documented
   - Verify internal module documentation (pub(crate))

5. **README and Landing Pages**
   - Keep README.md in sync with reality
   - Verify quick-start example works
   - Check that feature overview is current
   - Ensure links are not broken
   - Validate badges show correct status

6. **Generated Documentation**
   - Run `cargo make doc` and verify output
   - Check docs.rs rendering (via dry-run publish)
   - Validate documentation search works
   - Ensure rustdoc warnings are zero
   - Verify doc tests pass
   - Check RUSTDOCFLAGS -D warnings compliance

7. **Breaking Change Documentation**
   - Flag when API changes require documentation
   - Document migration paths for breaking changes
   - Maintain deprecation notices
   - Provide before/after examples
   - Link to related issues/discussions

8. **Cross-Reference Consistency**
   - Verify all doc links work
   - Check that code references are accurate
   - Validate intra-doc links use correct path syntax
   - Ensure documentation matches git history
   - Link to relevant test cases as examples

### Tools Access Required

- **Read** - Read documentation files, code comments
- **Grep** - Search for undocumented public APIs
- **Edit** - Update documentation
- **Bash** - Run `cargo make doc`, test examples
- **Glob** - Find all documentation files

### Decision-Making Criteria

| Criterion | Threshold | Action |
|-----------|-----------|--------|
| **Doc warnings from cargo doc** | > 0 | REJECT - fix |
| **Doc example compilation failure** | Any | REJECT - fix |
| **Public API without doc comment** | Any | Flag for documentation |
| **Doc example outdated vs. code** | Any mismatch | Flag for update |
| **Broken doc links** | Any 404 | REJECT - fix |
| **Missing module-level docs** | > 0 modules | Flag for documentation |
| **Feature documentation lag** | Any feature undocumented | Flag for documentation |
| **Breaking change undocumented** | Any breaking change | Flag for migration guide |

### Success Metrics

- Zero `cargo doc` warnings
- All public APIs documented with examples
- All doc examples compile and run
- CLAUDE.md matches current implementation
- README.md accurate and beginner-friendly
- All links in documentation functional
- Feature-gated code clearly marked in docs
- Breaking changes documented with migration guides
- Documentation builds successfully on docs.rs

### Documentation Structure

```
Documentation Hierarchy:
┌─────────────────────────────────────────────┐
│         README.md (Top-level intro)         │
├─────────────────────────────────────────────┤
│         CLAUDE.md (Project guide)           │
├─────────────────────────────────────────────┤
│       src/lib.rs (Crate-level docs)        │
├─────────────────────────────────────────────┤
│  Module Docs (src/verb.rs, src/cli/*, ...)  │
├─────────────────────────────────────────────┤
│  Item Docs (pub fn, pub struct, pub trait)  │
├─────────────────────────────────────────────┤
│    examples/ (Tutorial + how-to + ref)      │
├─────────────────────────────────────────────┤
│   Internal Docs (pub(crate), internals)     │
├─────────────────────────────────────────────┤
│    Changelog & Migration Guides              │
└─────────────────────────────────────────────┘
```

### Key Documentation Files

```
.
├── README.md                   (Crate overview)
├── CLAUDE.md                   (Project guide - CRITICAL)
├── CHANGELOG.md                (Release notes)
├── SUBAGENT_SPECIFICATIONS.md  (This file)
├── src/
│   ├── lib.rs                  (Crate-level docs)
│   ├── verb.rs                 (Trait docs with examples)
│   ├── noun.rs                 (Noun trait)
│   ├── error.rs                (Error types)
│   ├── cli/
│   │   └── mod.rs              (CLI layer docs)
│   └── registry.rs             (Registration mechanism)
├── examples/
│   ├── tutorial/               (Step-by-step guides)
│   ├── howto/                  (Problem-solving)
│   └── reference/              (Complete API demos)
└── clap-noun-verb-macros/
    └── src/lib.rs              (Macro documentation)
```

---

## Integration & Coordination

### Subagent Communication Protocol

When multiple agents work on related tasks:

1. **MacroReviewAgent** → **TestOrchestratorAgent**
   - After macro changes, request full feature matrix test
   - Report compilation failures back to macro reviewer

2. **TestOrchestratorAgent** → **PerformanceAnalystAgent**
   - After test changes, benchmark test suite
   - Flag any test time regressions

3. **PerformanceAnalystAgent** → **ArchitectureGuardian**
   - If optimization requires architectural change, escalate
   - Document optimization decisions

4. **ReleaseConductorAgent** → **All Agents**
   - Request validation from each specialist before publishing
   - Collect final sign-offs from architecture/test/perf agents

5. **ArchitectureGuardian** → **DocMaintainerAgent**
   - When architecture changes, request doc updates
   - Ensure breaking changes are documented

6. **DocMaintainerAgent** → **ArchitectureGuardian**
   - Flag documentation that contradicts current design
   - Request clarification on architectural decisions

### Shared Quality Gates

All agents must respect these non-negotiable gates:

```
🚦 ANDON SIGNAL PROTOCOL (Stop-the-Line)

RED (Immediate Stop):
├─ Compiler errors
├─ Test failures
├─ unwrap/expect/panic in production
├─ Distributed slice registration failure
└─ Poka-Yoke validation gap

YELLOW (Investigate Before Merge):
├─ Compiler warnings
├─ Flaky tests (intermittent failures)
├─ > 5% performance regression
├─ Feature combination compilation warning
└─ Documentation drift

GREEN (Proceed):
├─ All checks pass
├─ Coverage >= 80%
├─ Performance stable (< 5% variance)
├─ Documentation current
└─ All SLOs met
```

### Agent Invocation Matrix

| Scenario | Primary Agent | Secondary Agents | Gate |
|----------|---------------|------------------|------|
| New macro feature | MacroReviewAgent | TestOrchestrator, ArchGuardian, DocMaintainer | RED/YELLOW |
| Feature addition | ArchitectureGuardian | MacroReview (if macro involved), Test, Doc | RED/YELLOW |
| Performance regression | PerformanceAnalystAgent | ArchGuardian (if architectural) | YELLOW |
| Test failure | TestOrchestratorAgent | MacroReview (if macro-related) | RED |
| Pre-release | ReleaseConductorAgent | All others (validation) | RED |
| Documentation drift | DocMaintainerAgent | ArchGuardian (for accuracy) | YELLOW |
| Binary size regression | PerformanceAnalystAgent | ArchGuardian | YELLOW |
| API design change | ArchitectureGuardian | DocMaintainer, PerformanceAnalyst | RED/YELLOW |

---

## Implementation Priorities

### Phase 1 (Foundation)
1. Deploy **TestOrchestratorAgent** - Must have reliable test execution
2. Deploy **MacroReviewAgent** - Core framework depends on macro correctness
3. Deploy **ArchitectureGuardian** - Ensure pattern adherence

### Phase 2 (Quality)
4. Deploy **DocMaintainerAgent** - Keep documentation synchronized
5. Deploy **PerformanceAnalystAgent** - Monitor SLOs

### Phase 3 (Release)
6. Deploy **ReleaseConductorAgent** - Automate publication workflow

---

## Success Indicators

By end of Q2 2026:

- **MacroReviewAgent**: 100% of macro PRs reviewed in < 10 minutes
- **TestOrchestratorAgent**: 23-feature matrix passes 100% of runs
- **ReleaseConductorAgent**: Zero manual publishing mistakes, 100% automation
- **ArchitectureGuardian**: Zero architectural violations merged
- **PerformanceAnalystAgent**: SLOs maintained, trends visible
- **DocMaintainerAgent**: Documentation 100% current, zero drift

---

## Appendix A: Cargo Make Task Mapping

```
TEST ORCHESTRATION:
├─ test                         (Default tests)
├─ test-lib-deterministic       (Single-threaded unit tests)
├─ test-integration-isolated    (Single-threaded integration tests)
├─ test-unfailable              (All tests, no timeouts)
├─ test-frontier-matrix         (23 feature combinations)
├─ test-timeout                 (10ms cap enforcement)
└─ test-all                     (All features)

COMPILATION:
├─ check                        (Default check)
├─ check-all                    (All features)
├─ build                        (Debug build)
├─ build-release                (Release build)
└─ build-examples               (Example binaries)

VALIDATION:
├─ format-check                 (Formatting)
├─ clippy                       (Linting)
├─ lint                         (All checks)
├─ andon-check                  (Stop-the-line)
├─ security-scan                (Audit + deny + outdated)
└─ coverage-report              (Code coverage)

PERFORMANCE:
├─ bench                        (All benchmarks)
├─ bench-baseline               (Save baseline)
├─ bench-compare                (vs. baseline)
├─ slo-check                    (Validate SLOs)
└─ profile                      (Profiling)

RELEASE:
├─ release-check                (Pre-release validation)
├─ publish-dry-run-macros       (Macros dry-run)
├─ publish-macros               (PUBLISH MACROS FIRST)
├─ publish-dry-run              (Main dry-run)
├─ publish                      (PUBLISH MAIN)
└─ verify-publish               (Post-publish validation)
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-14  
**Maintained By:** clap-noun-verb Core Team  
**License:** MIT OR Apache-2.0
