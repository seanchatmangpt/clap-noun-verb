> Archived 2026-08-20: superseded/stale as of v26.8.20.

# Specialized Agent System: Complete Index and Navigation

This document serves as the master index for the specialized subagent system designed for clap-noun-verb.

---

## Documentation Structure

### Core Documents (Read in Order)

1. **AGENT_DESIGN_SUMMARY.md** (Start here)
   - Executive overview of all 6 agents
   - Design principles and philosophy
   - Quick reference tables and mappings
   - Agent-to-ADL correspondence
   - **Time to read**: 15-20 minutes
   - **Best for**: Understanding the big picture

2. **AGENT_SPECIFICATIONS.md** (Reference guide)
   - Detailed specifications for each of 6 agents
   - Core responsibilities (3-5 per agent)
   - Tool access requirements
   - Decision-making criteria (5-7 per agent)
   - Success metrics (quantitative and qualitative)
   - Example review checklists
   - **Time to read**: 30-40 minutes
   - **Best for**: Understanding what each agent does and how to evaluate success

3. **AGENT_IMPLEMENTATION_GUIDE.md** (Practical handbook)
   - Templates for invoking each agent
   - Command checklists for manual execution
   - CI/CD pipeline integration patterns
   - GitHub Actions workflow examples
   - Agent communication patterns (serial and parallel)
   - Troubleshooting specific agent issues
   - Customization and extension guidance
   - **Time to read**: 20-30 minutes
   - **Best for**: Actually using agents, setting up automation, troubleshooting

4. **AGENT_DECISION_TREES.md** (Operational playbooks)
   - Decision trees for each agent
   - Tactical playbooks for common scenarios
   - When to PASS, FAIL, or YELLOW each situation
   - Universal decision rules
   - **Time to read**: 25-35 minutes
   - **Best for**: Agents making decisions, understanding nuance and exceptions

5. **AGENT_SYSTEM_INDEX.md** (This file)
   - Master navigation guide
   - Quick lookup tables
   - FAQ
   - Glossary
   - **Time to read**: 5-10 minutes
   - **Best for**: Finding what you need, navigating between documents

---

## Quick Navigation by Use Case

### I want to understand the overall design
→ Read **AGENT_DESIGN_SUMMARY.md**

### I want to implement these agents
→ Read **AGENT_IMPLEMENTATION_GUIDE.md**

### I want to know what each agent does
→ Read **AGENT_SPECIFICATIONS.md**

### I need to configure an agent to make a decision
→ Read **AGENT_DECISION_TREES.md**

### I need to find something specific
→ Use the **Quick Lookup** tables below

### I'm troubleshooting an agent
→ See **Troubleshooting** section in **AGENT_IMPLEMENTATION_GUIDE.md**

### I want to understand ADL correspondence
→ See **Agent-to-ADL Mapping** in **AGENT_DESIGN_SUMMARY.md**

### I need to set up CI/CD integration
→ See **GitHub Actions Integration** in **AGENT_IMPLEMENTATION_GUIDE.md**

---

## Quick Lookup: Agents at a Glance

| Agent | Specialization | Key Focus | SLO | Enforces |
|-------|---|---|---|---|
| **MacroReviewAgent** | Proc-macros | Token safety, distributed slices, validation | 0 clippy warnings | ADL-002, ADL-008 |
| **TestOrchestratorAgent** | Quality assurance | Feature matrix, flakiness, behavioral tests | <1s, 100% pass | Test quality rules |
| **ReleaseConductorAgent** | Publishing | Macros-first, version sync, crates.io | 0 errors | Publishing workflow |
| **ArchitectureGuardian** | Design | ADL compliance across all 10 ADLs | 0 violations | ADL-001 to ADL-010 |
| **PerformanceAnalystAgent** | Performance | Compilation time, binary size, benchmarks | 2s/10MB | ADL-009 |
| **DocMaintainerAgent** | Documentation | Version sync, examples, ADL accuracy | 100% doc-tests | CLAUDE.md sync |

