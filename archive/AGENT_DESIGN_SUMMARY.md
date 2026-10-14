> Archived 2026-08-20: superseded/stale as of v26.8.20.

# Agent Design Summary: clap-noun-verb Specialized Subagents

## Executive Overview

This document summarizes the design of 6 specialized subagent types for the clap-noun-verb Rust CLI framework. Each agent is optimized for a specific domain within the development lifecycle and enforces quality gates defined by the Architecture Decision Log (ADL) and CLAUDE.md critical rules.

---

## The 6 Agents at a Glance

### 1. **MacroReviewAgent** - Procedural Macro Specialist
- **Domain**: `clap-noun-verb-macros/` crate changes
- **Key Focus**: Token safety, compile-time validation, distributed slice correctness
- **Enforces**: ADL-002 (Proc-Macro Over Derive-Based Registration)
- **SLO**: Zero clippy warnings, <5% compile time regression
- **Tools**: Read, Grep, Bash, Edit

### 2. **TestOrchestratorAgent** - Quality Assurance Lead
- **Domain**: Test execution and flakiness detection
- **Key Focus**: Full feature matrix (23 combinations), deterministic runs, behavioral assertions
- **Enforces**: <1 second suite, 100% behavioral tests (AAA pattern), zero flakiness
- **SLO**: All tests pass across all feature combinations
- **Tools**: Bash, Read, Grep, Edit

### 3. **ReleaseConductorAgent** - Publishing Orchestrator
- **Domain**: Release pipeline (macros-first publishing to crates.io)
- **Key Focus**: Version sync, dry-run validation, SLO compliance, crates.io verification
- **Enforces**: Macros published before main, both versions match, CHANGELOG updated
- **SLO**: Zero publish errors, successful crates.io listing within 5 minutes
- **Tools**: Bash, Read, Edit, Grep

### 4. **ArchitectureGuardian** - Design Architect
- **Domain**: Architecture Decision Log (ADL) compliance
- **Key Focus**: 10 ADL principles (noun-verb pattern, JSON output, no panics, minimal core, etc.)
- **Enforces**: Architectural consistency across all changes
- **SLO**: Zero ADL violations, all 10 ADLs maintained
- **Tools**: Read, Grep, Bash, Edit

### 5. **PerformanceAnalystAgent** - Performance Optimizer
- **Domain**: Compilation time and binary size SLOs
- **Key Focus**: Incremental build tracking (target: ≤2s), binary size (target: ≤10MB), benchmarks
- **Enforces**: ADL-009 (SLO Targets), no regressions >5%, baseline tracking
- **SLO**: Compilation ≤2s (currently 0.66s), binary ≤10MB (currently 2.2MB)
- **Tools**: Bash, Read, Grep, Edit

### 6. **DocMaintainerAgent** - Documentation Steward
- **Domain**: Documentation synchronization and accuracy
- **Key Focus**: CLAUDE.md/README consistency, example code, doc-tests, ADL accuracy
- **Enforces**: Zero outdated references, all examples compile, doc-tests pass
- **SLO**: 100% doc-test pass rate, zero dead documentation
- **Tools**: Read, Grep, Bash, Edit

---

## Design Principles

### 1. **Specialization Over Generality**
Each agent has deep domain expertise rather than broad capabilities. This allows for:
- Focused decision-making with clear criteria
- Meaningful success metrics aligned to domain
- Efficient tool usage (knows which tools matter most)
- Clear escalation paths when issues arise

### 2. **Enforcement of Existing Standards**
Agents don't make new rules—they enforce existing project standards:
- **CLAUDE.md Critical Rules**: Error handling, logging, testing, git practices
- **ADL (1-10)**: Architectural decisions already made
- **Makefile.toml**: Build and test tasks already defined
- **SLOs**: Documented performance targets

### 3. **Measurable Success Metrics**
Each agent has clear, objective success criteria:
- **MacroReviewAgent**: All clippy deny rules pass
- **TestOrchestratorAgent**: 23/23 feature combos pass, <1 second
- **ReleaseConductorAgent**: Both crates published to crates.io
- **ArchitectureGuardian**: All 10 ADLs upheld
- **PerformanceAnalystAgent**: SLOs maintained (2s compile, 10MB binary)
- **DocMaintainerAgent**: 100% doc-test pass, zero dead links

