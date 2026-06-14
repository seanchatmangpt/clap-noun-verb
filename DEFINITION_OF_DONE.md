# Definition of Done (DoD) Checklist

**Project:** clap-noun-verb v26.6.14  
**Last Updated:** 2026-06-14  
**Audience:** Development team, reviewers, release managers

---

## Overview

This checklist defines the criteria a feature must satisfy before it is considered complete and ready for release. It covers requirements, code quality, testing, documentation, review, release readiness, and sign-off.

**Legend:**
- 🔴 **CRITICAL** — Must pass; blocking release
- 🟠 **HIGH** — Should pass; exceptions rare and documented
- 🟡 **MEDIUM** — Expected; can defer with justification
- 🟢 **LOW** — Nice-to-have; optional for this release

---

## 1. Requirements & Scope

- [ ] 🔴 **Feature is scoped and bounded** — Clear problem statement, acceptance criteria documented
- [ ] 🔴 **Acceptance criteria are testable** — Behavior is observable and measurable
- [ ] 🔴 **Dependencies are resolved** — No blocking upstream features; all prerequisites complete
- [ ] 🟠 **Architecture aligns with CLAUDE.md patterns** — Follows noun-verb dispatch, trait design, error handling rules
- [ ] 🟠 **No breaking changes without justification** — Backward compatibility maintained or migration guide written
- [ ] 🟡 **Feature branch follows naming convention** — `claude/*`, `feat/*`, `fix/*`, or `refactor/*` prefix
- [ ] 🟡 **Commit history is clean** — Logical, squashed, descriptive messages following project style

**Sign-off:** Feature owner confirms all requirements met

---

## 2. Code Quality

### Style & Formatting

- [ ] 🔴 **Code passes `cargo make format-check`** — rustfmt rules applied (100 char max, 4 spaces)
- [ ] 🔴 **Code passes `cargo make clippy`** — No clippy warnings; `-D warnings` enforced
- [ ] 🔴 **No `unwrap()`, `expect()`, `panic!()`** — All error paths use `Result<T>` with `?` operator
- [ ] 🔴 **No `todo!()`, `unimplemented!()`** — These raise to `deny` at CI time; use `bail!()` or return `Err()`
- [ ] 🔴 **No `println!()` in library code** — Log via `log::*` macros only (except `src/bin/`, tests, examples)
- [ ] 🟠 **Crate version bumped correctly** — Follows SemVer; CHANGELOG.md updated with this release
- [ ] 🟠 **Public API is `pub` only if necessary** — Modules/functions marked `pub` are intentionally exported
- [ ] 🟡 **Comments explain the why, not the what** — Code is self-documenting; comments justify design choices

### Type & Trait Design

- [ ] 🔴 **Traits are `dyn`-compatible** — No async methods; no inherent trait objects required
- [ ] 🔴 **Return types serialize correctly** — `#[derive(Serialize)]` on handler output types
- [ ] 🟠 **Error types implement `std::error::Error`** — `NounVerbError` wrapper or compatible error
- [ ] 🟠 **Trait bounds are minimal** — No unnecessary trait bounds; use `impl Trait` where possible
- [ ] 🟡 **Type names follow convention** — PascalCase; no abbreviations unless very standard

### Async Handling (if applicable)

- [ ] 🟠 **Async code behind `async` feature gate** — Feature-gated module in `src/async_verb.rs`
- [ ] 🟠 **Sync traits for dispatch** — `VerbCommand` stays sync; async wrapped in handlers
- [ ] 🟡 **Tokio runtime usage documented** — How async is driven (tokio full features standard)

---

## 3. Testing

### Unit Tests

- [ ] 🔴 **All public functions have unit tests** — Coverage ≥80% (measured via cargo-tarpaulin or similar)
- [ ] 🔴 **Tests follow AAA pattern** — Arrange, Act, Assert clearly separated
- [ ] 🔴 **Tests verify behavior, not implementation** — No `assert!(result.is_ok())` alone; check actual output
- [ ] 🔴 **Edge cases tested** — Empty inputs, None, zero, negative, max/min values, boundary conditions
- [ ] 🟠 **No flaky tests** — Tests pass consistently; no random timeouts or race conditions
- [ ] 🟠 **Test names are descriptive** — `test_verb_action_with_required_args_succeeds` not `test_1`
- [ ] 🟡 **Doctests compile and pass** — `cargo make doc` with `-D warnings` succeeds
- [ ] 🟡 **Tests run in parallel** — No hidden `SerialTest` unless genuinely necessary

