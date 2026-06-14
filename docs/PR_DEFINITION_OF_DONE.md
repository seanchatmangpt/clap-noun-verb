# Pull Request Definition of Done

A pull request for **clap-noun-verb** is considered "done" when all items in this checklist are satisfied. This ensures consistent quality, maintainability, and adherence to the project's architectural principles.

**Use this document as your PR review checklist before requesting reviews and as a reviewer's acceptance criteria.**

---

## 1. Code Changes ✅

Code must be correct, idiomatic, and follow the project's strict quality standards.

### Formatting & Style
- [ ] Code is formatted with `cargo make format` (max width 100, 4-space tabs)
- [ ] No formatting errors: `cargo make format-check` passes
- [ ] Line length does not exceed 100 characters
- [ ] Uses consistent indentation (4 spaces, no mixed tabs/spaces)

### Linting & Quality
- [ ] All clippy warnings resolved: `cargo make clippy` passes with 0 warnings
- [ ] Full lint suite passes: `cargo make lint` succeeds
- [ ] No `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` in production code
  - These are enforced by Clippy deny lints: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `exit`
  - Only allowed in tests and bin code (`src/bin/`, build.rs)
- [ ] Error handling uses `Result<T>` with `?` operator or `map_err()` consistently

### Code Architecture
- [ ] Follows CLAUDE.md guidelines and project conventions
- [ ] If modifying core modules: changes align with noun-verb pattern (`src/noun.rs`, `src/verb.rs`, `src/router.rs`, `src/registry.rs`)
- [ ] If modifying proc-macros: changes validated by `clap-noun-verb-macros/` crate
- [ ] No breaking changes to public APIs without ADL (Architecture Decision Log) approval
- [ ] Trait methods remain sync and object-safe (no `async` in trait methods)
- [ ] Use `&'static str` for trait method returns where applicable
- [ ] Library code uses `log::` macros, not `print!`/`println!` (except in `src/bin/`, `build.rs`)

### Compilation & Builds
- [ ] Project builds successfully: `cargo make build` passes
- [ ] All feature combinations compile:
  - [ ] Default features: `cargo make check`
  - [ ] All features: `cargo make check-all`
  - [ ] Frontier features: `cargo make check-frontier` (if applicable)
- [ ] No new dependencies that increase binary size beyond SLO (<=10MB)
- [ ] No new dependencies with copyleft licenses (denied by `deny.toml`: AGPL, GPL, LGPL)
- [ ] License compliance verified: `cargo deny check` passes

---

## 2. Testing ✅

Comprehensive tests ensure reliability, catch regressions, and enable confident refactoring.

### Test Coverage
- [ ] All new public functions have tests
- [ ] New public verbs have dedicated test functions
- [ ] Tests cover both happy path and edge cases (error conditions, boundary values)
- [ ] Entire test suite passes: `cargo make test` (parallelized)
- [ ] Tests pass deterministically: `cargo make test-lib-deterministic` (single-threaded)
- [ ] If adding features: `cargo make test-all` passes (all features enabled)
- [ ] If touching frontier code: `cargo make test-frontier` passes

### Test Quality
- [ ] Follow AAA (Arrange, Act, Assert) pattern
  ```rust
  #[test]
  fn test_verb_behavior_with_input() {
      // Arrange: Set up preconditions
      let input = prepare_input();
      
      // Act: Perform the action
      let result = function_under_test(input);
      
      // Assert: Verify behavior
      assert_eq!(result, expected_value);
  }
  ```
- [ ] Tests verify **behavior** (observable outputs/side effects), not implementation details
- [ ] No trivial assertions like `assert!(result.is_ok())` — verify actual return values
- [ ] No time-dependent assertions (sleep calls, time-based logic)
- [ ] Descriptive test names explain what is being tested: `test_verb_command_executes_with_required_args` not `test_it_works`
- [ ] Test functions are small and focused (test one behavior per test)

### Performance & Determinism
- [ ] Full test suite completes in <1 second with parallel execution
- [ ] No flaky tests (tests must pass 100% of the time with parallel and single-threaded execution)
- [ ] No uncontrolled randomness in tests (use `proptest` or `rand` with seed control if randomization is needed)
- [ ] For async tests: use `#[tokio::test]` attribute
- [ ] For determinism-sensitive tests: use `cargo make test-lib-deterministic` to validate

