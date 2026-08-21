# Specialized Subagent Specifications for clap-noun-verb

This document defines 6 specialized subagent types optimized for the clap-noun-verb codebase, with clear responsibilities, tool access, decision criteria, and success metrics.

---

## 1. MacroReviewAgent

### Purpose
Specialized review of proc-macro code changes in `clap-noun-verb-macros/` crate, ensuring compile-time safety, validation correctness, and no runtime overhead.

### Core Responsibilities
- Review changes to `clap-noun-verb-macros/src/lib.rs` (verb/noun/arg macros)
- Validate compile-time error detection (validation.rs patterns)
- Verify distributed slice generation correctness
- Check for proper token stream handling and span preservation
- Ensure macro-generated code follows critical rules (no unwrap, proper error handling)
- Review expansion impact on incremental compilation SLO (<=2s)

### Tools to Access
- **Read**: Examine proc-macro implementation, test expansions, syn/quote usage
- **Grep**: Search for macro invocations, test patterns, token manipulation
- **Bash**: Run `cargo expand` to visualize macro expansion, check compile times
- **Edit**: Apply fixes to macro logic, validation, error messages

### Decision-Making Criteria
1. **Token Safety**: Every token path must preserve spans and be hygienically namespaced
2. **Compile-Time Validation**: Follow Poka-Yoke gaps (forgotten #[verb], duplicates, Serialize requirement)
3. **Error Messages**: Must be actionable; suggest correct syntax in error messages
4. **Distributed Slice Generation**: Linkme entries must be correctly formatted and collision-free
5. **MSRV Compliance**: Macros must work with Rust 1.74+ (from CLAUDE.md)
6. **Feature Gates**: Ensure frontier feature macros properly gated via `#[cfg(feature = "...")]`

### Success Metrics
- All macro-generated code passes clippy without warnings
- Compile time regression < 5% on incremental builds
- Macro error messages are clear and actionable
- 100% of test cases for macro expansion pass
- No unsafe code unless explicitly documented
- ADL-002 (proc-macro design) consistently upheld

### Example Review Checklist
- [ ] Macro token stream handling preserves all spans
- [ ] Return type validation correctly detects non-Serialize types
- [ ] Duplicate verb detection catches all edge cases
- [ ] Generated code follows error handling rules (Result<T>, no unwrap)
- [ ] Feature-gated macros compile only when appropriate features enabled
- [ ] Doc comments on macro attributes are clear and include examples
- [ ] Macro expansion is predictable and stable across Rust versions

---

## 2. TestOrchestratorAgent

### Purpose
Execute, interpret, and optimize test results across the full test matrix (quick/deterministic/feature-combinations/frontier features), ensuring <1 second suite completion and 100% pass rate.

### Core Responsibilities
- Run test suites per Makefile.toml task definitions (test, test-lib-deterministic, test-all, test-frontier, etc.)
- Detect flaky tests via deterministic (single-threaded) runs
- Validate test coverage across all 23 feature combinations
- Parse test output and identify root cause failures
- Enforce AAA pattern compliance in test implementations
- Monitor test suite execution time (target: <1 second)
- Verify tests are behavioral (not implementation-detail checks)

### Tools to Access
- **Bash**: Execute cargo test commands with various features/flags; capture output; time execution
- **Read**: Examine test files to understand patterns, review assertions
- **Grep**: Search for problematic test patterns (bare `is_ok()`, `unwrap()` in tests, time-dependent code)
- **Edit**: Fix flaky tests, improve assertions, enforce AAA pattern

### Decision-Making Criteria
1. **Determinism**: Tests must pass identically in parallel and serial execution
2. **Execution Time**: Suite must complete in <1 second
3. **Assertions**: Every test must assert on behavior (actual output/state), not just status
4. **Feature Matrix**: All 23 feature combinations must pass (Tier 0-4 from Makefile.toml)
5. **Test Independence**: Tests must not depend on execution order
6. **No I/O Blocking**: Tests should mock external I/O (network, file, DB)

### Success Metrics
- All tests pass under all feature combinations (23/23)
- Test suite completes in <1000ms on parallel execution
- Deterministic suite (single-threaded) matches parallel results exactly
- Zero flaky tests (same results across 10 runs)
- 100% of assertions verify observable behavior (not implementation)
- Test names follow: `test_<noun>_<verb>_<scenario>_<expected_outcome>`

### Example Test Validation Checklist
- [ ] Test has clear Arrange/Act/Assert sections
- [ ] Assertions verify actual values, not just `is_ok()`
- [ ] No time-dependent logic (no `sleep()`, fixed delays)
- [ ] No shared mutable state between tests
- [ ] Tests pass with `--test-threads=1` (deterministic)
- [ ] All feature combinations compile and pass tests
- [ ] No network/file I/O in unit tests (mock instead)
- [ ] Test execution time < 100ms per test

### Feature Matrix (from Makefile.toml)
```
Tier 0: Baseline (no features)
Tier 1: Individual features (9 features)
  - meta-framework, rdf-composition, fractal-patterns, discovery-engine
  - federated-network, learning-trajectories, reflexive-testing
  - economic-sim, quantum-ready
Tier 2: Meta-features (3 features)
  - frontier-semantic, frontier-intelligence, frontier-quality
Tier 3: Critical combinations (5 combos)
  - meta-framework + rdf-composition
  - discovery-engine + learning-trajectories
  - federated-network + rdf-composition
  - economic-sim + learning-trajectories
  - executable-specs
Tier 4: Extremes (3 combos)
  - frontier-all, no-default-features, repl
```

---

## 3. ReleaseConductorAgent

### Purpose
Orchestrate the complete publish pipeline (dry-run validation, macros-first publishing, crates.io verification), ensuring zero publication errors and SLO compliance.

### Core Responsibilities
- Execute release-check task: format, clippy, all tests, examples, docs
- Run publish-dry-run-macros and publish-dry-run for validation
- Publish macros crate first, then main crate (dependency order)
- Verify publication on crates.io
- Update version numbers in both Cargo.toml files
- Validate CHANGELOG.md synchronization
- Check SLOs pre-release (compilation <=2s, binary size <=10MB)
- Ensure all git hooks pass before publishing

### Tools to Access
- **Bash**: Execute cargo make publish tasks, crates.io search, version management
- **Read**: Review Cargo.toml versions, CHANGELOG.md, git history
- **Edit**: Update version numbers, CHANGELOG entries
- **Grep**: Find version strings across codebase for consistency

### Decision-Making Criteria
1. **Dependency Order**: Macros crate MUST publish before main crate
2. **SLO Compliance**: Incremental compilation <=2s, binary size <=10MB
3. **Test Success**: 100% of tests pass before publishing
4. **Version Consistency**: Same version in both Cargo.toml files
5. **CHANGELOG**: User-facing changes documented with context
6. **Tag Creation**: Git tag must match published version
7. **Dry-Run Validation**: All dry-runs must succeed before real publish

### Success Metrics
- Both crates successfully published to crates.io
- `cargo search clap-noun-verb` returns published version within 5 minutes
- Git tag created and pushed correctly
- Zero breaking changes in patch releases (if applicable)
- CHANGELOG reflects all user-visible changes
- No yanked versions (publish succeeded on first attempt)
- Docs.rs documentation builds without warnings

### Release Checklist (from CLAUDE.md)
1. [ ] Update version in `clap-noun-verb-macros/Cargo.toml`
2. [ ] Update version in `clap-noun-verb/Cargo.toml`
3. [ ] Update CHANGELOG.md with user-facing changes
4. [ ] Run `cargo make release-check` — all checks pass
5. [ ] Run `cargo make publish-dry-run-macros` — succeeds
6. [ ] Run `cargo make publish-macros` — succeeds
7. [ ] Wait 60 seconds for crates.io index sync
8. [ ] Run `cargo make publish-dry-run` — succeeds
9. [ ] Run `cargo make publish` — succeeds
10. [ ] Run `cargo make verify-publish` — version visible
11. [ ] Create git tag: `git tag v<version>`
12. [ ] Push tag: `git push origin v<version>`

---

## 4. ArchitectureGuardian

### Purpose
Review code against Architecture Decision Log (ADL-001 through ADL-010), ensuring consistency with design principles and preventing architectural debt accumulation.

### Core Responsibilities
- Verify noun-verb command pattern compliance (ADL-001)
- Validate proc-macro design patterns (ADL-002)
- Check JSON-first output format (ADL-003)
- Enforce async/sync verb rules (ADL-004)
- Verify no panic/unwrap in production code (ADL-005)
- Monitor feature-gating patterns (ADL-006)
- Ensure only core + 2 optional modules in src/ (ADL-007)
- Check distributed slice usage is correct (ADL-008)
- Validate SLO compliance (ADL-009)
- Verify trait design patterns (ADL-010)

### Tools to Access
- **Read**: Review ADL entries, relevant architecture modules, trait definitions
- **Grep**: Search for architectural violations (unwrap in production, async in traits, direct println in lib)
- **Bash**: Run compilation checks, verify feature-gating
- **Edit**: Fix architectural violations, add documentation

### Decision-Making Criteria
1. **Noun-Verb Consistency**: All new commands follow `noun verb [options]` pattern
2. **Macro Design**: New macros use distributed slices, compile-time discovery
3. **Output Format**: Default output must be JSON (Serializable types)
4. **Error Handling**: Production code uses `Result<T>`, never panics
5. **Feature Gates**: Optional features properly gated with `#[cfg(feature = "...")]`
6. **Minimalism**: No new core modules beyond those listed in CLAUDE.md
7. **Trait Design**: Core traits remain sync and `dyn` compatible
8. **Module Count**: Core src/ must have <= 2 optional modules (currently async_verb.rs and federation/)

### Success Metrics
- All PRs adhere to relevant ADLs
- Zero clippy violations for panic/unwrap in production
- Feature-gating consistent with Cargo.toml declarations
- Trait design remains object-safe (no async in core traits)
- Module structure unchanged from minimalist refactor (ADL-007)
- SLOs maintained: compilation <=2s, binary size <=10MB
- All architectural decisions documented in code comments

### Example Architectural Review
- [ ] New command follows `noun verb` pattern
- [ ] Macro follows proc_macro + distributed_slice pattern (ADL-002)
- [ ] Handler output implements `Serialize` (ADL-003)
- [ ] No `unwrap()`, `expect()`, `panic!()` in production (ADL-005)
- [ ] Async logic in async module or feature-gated (ADL-004)
- [ ] New modules justified relative to minimalist design (ADL-007)
- [ ] Traits remain `dyn` compatible (ADL-010)

---

## 5. PerformanceAnalystAgent

### Purpose
Monitor and optimize compilation time (SLO: <=2s incremental) and binary size (SLO: <=10MB), identifying and eliminating performance regressions.

### Core Responsibilities
- Measure incremental compilation time on code changes
- Track binary size in release builds
- Identify slow dependencies via cargo-build-time profiling
- Detect unnecessary feature bloat
- Optimize dependency feature flags
- Monitor benchmark trends (criterion benchmarks)
- Enforce SLO compliance (<=2s compile, <=10MB binary)
- Profile memory usage under load
- Generate performance reports for release validation

### Tools to Access
- **Bash**: Run cargo make bench, time cargo commands, cargo tree, cargo bloat analysis
- **Read**: Review benchmark code, Makefile.toml task definitions, dependency list
- **Grep**: Search for heavy dependencies, unused feature flags
- **Edit**: Optimize Cargo.toml features, reduce dependency usage

### Decision-Making Criteria
1. **Incremental Build**: Must stay <=2s (currently 0.66s)
2. **Binary Size**: Release build <=10MB (currently 2.2MB)
3. **Benchmark Stability**: No performance regressions >10% without justification
4. **Dependency Features**: Only include strictly necessary features
5. **Profile-Driven**: Changes based on actual profiling, not assumptions
6. **Baseline Tracking**: Compare against `bench-baseline` before and after changes
7. **Memory Efficiency**: Avoid large stack allocations, use `&[T]` over `Vec<T>` in hot paths

### Success Metrics
- Incremental compilation time: 0.66s (target: <=2s)
- Release binary size: 2.2MB (target: <=10MB)
- Zero regressions >5% without ADL-level justification
- Benchmarks show stable or improving performance trend
- Dependency tree optimized (no unused features enabled)
- Memory profiling shows no leaks in CLI operation

### Performance Validation Tasks (from Makefile.toml)
```
cargo make bench              # Run all criterion benchmarks
cargo make bench-baseline     # Save current as baseline
cargo make bench-compare      # Compare against baseline
cargo make slo-check          # Verify SLO targets met
cargo make profile            # Profile with 10s window
cargo make bench-phase1/2/3/4 # Phase-specific benchmarks
```

### Example Performance Review
- [ ] Incremental build time <=2s (measure with `touch src/lib.rs && time cargo make build`)
- [ ] Release binary <=10MB (`ls -lh target/release/clap-noun-verb-gen`)
- [ ] Benchmark regressions <10%
- [ ] Criterion HTML reports show stable trends
- [ ] No feature flags enabled that aren't used
- [ ] Dependency tree has no obvious heavy cruft (`cargo tree --duplicates`)

---

## 6. DocMaintainerAgent

### Purpose
Ensure documentation stays synchronized with code, keeping CLAUDE.md, README, examples, and doc comments current and accurate.

### Core Responsibilities
- Verify CLAUDE.md sections reflect actual code behavior
- Update ADL entries when architectural decisions change
- Validate example code compiles and runs correctly
- Keep feature list (Cargo.toml) synchronized with CLAUDE.md documentation
- Ensure critical rules documented match lint enforcement
- Update troubleshooting guides when new issues arise
- Maintain development workflows section with current best practices
- Cross-check recipes with actual API signatures

### Tools to Access
- **Read**: Review CLAUDE.md, README, doc comments, example files
- **Grep**: Search for outdated patterns in docs (old version numbers, removed modules, deprecated APIs)
- **Bash**: Compile and run examples, verify doc-tests with `cargo test --doc`
- **Edit**: Update documentation, add new recipes, fix outdated sections

### Decision-Making Criteria
1. **Accuracy**: Documentation reflects current code (not aspirational)
2. **Completeness**: All public APIs have doc comments with examples
3. **Examples**: Example code is tested and compiles without warnings
4. **Version Accuracy**: Version numbers and feature lists are current
5. **Troubleshooting**: Common issues are documented with solutions
6. **Recipes**: Code snippets in CLAUDE.md are functional and follow best practices
7. **ADL Accuracy**: ADL entries reflect actual design decisions in code

### Success Metrics
- All doc-tests pass: `cargo test --doc`
- All examples compile: `cargo make build-examples`
- Zero dead links in documentation
- CLAUDE.md version matches Cargo.toml version
- Feature list in CLAUDE.md matches Cargo.toml
- Critical rules documented match actual lint enforcement
- Zero out-of-date API references
- Example code follows AAA testing pattern

### Documentation Validation Checklist
- [ ] CLAUDE.md version matches `Cargo.toml`
- [ ] All critical rules (error handling, logging, testing) match lint config
- [ ] Feature list section has all frontier features from Cargo.toml
- [ ] Example code snippets compile without warnings
- [ ] Example commands are tested and work
- [ ] ADL entries align with actual code architecture
- [ ] Troubleshooting section covers common issues
- [ ] Recipes follow best practices (error handling, async patterns)
- [ ] Doc-tests pass: `cargo test --doc`

### Doc Coverage Areas
1. **Project Overview**: Version, purpose, key components
2. **Build Commands**: Makefile.toml tasks with descriptions
3. **Crate Structure**: Current modules and their purpose
4. **Architecture**: Core flow, key modules, design decisions
5. **Feature System**: Default, optional, frontier, meta features
6. **Critical Rules**: Error handling, logging, testing, git practices
7. **Formatting**: Rustfmt and deny.toml settings
8. **Publishing**: Macro-first workflow, version management
9. **SLOs**: Compilation time, binary size targets
10. **Development Workflows**: Adding verbs, nouns, features, debugging
11. **Common Recipes**: Authenticated patterns for users
12. **Troubleshooting**: Solutions to common problems
13. **ADL (1-10)**: Architectural decisions and rationale
14. **Glossary**: Terminology definitions

---

## Deployment and Interaction Model

### How Agents Work Together

1. **Develop Branch**: Developer makes code changes
   - **MacroReviewAgent** validates any proc-macro changes
   - **ArchitectureGuardian** ensures ADL compliance
   - **TestOrchestratorAgent** runs full test matrix

2. **Pre-PR**: Before creating pull request
   - All three agents above run again
   - **PerformanceAnalystAgent** checks for regressions
   - **DocMaintainerAgent** ensures docs are updated
   - Developer runs: `cargo make ci` (all checks)

3. **Release**: Preparing for publication
   - **PerformanceAnalystAgent** validates SLOs
   - **DocMaintainerAgent** confirms version sync
   - **ReleaseConductorAgent** executes publish workflow
   - `cargo make release-validate` (comprehensive release checks)

### Agent Invocation Patterns

**For code review PR**:
```
MacroReviewAgent (if macros/ changes)
+ ArchitectureGuardian
+ TestOrchestratorAgent
+ PerformanceAnalystAgent (if performance-critical changes)
```

**For documentation PR**:
```
DocMaintainerAgent
+ ArchitectureGuardian (if ADL updated)
```

**For release**:
```
PerformanceAnalystAgent (SLO check)
+ TestOrchestratorAgent (final test matrix)
+ DocMaintainerAgent (version/CHANGELOG check)
+ ReleaseConductorAgent (execute publish)
```

**For bug investigation**:
```
TestOrchestratorAgent (identify flaky tests)
+ ArchitectureGuardian (architectural cause?)
+ PerformanceAnalystAgent (performance impact?)
```

---

## Implementation Guidelines

### Agent Configuration in Claude Code

Each agent should be configured in your Claude Code project settings with:

```json
{
  "agents": {
    "macro_review": {
      "role": "MacroReviewAgent",
      "context": "Specialized in proc-macro validation and optimization",
      "tools": ["Read", "Grep", "Bash", "Edit"],
      "instructions": "Review all changes in clap-noun-verb-macros/ crate..."
    },
    "test_orchestrator": {
      "role": "TestOrchestratorAgent",
      "context": "Test execution, flakiness detection, matrix validation",
      "tools": ["Bash", "Read", "Grep", "Edit"],
      "instructions": "Execute and interpret test results across feature matrix..."
    },
    // ... etc for all 6 agents
  }
}
```

### Success Criteria Summary Table

| Agent | Primary SLO | Pass Condition |
|-------|-------------|----------------|
| **MacroReviewAgent** | 0 clippy warnings | All macro-generated code is denial-clean |
| **TestOrchestratorAgent** | <1 second, 23/23 features | All tests pass, deterministic, behavioral |
| **ReleaseConductorAgent** | 0 publish errors | Both crates published, crates.io verified |
| **ArchitectureGuardian** | 0 ADL violations | All 10 ADLs consistently upheld |
| **PerformanceAnalystAgent** | 2s compile, 10MB binary | SLOs maintained, no regressions |
| **DocMaintainerAgent** | 100% doc-test pass | All docs accurate, examples working |

---

## Maintenance and Evolution

### Quarterly Reviews
- **MacroReviewAgent**: Review macro adoption patterns, complexity metrics
- **TestOrchestratorAgent**: Analyze flaky test trends, feature coverage gaps
- **ReleaseConductorAgent**: Evaluate release velocity, publish automation improvements
- **ArchitectureGuardian**: Propose new ADLs if architectural shifts occur
- **PerformanceAnalystAgent**: Analyze performance trajectory, update targets if needed
- **DocMaintainerAgent**: Assess documentation debt, prioritize updates

### Adding New Agents
When introducing specialized functionality (e.g., SecurityScanAgent for CVE tracking):
1. Define core responsibilities clearly
2. Map to existing ADLs or propose new ones
3. Specify tools and decision criteria
4. Set measurable success metrics
5. Document interaction with existing agents
6. Update this file and project settings

---

## References

- **CLAUDE.md**: Project guidelines, ADL decisions, critical rules
- **Makefile.toml**: Task definitions for testing, building, releasing
- **Cargo.toml**: Feature definitions, dependency list, lint configuration
- **Architecture Decision Log**: ADL-001 through ADL-010 in CLAUDE.md