### Integration Tests

- [ ] 🔴 **Command execution end-to-end tested** — Full CLI flow from args to output
- [ ] 🔴 **Feature combinations tested** — If feature-gated, test with/without feature enabled
- [ ] 🟠 **Error paths tested** — Invalid args, missing files, network errors all covered
- [ ] 🟠 **Output format tested** — JSON/YAML/Table/TSV all generate valid output
- [ ] 🟡 **Real example provided** — At least one working tutorial or how-to example

### Performance & SLOs

- [ ] 🔴 **Incremental compile time ≤2s** — New code doesn't regress compilation (baseline: 0.66s)
- [ ] 🔴 **Binary size ≤10MB** — New features don't bloat binary (baseline: 2.2MB)
- [ ] 🟠 **Benchmarks run without regression** — `cargo make bench-compare` shows no slowdown
- [ ] 🟠 **Memory usage is reasonable** — No unbounded allocations; linear scaling
- [ ] 🟡 **Concurrency safe** — If multi-threaded, tested with `loom` or equivalent

### Test Suite Completion

- [ ] 🔴 **`cargo make test` passes** — All unit tests pass, quick mode
- [ ] 🔴 **`cargo make test-lib-deterministic` passes** — Single-threaded, no flakiness
- [ ] 🟠 **`cargo make test-all` passes** — All features enabled, all tests run
- [ ] 🟠 **`cargo make test-frontier` passes** — If frontier features exist, all combinations work
- [ ] 🟡 **`cargo make ci` passes** — Full CI suite succeeds

---

## 4. Documentation

### Inline Code Comments

- [ ] 🔴 **All public modules have module-level docs** — `//! Module description` at top
- [ ] 🔴 **All public items have rustdoc** — `///` comments with at least one sentence
- [ ] 🟠 **Examples in rustdoc** — Complex functions include usage examples
- [ ] 🟠 **# Errors section** — Documented which `Error` variants are returned
- [ ] 🟡 **# Panics section** — Documented if function can panic (should be none per CLAUDE.md)

### Examples

- [ ] 🔴 **Tutorial example exists** — Step-by-step learning guide in `examples/tutorial/`
- [ ] 🔴 **How-to example exists** — Task-focused example in `examples/howto/`
- [ ] 🟠 **Reference example exists** — Complete API showcase in `examples/reference/`
- [ ] 🟠 **All examples compile without warnings** — `cargo make build-examples` succeeds
- [ ] 🟡 **README covers the feature** — Getting started section mentions new capability

### CHANGELOG & Versioning

- [ ] 🔴 **CHANGELOG.md updated** — Entry under `[Unreleased]` or version heading
- [ ] 🔴 **Version bumped in Cargo.toml** — SemVer: patch (fix), minor (feature), major (breaking)
- [ ] 🟠 **Migration guide written (if breaking)** — Explains old vs. new API with examples
- [ ] 🟠 **Deprecation notices issued** — Old APIs marked `#[deprecated]` with message
- [ ] 🟡 **Release notes prepared** — Summary for announcement (GitHub releases)

### API Documentation

- [ ] 🔴 **Public types documented** — Struct/enum fields explained
- [ ] 🟠 **Trait implementations documented** — Why this type implements these traits
- [ ] 🟠 **Feature gates documented** — Which APIs require which features
- [ ] 🟡 **Links to examples in docs** — Rustdoc links point to concrete code

---

## 5. Code Review

### Peer Review

- [ ] 🔴 **At least 1 peer review approval** — Reviewed by someone other than author
- [ ] 🔴 **All review comments addressed** — Changes made, discussed, or acknowledged
- [ ] 🟠 **No unresolved threads** — GitHub PR review threads marked resolved
- [ ] 🟠 **Code style accepted** — Reviewer approves formatting and naming
- [ ] 🟡 **Reviewer tests locally** — `cargo make test` run on reviewer's machine

### Architecture Review (for substantial features)

