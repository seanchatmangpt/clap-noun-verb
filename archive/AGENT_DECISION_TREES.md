# Agent Decision Trees and Tactical Playbooks

This document provides decision trees and tactical playbooks for each agent to use when encountering common scenarios.

---

## MacroReviewAgent Decision Tree

### Scenario: New Proc-Macro Change Detected

```
START: Macro code change in clap-noun-verb-macros/
│
├─ Is this a new macro (#[xxx])?
│  │
│  ├─ YES: Check distributed_slice generation
│  │  │
│  │  ├─ Does macro use linkme::distributed_slice! correctly?
│  │  │  ├─ YES → Check token safety (next step)
│  │  │  └─ NO → FAIL: Document required linkme changes, suggest fix
│  │  │
│  │  └─ Generate expansion: cargo expand --lib
│  │     ├─ Inspect generated code for span preservation
│  │     └─ Run: cargo make clippy (deny rules)
│  │
│  └─ NO: Is this a fix to existing macro?
│     │
│     └─ Run: cargo make check && cargo make clippy
│        ├─ All pass? → PASS: Verify test expansion
│        └─ Violations? → FAIL: List specific violations
│
├─ Check error messages
│  │
│  ├─ Are error messages actionable?
│  │  ├─ YES: Include code snippet in error
│  │  └─ NO: → FAIL: Rewrite error with example
│  │
│  └─ Example: #[verb] on non-async fn should show correct syntax
│
├─ Verify compile-time validation completeness
│  │
│  ├─ Does it detect forgotten #[verb]?
│  ├─ Does it detect duplicate verbs?
│  └─ Does it check return type implements Serialize?
│     ├─ All YES? → PASS
│     └─ Any NO? → FAIL: Add validation
│
├─ Measure impact on SLO
│  │
│  ├─ Incremental build time regression?
│  │  ├─ <5% → PASS
│  │  └─ >=5% → FAIL: Optimize macro expansion
│  │
│  └─ Feature-gated macro properly cfg'd?
│     ├─ YES → PASS
│     └─ NO → FAIL: Add #[cfg(feature = "...")] guard
│
└─ DECISION: PASS all checks OR list specific failures and fixes
```

### Tactical Decisions

| Situation | Decision | Action |
|-----------|----------|--------|
| Macro expansion uses temp variables | Hygiene concern | Verify no name collisions; ensure 100% namespaced |
| Error message unclear to users | UX issue | Rewrite to include code example and link to docs |
| Compile time increases 3% | Within threshold | Log in commit; monitor trend |
| Compile time increases 10% | Over threshold | Reject; optimize or redesign macro |
| New macro adds validation gap | Quality issue | Implement poka-yoke check before approving |

---

## TestOrchestratorAgent Decision Tree

### Scenario: Test Suite Execution