### Test Coverage Target
- [ ] Code coverage for new code is ≥80% (measured by `cargo tarpaulin` or similar)
- [ ] Changes to critical paths (router, registry, macro expansion) have >90% coverage
- [ ] Untested code paths are marked with `#[cfg(test)]` skip reasons or documented

---

## 3. Documentation ✅

Clear documentation enables users to adopt the code and maintainers to understand design decisions.

### Code Comments
- [ ] **Public API documented**: All public functions/types have doc comments
- [ ] Comments explain **WHY**, not WHAT:
  - Bad: `// increment counter`
  - Good: `// Use smaller threshold for high-frequency commands to reduce latency`
- [ ] Doc comments include `# Examples` section with executable code
- [ ] Doc comments include `# Errors` section if function returns `Result`
- [ ] Doc comments include `# Panics` section only if panics are possible (should be rare)
- [ ] No commented-out code blocks (delete or create an issue)

### Rustdoc
- [ ] `cargo make doc` builds successfully without warnings
- [ ] All public items are documented (use `#![deny(missing_docs)]` locally to verify)
- [ ] Doc examples are correct and idiomatic
- [ ] Cross-references between docs are accurate

### Changelog Entry
- [ ] New entry added to `CHANGELOG.md` (or relevant version section) describing user-facing changes
- [ ] Changelog entry is in one of these categories: `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`
- [ ] Changelog entry references the PR number or issue number: `Closes #123`

### Architecture Decisions
- [ ] If making architectural changes: update ADL (Architecture Decision Log) in `CLAUDE.md` or create a new ADL entry
- [ ] If changing verb/noun patterns: document rationale in code comments or create an ADL
- [ ] If modifying SLO targets: justify changes and update `CLAUDE.md`

---

## 4. Review & Approval ✅

Code quality is ensured through collaborative review and proper merge discipline.

### Review Process
- [ ] At least one maintainer approval received
- [ ] All review comments addressed (no unresolved conversations)
- [ ] Responses to feedback are constructive and complete
- [ ] If feedback is disagreed with: discussion resolved with consensus or maintainer decision
- [ ] Re-reviewed after making changes: original reviewer signs off on fixes

### GitHub Actions & CI
- [ ] All GitHub Actions workflows pass (green checkmarks on all required checks)
- [ ] Full CI suite passes: `cargo make ci` succeeds locally before merge
- [ ] No skipped tests in CI (test suite must run fully)
- [ ] Code coverage report (if enabled) shows ≥80% for new code

### Commit History
- [ ] Commits are logical and atomic (one feature/fix per commit)
- [ ] Commit messages follow convention: `<type>: <subject>` with optional body
  - Types: `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `perf`
  - Subject: lowercase, no period, <50 characters
  - Body (if present): explains WHY, not WHAT; wrapped at 72 characters
- [ ] Commits do NOT rebase main (only merge: `git merge main`)
- [ ] Commit history is clean (no "fixup" or "merge conflict" commits if avoidable)

### Merge Readiness
- [ ] Branch is up-to-date with main (consider `git merge main` if stale)
- [ ] All branch protection rules satisfied (approvals, CI checks, etc.)
- [ ] Ready to merge via GitHub UI (no manual merge commands needed)
- [ ] No force-pushes or destructive git operations in history

---

## 5. Performance ✅

Changes must not regress performance metrics or violate project SLOs.

### Build Performance SLO
- [ ] Incremental compilation time <=2 seconds (target: 0.66s, currently: 0.66s)
  - Test: `touch src/lib.rs && time cargo make build` 
  - If regression: profile with `cargo build -Z timings` and justify addition
- [ ] No new heavyweight dependencies added without justification
- [ ] Feature flags minimize compile-time impact on default builds

### Binary Size SLO
- [ ] Final binary size <=10MB (target achieved: 2.2MB)
  - Test: `cargo make build-release && ls -lh target/release/clap-noun-verb*`
  - If regression: profile with `cargo bloat --release -n 20` and optimize

### Runtime Performance
- [ ] Verb dispatch is fast (<1ms typical latency)
- [ ] CommandRegistry initialization does not block startup
- [ ] No new unbounded allocations in hot paths
- [ ] If changes affect router or registry: benchmark with `cargo make bench`

### Benchmark Changes
- [ ] If adding a new verb or modifying hot paths: add benchmark in `benches/dispatch.rs`
- [ ] Benchmarks show no regression vs. previous version
- [ ] Benchmarks run cleanly: `cargo make bench`

---

## 6. Compatibility ✅

Changes maintain backward compatibility and clear upgrade paths.

### Semantic Versioning
- [ ] Changes follow semantic versioning (major.minor.patch)
- [ ] **Breaking changes** require major version bump and ADL approval
- [ ] **New features** require minor version bump
- [ ] **Bug fixes** require patch version bump

### API Stability
- [ ] No breaking changes to public traits (`NounCommand`, `VerbCommand`)
- [ ] No breaking changes to public structs/enums without deprecation period
- [ ] No removal of public functions (use `#[deprecated]` if removing)
- [ ] Feature flags remain additive (no removing features)

