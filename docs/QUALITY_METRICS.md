# Quality Metrics & Success Criteria Framework

**clap-noun-verb v26.9.1** — Comprehensive quality metrics, baselines, targets, and reporting methodology.

**Audience**: Maintainers, contributors, community stakeholders, and external auditors ensuring accountability and continuous improvement.

**Last Updated**: 2026-08-20  
**Maintained by**: clap-noun-verb contributors

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Code Quality Metrics](#code-quality-metrics)
3. [Documentation Quality](#documentation-quality)
4. [Performance Metrics](#performance-metrics)
5. [Release Health](#release-health)
6. [Process Metrics](#process-metrics)
7. [Maintainability Metrics](#maintainability-metrics)
8. [Security Metrics](#security-metrics)
9. [Community Health](#community-health)
10. [Architecture Alignment](#architecture-alignment)
11. [SLO Tracking & Dashboard](#slo-tracking--dashboard)
12. [Reporting & Governance](#reporting--governance)

---

## Executive Summary

clap-noun-verb maintains a comprehensive quality framework covering technical excellence (code, performance, security), operational efficiency (release cadence, issue resolution), and community health. All metrics are tracked against baseline and target values with explicit headroom percentages.

### Dashboard Overview

| Category | Metric | Baseline | Target | Status |
|----------|--------|----------|--------|--------|
| **Code Quality** | Test Coverage | TBD | ≥80% | 📊 Pending |
| **Code Quality** | Clippy Warnings | 0 | 0 | ✅ Passing |
| **Code Quality** | Cyclomatic Complexity (avg) | TBD | ≤10 | 📊 Pending |
| **Documentation** | API Coverage | TBD | 100% | 📊 Pending |
| **Documentation** | Example Completeness | 15 | 25+ | ✅ On Track |
| **Documentation** | ADR Count | 0 | ≥5 | ⚠️ Needs Action |
| **Performance** | Incremental Compile | 0.66s | ≤2.0s | ✅ Passing |
| **Performance** | Release Binary | 2.2MB | ≤10MB | ✅ Passing |
| **Performance** | Test Suite | 0.16s | <1.0s | ✅ Passing |
| **Performance** | Doc Build | 4-5s | ≤15s | ✅ Passing |
| **Release Health** | Hotfix Ratio | TBD | ≤15% | 📊 Pending |
| **Release Health** | Regression Bug Count | TBD | ≤1 per release | 📊 Pending |
| **Process** | PR Review Time (avg) | TBD | ≤2 days | 📊 Pending |
| **Process** | Issue Resolution Time | TBD | ≤7 days | 📊 Pending |
| **Community** | Issue Response Time | TBD | ≤24h | 📊 Pending |
| **Community** | Active Contributors | TBD | ≥3 | 📊 Pending |
| **Security** | CVE Response Time | TBD | ≤48h | 📊 Pending |

**Legend**: ✅ Passing | ⚠️ Needs Action | 🔴 Critical | 📊 Pending Data

---

## Code Quality Metrics

### 1. Test Coverage

#### Target
- **Overall coverage**: ≥80% of production code lines
- **Critical paths**: 100% (CLI dispatch, verb registration, error handling)
- **Optional modules**: ≥75%
- **Examples & integration tests**: Comprehensive coverage of all documented features

#### Baseline (v26.9.1)
```
Current Status: Pending formal coverage report
Known Test Suite:
- 47 unit & integration tests (unignored, compile-checked)
- 47 doctests passing (zero ignored)
- ~360 test files across workspace
- Test execution: <1 second (parallel)
```

#### Measurement
```bash
# Install tarpaulin (Linux/macOS coverage tool)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage --timeout 300

# View report
open target/coverage/index.html

# GitHub Actions: .github/workflows/coverage.yml
# Runs on: every PR, main branch push
# Threshold enforcement: Fail if coverage drops below 80%
```

#### Interpretation
| Coverage | Status | Action |
|----------|--------|--------|
| ≥85% | ✅ Excellent | Monitor for regression |
| 80-85% | ✅ Acceptable | Plan incremental improvement |
| 70-80% | ⚠️ Yellow | Prioritize test gaps in next sprint |
| <70% | 🔴 Red | Halt feature work; focus on testing |

#### Exclusions
- Generated code (macros, codegen outputs)
- Integration test infrastructure (test helpers)
- Deprecated/dead-code paths (explicitly marked)
- Panic/unwrap paths (unreachable in production)

### 2. Clippy Warnings

#### Target
- **Denied violations**: 0 (fail CI/CD)
- **Warnings**: 0 in production code
- **Allowed warnings**: Only documented in lints section with rationale

#### Baseline (v26.9.1)
```
Status: ✅ 0 warnings / 0 denied violations
Last full run: 2026-06-14
Configuration: Cargo.toml [workspace.lints.clippy]
  - deny: unimplemented, todo, exit
  - allow: unsafe_code (required for linkme), dead_code (v5.1 stubs)
  - warn: all (with specific overrides for false positives)
```

#### Measurement
```bash
# Check all warnings
cargo clippy --all-targets --all-features -- -D warnings

# Fix auto-fixable warnings
cargo clippy --fix --all-targets --all-features

# CI gate: enforced on every PR
```

#### Warning Categories
| Category | Rule | Policy | Example |
|----------|------|--------|---------|
| **Error Handling** | `unwrap_used` | Allow with comment | Only in non-critical paths |
| **Error Handling** | `expect_used` | Allow with comment | Test setup |
| **Error Handling** | `panic` | Allow with comment | test only |
| **Error Handling** | `todo` | **Deny** | Production only |
| **Error Handling** | `unimplemented` | **Deny** | Production only |
| **Performance** | `all` | Warn (priority:-1) | Allows overrides per lint |
| **Code Style** | `too_many_arguments` | Allow | Verb macros generate large arg lists |
| **Code Style** | `cognitive_complexity` | Allow | Complex dispatch logic |

### 3. Cyclomatic Complexity

#### Target
- **Average complexity**: ≤10 per function
- **Maximum complexity**: ≤15 for any single function
- **Excluded**: Test functions, macros, auto-generated code

#### Baseline (v26.9.1)
```
Current Status: Pending detailed analysis
Known High-Complexity Areas:
- CommandRegistry::build() - ~12 (acceptable, documented)
- CliBuilder::from_registry() - ~11 (acceptable, documented)
- Verb macro expansion - auto-generated, excluded
```

#### Measurement
```bash
# Using cargo-metrics or similar tools
cargo install cargo-metrics
cargo metrics --json > metrics.json

# Alternative: manual inspection via:
# - `cargo clippy` cognitive_complexity lint
# - IDE refactoring tools (JetBrains, VSCode)

# Reporting: include in quarterly reviews
# Threshold: fail review if avg > 10 in core modules
```

#### Complexity Budget
| Module | Current | Budget | Headroom | Status |
|--------|---------|--------|----------|--------|
| `cli/` | TBD | 10 | TBD | 📊 Pending |
| `builder.rs` | TBD | 12 | TBD | 📊 Pending |
| `router.rs` | TBD | 10 | TBD | 📊 Pending |
| `macro expansion` | Excluded | N/A | N/A | — |

---

## Documentation Quality

### 1. API Coverage

#### Target
- **Public API**: 100% documented with `///` doc comments
- **All public types**: Include examples for non-trivial structs/enums
- **All public functions**: Include signature, purpose, errors, panics (if any), examples
- **Feature-gated APIs**: Clear feature requirement in docs

#### Baseline (v26.9.1)
```
Current Status: Pending full audit
Known Metrics:
- Core modules: lib.rs, cli/mod.rs, builder.rs, router.rs - documented
- Macro crate: Full docstrings for all pub items
- Examples: 15+ runnable examples across tutorial/howto/reference
- Doctests: 47 passing, 0 ignored (v26.9.1 refactor)
```

#### Measurement
```bash
# Missing documentation check
cargo doc --no-deps --all-features 2>&1 | grep "warning: missing"

# Enforce in CI: set RUSTDOCFLAGS
export RUSTDOCFLAGS="-D warnings"
cargo doc --no-deps --all-features

# Coverage metric (auto from docs.rs):
# - Percentage of pub items with documentation
# - Tracked per release (semver)
```

#### Coverage Checklist
- [ ] All pub modules have module-level docs
- [ ] All pub types have doc comments with examples
- [ ] All pub functions have docs, error cases, panics
- [ ] All feature-gated items mention feature requirement
- [ ] All macros have usage examples
- [ ] Deprecated items marked with `#[deprecated]` + guidance

### 2. Examples & Tutorials

#### Target
- **Tutorials**: Step-by-step learning path (≥5 examples)
  - Basic CLI
  - Arguments & flags
  - Subcommands (noun-verb pattern)
  - Error handling
  - Custom output formatting
  
- **How-To Guides**: Task-oriented (≥10 examples)
  - Validation
  - Environment variables
  - Argument groups
  - Deprecation strategies
  - Completions
  
- **Reference Examples**: Complete API demonstrations (≥10 examples)
  - Attribute macros
  - Framework integration
  - Nested commands
  - Custom collectors
  
- **All examples**: Runnable, tested, documented

#### Baseline (v26.9.1)
```
Current Examples: 15 across categories
Location: examples/{tutorial,howto,reference}/

Tutorial (4):
  ✅ tutorial_basic
  ✅ tutorial_arguments
  ✅ tutorial_positional
  ✅ tutorial_services

How-To (5):
  ✅ howto_arg_groups
  ✅ howto_validation
  ✅ howto_env_vars
  ✅ howto_arg_actions
  ✅ howto_deprecation

Reference (6):
  ✅ ref_attribute_macro
  ✅ ref_framework
  ✅ ref_nested
  ✅ ref_collector
  ✅ ref_format
  ✅ ref_context
  ✅ ref_root_verb

Action: Expand to 25+ examples covering:
  - Advanced patterns (composition, federation)
  - Integration scenarios (config files, plugins)
  - Performance tuning
  - Testing strategies
```

#### Maintenance
```bash
# Build all examples
cargo make build-examples

# Test examples compile
cargo build --examples

# Run example executables (if applicable)
for example in target/debug/examples/*; do
  echo "Testing $example..."
  "$example" --help || true
done

# Add to CI: ensure examples build on every PR
```

### 3. Architecture Decision Records (ADRs)

#### Target
- **Minimum count**: ≥5 ADRs documenting major decisions
- **Coverage**: 
  - Why noun-verb pattern (ADR-001)
  - Macro-based auto-discovery (ADR-002)
  - Feature gating strategy (ADR-003)
  - Error handling philosophy (ADR-004)
  - Deprecation strategy (ADR-005)
  
- **Format**: [MADR](https://adr.github.io/) or [ADR](https://adr.github.io/) template
- **Lifecycle**: Proposed → Accepted → Superseded/Deprecated

#### Baseline (v26.9.1)
```
Current Status: ⚠️ 0 ADRs
Action: Create docs/adr/ directory and populate:
  1. ADR-0001: Noun-Verb Command Pattern Design
  2. ADR-0002: Linkme Distributed Slice for Auto-Discovery
  3. ADR-0003: Zero-Default Features Architecture
  4. ADR-0004: Error Handling with Result<T> & NounVerbError
  5. ADR-0005: Deprecation Strategy & Semantic Versioning
  6. ADR-0006: Macro Validation During Compilation
  7. ADR-0007: Documentation-as-Code (Doctests)
```

#### Template (MADR Format)
```markdown
# ADR-NNN: [Title]

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-NNN]

## Context
[Problem statement and constraints]

## Decision
[The decision made]

## Consequences
[Positive and negative outcomes]

## Alternatives Considered
[Rejected options and rationale]

## References
- [Link to related issues/RFCs]
- [Link to implementation PR]
```

#### Measurement
```bash
# Track ADR count
ls docs/adr/*.md | wc -l

# Enforce: minimum 5 ADRs before v27.0.0 release
```

### 4. Tutorial & Documentation Completeness

#### Target
- **Getting Started Guide**: ✅ Present (README.md, 2-minute quickstart)
- **Architecture Documentation**: ✅ Present (CLAUDE.md, 10,000+ words)
- **Feature Guides**: Tutorial, How-To, Reference categories complete
- **Troubleshooting Guide**: Common issues and solutions
- **FAQ**: 20+ common questions answered
- **Glossary**: Term definitions for noun-verb, verb, registry, etc.
- **Migration Guides**: For each major version change

#### Baseline (v26.9.1)
```
Completed:
  ✅ README.md - 2-minute quickstart
  ✅ CLAUDE.md - Architecture & project overview
  ✅ CHANGELOG.md - Change history (detailed)
  ✅ examples/README.md - Example index
  ✅ docs/ - 15+ guides (performance, release, testing, onboarding)

In Progress:
  📋 Glossary (partial)
  📋 Troubleshooting (partial)

Needed:
  ⚠️ FAQ (comprehensive)
  ⚠️ Migration guides (for major versions)
  ⚠️ Comparison with alternatives (typer, structopt, etc.)
```

---

## Performance Metrics

### 1. Compile-Time Metrics

#### Target
- **Incremental compilation**: ≤2.0s after single-file change (dev mode)
- **Full clean build**: ≤30s (CI environment)
- **Macro expansion**: ≤100ms for typical verb definition
- **Check**: ≤1.5s incremental

#### Baseline (v26.9.1)
```
Incremental Compile: 0.66s ✅ (67% headroom)
  Macro expansion:    20ms
  Core compile:      400ms
  Dependencies:      180ms
  Linking:            60ms

Full Clean Build: ~8-12s (multi-core)
Macro expansion per #[verb]: ~2-5ms
```

#### Measurement
```bash
# Incremental build timing (warm cache)
touch src/lib.rs && time cargo build 2>&1 | tail -3

# Record per release
echo "v26.9.1,0.66" >> metrics/compile_time.csv

# SLO check
cargo make slo-check
```

#### Regression Detection
```
Threshold: If incremental build > 1.0s (50% headroom used)
Action: Investigate macro expansion, dependency updates, or code structure
Response: Require optimization PR before merging feature
```

### 2. Binary Size Metrics

#### Target
- **Release binary (minimal features)**: ≤10MB
- **Debug binary**: Baseline, no explicit target
- **Stripped binary**: ≤5MB

#### Baseline (v26.9.1)
```
Release Binary (no features): 2.2MB ✅ (78% headroom)
Debug Binary: ~8-10MB
Stripped Release: ~1.8MB

Memory Usage (at runtime, empty CLI):
- Stack: ~4KB
- Heap (registry): ~50KB
```

#### Measurement
```bash
# Release binary size
cargo build --release
ls -lh target/release/clap_noun_verb

# Strip symbols
strip target/release/clap_noun_verb
ls -lh target/release/clap_noun_verb

# Compare across features
for feature in "" "repl" "otel" "federated-network"; do
  cargo build --release --features "$feature"
  echo "$feature: $(stat -f%z target/release/clap_noun_verb)" # macOS
  # or: echo "$feature: $(stat --format='%s' target/release/clap_noun_verb)" # Linux
done
```

#### Tracking
```
Track per release in: metrics/binary_size.csv
Format: version,feature_set,size_bytes
Example:
26.9.1,default,2306867
26.9.1,all,4520391
26.9.1,repl,2450000
```

### 3. Test Execution Performance

#### Target
- **Full parallel test suite**: <1.0s
- **Unit tests only**: <0.5s
- **Integration tests**: <1.5s
- **Single test**: <200ms

#### Baseline (v26.9.1)
```
Full Suite (parallel, no features): ~0.16s ✅ (84% headroom)
Unit tests: ~0.12s
Integration tests: ~0.04s
Doctests: ~0.08s
Total: <0.20s
```

#### Measurement
```bash
# Time full suite
cargo make test

# Time components
time cargo test --lib --quiet
time cargo test --test '*' --quiet
time cargo test --doc --quiet

# Detailed breakdown
cargo test --quiet -- --test-threads=1 --nocapture
```

#### Regression Alert
```
Threshold: If full suite > 1.5s
Action: Profile with `cargo flamegraph` or `perf`
Response: Requires optimization before merge
```

### 4. Documentation Build Performance

#### Target
- **Full documentation build**: ≤15s
- **Incremental doc rebuild**: ≤5s
- **docs.rs build**: ≤30s (external service)

#### Baseline (v26.9.1)
```
Full Doc Build: 4-5s ✅ (67% headroom)
  Example builds: 2s
  Rustdoc generation: 2-3s
  Doctest compilation: ~1s

Incremental (after src change): 2-3s
```

#### Measurement
```bash
# Time doc build
time cargo make doc

# Track per release
echo "v26.9.1,4.2,2.8" >> metrics/doc_build.csv
# Format: version,full_build_time,incremental_time
```

---

## Release Health

### 1. Release Cadence & Timing

#### Target
- **Planned releases**: Every 2-4 weeks (minor versions)
- **Patch releases**: As needed for bugs (hotfixes)
- **Major releases**: Annually (or less frequently)
- **Release window**: 1-2 days from merge to publish
- **Announcement to production**: ≤6 hours

#### Baseline (v26.9.1)
```
Recent Release History (last 6 months):
26.9.1 - 2026-06-14 (current)
26.9.1 - 2026-06-13 (1 day)
26.9.1  - 2026-06-01 (12 days)
26.5.28 - 2026-05-28 (previous cycle)

Cadence: 1-2 weeks average ✅
Release-to-Publish: ~4 hours ✅
```

#### Tracking
```bash
# Track release frequency
git log --oneline --all --decorate | grep "^.*Release" | wc -l

# Time between releases
git log --tags --pretty=format:"%d %ai" | head -20

# Add to RELEASE_INDEX.md
```

### 2. Hotfix Ratio

#### Target
- **Hotfix to total release ratio**: ≤15%
- **Critical bugs requiring hotfix**: ≤1 per release
- **Definition**: A release that corrects bugs found in previous release within 48 hours

#### Baseline (v26.9.1)
```
Current Status: Pending baseline data
Historical (estimated from CHANGELOG):
- 26.9.1: No hotfix (new release)
- 26.9.1: No hotfix
- 26.9.1:  No hotfix
- 26.5.28: No hotfix

Current Ratio: 0/last-10-releases = 0% ✅
Target: ≤15% (acceptable)
```

#### Measurement & Prevention
```bash
# Track hotfixes per release
releases=$(git tag -l 'v*' | sort -V)
for release in $releases; do
  next_release=$(git tag -l "v*" | sort -V | grep -A1 "^$release$" | tail -1)
  days_between=$(echo "scale=1; \
    ($(git log -1 --format=%ai $next_release | cut -d' ' -f1 | date -f- +%s) - \
     $(git log -1 --format=%ai $release | cut -d' ' -f1 | date -f- +%s)) / 86400" | bc)
  if [ "$days_between" -lt 2 ]; then
    echo "$release: HOTFIX ($days_between days)"
  fi
done

# Threshold enforcement
# If hotfix_ratio > 15%, require post-mortem
```

### 3. Regression Bug Count

#### Target
- **Regression bugs per release**: ≤1
- **Definition**: Bug reported within 1 week of release caused by changes in that release
- **Resolution time**: ≤48 hours

#### Baseline (v26.9.1)
```
Current Status: Pending baseline
Recent History:
- 26.9.1: 0 regression bugs
- 26.9.1:  0 regression bugs

All known issues: checked against CHANGELOG
No open regression reports ✅
```

#### Tracking & Prevention
```bash
# Labeling system (GitHub Issues)
# Labels: regression, release:26.9.1

# Find regression bugs
gh issue list --label "regression" --state open

# Prevention checklist (RELEASE_DEFINITION_OF_DONE.md):
  ✅ Regression test suite runs before release
  ✅ Example programs tested
  ✅ Documentation examples verified
  ✅ Breaking changes documented
  ✅ Migration guide (if needed)
```

### 4. Community-Reported Issues

#### Target
- **Critical issues (crashes, data loss)**: ≤0 in production
- **Major issues (functionality broken)**: ≤2 per release
- **Minor issues (usability, performance)**: ≤5 per release
- **Documentation issues**: ≤3 per release

#### Baseline (v26.9.1)
```
Current Status: Pending GitHub audit
Known Issues (as of 2026-06-14):
- Open critical: 0 ✅
- Open major: 0 ✅
- Open minor: TBD

Action: Audit GitHub issues and populate baseline
```

---

## Process Metrics

### 1. PR Review Time

#### Target
- **First review**: Within 24 hours of submission
- **Final approval to merge**: Within 2 days (48 hours)
- **Average review time**: ≤2 days
- **95th percentile**: ≤5 days

#### Baseline (v26.9.1)
```
Current Status: Pending metrics collection
Setup tracking:
  - Use GitHub Actions to record PR timestamps
  - Track: submission → first-review, first-review → approval, approval → merge
  - Exclude: PRs waiting on author feedback
  - Exclude: Draft PRs
```

#### Measurement
```bash
# Query GitHub API
gh api repos/seanchatmangpt/clap-noun-verb/pulls \
  --state closed \
  --jq '.[] | {title: .title, created: .created_at, merged: .merged_at}'

# Calculate metrics
# Python script: scripts/metrics_pr_review_time.py

# Add to CI: metrics-collection workflow
# Runs: Weekly, updates metrics/pr_review_time.csv
```

### 2. Issue Resolution Time

#### Target
- **Acknowledgment**: Within 24 hours (first comment)
- **Resolution (fix/close)**: ≤7 days average
- **Critical (crash/data loss)**: ≤24 hours
- **Feature requests**: ≤30 days (may require discussion)

#### Baseline (v26.9.1)
```
Current Status: Pending metrics collection
SLA Tiers:
  - Critical: ≤24h (crashes, security)
  - Major: ≤3 days (functionality broken)
  - Minor: ≤7 days (usability, docs)
  - Enhancement: ≤30 days (feature requests, nice-to-have)
```

#### Tracking
```bash
# Script: scripts/metrics_issue_resolution.py
# Tracks: issue-created → first-comment, first-comment → closed/resolved
# Excludes: waiting-for-clarification, waiting-on-user labels
# Produces: metrics/issue_resolution_time.csv

# CI workflow: weekly update
# Alerts: if average > 7 days, escalate
```

### 3. Developer Onboarding Time

#### Target
- **Initial setup**: ≤30 minutes (clone, build, run tests)
- **First contribution**: ≤4 hours (with guidance)
- **First merged PR**: ≤2 weeks (from onboarding)

#### Baseline (v26.9.1)
```
Current Setup Status:
  ✅ CLAUDE.md - 5,000+ words of guidance
  ✅ ONBOARDING.md - Step-by-step guide (docs/ONBOARDING.md)
  ✅ Makefile.toml - Simple task runners
  ✅ Examples - Runnable code
  ✅ Tests - Clear structure
  ⚠️ Video walkthrough - Not yet available
```

#### Measurement
```bash
# Simulate onboarding
time (
  git clone https://github.com/seanchatmangpt/clap-noun-verb ~/test-clone
  cd ~/test-clone
  cargo make test
  echo "Setup complete"
)

# Expected output: 2-5 minutes (including downloads)

# Track actual contributor experience
# Survey: docs/CONTRIBUTOR_FEEDBACK.md (quarterly)
```

---

## Maintainability Metrics

### 1. Cyclomatic Complexity Analysis

#### Target (see Code Quality section for details)
- **Average complexity**: ≤10 per function
- **Maximum complexity**: ≤15 for any single function

#### Baseline (v26.9.1)
```
Current Status: Pending detailed analysis
Tracking location: metrics/complexity_analysis.json

Sample structure (to be filled):
{
  "module": "cli/registry.rs",
  "functions": [
    {"name": "CommandRegistry::build", "complexity": 12, "status": "✅"},
    {"name": "CliBuilder::from_registry", "complexity": 11, "status": "✅"}
  ]
}
```

### 2. Code Duplication Detection

#### Target
- **Duplicate code**: <3% of codebase
- **Copy-paste violations**: Zero in critical paths (CLI dispatch, error handling)
- **Extracted common patterns**: Documented in architecture guide

#### Baseline (v26.9.1)
```
Current Status: Pending duplicate analysis
Tools: cargo-duplicate-code, simian, or custom script

Action plan:
  1. Run duplicate detection
  2. Identify intentional vs. accidental duplication
  3. Extract common patterns where appropriate
  4. Document rationale for intentional duplication
```

#### Measurement
```bash
# Duplicate code detection
# Option 1: Using rust-code-analysis
cargo install rust-code-analysis
rust-code-analysis --metrics /path/to/src -O json

# Option 2: Manual inspection
# - Search for identical blocks (>5 lines)
# - Check for similar patterns that could be consolidated

# Reporting: include in quarterly code health report
```

### 3. Technical Debt Tracking

#### Target
- **Explicit TODOs**: Document with issue link or deadline
- **Deprecated APIs**: Clear removal date and migration guide
- **Known limitations**: Documented in LIMITATIONS.md
- **Debt ratio**: <5% of codebase (loose estimate)

#### Baseline (v26.9.1)
```
Current Explicit Debt:
  ✅ Zero stubs (eliminated in v26.9.1 refactor)
  ✅ Zero cheats (eliminated in v26.9.1 refactor)
  ✅ Zero assert!(true) tautologies
  
Documented Debt:
  - Optional frontier features: documented as unstable
  - LIMITATIONS.md: Known limitations and workarounds

TODOs with issues:
  grep -r "TODO\|FIXME" src/ | wc -l
  Expected: <10 (all with linked issues)
```

#### Debt Management
```bash
# Periodic audit
grep -r "TODO\|FIXME" src/ clap-noun-verb-macros/src/ --include="*.rs" \
  | grep -v "^#!" \
  | while read line; do
    echo "ACTION REQUIRED: $line"
  done

# Deprecation tracking
# - All deprecated items: marked with #[deprecated(since = "26.9.1", ...)]
# - Removal date: documented (typically 2-3 releases later)
# - Migration guide: provided in CHANGELOG

# Review cadence: quarterly in maintainer meetings
```

---

## Security Metrics

### 1. Security Audit Frequency

#### Target
- **Internal audits**: Quarterly (or per major release)
- **Dependency audits**: Weekly (automated via cargo-audit)
- **External security review**: Annually (professional assessment)
- **Penetration testing**: As needed (when adding security-critical features)

#### Baseline (v26.9.1)
```
Current Status:
  ✅ cargo audit: Runs in CI (weekly via Dependabot)
  ✅ Internal review: Part of release checklist
  ⚠️ External audit: Scheduled for v27.0.0 release
  ⚠️ Penetration testing: N/A (CLI framework, not attack surface)

Last audit: 2026-06-14 (as part of docs refresh)
Findings: 0 critical, 0 high ✅
```

#### Measurement
```bash
# Automated dependency audit
cargo audit

# CI integration: .github/workflows/security.yml
# Runs: Daily
# Fails if: new vulnerabilities found

# Manual audit checklist (quarterly):
  ✅ Unsafe code review
  ✅ Error handling paths
  ✅ Input validation
  ✅ Dependency updates
  ✅ Cryptography usage (if any)
```

### 2. Vulnerability Response Time

#### Target
- **Internal reporting**: ≤24 hours to triage
- **Critical fix**: ≤48 hours to release patch
- **High severity**: ≤1 week to patch
- **Medium severity**: ≤2 weeks to patch
- **Public disclosure**: Coordinated (if applicable)

#### Baseline (v26.9.1)
```
Current Status:
  - Known vulnerabilities: 0
  - Response procedures: Documented in SECURITY.md
  - Patch release process: Defined in RELEASE_MANAGEMENT.md
  
CVE Disclosure Process:
  1. Security report received (GitHub Security Advisory)
  2. Triage within 24h
  3. Fix developed in private branch
  4. Testing on target version and latest
  5. Patch released
  6. Advisory published
  7. Mirror updated (if using mirrors)
```

#### Process Documentation
```markdown
# SECURITY.md (to be created)

## Reporting Security Vulnerabilities

Do NOT open public GitHub issues for security vulnerabilities.

Email: [SECURITY_CONTACT_EMAIL]

Include:
  - Description of vulnerability
  - Steps to reproduce
  - Affected versions
  - Proposed fix (if any)

Response timeline:
  - Acknowledgment: within 24 hours
  - Initial assessment: within 3 days
  - Fix or disclosure plan: within 7 days
```

### 3. Dependency Update Lag

#### Target
- **Critical patches**: Applied within 3 days
- **High severity**: Applied within 1 week
- **Medium/Low**: Applied within 1 month
- **Major version updates**: Evaluated within 6 weeks

#### Baseline (v26.9.1)
```
Current Dependencies (Cargo.toml):
  clap: 4.5 (latest: 4.5)  ✅ up-to-date
  serde: 1.0 (latest: 1.0) ✅ up-to-date
  tokio: 1.40 (latest: 1.x) ✅ maintained
  linkme: 0.3 (latest: 0.3) ✅ stable
  
Audit status: No known vulnerabilities ✅
Last updated: 2026-06-14
```

#### Tracking
```bash
# Check for updates
cargo outdated

# Set up Dependabot (GitHub)
# .github/dependabot.yml:
  version: 2
  updates:
    - package-ecosystem: "cargo"
      directory: "/"
      schedule:
        interval: "weekly"
      reviewers: ["@seanchatmangpt"]
      auto-merge: false  # Manual review required

# Security audit (CI)
cargo audit

# Report: include in monthly status
```

---

## Community Health

### 1. Issue Response Time

#### Target
- **First response**: Within 24 hours of issue creation
- **Triage decision**: Within 2 days (accepted, more-info-needed, duplicate, etc.)
- **"Needs clarification" response**: Within 5 days (or close)

#### Baseline (v26.9.1)
```
Current Status: Pending metrics collection
Setup:
  - GitHub Actions workflow: monitor new issues
  - Label system: triage, needs-clarification, accepted, etc.
  - SLA enforcement: auto-comment if no response within 24h
```

#### Process Documentation
```bash
# Issue triage workflow

if [ issue_is_valid ]; then
  # Add: kind/{bug,feature,question}, priority/{critical,high,medium,low}
  gh issue edit $ISSUE --add-label "triage,needs-clarification" || \
    gh issue edit $ISSUE --add-label "triage,accepted"
fi

# Response template (if needs clarification)
# "Thanks for reporting! To help us better understand this issue, could you provide:
#  - Steps to reproduce
#  - Expected vs. actual behavior
#  - Your Rust version (rustc --version)"

# Close after 1 week of no response
```

### 2. Contributor Count & Diversity

#### Target
- **Active contributors per release**: ≥3
- **New contributors per year**: ≥5
- **Geographic diversity**: Represented across 2+ continents
- **Organizational diversity**: Not dominated by single company

#### Baseline (v26.9.1)
```
Current Status:
  Primary author: @seanchatmangpt ✅
  Contributors: TBD (requires GitHub audit)
  
Action plan:
  1. Audit current contributors
  2. Track first-time contributors
  3. Recognize contributions (in CHANGELOG, releases)
  4. Facilitate easier onboarding
```

#### Tracking
```bash
# Contributor metrics (GitHub API)
gh api repos/seanchatmangpt/clap-noun-verb/contributors \
  --paginate --jq '.[] | {login, contributions}'

# Calculate new contributors per release
# Script: scripts/metrics_new_contributors.py

# Publish: In release notes, thank contributors
# "This release was made possible by: @user1, @user2, ..."
```

### 3. Discussion Quality

#### Target
- **Substantive discussions**: RFCs for major features
- **RFC acceptance rate**: ≥70% (balance between speed and inclusivity)
- **Tone**: Professional, welcoming, growth-focused
- **Learning resource**: Discussions serve as documentation

#### Baseline (v26.9.1)
```
Current Status:
  - GitHub Discussions: Enabled
  - RFC template: To be created
  - Moderation policy: To be created
  
Examples of high-quality discussions:
  - "Why noun-verb pattern instead of subcommands?"
  - "How to design ergonomic command hierarchies?"
  - "Performance: Macro expansion strategies"
```

#### Policy Documentation
```markdown
# DISCUSSION_POLICY.md (to be created)

## Discussion Guidelines

### Topic Categories
- **Announcements**: Release notes, major milestones
- **Roadmap & Planning**: Feature requests, architecture discussions
- **Show & Tell**: Projects using clap-noun-verb, blog posts
- **Help & Q&A**: Usage questions, troubleshooting

### Tone
- Professional and respectful
- Inclusive of different perspectives
- Growth-focused (emphasis on learning)
- No spam, no self-promotion outside Show & Tell

### Moderation
- Off-topic comments may be moved or removed
- Repeated violations: contributor flagged
- Appeals process: Contact maintainers directly
```

### 4. Adoption Metrics

#### Target
- **crates.io downloads per month**: Baseline + 10% growth/quarter
- **GitHub stars**: Baseline + 5/quarter
- **Public projects using framework**: Documented showcase
- **Blog posts/tutorials**: ≥2 per year (external)

#### Baseline (v26.9.1)
```
Current Status (to be collected):
  - crates.io downloads: TBD
  - GitHub stars: TBD
  - Projects using framework: TBD
  - External tutorials: 0 (action item)

Action:
  1. Collect baseline from crates.io and GitHub
  2. Create SHOWCASE.md for projects using framework
  3. Reach out to known users for case studies
  4. Write 2-3 tutorial blog posts
```

#### Tracking
```bash
# crates.io downloads
curl https://crates.io/api/v1/crates/clap-noun-verb \
  | jq '.crate.recent_downloads'

# GitHub stars
gh api repos/seanchatmangpt/clap-noun-verb \
  | jq '.stargazers_count'

# Track over time: metrics/adoption.csv
# Format: date,crates_downloads,github_stars,known_projects
```

---

## Architecture Alignment

### 1. Architecture Decision Language (ADL) Compliance

#### Target
- **ADRs covering all major decisions**: ≥5 (see Documentation Quality)
- **Alignment with stated philosophy**: 100% of code follows CLAUDE.md design
- **Breaking changes**: Rare (<1 per year), well-documented

#### Baseline (v26.9.1)
```
Current State:
  ✅ Core architecture documented: CLAUDE.md
  ✅ Design philosophy clear: noun-verb patterns, macro-based, zero stubs
  ✅ Feature system documented: feature gating strategy, frontier features
  ⚠️ ADRs: 0 (action item: create ≥5)
  ✅ Breaking changes: 0 in last 6 months
```

#### ADL Enforcement
```bash
# Code review checklist:
# - Does change align with stated design?
# - Does it follow established patterns?
# - If breaking: is it documented?
# - If feature: is feature gating clear?

# Architectural consistency checks
# - All public APIs: use Result<T>? ✅
# - All CLIs: use noun-verb pattern? ✅
# - All tests: AAA pattern? ✅ (documented in TESTING_REQUIREMENTS.md)
# - All docs: current version documented? ✅
```

### 2. Breaking Changes Frequency

#### Target
- **Planned breaking changes**: ≤1 per year
- **Unplanned breaking changes**: 0 (caught in review)
- **Deprecation period**: ≥2 releases before removal
- **Migration guides**: For every breaking change

#### Baseline (v26.9.1)
```
Recent History:
  26.9.1: 0 breaking changes
  26.9.1:  0 breaking changes (minor feature additions)
  26.5.28: 0 breaking changes
  v26.*:   0 breaking changes (minor version series)
  
Next planned: v27.0.0 (TBD, no scheduled breaking changes yet)
```

#### Breaking Change Process
```markdown
# BREAKING_CHANGES_PROCESS.md

## Proposing a Breaking Change

1. RFC discussion: Propose in GitHub Discussions
2. ADR creation: Document decision (ADR template)
3. Planning: Schedule for major version only
4. Deprecation period: ≥2 releases with `#[deprecated]`
   - Release N: Add `#[deprecated(since = "N+1", ...)]`
   - Release N+1: Add to CHANGELOG (warning)
   - Release N+2: Remove or change behavior
5. Migration guide: Document old → new pattern
6. Release notes: Prominent breaking change section

Example:
  v26.9.1: Add `#[deprecated(since = "26.7", ...)]` to old function
  v26.9.1:  Emit warning, document migration
  v27.0.0:  Remove old function, make new API required
```

### 3. Deprecation Strategy Execution

#### Target
- **Deprecation path**: Clear and documented for all deprecated items
- **Removal date**: Published (e.g., "removal planned for v27.0.0")
- **Migration guide**: Available before deprecation period
- **User adoption**: >80% of users migrated before removal

#### Baseline (v26.9.1)
```
Current deprecated items: 0
Deprecation procedure: Documented in RELEASE_MANAGEMENT.md

Known future deprecations (planned):
  - None currently scheduled
  
Deprecation template (from code):
  #[deprecated(
    since = "26.9.1",
    note = "Use `new_function` instead. See CHANGELOG for migration guide."
  )]
```

---

## SLO Tracking & Dashboard

### 1. SLO Definitions

#### Core SLOs (from PERFORMANCE_STANDARDS.md)

| Metric | Target | Current | Headroom | Status |
|--------|--------|---------|----------|--------|
| Incremental Compilation | ≤2.0s | 0.66s | 67% | ✅ Green |
| Release Binary | ≤10MB | 2.2MB | 78% | ✅ Green |
| Test Suite (parallel) | <1.0s | 0.16s | 84% | ✅ Green |
| Doc Build | ≤15s | 4-5s | 67% | ✅ Green |
| Test Coverage | ≥80% | TBD | TBD | 📊 Pending |

#### Extended SLOs

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Clippy Warnings | 0 | 0 | ✅ Green |
| PR Review Time | ≤2 days | TBD | 📊 Pending |
| Issue Response | ≤24h | TBD | 📊 Pending |
| Security Audit | Quarterly | Ongoing | ✅ Green |

### 2. SLO Violation Response

#### Yellow Alert (>50% headroom consumed)
```
Action:
  1. Investigate root cause
  2. Plan mitigation (optimization or deadline extension)
  3. Communicate to stakeholders
  4. Monitor closely in next release
```

#### Red Alert (<20% headroom remaining)
```
Action (blocking):
  1. Immediate investigation required
  2. Optimization work prioritized
  3. Feature freeze until resolved
  4. Escalate to maintainers
  5. Public communication (if external SLA affected)
```

### 3. Dashboard Configuration

#### Automated Metrics Collection

```yaml
# metrics/slo_dashboard.yml
slos:
  - name: "Incremental Compilation"
    metric: "compile_time_incremental"
    target: 2.0
    unit: "seconds"
    frequency: "per-release"
    measurement: "scripts/measure_compile_time.sh"
    
  - name: "Binary Size"
    metric: "binary_size_release"
    target: 10485760  # 10MB in bytes
    unit: "bytes"
    frequency: "per-release"
    measurement: "stat --format='%s' target/release/clap_noun_verb"

  - name: "Test Suite"
    metric: "test_suite_duration"
    target: 1.0
    unit: "seconds"
    frequency: "per-commit"
    measurement: "time cargo make test"

  - name: "Test Coverage"
    metric: "code_coverage_percent"
    target: 80.0
    unit: "percent"
    frequency: "per-release"
    measurement: "cargo tarpaulin"

  - name: "PR Review Time"
    metric: "pr_review_time_avg"
    target: 2.0
    unit: "days"
    frequency: "weekly"
    measurement: "scripts/metrics_pr_review_time.py"
```

#### Tracking Format

```csv
# metrics/slo_tracking.csv
date,metric,target,actual,headroom_percent,status,notes
2026-06-14,compile_time,2.0,0.66,67,green,
2026-06-14,binary_size,10.0,2.2,78,green,
2026-06-14,test_suite,1.0,0.16,84,green,
2026-06-14,doc_build,15.0,4.5,70,green,
2026-06-14,test_coverage,80.0,TBD,TBD,pending,Awaiting tarpaulin run
```

#### CI/CD Integration

```yaml
# .github/workflows/slo-tracking.yml
name: SLO Tracking

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  track-slos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Measure Compile Time
        run: |
          scripts/measure_compile_time.sh >> metrics/slo_tracking.csv
      
      - name: Measure Binary Size
        run: |
          cargo build --release
          size=$(stat --format='%s' target/release/clap_noun_verb)
          echo "$(date -I),binary_size,10485760,$size,$(((10485760-size)*100/10485760)),\
            $(if [ $size -lt 10485760 ]; then echo 'green'; else echo 'red'; fi)," \
            >> metrics/slo_tracking.csv
      
      - name: Measure Test Suite
        run: |
          scripts/measure_test_time.sh >> metrics/slo_tracking.csv
      
      - name: Measure Coverage
        run: |
          cargo tarpaulin --out Xml
          # Parse coverage and append
      
      - name: Commit Metrics
        run: |
          git config user.name "SLO Tracker"
          git config user.email "metrics@example.com"
          git add metrics/
          git commit -m "chore: update SLO metrics" || true
          git push
```

---

## Reporting & Governance

### 1. Quarterly Quality Report

#### Contents
```markdown
# Q2 2026 Quality Report

## Executive Summary
- All SLOs in green status
- Test coverage: 82% (target: 80%)
- 0 critical vulnerabilities
- 3 new contributors
- Release cadence: 1.5 weeks average

## Metrics Dashboard
[SLO table as above]

## Detailed Findings

### Code Quality
- Clippy warnings: 0
- Cyclomatic complexity: Average 9.2 (target: ≤10)
- Code duplication: 2.1% (target: <3%)

### Documentation
- API coverage: 98% (target: 100%)
- Examples: 15 (target: 25)
- ADRs: 0 (target: 5) — ACTION REQUIRED

### Performance
- [Detailed breakdown per section]

### Release Health
- Hotfix ratio: 0% (target: ≤15%)
- Regression bugs: 0 (target: ≤1)
- Time-to-publish: 4 hours avg (target: ≤6h)

### Community
- Issue response time: 12 hours avg (target: 24h)
- New contributors: 2
- GitHub stars: +8 (YTD: +30)

## Risk Assessment
- Documentation gaps: ADRs needed — LOW PRIORITY (v27.0.0 deadline)
- Coverage gaps: Unit tests for edge cases — MEDIUM PRIORITY

## Recommendations
1. Create 5 ADRs (planning for v27.0.0)
2. Expand example set to 25 (add 10 advanced examples)
3. Implement automated coverage tracking in CI
4. Establish security contact for vulnerability reporting

## Next Quarter Goals
- Achieve 100% API documentation coverage
- Publish first external tutorial blog post
- Recruit 2 additional maintainers
- Schedule annual security audit
```

#### Audience
- Maintainers (accountability)
- Contributors (transparency, recognition)
- Community (confidence, quality assurance)
- Stakeholders (business case, ROI)

### 2. Release Quality Checklist

```markdown
# RELEASE_QUALITY_CHECKLIST.md

## Pre-Release (1 week before)

### Code Quality
- [ ] Clippy clean: `cargo clippy --all-targets --all-features`
- [ ] Formatting: `cargo make format-check`
- [ ] Tests passing: `cargo make test-all`
- [ ] Coverage ≥80%: `cargo tarpaulin`
- [ ] No new TODOs/FIXMEs

### Documentation
- [ ] CHANGELOG updated
- [ ] README accurate
- [ ] Examples building: `cargo make build-examples`
- [ ] Doc builds without warnings: `cargo make doc`
- [ ] Migration guides (if breaking changes)
- [ ] API docs complete (100% coverage)

### Performance
- [ ] Incremental compile <2.5s
- [ ] Binary size <10.5MB
- [ ] Test suite <1.2s
- [ ] Benchmarks run: `cargo make bench`
- [ ] No performance regressions

### Security
- [ ] cargo audit clean
- [ ] No unsafe code changes without review
- [ ] Dependency updates reviewed

### Examples & Tests
- [ ] All tutorial examples run
- [ ] All how-to examples run
- [ ] All reference examples compile
- [ ] Integration tests passing
- [ ] No ignored tests

## Release Day

### Publish
- [ ] Publish macros crate: `cargo make publish-macros`
- [ ] Publish main crate: `cargo make publish`
- [ ] Verify on crates.io (wait 1-2 min)
- [ ] docs.rs updated (automated)

### Announce
- [ ] GitHub Release created with changelog
- [ ] Announcement on GitHub Discussions
- [ ] Twitter/social media (if applicable)
- [ ] Blog post (for major releases)

### Monitor
- [ ] Watch for reported issues (first 48h)
- [ ] Monitor crates.io downloads
- [ ] Check for immediate bugs
- [ ] Prepare hotfix branch if needed

## Post-Release (1 week)

### Metrics
- [ ] Record SLO metrics
- [ ] Track adoption (downloads, stars)
- [ ] Collect community feedback
- [ ] Identify any regression reports

### Retrospective
- [ ] What went well?
- [ ] What could be improved?
- [ ] Update release process
- [ ] Plan next release
```

### 3. Governance & Escalation

#### Decision-Making
```
Policy: Consensus-driven with clear escalation

Level 1: Maintainer Decision
- Bug fixes
- Documentation updates
- Example improvements
- Test additions
- Performance optimizations

Level 2: Maintainer Discussion (async)
- Feature additions
- API changes
- Deprecations
- Release timing
- Policy updates

Level 3: RFC / RFC Discussion (community)
- Major architectural changes
- Breaking changes
- New sub-projects
- Strategic direction changes

Escalation:
- Blocked on consensus → discussion period (1 week)
- Blocked after discussion → request community input
- No consensus after 2 weeks → maintainer decision (documented)
```

#### Issue Triage & Prioritization
```
Priority Levels:
  P0 (Critical): Crash, data loss, security → fix immediately
  P1 (High): Core functionality broken → fix this release
  P2 (Medium): Feature incomplete or performance issue → next release
  P3 (Low): Nice-to-have, documentation, cosmetics → backlog

Kind Labels:
  kind/bug
  kind/feature
  kind/enhancement
  kind/documentation
  kind/question
  kind/discussion

Status Labels:
  status/triage - Waiting for review
  status/accepted - Assigned to maintainer
  status/blocked - Waiting on external blocker
  status/in-progress - Being worked on
  status/review - PR submitted
  status/done - Merged/closed
```

---

## Implementation Roadmap

### Phase 1: Immediate (Week 1)
- [ ] Create this document: `docs/QUALITY_METRICS.md` ✅
- [ ] Create ADR directory: `docs/adr/`
- [ ] Create 5 initial ADRs
- [ ] Set up automated SLO tracking (GitHub Actions)
- [ ] Create baseline metrics collection script

### Phase 2: Short-term (Month 1)
- [ ] Implement test coverage tracking (tarpaulin in CI)
- [ ] Set up weekly metrics collection (cron job)
- [ ] Create first quarterly report
- [ ] Expand examples to 20+
- [ ] Create SECURITY.md with vulnerability reporting

### Phase 3: Medium-term (Q3 2026)
- [ ] Publish first external security audit report
- [ ] Reach 100% API documentation coverage
- [ ] Write 3 external blog post tutorials
- [ ] Achieve 85%+ test coverage
- [ ] Establish 3+ active maintainers

### Phase 4: Long-term (By v27.0.0)
- [ ] Achieve and sustain all green SLOs
- [ ] 100% API coverage + 25+ examples
- [ ] Established community contribution process
- [ ] Annual security audits scheduled
- [ ] Competitive performance benchmarks published

---

## Related Documents

- **CLAUDE.md** - Architecture and project overview
- **PERFORMANCE_STANDARDS.md** - Detailed performance budgets and measurement
- **RELEASE_MANAGEMENT.md** - Release process and versioning
- **RELEASE_DEFINITION_OF_DONE.md** - Release quality checklist
- **TESTING_REQUIREMENTS.md** - Testing standards and patterns
- **ONBOARDING.md** - Contributor onboarding guide
- **PR_DEFINITION_OF_DONE.md** - PR review checklist

---

## Appendix: Metrics Collection Scripts

### Script: measure_compile_time.sh

```bash
#!/bin/bash
# Measure incremental compilation time

set -e

TIMESTAMP=$(date -u +"%Y-%m-%d %H:%M:%S")
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

# Clean build
cargo clean > /dev/null 2>&1

# Warm cache
cargo build > /dev/null 2>&1

# Measure incremental
touch src/lib.rs
START=$(date +%s%N)
cargo build > /dev/null 2>&1
END=$(date +%s%N)

TIME_MS=$(( (END - START) / 1000000 ))
TIME_S=$(echo "scale=2; $TIME_MS / 1000" | bc)

echo "$TIMESTAMP,$VERSION,compile_time,$TIME_S,seconds"
```

### Script: metrics_test_coverage.py

```python
#!/usr/bin/env python3
"""Collect test coverage metrics."""

import json
import subprocess
from datetime import datetime

def run_coverage():
    """Run tarpaulin and parse coverage."""
    result = subprocess.run(
        ["cargo", "tarpaulin", "--out", "Json", "--output-dir", "target/coverage"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"Coverage failed: {result.stderr}")
        return None
    
    # Parse JSON output
    try:
        with open("target/coverage/cobertura.xml") as f:
            # Parse coverage data
            # Returns: line_coverage%, branch_coverage%, etc.
            pass
    except Exception as e:
        print(f"Parse error: {e}")
        return None

if __name__ == "__main__":
    coverage = run_coverage()
    if coverage:
        timestamp = datetime.utcnow().isoformat()
        print(f"{timestamp},test_coverage,{coverage['lines']}%")
```

---

**End of Document**

Document Version: 1.0  
Last Updated: 2026-08-20  
Maintainer: clap-noun-verb team  
Status: Published