```
START: Run test command
│
├─ Quick Tests (cargo make test)
│  │
│  ├─ Execution time <1000ms?
│  │  ├─ YES → PASS: Log time, continue
│  │  └─ NO → YELLOW: Identify slow test, flag for next review
│  │
│  └─ All tests pass?
│     ├─ YES → Continue to deterministic
│     └─ NO → FAIL: List failing tests with assertion details
│
├─ Deterministic Tests (cargo make test-lib-deterministic)
│  │
│  ├─ Single-threaded results == parallel results?
│  │  ├─ YES → PASS: No flakiness detected
│  │  └─ NO → FAIL: Flaky test detected; requires fix before merge
│  │
│  └─ Test list:
│     ├─ All follow AAA pattern? (Arrange/Act/Assert)
│     ├─ All assert on behavior (not just is_ok())?
│     ├─ No time-dependent logic (sleep, fixed delays)?
│     ├─ No shared mutable state between tests?
│     └─ Any NO? → FAIL: Improve test quality
│
├─ Feature Matrix Tests (cargo make test-frontier-matrix)
│  │
│  ├─ For each of 23 feature combinations:
│  │  │
│  │  ├─ Tests compile?
│  │  │  ├─ YES → Run tests
│  │  │  └─ NO → FAIL: List compilation errors for feature combo
│  │  │
│  │  └─ Tests pass?
│  │     ├─ YES → Record as passing
│  │     └─ NO → FAIL: List failures by feature
│  │
│  └─ All 23/23 passing?
│     ├─ YES → PASS
│     └─ NO → FAIL: List which combos failed
│
├─ Flakiness Detection
│  │
│  ├─ Run deterministic suite 3 times
│  │  ├─ Identical results all 3 times?
│  │  │  ├─ YES → PASS: Tests are deterministic
│  │  │  └─ NO → FAIL: List flaky tests, require fixes
│  │
│  └─ Run under stress (RUST_TEST_THREADS=1 with high load)?
│     ├─ All pass under stress?
│     │  ├─ YES → PASS
│     │  └─ NO → FAIL: Tests fail under contention
│
├─ Code Quality Assessment
│  │
│  ├─ Grep for test anti-patterns:
│  │  │
│  │  ├─ Any bare assert!(result.is_ok())?
│  │  │  └─ YES → YELLOW: Ask to verify actual behavior assertion
│  │  │
│  │  ├─ Any #[tokio::test] without proper assertion?
│  │  │  └─ YES → YELLOW: Review for meaningful assertions
│  │  │
│  │  ├─ Any std::thread::sleep()?
│  │  │  └─ YES → FAIL: Replace with proper event/assertion
│  │  │
│  │  └─ Any unwrap()/expect() in tests?
│  │     └─ PASS (allowed in tests; verify used appropriately)
│  │
│  └─ All checks clean? → PASS: Tests are high-quality
│
└─ FINAL DECISION: PASS, YELLOW (warning), or FAIL with specific issues
```

### Tactical Decisions

| Situation | Decision | Action |
|-----------|----------|--------|
| Test passes parallel but fails serial | Flaky test | Rerun 10x serially; if any fail, requires fix |
| Test asserts only is_ok() | Weak test | Ask dev to verify actual output values |
| Feature combo fails to compile | Feature interaction | Investigate feature dependencies |
| Test takes 200ms | Slow but acceptable | Log; if >500ms total, investigate |
| 22/23 features pass | Almost there | List which combo fails; requires fix |
| Entire suite <500ms | Excellent | Green light, merge-ready |

---

## ReleaseConductorAgent Decision Tree

### Scenario: Release Workflow Execution