- [ ] 🟠 **Design aligns with core patterns** — Dispatch loop, trait registry, error handling
- [ ] 🟠 **No breaking changes to public API** — Or breaking change is intentional and versioned
- [ ] 🟠 **Performance-critical paths profiled** — Hot loops verified not to regress
- [ ] 🟡 **Complexity justified** — If code is complex, comment explains why

### Security Review (if handling user input, files, or network)

- [ ] 🔴 **Input validation on all user-supplied data** — No unsafe parsing, no injection vectors
- [ ] 🔴 **Error messages don't leak secrets** — Paths, tokens, credentials redacted
- [ ] 🟠 **File operations use safe APIs** — `std::fs` or `tempfile` crate, not raw I/O
- [ ] 🟠 **Network calls use secure defaults** — TLS verification enabled, timeouts set
- [ ] 🟠 **Dependencies audited** — `cargo make security-scan` passes (audit, deny, outdated)
- [ ] 🟡 **OWASP risks assessed** — SQL injection, path traversal, etc., mitigated if applicable

---

## 6. Release Readiness

### Compilation & Build

- [ ] 🔴 **`cargo make check` passes** — Full crate compiles
- [ ] 🔴 **`cargo make check-all` passes** — All feature combinations compile
- [ ] 🔴 **`cargo make build-release` succeeds** — No warnings in release mode
- [ ] 🟠 **No unsafe code** — Only `unsafe` allowed is in linkme proc-macro context
- [ ] 🟡 **LLVM IR optimized** — `--release` flags applied; link-time optimization considered

### Documentation Build

- [ ] 🔴 **`cargo make doc` succeeds** — No rustdoc warnings (`-D warnings` enforced)
- [ ] 🔴 **docs.rs build would pass** — Uses `#![cfg_attr(docsrs, ...)]` for nightly features
- [ ] 🟠 **All public items documented** — No missing `///` comments flagged by CI
- [ ] 🟡 **Documentation is rendered correctly** — Images, links, code examples display properly

### Dependency Health

- [ ] 🔴 **No denied licenses** — `cargo make security-scan` + `cargo deny check licenses` pass
- [ ] 🔴 **No known security advisories** — `cargo audit` passes; allowed denials documented
- [ ] 🟠 **Minimal dependency footprint** — No unnecessarily heavy transitive deps
- [ ] 🟠 **MSRV verified** — Code works with Rust 1.74+ (from Cargo.toml)
- [ ] 🟡 **Dependency versions up-to-date** — No outdated crates; updates don't break API

### Macro Crate Publishing (if applicable)

- [ ] 🔴 **Macros crate published first** — `clap-noun-verb-macros` published before main crate
- [ ] 🔴 **Main crate references published macros** — Cargo.toml points to crates.io version
- [ ] 🟠 **Macro tests pass** — `clap-noun-verb-macros/src/` validation tests succeed
- [ ] 🟡 **Macro documentation complete** — Proc-macro attribute behavior documented

### Backward Compatibility

- [ ] 🔴 **Existing code compiles** — No `cargo test` breakage in downstream projects
- [ ] 🔴 **Behavior is stable** — Output format, exit codes unchanged (except for features)
- [ ] 🟠 **Deprecation warnings only if intentional** — Compiler warnings are feature-intentional
- [ ] 🟡 **Upgrade path clear** — Migration guide provided for major version changes

---

## 7. Sign-Off & Approval

### Pre-Release Checklist

- [ ] 🔴 **Feature owner signs off** — Original author confirms feature complete and tested
- [ ] 🔴 **Tech lead reviews final state** — Architecture/design approved
- [ ] 🔴 **QA confirms test coverage** — Testing checklist verified
- [ ] 🟠 **Product owner approves feature** — Stakeholder confirms acceptance criteria met
- [ ] 🟡 **Release manager prepares package** — Version tags, build artifacts staged

### Release Gate Sign-Off

Before merging to `main` and publishing:

- [ ] 🔴 **All CI checks pass** — `cargo make ci` succeeds in CI environment
- [ ] 🔴 **All security checks pass** — `cargo make security-scan` clean
- [ ] 🔴 **Release validation complete** — `cargo make release-validate` passes
- [ ] 🟠 **Git history clean** — Merge commits only (no rebases per CLAUDE.md)
- [ ] 🟠 **No uncommitted changes** — Working tree is clean
- [ ] 🟡 **Tag is signed** — `git tag -s` with GPG key (if enforced)