---

## Quick Lookup: Tools and Responsibilities

### MacroReviewAgent Tools
```
Read: Review macro implementation, test code
Grep: Search for token patterns, validate syntax
Bash: Execute cargo expand, cargo make clippy
Edit: Fix macro implementations
```

### TestOrchestratorAgent Tools
```
Bash: Execute all test commands, time runs
Read: Examine test implementations
Grep: Find test patterns, detect anti-patterns
Edit: Improve test quality
```

### ReleaseConductorAgent Tools
```
Bash: Execute publish commands, manage versions
Read: Review version files, CHANGELOG
Edit: Update versions and changelog entries
Grep: Verify version consistency
```

### ArchitectureGuardian Tools
```
Read: Review architecture, ADL documents
Grep: Search for violations (unwrap, async in traits, etc.)
Bash: Run compliance checks (clippy, slo-check, frontier checks)
Edit: Fix architectural violations
```

### PerformanceAnalystAgent Tools
```
Bash: Run cargo bench, cargo build with timing, cargo bloat
Read: Review dependency list, benchmark code
Grep: Find performance issues (large arrays, excessive clones)
Edit: Optimize Cargo.toml, code optimizations
```

### DocMaintainerAgent Tools
```
Read: Review documentation, examples, ADLs
Grep: Find broken links, version references, outdated patterns
Bash: Run cargo test --doc, cargo make build-examples
Edit: Update documentation, examples, ADLs
```

---

## Quick Lookup: Success Metrics

| Agent | Primary Metric | Target | Current |
|-------|---|---|---|
| **MacroReviewAgent** | Clippy violations | 0 | 0 |
| **TestOrchestratorAgent** | Suite execution time | <1000ms | TBD |
| **TestOrchestratorAgent** | Feature combos passing | 23/23 | TBD |
| **ReleaseConductorAgent** | Publish errors | 0 | 0 |
| **ArchitectureGuardian** | ADL violations | 0 | 0 |
| **PerformanceAnalystAgent** | Compile time | <=2000ms | 660ms |
| **PerformanceAnalystAgent** | Binary size | <=10MB | 2.2MB |
| **DocMaintainerAgent** | Doc-test pass rate | 100% | TBD |

---

## Quick Lookup: Decision Criteria

### MacroReviewAgent Criteria
1. Token streams preserve spans (hygiene)
2. Compile-time validation completeness (4 Poka-Yoke gaps)
3. Error messages are actionable
4. Distributed slice generation correct
5. MSRV compliance (Rust 1.74+)
6. Feature gates respected

### TestOrchestratorAgent Criteria
1. Determinism (serial == parallel)
2. Execution time <1 second
3. Behavioral assertions (not implementation)
4. Feature matrix: 23/23 passing
5. Test independence (no execution order dependency)
6. No I/O blocking in tests

### ReleaseConductorAgent Criteria
1. Macros published before main (dependency order)
2. Version consistency across Cargo.toml files
3. CHANGELOG updated
4. SLOs met
5. Dry-runs succeed
6. Git tag created and pushed

### ArchitectureGuardian Criteria
1. Noun-verb command pattern (ADL-001)
2. Proc-macro design (ADL-002)
3. JSON output format (ADL-003)
4. Async/sync verb rules (ADL-004)
5. No panics (ADL-005)
6. Feature gating (ADL-006)
7. Minimalist core (ADL-007)
8. Distributed slices (ADL-008)
9. SLO targets (ADL-009)
10. Trait design (ADL-010)

### PerformanceAnalystAgent Criteria
1. Incremental compile <=2s (target), <5% regression tolerance
2. Binary size <=10MB (target), <20% regression tolerance
3. Benchmark regressions <10%
4. No feature bloat
5. No large stack allocations
6. Minimal unnecessary clones