```
START: Release requested
│
├─ PRE-FLIGHT CHECKS
│  │
│  ├─ Versions synchronized?
│  │  │
│  │  ├─ Read clap-noun-verb-macros/Cargo.toml → extract version
│  │  ├─ Read clap-noun-verb/Cargo.toml → extract version
│  │  │
│  │  ├─ Versions match (e.g., both 26.6.2)?
│  │  │  ├─ YES → Continue
│  │  │  └─ NO → FAIL: Update both to same version, provide command
│  │
│  ├─ CHANGELOG updated?
│  │  │
│  │  ├─ CHANGELOG.md exists?
│  │  │  ├─ YES → Check for this version entry
│  │  │  └─ NO → FAIL: Create CHANGELOG.md with entry
│  │  │
│  │  └─ Version entry has user-facing changes?
│  │     ├─ YES → Continue
│  │     └─ NO → FAIL: Add meaningful changelog entry
│  │
│  ├─ Git status clean?
│  │  │
│  │  ├─ Uncommitted changes?
│  │  │  ├─ NO → Continue
│  │  │  └─ YES → FAIL: Commit or stash changes first
│  │  │
│  │  └─ All changes on main/release branch?
│  │     ├─ YES → Continue
│  │     └─ NO → FAIL: Merge branch first
│
├─ VALIDATION PHASE
│  │
│  ├─ Run: cargo make release-check
│  │  │
│  │  ├─ All checks pass? (format, clippy, tests, build, docs)
│  │  │  ├─ YES → Continue
│  │  │  └─ NO → FAIL: List specific failures, require fixes
│  │  │
│  │  └─ SLO met? (compile <=2s, binary <=10MB)
│  │     ├─ YES → Continue
│  │     └─ NO → FAIL: Performance regression; requires optimization
│
├─ DRY-RUN PHASE
│  │
│  ├─ Run: cargo make publish-dry-run-macros
│  │  │
│  │  ├─ Succeeds (no errors)?
│  │  │  ├─ YES → Continue
│  │  │  └─ NO → FAIL: List errors (usually dependency or manifest issues)
│  │  │
│  │  └─ Warnings (deprecation, unused)?
│  │     ├─ Critical → FAIL: Fix warnings
│  │     └─ Non-critical → YELLOW: Log for next review
│  │
│  ├─ Run: cargo make publish-dry-run
│  │  │
│  │  ├─ Succeeds (no errors)?
│  │  │  ├─ YES → Ready to publish
│  │  │  └─ NO → FAIL: List errors (check macros are published first)
│
├─ PUBLISH PHASE
│  │
│  ├─ MACROS FIRST (required: main crate depends on macros)
│  │  │
│  │  ├─ Run: cargo make publish-macros
│  │  │  │
│  │  │  ├─ Succeeds?
│  │  │  │  ├─ YES → Wait 60 seconds for index sync
│  │  │  │  └─ NO → FAIL: Diagnose; usually manifest or auth issue
│  │  │  │
│  │  │  └─ Verify on crates.io
│  │  │     └─ `cargo search clap-noun-verb-macros` shows new version?
│  │  │        ├─ YES → Continue
│  │  │        └─ NO → Wait up to 5 minutes, recheck
│  │
│  ├─ MAIN CRATE
│  │  │
│  │  ├─ Run: cargo make publish
│  │  │  │
│  │  │  ├─ Succeeds?
│  │  │  │  ├─ YES → Log success, wait 60 seconds
│  │  │  │  └─ NO → FAIL: Diagnose; check macros published first
│  │  │  │
│  │  │  └─ Verify on crates.io
│  │  │     └─ `cargo search clap-noun-verb` shows new version?
│  │  │        ├─ YES → Continue
│  │  │        └─ NO → Wait up to 5 minutes, recheck
│
├─ GIT TAGGING
│  │
│  ├─ Create tag: git tag v<version>
│  │  │
│  │  ├─ Tag created successfully?
│  │  │  ├─ YES → Push tag
│  │  │  └─ NO → FAIL: Debug git issue
│  │
│  └─ Push: git push origin v<version>
│     │
│     ├─ Push succeeds?
│     │  ├─ YES → Release complete
│     │  └─ NO → FAIL: Check git auth/permissions
│     │
│     └─ Tag visible on GitHub?
│        ├─ YES → Release successful
│        └─ NO → Wait 1-2 minutes, recheck
│
└─ FINAL DECISION: Release successful with publish URLs, or FAIL with remediation steps
```

### Tactical Decisions

| Situation | Decision | Action |
|-----------|----------|--------|
| Versions don't match | Blocker | Run: `sed -i 's/26.6.1/26.6.2/g' */Cargo.toml` (adjust version) |
| CHANGELOG missing | Blocker | Require human to write changelog entry |
| Release check fails | Blocker | Identify test/lint failures; require fixes |
| Dry-run fails (manifest issue) | Blocker | Check Cargo.toml syntax; likely metadata/keywords issue |
| Macros publish but main fails | Critical | Verify macros crate published; wait 60s for index sync |
| crates.io shows old version | Wait scenario | Recheck every 30 seconds for up to 5 minutes |
| Tag push fails | Auth issue | Verify GitHub token/SSH key; escalate if needed |

---

## ArchitectureGuardian Decision Tree

### Scenario: Code Change Architecture Review