### Post-Release

- [ ] 🔴 **Crate published to crates.io** — Visible in `cargo search clap-noun-verb`
- [ ] 🔴 **Documentation published to docs.rs** — API docs available online
- [ ] 🟠 **GitHub release created** — Tag annotated with CHANGELOG entry
- [ ] 🟠 **Announcement made** — Release notes shared on channels (if major)
- [ ] 🟡 **Downstreams notified** — Maintainers of known dependents informed of breaking changes

---

## 8. Checklist by Feature Type

Use these focused checklists for common feature patterns:

### New Verb Command

- [ ] Verb function decorated with `#[verb("name")]`
- [ ] Return type implements `Serialize`
- [ ] Argument types auto-documented with doc comments
- [ ] Integration test calls the verb and checks output
- [ ] Example in `examples/tutorial/` or `examples/howto/`
- [ ] Rustdoc includes usage example
- [ ] Error cases tested (invalid args, missing params)
- [ ] Output tested in all formats (JSON default, YAML/Table if implemented)

### New Module (Feature-Gated)

- [ ] Module marked `#[cfg(feature = "...")]` in lib.rs
- [ ] Module documentation explains feature purpose
- [ ] Tests gated with same `#[cfg(test)]` + feature
- [ ] `Cargo.toml` entry for feature with descriptive comment
- [ ] Example demonstrates feature when enabled
- [ ] Works with or without feature (no cascading panic)
- [ ] Clippy/format pass in all configurations

### Macro Enhancement

- [ ] Proc-macro in `clap-noun-verb-macros/src/`
- [ ] Expand output verified with `cargo expand`
- [ ] Compile-time validation tests in `validation.rs`
- [ ] Macro crate version bumped first
- [ ] Main crate updated to reference new version
- [ ] Both crates published in order

### Breaking Change

- [ ] SemVer major version bump
- [ ] CHANGELOG entry under `### Breaking Changes`
- [ ] Migration guide in CHANGELOG with before/after code
- [ ] Deprecation warnings issued one release prior (if possible)
- [ ] All examples updated to new API
- [ ] `#[deprecated]` attributes added to old API (if not removing)
- [ ] Release notes emphasize migration steps

---

## 9. Continuous Integration Metrics

These metrics are checked automatically at `cargo make ci` and `cargo make release-validate`:

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **Compilation (incremental)** | ≤2.0s | 0.66s | ✅ PASS |
| **Binary size (release)** | ≤10MB | 2.2MB | ✅ PASS |
| **Test suite completion** | <1s (parallel) | <500ms | ✅ PASS |
| **Code coverage** | ≥80% | TBD per branch | 🔍 VERIFY |
| **Clippy warnings** | 0 | 0 | ✅ PASS |
| **Formatting violations** | 0 | 0 | ✅ PASS |
| **Doc warnings** | 0 | 0 | ✅ PASS |
| **Security advisories** | 0 (unless approved) | 2 approved | ✅ PASS |
| **Feature compile test** | 23 combinations | All tested | ✅ PASS |

---

## 10. Common Failure Modes & Corrections

| Issue | Why It Fails DoD | How to Fix | Severity |
|-------|------------------|-----------|----------|
| `unwrap()` in handler | Violates error handling rule | Replace with `?` operator or `map_err()` | CRITICAL |
| Missing rustdoc on public item | Blocks docs.rs build | Add `///` comment with example | CRITICAL |
| Flaky test (intermittent fail) | Violates test reliability | Use `serial_test`, mock time, eliminate race | HIGH |
| Feature doesn't compile standalone | Breaking assumption of feature gates | Add `#[cfg(feature = "...")]` or make unconditional | HIGH |
| `println!()` in library code | Violates logging rule | Use `log::info!()`, `log::debug!()` | HIGH |
| No integration test | Behavior not verified end-to-end | Add test in `tests/` directory | HIGH |
| Breaking change, no CHANGELOG | Release notes incomplete | Add entry under `### Breaking Changes` | HIGH |
| Clippy warning allowed globally | Masks future real issues | Fix the code instead of allowing lint | MEDIUM |
| Example doesn't build | Documentation not verified | Run `cargo make build-examples` | MEDIUM |
| Panic in error handler | Violates "no panic" rule | Return `Result<T>` or use `log::error!()` then graceful shutdown | CRITICAL |