### DocMaintainerAgent Criteria
1. Version consistency (Cargo.toml == CLAUDE.md)
2. Feature list sync (Cargo.toml == CLAUDE.md)
3. Critical rules match lint configuration
4. All examples compile and run
5. All doc-tests pass
6. All ADLs accurate
7. No dead links
8. No references to removed modules

---

## Quick Lookup: ADL Mapping

| ADL | Title | Agent Responsible |
|-----|-------|------------------|
| **ADL-001** | Noun-Verb Pattern | ArchitectureGuardian |
| **ADL-002** | Proc-Macro Design | MacroReviewAgent |
| **ADL-003** | JSON Output | ArchitectureGuardian |
| **ADL-004** | Async-First Verbs | ArchitectureGuardian |
| **ADL-005** | No Panics | MacroReviewAgent, ArchitectureGuardian |
| **ADL-006** | Feature Gating | ArchitectureGuardian |
| **ADL-007** | Minimalist Core | ArchitectureGuardian |
| **ADL-008** | Distributed Slices | MacroReviewAgent |
| **ADL-009** | SLO Targets | PerformanceAnalystAgent |
| **ADL-010** | Trait Design | ArchitectureGuardian |

---

## Quick Lookup: Makefile.toml Tasks

### Format and Lint
```
cargo make format-check    → Verify formatting
cargo make format          → Auto-format code
cargo make clippy          → Run clippy linter
cargo make lint            → All linting checks
```

### Testing
```
cargo make test                      → Quick tests (parallel)
cargo make test-lib-deterministic    → Deterministic (serial)
cargo make test-all                  → All features
cargo make test-frontier             → All features + frontier
cargo make test-frontier-matrix      → 23 feature combinations
cargo make test-integration          → Integration tests
```

### Building
```
cargo make build           → Debug build
cargo make build-release   → Release build
cargo make build-examples  → Build examples
cargo make check           → Type check
cargo make check-all       → Type check all features
```

### Performance
```
cargo make bench           → Run all benchmarks
cargo make bench-baseline  → Save baseline
cargo make bench-compare   → Compare to baseline
cargo make slo-check       → Verify SLO targets
```

### Verification
```
cargo make verify          → Quick verify (format, clippy, tests)
cargo make ci              → Full CI suite
cargo make release-check   → Pre-release validation
cargo make release-validate → Comprehensive release checks
```

### Publishing
```
cargo make publish-dry-run-macros  → Dry-run macros publish
cargo make publish-macros          → Publish macros
cargo make publish-dry-run         → Dry-run main publish
cargo make publish                 → Publish main
cargo make verify-publish          → Verify on crates.io
cargo make publish-all             → Complete workflow
```

---

## Quick Lookup: Common Command Sequences

### Pre-PR Check
```bash
cargo make format
cargo make lint
cargo make test
cargo make check-all
# All pass? → Ready for PR
```

### Pre-Merge Check
```bash
cargo make ci
# Full CI suite runs:
# - format-check, clippy, test-feature-combinations
# - test-unfailable, build-examples, check-all
```

### Pre-Release Check
```bash
cargo make release-validate
# Comprehensive checks:
# - andon-check, test-frontier-matrix, coverage-report
# - bench-compare, slo-check, security-scan
# - build-release, doc
```

### Performance Investigation
```bash
touch src/lib.rs && time cargo make build
cargo make bench-baseline
cargo make bench-compare
cargo bloat --release -n 20
cargo tree --duplicates
```

### Documentation Check
```bash
cargo make build-examples
cargo test --doc
grep -E 'version|Version' CLAUDE.md
grep -E '^version' Cargo.toml
```

---

## FAQ: Agent System

### Q: When should agents run?
**A**: Before merge (MacroReviewAgent, ArchitectureGuardian, TestOrchestratorAgent), and before release (all agents).

### Q: Can agents run in parallel?
**A**: Yes, when independent (e.g., MacroReviewAgent and TestOrchestratorAgent). Serial execution for release workflows.

### Q: What if an agent finds multiple issues?
**A**: Report all issues with specific locations and suggestions. Prioritize by severity (blocker > critical > warning).