```
START: Code change to review
│
├─ ADL-001: Noun-Verb Pattern?
│  │
│  ├─ New command added?
│  │  │
│  │  ├─ Does it follow "noun verb" pattern?
│  │  │  ├─ YES (e.g., "services status") → PASS
│  │  │  └─ NO (e.g., "get-services") → FAIL: Suggest noun-verb structure
│  │  │
│  │  └─ Single-level commands (no deep nesting)?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Noun-verb is 2 levels; deeper nesting violates ADL
│
├─ ADL-002: Proc-Macro Design?
│  │
│  ├─ New macros added?
│  │  │
│  │  ├─ Use linkme::distributed_slice!?
│  │  │  ├─ YES → PASS: Compile-time discovery pattern correct
│  │  │  └─ NO → FAIL: Suggest distributed_slice approach
│  │  │
│  │  └─ No manual registry boilerplate?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Distributed discovery should eliminate boilerplate
│
├─ ADL-003: JSON Output Format?
│  │
│  ├─ New verb added?
│  │  │
│  │  ├─ Return type implements Serialize?
│  │  │  │
│  │  │  ├─ Grep for verb function
│  │  │  ├─ Extract return type
│  │  │  ├─ Verify #[derive(Serialize)] or serde_json::Value
│  │  │  │
│  │  │  ├─ YES → PASS: Default JSON output enabled
│  │  │  └─ NO → FAIL: Add Serialize derive or return JSON type
│  │  │
│  │  └─ No custom text/binary formatters in core?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Keep output format neutral; let users format as needed
│
├─ ADL-004: Async/Sync Verbs?
│  │
│  ├─ New verb signature?
│  │  │
│  │  ├─ Is async fn?
│  │  │  ├─ YES → PASS: Async-first is correct
│  │  │  └─ NO: Is it sync by necessity?
│  │  │     ├─ YES (CPU-bound, no I/O) → PASS
│  │  │     └─ NO (I/O-bound) → WARN: Consider async for I/O operations
│  │  │
│  │  └─ No async in core traits?
│  │     ├─ YES → PASS: Traits remain dyn compatible
│  │     └─ NO → FAIL: Move async to async_verb.rs module, feature-gated
│
├─ ADL-005: No Panics in Production?
│  │
│  ├─ Grep for panic/unwrap/expect in src/ (not tests/)
│  │  │
│  │  ├─ Any instances found?
│  │  │  ├─ NO → PASS: Error handling is correct
│  │  │  └─ YES: Are they in production code?
│  │  │     │
│  │  │     ├─ In src/bin/ only? → PASS: Binaries can panic
│  │  │     ├─ In tests? → PASS: Tests can panic
│  │  │     └─ In src/lib.rs or modules? → FAIL: Replace with Result<T>
│  │  │
│  │  └─ Run: cargo make clippy
│  │     ├─ Deny rules pass? → PASS
│  │     └─ Violations? → FAIL: List and fix
│
├─ ADL-006: Feature Gating?
│  │
│  ├─ Frontier feature code added?
│  │  │
│  │  ├─ Code wrapped in #[cfg(feature = "...")] guard?
│  │  │  ├─ YES → PASS: Feature properly gated
│  │  │  └─ NO → FAIL: Gate frontier features; they shouldn't affect core
│  │  │
│  │  └─ Feature declared in Cargo.toml [features]?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Declare feature in Cargo.toml
│
├─ ADL-007: Minimalist Core?
│  │
│  ├─ New modules added to src/?
│  │  │
│  │  ├─ Is this a core feature (verb routing, registry)?
│  │  │  ├─ YES → PASS: Core modules justified
│  │  │  └─ NO: Is it essential?
│  │  │     │
│  │  │     ├─ YES → Propose as new core module (requires ADL discussion)
│  │  │     └─ NO → FAIL: Gate as frontier feature, not core
│  │  │
│  │  └─ Total optional modules in src/ (not counting core)?
│  │     ├─ Currently 2: async_verb.rs, federation/
│  │     ├─ Adding more? → FAIL: Explain necessity
│  │     └─ At/below limit? → PASS
│
├─ ADL-008: Distributed Slice Registration?
│  │
│  ├─ Verb registration using distributed slices?
│  │  │
│  │  ├─ Use #[verb] macro?
│  │  │  ├─ YES → PASS: Automatic discovery enabled
│  │  │  └─ NO: Manual registry.register() calls?
│  │  │     └─ YES → FAIL: Use #[verb] macro for distributed discovery
│  │  │
│  │  └─ CommandRegistry correctly collects entries?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Verify CommandRegistry implementation
│
├─ ADL-009: SLO Targets?
│  │
│  ├─ Dependency or code change affecting performance?
│  │  │
│  │  ├─ Measure: touch src/lib.rs && time cargo make build
│  │  │  │
│  │  │  ├─ <=2000ms? → PASS
│  │  │  └─ >2000ms? → FAIL: Optimization required before merge
│  │  │
│  │  └─ Measure: cargo make build-release && ls -lh target/release/binary
│  │     │
│  │     ├─ <=10MB? → PASS
│  │     └─ >10MB? → FAIL: Size regression; requires investigation
│
├─ ADL-010: Trait Design?
│  │
│  ├─ New trait definitions?
│  │  │
│  │  ├─ All methods sync (no async)?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: Use async_verb.rs module for async
│  │  │
│  │  ├─ Trait uses only 'static lifetimes?
│  │  │  ├─ YES → PASS: dyn compatible
│  │  │  └─ NO → FAIL: Remove lifetime constraints for object safety
│  │  │
│  │  └─ Trait object-safe (no Self::Assoc types)?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Refactor for dyn compatibility
│
└─ FINAL DECISION: PASS all 10 ADLs or list specific violations with fixes
```