---

## 11. Sign-Off Template

When a feature is ready for release, fill out this sign-off:

```markdown
## Feature: [Feature Name]
**Author:** [Your Name]  
**Date:** [ISO 8601 date]  
**PR:** [Link to pull request]

### Requirements Met
- [x] All acceptance criteria satisfied
- [x] No blocking dependencies

### Code Quality
- [x] Formatting: `cargo make format-check` ✅
- [x] Linting: `cargo make clippy` ✅
- [x] No panics/unwraps in production code

### Testing
- [x] Unit tests: [X]% coverage, all pass
- [x] Integration tests: End-to-end flow verified
- [x] Edge cases tested: [list examples]

### Documentation
- [x] Rustdoc complete with examples
- [x] CHANGELOG entry added
- [x] Example in `examples/` added/updated
- [x] Version bumped: v26.6.X → v26.6.Y

### Review
- [x] Peer review: approved by [Reviewer Name]
- [x] Architecture review: [Approved/Noted concerns]
- [x] Security review: [N/A / Approved / See notes]

### Release Readiness
- [x] `cargo make release-check` passes
- [x] No breaking changes (or versioned appropriately)
- [x] Backward compatibility verified
- [x] Macros published first (if applicable)

### Sign-Off
- **Feature Owner:** [Signature] — I confirm this feature is complete and ready
- **Tech Lead:** [Signature] — I approve the architecture and implementation
- **Release Manager:** [Signature] — I'm ready to publish this version

---
```

---

## 12. Appendix: Useful Commands

All commands use `cargo make` per CLAUDE.md:

```bash
# Formatting & linting
cargo make format                # Auto-fix formatting
cargo make format-check          # Check without modifying
cargo make clippy                # Run clippy linter
cargo make lint                  # Run all linting (format-check + clippy)

# Testing
cargo make test                  # Quick test suite
cargo make test-lib-deterministic # Single-threaded, no flakiness
cargo make test-all              # All features enabled
cargo make test-frontier         # All frontier features
cargo make test-frontier-matrix  # 23 feature combinations

# Building & docs
cargo make build                 # Debug build
cargo make build-release         # Optimized build
cargo make build-examples        # All examples
cargo make doc                   # Build and check docs (-D warnings)
cargo make doc-open              # Build and open in browser

# Release workflow
cargo make release-check         # Full pre-release validation
cargo make release-validate      # Complete release suite
cargo make ci                    # CI pass/fail gate
cargo make security-scan         # Audit + deny + outdated

# Publishing (macro crate first!)
cargo make publish-dry-run-macros # Test publish macros
cargo make publish-macros         # Publish macros to crates.io
cargo make publish-dry-run        # Test publish main crate
cargo make publish                # Publish main crate

# Benchmarking & SLOs
cargo make bench                 # Run all benchmarks
cargo make bench-baseline        # Save current as baseline
cargo make bench-compare         # Compare to baseline
cargo make slo-check             # Verify SLO targets met
```

---

## 13. Questions & Escalation

**Q: Can I merge without 80% test coverage?**  
A: High-risk code (error paths, security) requires coverage. Low-risk (examples) can defer. Document the justification in PR.

**Q: What if a breaking change is unavoidable?**  
A: Bump major version, add `### Breaking Changes` section to CHANGELOG, write migration guide, announce prominently.

**Q: Can I use `unwrap()` in tests?**  
A: Yes, test code is allowed. But if the handler function has `unwrap()`, that's a CRITICAL failure.

**Q: What if my PR doesn't fit one feature type?**  
A: Use section 2-7 (Code Quality, Testing, Documentation, Review, Release Readiness, Sign-Off) as the baseline.

**Q: How do I request an exception?**  
A: Document the exception in the PR description with a link to this DoD. Get tech lead approval. Risk is on them.

---

**Document Version:** 1.0  
**Effective Date:** 2026-06-14  
**Next Review:** 2026-12-14  
**Owner:** Development Lead / Release Manager