### Q: How do I customize SLOs?
**A**: Update CLAUDE.md ADL section and Makefile.toml. Communicate changes to team; current SLOs are well under targets.

### Q: Can I add new agents?
**A**: Yes. Follow the template: responsibilities (3-5), tools, criteria (5-7), metrics. Update this index.

### Q: What if a tool is missing?
**A**: File a request or update agent spec. Core tools: Read, Grep, Bash, Edit. Others available if specified.

### Q: How do agents interact?
**A**: MacroReviewAgent checks code → ArchitectureGuardian checks design → TestOrchestratorAgent validates tests → PerformanceAnalystAgent measures → DocMaintainerAgent syncs docs → ReleaseConductorAgent publishes.

### Q: What's the difference between PASS, FAIL, and YELLOW?
**A**: PASS = Accept/proceed. FAIL = Block/require fix. YELLOW = Caution/monitor/review recommended.

### Q: How often should I run agents?
**A**: Per commit (automated in CI), before PR (local), before release (comprehensive).

### Q: Where do agents get their standards from?
**A**: CLAUDE.md (critical rules, ADLs), Makefile.toml (build tasks), Cargo.toml (lint rules, features).

---

## Glossary: Key Terms

### Agent-Specific Terms
- **PASS** - Agent accepts the code/release; no blocking issues
- **FAIL** - Agent rejects; blocking issue(s) found; must be fixed
- **YELLOW** - Agent warns; monitor/review recommended; not blocking
- **Blocker** - Issue that must be fixed before merge
- **SLO** - Service Level Objective; target metric (e.g., <=2s compile)

### Architecture Terms
- **ADL** - Architecture Decision Log (10 decisions documented in CLAUDE.md)
- **Noun-Verb** - Command pattern (e.g., `services status`)
- **Verb** - Action/operation (e.g., `status`, `create`, `delete`)
- **Noun** - Logical grouping (e.g., `services`, `database`)
- **Distributed Slice** - Compile-time verb discovery via `linkme`
- **Proc-Macro** - Procedural macro (`#[verb]`, `#[noun]`)

### Quality Terms
- **AAA Pattern** - Test structure (Arrange, Act, Assert)
- **Behavioral Test** - Verifies observable output/state, not implementation
- **Flaky Test** - Test that passes/fails intermittently
- **Deterministic** - Same results every time (no randomness)
- **Feature Matrix** - All combinations of feature flags (23 combos)

### Performance Terms
- **SLO** - Service Level Objective (target metric)
- **Incremental Compile** - Rebuild with small change (target: <=2s)
- **Binary Size** - Release artifact size (target: <=10MB)
- **Regression** - Performance decrease from baseline
- **Profiling** - Measuring execution/memory to find bottlenecks

### Documentation Terms
- **CLAUDE.md** - Central project guidance document
- **Doc-Test** - Code examples in documentation that are tested
- **Dead Link** - Reference to non-existent section/file
- **Version Sync** - Keeping version numbers consistent across files
- **ADL Accuracy** - Ensuring ADL entries match actual code

---

## Implementation Roadmap

### Phase 1: Design Review (Week 1)
- [ ] Read AGENT_DESIGN_SUMMARY.md
- [ ] Review AGENT_SPECIFICATIONS.md
- [ ] Discuss design with team
- [ ] Identify any customizations needed

### Phase 2: Local Implementation (Week 2)
- [ ] Follow templates in AGENT_IMPLEMENTATION_GUIDE.md
- [ ] Set up agents locally
- [ ] Run manual checks per Decision Trees
- [ ] Validate against example scenarios

### Phase 3: CI/CD Integration (Week 3)
- [ ] Copy GitHub Actions workflow from AGENT_IMPLEMENTATION_GUIDE.md
- [ ] Configure agents in your CI/CD system
- [ ] Test on a feature branch
- [ ] Verify feedback is clear and actionable