### Tactical Decisions

| ADL | Violation | Remedy |
|-----|-----------|--------|
| ADL-001 | Flat command "get-service" instead of "services get" | Restructure to noun verb pattern |
| ADL-002 | Manual registry() calls instead of #[verb] macro | Convert to #[verb] with distributed slice |
| ADL-003 | Verb returns plain String instead of Serialize struct | Wrap in struct with #[derive(Serialize)] |
| ADL-004 | Async method in NounCommand trait | Move to async_verb.rs, feature-gated |
| ADL-005 | unwrap() in verb implementation | Replace with Result<T> and ? operator |
| ADL-006 | Frontier feature not gated by #[cfg] | Add #[cfg(feature = "...")] guard |
| ADL-007 | New core module added (not async/federation) | Justify in CLAUDE.md ADL discussion |
| ADL-008 | Manual verb registration instead of distributed slice | Convert to #[verb] macro |
| ADL-009 | Incremental compile time 2.5s (over 2s SLO) | Profile with cargo-build-time; optimize |
| ADL-010 | Async method in trait definition | Use async_verb.rs module approach |

---

## PerformanceAnalystAgent Decision Tree

### Scenario: Performance Validation

```
START: Performance baseline or change detection
│
├─ INCREMENTAL COMPILE TIME
│  │
│  ├─ Establish baseline
│  │  │
│  │  ├─ Touch a source file: touch src/lib.rs
│  │  ├─ Measure compile: time cargo make build
│  │  ├─ Record time (ms)
│  │  │
│  │  └─ Target: <=2000ms (SLO)
│  │     ├─ Below 1000ms? → Green (excellent)
│  │     ├─ 1000-2000ms? → Green (acceptable)
│  │     └─ Above 2000ms? → Red (SLO violation)
│  │
│  ├─ Detect regression
│  │  │
│  │  ├─ Compare to previous baseline (currently 0.66s = 660ms)
│  │  │  │
│  │  │  ├─ <5% increase → Yellow (monitor)
│  │  │  ├─ 5-10% increase → Red (investigate)
│  │  │  └─ >10% increase → FAIL (requires optimization)
│  │  │
│  │  └─ If regression detected: cargo build --release -Z timings
│  │     ├─ Identify slow crate (e.g., serde_json, tokio)
│  │     ├─ Check if it's new dependency (if so, consider alternatives)
│  │     └─ Consider feature reduction or codegen unit optimization
│
├─ BINARY SIZE (RELEASE)
│  │
│  ├─ Establish baseline
│  │  │
│  │  ├─ Build: cargo make build-release
│  │  ├─ Measure: ls -lh target/release/clap-noun-verb-gen
│  │  ├─ Extract size (MB)
│  │  │
│  │  └─ Target: <=10MB (SLO)
│  │     ├─ <2MB? → Green (excellent; current: 2.2MB)
│  │     ├─ 2-8MB? → Green (acceptable)
│  │     ├─ 8-10MB? → Yellow (approaching limit)
│  │     └─ >10MB? → Red (SLO violation)
│  │
│  ├─ Detect regression
│  │  │
│  │  ├─ Compare to previous baseline (currently 2.2MB)
│  │  │  │
│  │  │  ├─ <5% increase (2.2MB * 1.05 = 2.31MB) → Yellow (monitor)
│  │  │  ├─ 5-20% increase → Red (investigate)
│  │  │  └─ >20% increase → FAIL (requires optimization)
│  │  │
│  │  └─ If regression detected: cargo bloat --release -n 20
│  │     ├─ Identify bloated symbols
│  │     ├─ Check if new feature dependencies added
│  │     └─ Consider removing unused features
│
├─ BENCHMARKS (Criterion)
│  │
│  ├─ Establish baseline
│  │  │
│  │  ├─ Run: cargo make bench-baseline
│  │  ├─ Save baseline measurements
│  │  │
│  │  └─ Key metrics to track:
│  │     ├─ Verb dispatch time (ms)
│  │     ├─ CLI parsing time (ms)
│  │     ├─ Command registry creation (ms)
│  │     └─ JSON serialization (ms)
│  │
│  ├─ Detect regression
│  │  │
│  │  ├─ Run: cargo make bench-compare
│  │  │  │
│  │  │  ├─ Compare against baseline
│  │  │  ├─ <5% regression → Green (acceptable noise)
│  │  │  ├─ 5-10% regression → Yellow (monitor)
│  │  │  ├─ >10% regression → Red (requires investigation)
│  │  │
│  │  └─ If regression: cargo make profile
│  │     ├─ Profile with 10-second window
│  │     ├─ Identify slow path (parsing? routing? serialization?)
│  │     └─ Optimize specific bottleneck
│
├─ DEPENDENCY ANALYSIS
│  │
│  ├─ Check for feature bloat
│  │  │
│  │  ├─ Run: cargo tree --duplicates
│  │  │  │
│  │  │  ├─ Any duplicates of major crates (serde, tokio)?
│  │  │  │  ├─ YES → Investigate; likely feature misconfiguration
│  │  │  │  └─ NO → Continue
│  │  │  │
│  │  │  └─ Review current features in Cargo.toml
│  │  │     ├─ tokio: ["full"] is heavy; consider minimal features
│  │  │     └─ serde: only "derive" needed; remove others
│  │
│  ├─ Check each dependency
│  │  │
│  │  ├─ For heavy crates (tokio, serde, clap, linkme):
│  │  │  │
│  │  │  ├─ Is it actually used?
│  │  │  │  ├─ NO → Remove it
│  │  │  │  └─ YES → Minimize its features
│  │  │  │
│  │  │  └─ Example: tokio full features vs runtime-only
│  │  │
│  │  └─ Run: cargo tree | head -30
│  │     └─ Review top-level and immediate deps
│
├─ MEMORY PROFILING
│  │
│  ├─ Check for large stack allocations
│  │  │
│  │  ├─ Grep: grep -r '\[.*; [0-9]{4,}' src/
│  │  │  │
│  │  │  ├─ Found large stack arrays?
│  │  │  │  ├─ YES → Convert to Vec<T> on heap
│  │  │  │  └─ NO → Continue
│  │
│  ├─ Check for unnecessary clones
│  │  │
│  │  ├─ Grep: grep -r '\.clone()' src/ | head -20
│  │  │  │
│  │  │  ├─ Each clone in hot path (verb dispatch, parsing)?
│  │  │  │  ├─ YES → Consider &T references instead
│  │  │  │  └─ NO → Acceptable
│  │
│  └─ Overall memory usage
│     │
│     └─ No specific target; monitor trend
│        ├─ If grows >20% between releases, investigate
│        └─ Profile with valgrind if needed
│
└─ FINAL DECISION: Performance pass/yellow/fail with metrics and recommendations
```