### 4. **Parallel Execution Where Possible**
Agents can run in parallel when independent:
```
Development → MacroReview (if changed)
           → ArchitectureGuardian
           → TestOrchestrator
           → PerformanceAnalyst (if perf-critical)
           → DocMaintainer (if docs changed)
           ↓
           Merge (if all pass)
           ↓
Release → PerformanceAnalyst (SLO check)
       → TestOrchestrator (final validation)
       → DocMaintainer (version sync)
       → ReleaseConductor (execute publish)
```

### 5. **Clear Tool Ownership**
Each agent has a clear set of tools it needs:
- **Read** (all agents): Review code and configuration
- **Grep** (all agents): Search for patterns and violations
- **Bash** (all agents): Execute cargo tasks
- **Edit** (most agents): Fix issues directly

This reduces complexity and ensures agents don't use tools outside their domain.

---

## Codebase Context

### Key Architectural Patterns
1. **Noun-Verb Commands**: `myapp services status` pattern (ADL-001)
2. **Proc-Macro Registration**: `#[verb]` generates distributed slice entries (ADL-002)
3. **JSON-First Output**: All verbs return Serializable types (ADL-003)
4. **Async-First Verbs**: Async by default, sync feature-gated (ADL-004)
5. **No Panics**: Production code uses `Result<T>`, never panics (ADL-005)
6. **Feature-Gated Experimentation**: Frontier features kept separate (ADL-006)
7. **Minimalist Core**: Only 2 optional modules in `src/` (ADL-007)
8. **Distributed Slice Registration**: Compile-time verb discovery (ADL-008)
9. **Performance SLOs**: 2s compile, 10MB binary (ADL-009)
10. **Sync Traits**: Core traits are `dyn` compatible (ADL-010)

### Test Structure
- **Quick Tests** (parallel): `cargo make test` — fast iteration
- **Deterministic Tests** (serial): `cargo make test-lib-deterministic` — flakiness detection
- **Feature Matrix** (23 combinations): `cargo make test-frontier-matrix` — exhaustive validation
- **All Features**: `cargo make test-all` — full coverage
- **Integration**: `cargo make test-integration` — example CLI testing

### Build Tasks
- **Format Check**: `cargo make format-check`
- **Lint**: `cargo make clippy`, `cargo make lint`
- **Test**: Various test commands per Makefile.toml
- **Benchmarks**: `cargo make bench`, `cargo make bench-compare`
- **SLO Check**: `cargo make slo-check`
- **CI Full Suite**: `cargo make ci`, `cargo make release-validate`

---

## Agent-to-ADL Mapping

| ADL | Agent | Enforcement |
|-----|-------|-------------|
| **ADL-001**: Noun-Verb Pattern | ArchitectureGuardian | All new commands follow pattern |
| **ADL-002**: Proc-Macro Design | MacroReviewAgent | Distributed slices, no unwrap |
| **ADL-003**: JSON Output | ArchitectureGuardian | Verbs return Serializable |
| **ADL-004**: Async/Sync Verbs | ArchitectureGuardian | Async in async_verb.rs or feature |
| **ADL-005**: No Panics | MacroReviewAgent, ArchitectureGuardian | Clippy deny rules pass |
| **ADL-006**: Feature Gating | ArchitectureGuardian | Frontier features properly gated |
| **ADL-007**: Minimalist Core | ArchitectureGuardian | Only 2 optional modules |
| **ADL-008**: Distributed Slices | MacroReviewAgent | Linkme usage correct |
| **ADL-009**: SLO Targets | PerformanceAnalystAgent | 2s compile, 10MB binary |
| **ADL-010**: Trait Design | ArchitectureGuardian | dyn compatible, no async |

---

## Success Metrics Summary