### Deprecation
- [ ] If deprecating a public item: add `#[deprecated(since = "X.Y.Z", note = "...")]`
- [ ] Deprecation message explains migration path: "Use foo() instead"
- [ ] Deprecation period: minimum 2 minor releases before removal
- [ ] Changelog includes deprecation notices

### Feature Compatibility
- [ ] Changes work with all combinations of feature flags
- [ ] No feature gates that conditionally change public APIs
- [ ] Frontier features remain experimental (clearly documented)
- [ ] No hidden dependencies on future unstable features

---

## 7. Security ✅

Security controls prevent vulnerabilities in dependencies and code.

### Unsafe Code
- [ ] No new `unsafe` blocks in library code (only allowed in `linkme` macro expansion)
- [ ] Existing `unsafe` code is documented with `// SAFETY:` comments
- [ ] All invariants for `unsafe` code are clearly stated and maintained

### Panics & Error Handling
- [ ] No panics possible from invalid user input
- [ ] All `Result` types properly propagated or handled
- [ ] Error messages don't leak sensitive information (paths, internal state)
- [ ] No `.unwrap()` or `.expect()` on user-controlled values

### Dependency Security
- [ ] No new dependencies with known CVEs: `cargo audit` passes
- [ ] License check passed: `cargo deny check` passes
- [ ] Dependency versions pinned (or use workspace dependencies for consistency)
- [ ] If upgrading dependencies: `cargo deny check` and `cargo audit` still pass

### Input Validation
- [ ] Command-line arguments validated before use
- [ ] File paths sanitized (no traversal attacks)
- [ ] JSON/YAML parsing errors handled gracefully
- [ ] No assumption of untrusted input being well-formed

---

## 8. Merge Readiness ✅

Final checks before merging ensure the branch is production-ready.

### Pre-Merge Checklist
- [ ] Local full CI passes: `cargo make ci` succeeds with 0 errors
- [ ] No uncommitted changes (`git status` shows clean working tree)
- [ ] Branch name follows convention: `feat/*`, `fix/*`, `refactor/*`, `docs/*`, or `claude/*`
- [ ] No merge conflicts (or conflicts resolved cleanly)

### Merge Strategy
- [ ] Merging via GitHub's "Create a merge commit" (never squash/rebase)
- [ ] Merge commit message follows convention: auto-generated or manually written following format
- [ ] Branch deleted after merge (enable auto-delete in GitHub)

### Documentation Updates
- [ ] `CHANGELOG.md` updated with user-facing changes
- [ ] `CLAUDE.md` updated if processes/SLOs changed
- [ ] Examples updated if public API changed
- [ ] README.md updated if feature set changed significantly

### Version Bump (if applicable)
- [ ] Version updated in `Cargo.toml` (all workspace crates affected)
- [ ] Version updated in `clap-noun-verb-macros/Cargo.toml` if macros changed
- [ ] Version updated consistently across workspace

### Final Validation
- [ ] Squash any WIP commits or interactive rebase for clean history (if using feature branches)
- [ ] Verify one last time: `cargo make ci` passes
- [ ] Create PR with summary explaining changes
- [ ] Tag release if version bumped (after merge)

---

## GitHub Actions Integration Notes

This checklist is designed to integrate with GitHub Actions CI/CD pipelines. Recommended workflow:

### Automatic Checks (GitHub Actions)
```yaml
# .github/workflows/ci.yml (example)
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo make ci  # Runs all checks from Definition of Done
      
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo audit
      - run: cargo deny check
      
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v3
```

### Manual Review Checklist (Reviewer)
Maintainers should verify:
1. **Code Review**: Read the diff, verify architecture and design
2. **Automated Checks**: All GitHub Actions pass (shown in PR status)
3. **Definition of Done**: Review this checklist in PR description (recommend PR template includes it)
4. **Approval**: Leave approval comment: "Approved for merge" or request changes

### PR Template (Recommended)
```markdown
## Summary
[Describe the change and why it's needed]

## Changes
- [List key changes]

## Testing
- [Describe testing performed]

## Checklist
- [ ] Passes `cargo make ci`
- [ ] Tests added/updated with ≥80% coverage
- [ ] No clippy warnings: `cargo make clippy`
- [ ] Formatted: `cargo make format-check`
- [ ] Documentation updated
- [ ] Changelog entry added
- [ ] No breaking changes (or ADL-approved)

## Definition of Done
Refer to `docs/PR_DEFINITION_OF_DONE.md` for complete checklist.
```

---

## Quick Reference Checklists

### For Feature Branches
```bash
# Before pushing
cargo make lint          # Format, clippy, full lint
cargo make test          # Run tests (parallelized)
cargo make test-lib-deterministic  # Run tests (single-threaded)
cargo make test-all      # Run all feature combinations
cargo make ci            # Full CI suite (catch everything)

# Before requesting review
git log main...HEAD      # Verify commits are logical
git diff main...HEAD     # Review your changes
```

### For Macro/Core Changes
```bash
# Additional validation
cargo make test-frontier         # If frontier features touched
cargo make build-release && ls -lh target/release/  # Check binary size
cargo deny check          # Verify licenses
cargo audit               # Check for CVEs
```

### For Release Candidates
```bash
# Complete pre-release validation
cargo make ci             # Full CI
cargo make doc            # Verify docs build
cargo bloat --release -n 20  # Check for bloat
cargo test --all-features --quiet  # Full test suite
```

---

## Common Definition of Done Failures & Fixes

| Failure | Root Cause | Fix |
|---------|-----------|-----|
| Clippy warnings | Code style issues | `cargo make clippy` then `cargo make format` |
| Test flakiness | Parallel test race condition | Run `cargo make test-lib-deterministic` and fix timing |
| Documentation missing | Forgot doc comments | Add `///` comments to public items |
| Changelog missing | Forgot to update CHANGELOG.md | Add entry under appropriate section |
| Format check fails | Code not formatted | `cargo make format` |
| Lint fails | Code quality issue | `cargo make lint` shows specific issues |
| Panic found | Used `unwrap()` or `expect()` | Replace with `Result<T>` and `?` operator |
| Binary size regression | Added heavy dependency | Profile with `cargo bloat` and optimize |
| Test coverage <80% | New code untested | Add tests to reach 80% coverage |
| Pre-commit hook fails | Checks not run locally | Run `cargo make ci` before committing |

---

## Notes for Contributors

1. **Don't wait for review to run checks**: Run `cargo make ci` locally before opening a PR
2. **Prioritize quality over speed**: A thorough PR takes longer but merges faster
3. **Think about edge cases**: Good tests save debugging time later
4. **Document your thinking**: Comments explain WHY, code explains WHAT
5. **Ask for help**: If you're unsure about anything, ask in the PR or create an issue
6. **Respect the merge protocol**: Never rebase or force-push; only merge and commit forward
7. **SLOs matter**: Performance and compile time are part of quality
8. **Security first**: Unsafe code and panics are red flags; avoid them

---

## References

- **CLAUDE.md**: Project guidelines, architecture, critical rules, workflows
- **Architecture Decision Log (ADL)** in CLAUDE.md: Rationale for design decisions
- **Cargo.toml**: Feature flags, dependencies, lint configuration
- **CHANGELOG.md**: Version history and user-facing changes
- **src/**: Core library code following noun-verb patterns
- **tests/**: Test suite structure and examples

---

**Last updated**: June 2026  
**Version**: 1.0  
**Maintainers**: Sean Chatman and contributors