### Tactical Decisions

| Scenario | Decision | Action |
|----------|----------|--------|
| Compile time 660ms (baseline) | Excellent | Continue; monitor for regressions |
| Compile time increases to 700ms (5.8%) | Monitor | Accept; log increase; watch for trend |
| Compile time increases to 780ms (18%) | Red | Profile; identify slow dependency; optimize before merge |
| Binary size 2.2MB (baseline) | Excellent | Continue; monitor for growth |
| Binary size increases to 2.5MB (14%) | Red | Run cargo bloat; remove unused features before merge |
| Benchmark: dispatch time 0.5ms → 0.6ms (20%) | Red | Profile dispatch; optimize or reject change |
| Found large array [u8; 50000] | Issue | Convert to Vec with heap allocation |
| Found 10 unnecessary clones in hot path | Issue | Replace with &T references or Arc |

---

## DocMaintainerAgent Decision Tree

### Scenario: Documentation Maintenance

```
START: Documentation change or sync check
│
├─ VERSION SYNCHRONIZATION
│  │
│  ├─ Check Cargo.toml version
│  │  │
│  │  ├─ Read clap-noun-verb-macros/Cargo.toml → version = "26.X.Y"
│  │  ├─ Read clap-noun-verb/Cargo.toml → version = "26.X.Y"
│  │  │
│  │  ├─ Versions match?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: Update both to same version
│
│  ├─ Check CLAUDE.md version reference
│  │  │
│  │  ├─ Grep: grep -E 'Version [0-9]' CLAUDE.md
│  │  ├─ Grep: grep -E 'v26\.' CLAUDE.md
│  │  │
│  │  ├─ Version matches Cargo.toml?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: Update CLAUDE.md version
│
│  └─ Check feature list
│     │
│     ├─ Extract [features] from Cargo.toml
│     ├─ Extract feature list from CLAUDE.md
│     │
│     ├─ Lists match?
│     │  ├─ YES → PASS
│     │  └─ NO → FAIL: Sync feature list
│
├─ CRITICAL RULES CONSISTENCY
│  │
│  ├─ Error handling rule documented
│  │  │
│  │  ├─ CLAUDE.md says: "NEVER unwrap/expect/panic"
│  │  ├─ Cargo.toml [lints.clippy] says: unwrap_used = "deny"
│  │  │
│  │  ├─ Match?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: Sync documentation with lint config
│
│  ├─ Logging rule documented
│  │  │
│  │  ├─ CLAUDE.md says: "use log! macros, never print!"
│  │  │
│  │  └─ Verify rule is upheld in code → PASS
│
│  ├─ Testing rule documented
│  │  │
│  │  ├─ CLAUDE.md says: "AAA pattern, behavioral assertions"
│  │  │
│  │  └─ Verify rules in critical tests → PASS
│
│  └─ All rules sync'd? → PASS or FAIL with specific mismatches
│
├─ EXAMPLE CODE VALIDATION
│  │
│  ├─ Extract all code examples from CLAUDE.md
│  │  │
│  │  ├─ Create temporary test file with all examples
│  │  ├─ Attempt to compile: rustc --crate-type lib example.rs
│  │  │
│  │  ├─ All examples compile?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: List failing examples
│  │  │
│  │  └─ Run example code:
│  │     ├─ "Create a new verb" example compiles? → PASS
│  │     ├─ "Add a noun" example compiles? → PASS
│  │     └─ "Error handling recipe" follows rules? → PASS
│
├─ DOC TESTS
│  │
│  ├─ Run: cargo test --doc
│  │  │
│  │  ├─ All doc tests pass?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: List failing doc-tests with locations
│  │
│  ├─ Check doc-comment coverage
│  │  │
│  │  ├─ All public functions have /// docs?
│  │  ├─ All public types have /// docs?
│  │  ├─ All examples in docs have /// examples?
│  │  │
│  │  └─ Coverage adequate?
│  │     ├─ YES → PASS
│  │     └─ NO → WARN: Add missing doc comments
│
├─ ADL ACCURACY
│  │
│  ├─ For each ADL in CLAUDE.md (ADL-001 through ADL-010)
│  │  │
│  │  ├─ Does ADL match actual code?
│  │  │  │
│  │  │  ├─ ADL-001 (Noun-Verb): Check router.rs implements pattern
│  │  │  ├─ ADL-002 (Proc-Macro): Check macros use distributed_slice
│  │  │  ├─ ADL-003 (JSON): Check verb traits require Serialize
│  │  │  ├─ ADL-004 (Async): Check async_verb.rs and trait definitions
│  │  │  ├─ ADL-005 (No Panics): Check clippy deny rules in Cargo.toml
│  │  │  ├─ ADL-006 (Features): Check #[cfg] gating in src/
│  │  │  ├─ ADL-007 (Minimalist): Check src/ has only 2 optional modules
│  │  │  ├─ ADL-008 (Distributed): Check linkme usage in CommandRegistry
│  │  │  ├─ ADL-009 (SLOs): Check Makefile.toml slo-check task
│  │  │  └─ ADL-010 (Traits): Check trait definitions are dyn compatible
│  │  │
│  │  └─ All ADLs accurate?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Update ADL entries to match code
│
├─ TROUBLESHOOTING GUIDE
│  │
│  ├─ Check common issues documented
│  │  │
│  │  ├─ "error: could not compile" → Solution present?
│  │  ├─ "error[E0425]: cannot find function" → Solution present?
│  │  ├─ "test suite panicked" → Solution present?
│  │  ├─ "test is flaky" → Solution present?
│  │  │
│  │  └─ All common issues covered?
│  │     ├─ YES → PASS
│  │     └─ NO → WARN: Add documented solutions for missing issues
│
├─ DEAD LINKS CHECK
│  │
│  ├─ Grep for internal references (e.g., "see ADL-005")
│  │  │
│  │  ├─ All references point to existing sections?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: Fix broken references
│  │
│  ├─ Grep for file references (e.g., "src/router.rs")
│  │  │
│  │  ├─ All files still exist?
│  │  │  ├─ YES → PASS
│  │  │  └─ NO → FAIL: Update to current file paths
│
│  └─ Check links don't reference removed modules
│     ├─ "src/io/" removed → Any references? → FAIL: Remove references
│     ├─ "src/kernel/" removed → Any references? → FAIL: Remove references
│
├─ RECIPES VALIDATION
│  │
│  ├─ For each "Recipe:" section in CLAUDE.md
│  │  │
│  │  ├─ Is recipe code complete?
│  │  ├─ Does it follow best practices (error handling, async)?
│  │  ├─ Does it match current API?
│  │  │
│  │  └─ All recipes valid?
│  │     ├─ YES → PASS
│  │     └─ NO → FAIL: Update invalid recipes
│
└─ FINAL DECISION: Documentation sync pass/fail with specific issues
```