### Quantitative Metrics
| Metric | Target | Current | Agent |
|--------|--------|---------|-------|
| Clippy violations | 0 | 0 | MacroReviewAgent, ArchitectureGuardian |
| Test pass rate | 100% | 100% | TestOrchestratorAgent |
| Feature combinations passing | 23/23 | TBD | TestOrchestratorAgent |
| Test suite time | <1000ms | TBD | TestOrchestratorAgent |
| Incremental compile time | <=2s | 0.66s | PerformanceAnalystAgent |
| Release binary size | <=10MB | 2.2MB | PerformanceAnalystAgent |
| Doc-test pass rate | 100% | TBD | DocMaintainerAgent |
| Publish errors | 0 | 0 | ReleaseConductorAgent |
| ADL violations | 0 | 0 | ArchitectureGuardian |

### Qualitative Metrics
- **MacroReviewAgent**: Macro error messages are actionable and helpful
- **TestOrchestratorAgent**: Tests follow AAA pattern and verify behaviors
- **ReleaseConductorAgent**: Release process is reproducible and well-documented
- **ArchitectureGuardian**: All 10 ADLs are consistently upheld across codebase
- **PerformanceAnalystAgent**: Performance regressions are tracked and justified
- **DocMaintainerAgent**: Documentation is up-to-date and examples are tested

---

## Tool Usage Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                    Agent Decision Layer                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  MacroReviewAgent           TestOrchestratorAgent            │
│  ├─ Read (macro code)       ├─ Bash (run tests)             │
│  ├─ Grep (token patterns)   ├─ Read (test files)            │
│  ├─ Bash (expand, compile)  ├─ Grep (test patterns)         │
│  └─ Edit (fix macros)       └─ Edit (improve tests)         │
│                                                               │
│  ReleaseConductorAgent      ArchitectureGuardian             │
│  ├─ Bash (publish, tag)     ├─ Read (architecture)          │
│  ├─ Read (versions)         ├─ Grep (ADL violations)        │
│  ├─ Edit (versions)         ├─ Bash (compile checks)        │
│  └─ Grep (consistency)      └─ Edit (fix architecture)      │
│                                                               │
│  PerformanceAnalystAgent    DocMaintainerAgent              │
│  ├─ Bash (bench, profile)   ├─ Read (documentation)        │
│  ├─ Read (dependencies)     ├─ Grep (version refs)         │
│  ├─ Grep (features)         ├─ Bash (doc tests)            │
│  └─ Edit (optimize)         └─ Edit (update docs)          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## Integration Points

### With CLAUDE.md
All agents reference and enforce CLAUDE.md:
- **Critical Rules** section (error handling, logging, testing, git)
- **ADL-001 through ADL-010** (architecture decisions)
- **SLOs** section (compilation and size targets)
- **Development Workflows** section (best practices)

### With Makefile.toml
Agents invoke specific tasks:
- **MacroReviewAgent**: `cargo make clippy`, `cargo make check`
- **TestOrchestratorAgent**: `cargo make test-*` tasks
- **ReleaseConductorAgent**: `cargo make publish-*` and `release-check` tasks
- **ArchitectureGuardian**: `cargo make lint`, `cargo make andon-check`, `slo-check`
- **PerformanceAnalystAgent**: `cargo make bench-*`, `slo-check`
- **DocMaintainerAgent**: `cargo test --doc`, `cargo make build-examples`

### With GitHub Actions
Agents can be orchestrated via CI/CD:
- Parallel runs for quick checks
- Serial runs for release validation
- Automatic failure notifications
- Performance trend tracking

---

## Lifecycle Workflow

### Development Phase
```
1. Developer writes code on feature branch
2. MacroReviewAgent (if macros/ changed) → PASS/FAIL
3. ArchitectureGuardian → PASS/FAIL
4. TestOrchestratorAgent → PASS/FAIL
5. PerformanceAnalystAgent (if critical) → PASS/FAIL
6. DocMaintainerAgent (if docs changed) → PASS/FAIL
7. All pass → Ready for PR
```

### Review Phase
```
1. PR created
2. All agents run again (can be automated)
3. Agents provide specific feedback (file, line, suggestion)
4. Developer addresses feedback
5. Agents re-run to verify fixes
6. All pass → Mergeable
```