### Phase 4: Team Adoption (Week 4)
- [ ] Train team on agent system
- [ ] Document any customizations
- [ ] Monitor metrics and SLO compliance
- [ ] Iterate based on feedback

### Phase 5: Optimization (Ongoing)
- [ ] Quarterly review of agent performance
- [ ] Adjust decision criteria based on experience
- [ ] Add new agents as needs evolve
- [ ] Keep documentation in sync

---

## Troubleshooting Quick Reference

### Agent Hangs
→ Check for infinite loops or network I/O in tests (TestOrchestratorAgent)
→ Reduce feature matrix size or run tests serially

### Agent False Positives
→ Add context to agent prompt (e.g., "allow 'latest version' references")
→ Review decision criteria; may need customization for your project

### Agent Output Unclear
→ Request detailed output with file paths and line numbers
→ Refer to example output in AGENT_IMPLEMENTATION_GUIDE.md

### SLO Violations
→ Run performance analysis (cargo make bench-compare)
→ Profile slow paths (cargo make profile)
→ Optimize dependencies or code before merge

### Test Failures
→ Run deterministic suite (cargo make test-lib-deterministic)
→ Check for time-dependent logic or shared state
→ Review decision tree in AGENT_DECISION_TREES.md

---

## Key Files to Review

### Project Configuration
- `CLAUDE.md` - Project guidelines and ADL
- `Cargo.toml` - Features, dependencies, lints
- `Makefile.toml` - Build tasks and test suites
- `.github/workflows/` - CI/CD configuration

### Agent Documentation (This System)
- `AGENT_DESIGN_SUMMARY.md` - Big picture
- `AGENT_SPECIFICATIONS.md` - Detailed specs
- `AGENT_IMPLEMENTATION_GUIDE.md` - How to use
- `AGENT_DECISION_TREES.md` - Decision logic
- `AGENT_SYSTEM_INDEX.md` - This file

### Codebase Architecture
- `src/lib.rs` - Main crate structure
- `src/verb.rs` - Verb trait definition
- `clap-noun-verb-macros/src/lib.rs` - Macro implementations
- `src/cli/registry.rs` - Command registry
- `src/error.rs` - Error handling

---

## References and Links (Logical)

### CLAUDE.md Sections
- Build Commands → Makefile.toml tasks
- Crate Structure → src/ layout
- Architecture → Core flow and modules
- Critical Rules → What agents enforce
- ADL (ADL-001 to ADL-010) → Architecture decisions
- Development Workflows → Best practices
- Troubleshooting → Common issues

### This Agent System Documents
- Summary → Design principles
- Specifications → Detailed agent specs
- Implementation → How to set up
- Decision Trees → How to decide
- Index → You are here

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-06-14 | Initial design for clap-noun-verb 26.6.14 |

---

## Contact and Escalation

### For Agent Questions
→ Review AGENT_DECISION_TREES.md for decision guidance
→ Check AGENT_IMPLEMENTATION_GUIDE.md troubleshooting section

### For Architectural Disputes
→ Reference CLAUDE.md ADL entries
→ Escalate if change conflicts with multiple ADLs
→ Document decision in ADL-11 if new pattern emerges

### For Performance Issues
→ Use AGENT_IMPLEMENTATION_GUIDE.md performance validation script
→ Profile with cargo tools (bench, bloat, build-time)
→ Review PerformanceAnalystAgent decision tree

### For Documentation Gaps
→ File issue with specific missing documentation
→ Reference DocMaintainerAgent checklist
→ Update CLAUDE.md or examples with clarification

---

## End of Index

For detailed information, see the referenced documents:
- **AGENT_DESIGN_SUMMARY.md** - Overview and philosophy
- **AGENT_SPECIFICATIONS.md** - Detailed agent specs
- **AGENT_IMPLEMENTATION_GUIDE.md** - Practical guide
- **AGENT_DECISION_TREES.md** - Decision logic

Last updated: 2026-06-14
System designed for: clap-noun-verb v26.6.14
Rust MSRV: 1.74+