### Tactical Decisions

| Issue | Decision | Action |
|-------|----------|--------|
| CLAUDE.md says v26.6.1, Cargo.toml says v26.6.2 | Blocker | Update CLAUDE.md to match Cargo.toml |
| Feature list in CLAUDE.md missing "quantum-ready" | Blocker | Add to CLAUDE.md features section |
| Example code doesn't compile | Critical | Fix example or remove if obsolete |
| Doc-test fails: `assert!(result.is_ok())` | Quality | Update to verify actual behavior |
| ADL-007 says "2 optional modules" but code has 3 | Critical | Update ADL or restructure code |
| Troubleshooting section doesn't cover new error | Enhancement | Add solution for new error type |
| Dead link to "src/rdf/" (removed module) | Cleanup | Remove reference to archived module |
| Recipe code doesn't match current API | Blocker | Update recipe or mark as deprecated |

---

## Universal Decision Rules

### Rule 1: When in doubt, enforce existing standards
If unclear whether something violates an ADL or rule, default to enforcing the existing CLAUDE.md standard.

### Rule 2: Measurable beats subjective
Prefer quantitative criteria (SLO times, feature counts, test pass rates) over opinions.

### Rule 3: Fail fast with specificity
Rather than vague "this doesn't look right," provide exact file paths, line numbers, and specific violation.

### Rule 4: Escalate architectural disputes
If an ADL seems outdated or a change conflicts with multiple ADLs, escalate for human decision rather than agent making new rules.

### Rule 5: Document your reasoning
Every agent decision should include the rule/criterion that led to it, not just the pass/fail.

---

## Appendix: Decision Tree Visual Symbols

```
START          = Entry point
├─             = Branch (decision point)
│  ├─          = Sub-decision
│  │  ├─       = Deep decision
│  │  └─       = End of branch
│  └─          = Alternate path
└─             = Final branch
PASS           = Accept/proceed
FAIL           = Reject/block/require fix
YELLOW/WARN    = Caution/monitor/review recommended
DECISION:      = Final outcome
```