### Release Phase
```
1. Version bumped in Cargo.toml files
2. CHANGELOG.md updated
3. ArchitectureGuardian → No ADL violations
4. TestOrchestratorAgent → 23/23 features pass
5. PerformanceAnalystAgent → SLOs met
6. DocMaintainerAgent → Docs synced, examples work
7. ReleaseConductorAgent → Execute publish
8. Verify on crates.io
9. Create git tag
10. Release complete
```

---

## Extensibility

### Adding New Agents
When new specialized domains emerge, follow this template:

1. **Define Core Responsibilities** (3-5 bullet points)
2. **Specify Tool Access** (Read, Grep, Bash, Edit, etc.)
3. **Set Decision Criteria** (5-7 clear rules)
4. **Establish Success Metrics** (quantitative where possible)
5. **Map to ADLs** (which architecture decisions does it enforce?)
6. **Document Interaction** (how does it work with other agents?)

### Customizing Existing Agents
Each agent spec includes customization hooks:
- **Extended criteria**: For domain-specific rules
- **Tool overrides**: For special environments (e.g., CI/CD)
- **Metric adjustments**: For changing project needs

---

## Quick Reference Commands

### Invoke All Agents (Full Validation)
```bash
# Quick status check
cargo make format-check
cargo make clippy
cargo make test-lib-deterministic
cargo make test-feature-combinations
cargo make slo-check
cargo test --doc

# Full CI suite (what would happen before merge)
cargo make ci

# Release validation (what ReleaseConductorAgent does)
cargo make release-validate
```

### Per-Agent Quick Checks
```bash
# MacroReviewAgent
cargo expand --lib && cargo make clippy

# TestOrchestratorAgent
cargo make test-lib-deterministic && cargo make test-feature-combinations

# ArchitectureGuardian
cargo make lint && cargo make slo-check

# PerformanceAnalystAgent
cargo make bench-compare && cargo make slo-check

# DocMaintainerAgent
cargo make build-examples && cargo test --doc

# ReleaseConductorAgent
cargo make release-check && cargo make publish-dry-run-macros
```

---

## Files in This Design

1. **AGENT_SPECIFICATIONS.md** — Detailed specs for each of 6 agents
2. **AGENT_IMPLEMENTATION_GUIDE.md** — Practical templates and configuration
3. **AGENT_DESIGN_SUMMARY.md** — This file, executive overview

---

## References

- **CLAUDE.md**: Project guidelines, ADL decisions, critical rules
- **Makefile.toml**: All build and test task definitions
- **Cargo.toml**: Feature flags, dependency list, lint configuration
- **src/lib.rs**: Core module structure
- **clap-noun-verb-macros/src/lib.rs**: Macro implementations

---

## Next Steps

1. **Review** AGENT_SPECIFICATIONS.md for detailed responsibilities and criteria
2. **Study** AGENT_IMPLEMENTATION_GUIDE.md for practical execution templates
3. **Integrate** agents into CI/CD workflow (see GitHub Actions section)
4. **Assign** agents to team members or automated workflows
5. **Monitor** agent metrics and adjust SLOs based on real-world performance
6. **Iterate** on agent designs quarterly as project evolves

---

## Questions and Clarifications

**Q: Can agents run in parallel?**
A: Yes, when independent (e.g., MacroReviewAgent and TestOrchestratorAgent). Serial execution is recommended only for release workflows where order matters.

**Q: What if an agent finds multiple issues?**
A: Agent should report all issues with specific locations (file, line) and suggestions for fixes, prioritized by severity.

**Q: How do agents interact with code review?**
A: Agents provide detailed, actionable feedback. Code reviewers can use agent reports as a starting point and add human judgment on design and approach.

**Q: Can we customize SLOs?**
A: Yes, but changes should be documented in CLAUDE.md ADL section. Current SLOs are well under targets, so future increases are unlikely.

**Q: What if a tool is missing?**
A: Agents are designed with 4 core tools (Read, Grep, Bash, Edit). If a different tool is needed, update both the agent spec and the implementation guide.

