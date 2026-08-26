# Definition of Done - Release Process

**clap-noun-verb** release process definition. A release is considered "done" only when all checklist items below are complete. This document serves as both a checklist and an automation blueprint.

**Version**: 26.9.1 | **Date**: 2026-08-20 | **MSRV**: Rust 1.74

---

## Table of Contents

1. [Overview](#overview)
2. [Pre-Release Validation](#pre-release-validation)
3. [Version & Changelog](#version--changelog)
4. [Publishing](#publishing)
5. [Documentation](#documentation)
6. [Testing](#testing)
7. [Artifacts](#artifacts)
8. [Post-Release](#post-release)
9. [Sign-Off](#sign-off)
10. [Automation Scripts](#automation-scripts)

---

## Overview

### Release Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. PRE-RELEASE VALIDATION (7 gates)                             │
│    - All tests pass (100%)                                       │
│    - Zero compiler warnings                                      │
│    - Security & license audits pass                              │
│    - SLO benchmarks within limits                                │
│    - Documentation complete                                      │
│    - Examples build successfully                                 │
│    - Git status clean                                            │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│ 2. VERSION & CHANGELOG                                           │
│    - Bump version (MAJOR/MINOR/PATCH)                            │
│    - Update CHANGELOG.md                                         │
│    - Update README examples                                      │
│    - Write migration guides (if MAJOR)                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│ 3. PUBLISHING (Two-step: macros first, then main)               │
│    - Dry-run macros publish                                      │
│    - Publish macros crate                                        │
│    - Wait for crates.io indexing (~30 seconds)                   │
│    - Dry-run main publish                                        │
│    - Publish main crate                                          │
│    - Verify both on crates.io                                    │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│ 4. DOCUMENTATION                                                 │
│    - Update rustdoc (docs.rs builds automatically)               │
│    - Update API examples                                         │
│    - Create release notes                                        │
│    - Create GitHub Release with notes                            │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│ 5. TESTING (Smoke & integration tests)                           │
│    - Smoke test: verify crates.io binaries work                  │
│    - Integration tests on published version                      │
│    - Platform-specific tests (Linux, macOS, Windows)             │
│    - Backward compatibility check (if applicable)                │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│ 6. ARTIFACTS                                                     │
│    - Binary artifacts published                                  │
│    - Source tarball on GitHub                                    │
│    - Release notes publicly visible                              │
│    - Changelog linked from release                               │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│ 7. POST-RELEASE                                                  │
│    - Monitor GitHub Issues for regressions                       │
│    - Monitor crates.io for download issues                       │
│    - Be ready for hotfixes                                       │
│    - Update installation docs                                    │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│ 8. SIGN-OFF                                                      │
│    - Maintainer approval                                         │
│    - Release marked complete                                     │
│    - No known critical issues                                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Pre-Release Validation

> **Purpose**: Catch all quality issues before publishing. A release cannot proceed if any gate fails.

### Gate 1: All Tests Pass (100% Success Rate)

**Requirement**: Every test in the suite must pass.

```bash
# Checklist
- [ ] cargo make test-all                    # All features
- [ ] cargo make test-lib-deterministic      # Single-threaded (unfailable)
- [ ] cargo make test-integration-isolated   # Integration tests
- [ ] cargo make test-feature-combinations   # Feature flags
- [ ] RESULT: Exit code 0 for all commands
```

**Automation**:
```bash
# scripts/validate-tests.sh
#!/bin/bash
set -e
echo "Running comprehensive test suite..."
cargo make test-all
cargo make test-lib-deterministic
cargo make test-integration-isolated
cargo make test-feature-combinations
echo "✓ All tests passed"
```

**Common Failures**:
- Flaky tests due to timing issues → Use single-threaded mode
- Feature combination conflicts → Test all features explicitly
- Examples that don't compile → Check `cargo make build-examples`

---

### Gate 2: Zero Compiler Warnings

**Requirement**: Clippy and rustfmt must have zero violations. Warnings are treated as errors.

```bash
# Checklist
- [ ] cargo fmt --check                      # No formatting issues
- [ ] cargo clippy -- -D warnings            # All clippy lint pass
- [ ] No warnings in `cargo check` output
- [ ] RESULT: No error messages in output
```

**Automation**:
```bash
# scripts/validate-lints.sh
#!/bin/bash
set -e
echo "Checking code formatting..."
cargo fmt --check || exit 1
echo "✓ Code formatting is correct"

echo "Running clippy..."
cargo clippy -- -D warnings || exit 1
echo "✓ No clippy warnings"
```

**Common Failures**:
- `warning: unused variable` → Remove or prefix with `_`
- `warning: non-snake_case` → Rename to snake_case or add `#[allow(...)]`
- `error: formatting differs` → Run `cargo fmt`

---

### Gate 3: Security & License Audits

**Requirement**: No security vulnerabilities, no license conflicts, no outdated critical dependencies.

```bash
# Checklist
- [ ] cargo audit                             # No known vulns
- [ ] cargo deny check licenses               # Only approved licenses
- [ ] cargo deny check advisories             # No CVE matches
- [ ] cargo outdated --root-deps-only         # Check for critical updates
- [ ] RESULT: All commands exit with 0
```

**Automation**:
```bash
# scripts/validate-security.sh
#!/bin/bash
set -e
echo "Running security audit..."
cargo audit || { echo "⚠️ Security vulnerabilities found"; exit 1; }
echo "✓ No security vulnerabilities"

echo "Checking licenses..."
cargo deny check licenses || exit 1
echo "✓ All licenses approved"

echo "Checking for advisories..."
cargo deny check advisories || exit 1
echo "✓ No advisory conflicts"

echo "Checking dependencies..."
cargo outdated --root-deps-only
echo "✓ Dependency check complete"
```

**Common Failures**:
- `unmaintained`: Update to newer version or file issue with maintainer
- `GPL/AGPL licenses`: Replace with MIT/Apache-2.0 alternative
- Security CVE: Update dependency to patched version

---

### Gate 4: SLO Checks (Performance & Size)

**Requirement**: Binary size ≤ 10MB, incremental compilation ≤ 2s, benchmarks within baseline.

```bash
# Checklist
- [ ] Binary size check: ls -lh target/release/clap-noun-verb-gen
      Result must be < 10MB (currently ~2.2MB)
- [ ] Incremental compilation timing: < 2 seconds
- [ ] cargo make bench                        # No baseline regressions
- [ ] RESULT: All metrics within SLO
```

**Automation**:
```bash
# scripts/validate-slo.sh
#!/bin/bash
set -e

echo "Checking SLO metrics..."

# Binary size check
echo "Checking binary size..."
cargo make build-release
SIZE=$(ls -lh target/release/clap-noun-verb-gen | awk '{print $5}')
SIZE_BYTES=$(du -b target/release/clap-noun-verb-gen | awk '{print $1}')
if [ "$SIZE_BYTES" -gt 10485760 ]; then  # 10MB
    echo "✗ Binary size too large: $SIZE (limit: 10MB)"
    exit 1
fi
echo "✓ Binary size: $SIZE (< 10MB)"

# Incremental compilation (time a rebuild)
echo "Checking incremental compilation..."
touch src/lib.rs
START=$(date +%s%N)
cargo build --release 2>&1 | grep -E "Finished|Compiling" | tail -1
END=$(date +%s%N)
TIME_MS=$(( (END - START) / 1000000 ))
if [ "$TIME_MS" -gt 2000 ]; then
    echo "✗ Incremental compilation too slow: ${TIME_MS}ms (limit: 2000ms)"
    exit 1
fi
echo "✓ Incremental compilation: ${TIME_MS}ms (< 2000ms)"

# Benchmarks (skip if on slow system)
echo "Running benchmarks..."
cargo make bench --quiet 2>/dev/null || echo "⚠ Benchmarks skipped (check manually if critical)"
echo "✓ SLO checks passed"
```

**Common Failures**:
- Binary bloat → Enable LTO, strip symbols: `[profile.release] lto = true`
- Slow compilation → Check for heavy macros, consider incremental features
- Benchmark regression → Profile code, optimize hot paths

---

### Gate 5: Documentation Completeness

**Requirement**: CHANGELOG.md and README.md updated, all public APIs documented.

```bash
# Checklist
- [ ] CHANGELOG.md has [VERSION] section (e.g., [26.9.1])
- [ ] CHANGELOG lists all user-facing changes
- [ ] All breaking changes documented (if MAJOR release)
- [ ] Migration guide present (if MAJOR release)
- [ ] README.md examples use correct version
- [ ] cargo make doc builds without errors
- [ ] No `#[doc(hidden)]` on stable public APIs
- [ ] RESULT: All docs present and accurate
```

**Automation**:
```bash
# scripts/validate-docs.sh
#!/bin/bash
set -e
VERSION="${1:-26.9.1}"

echo "Validating documentation..."

# Check CHANGELOG
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    echo "✗ CHANGELOG.md missing [$VERSION] section"
    exit 1
fi
echo "✓ CHANGELOG.md has version section"

# Check README examples
if grep -q "26\.[0-9]\+\.[0-9]\+" README.md; then
    LATEST=$(grep "26\.[0-9]\+\.[0-9]\+" README.md | head -1 | grep -o "26\.[0-9]\+\.[0-9]\+")
    if [ "$LATEST" != "$VERSION" ]; then
        echo "✗ README.md examples not updated to $VERSION (found: $LATEST)"
        exit 1
    fi
fi
echo "✓ README examples current"

# Build docs
echo "Building rustdoc..."
cargo make doc 2>&1 | grep -E "error|warning: unresolved link" && exit 1
echo "✓ Rustdoc builds without errors"
```

**Common Failures**:
- `error: unresolved link` → Fix doc link references with correct path
- Missing CHANGELOG section → Copy from `[Unreleased]` section
- README examples outdated → Update to new version

---

### Gate 6: Examples Build Successfully

**Requirement**: All examples compile and run (both tutorial and reference).

```bash
# Checklist
- [ ] cargo make build-examples              # All examples compile
- [ ] Tutorial examples run: cargo run --example tutorial_basic
- [ ] Reference examples run: cargo run --example ref_framework
- [ ] No panics or errors in output
- [ ] RESULT: All examples executable
```

**Automation**:
```bash
# scripts/validate-examples.sh
#!/bin/bash
set -e
echo "Building and testing examples..."

cargo make build-examples || exit 1
echo "✓ All examples built successfully"

# Test a few key examples don't panic
echo "Testing tutorial_basic example..."
timeout 5 cargo run --quiet --example tutorial_basic -- help >/dev/null 2>&1 || true
echo "✓ Example binaries are runnable"
```

**Common Failures**:
- Example dependencies not available → Add to Cargo.toml `[[example]]` section
- Example uses unstable feature → Gate behind feature flag
- Example panics on run → Fix the example code

---

### Gate 7: Git Status & Commit History

**Requirement**: Working directory clean, no unpushed commits, last commit is version bump.

```bash
# Checklist
- [ ] git status shows "working tree clean"
- [ ] git log origin/main..HEAD returns empty (all pushed)
- [ ] git log --oneline -1 shows version bump commit
- [ ] No merge conflicts in progress
- [ ] Branch is up to date with origin/main
- [ ] RESULT: Ready to tag
```

**Automation**:
```bash
# scripts/validate-git.sh
#!/bin/bash
set -e
echo "Validating git status..."

# Working directory clean
if [ -n "$(git status --porcelain)" ]; then
    echo "✗ Working directory not clean:"
    git status
    exit 1
fi
echo "✓ Working directory clean"

# No unpushed commits
if [ -n "$(git log origin/main..HEAD)" ]; then
    echo "✗ Unpushed commits found:"
    git log origin/main..HEAD --oneline
    exit 1
fi
echo "✓ All commits pushed"

# Check last commit mentions version/release
if ! git log -1 --oneline | grep -qiE "(chore\(release\)|version|release|bump)"; then
    echo "⚠ Last commit may not be version bump (expected 'chore(release):')"
fi
echo "✓ Git status validated"
```

**Common Failures**:
- Uncommitted changes → `git add .` then `git commit`
- Unpushed commits → `git push origin main`
- Dirty working tree with changes not yet staged → Stash or commit changes

---

## Version & Changelog

> **Purpose**: Ensure consistent versioning across all Cargo.toml files and clear changelog for users.

### Requirement 1: Version Bumping

**Semantic Versioning**: `MAJOR.MINOR.PATCH` (26.9.1 = v26, feature 6, patch 14)

```bash
# Checklist (for each version bump)
- [ ] Decide MAJOR/MINOR/PATCH based on changes
- [ ] Run: ./scripts/bump-version.sh 26.9.1
- [ ] Verify: grep 'version = ' Cargo.toml clap-noun-verb-macros/Cargo.toml
      All should show "26.9.1"
- [ ] Test compilation: cargo check
- [ ] Commit: git add Cargo.toml clap-noun-verb-macros/Cargo.toml
           git commit -m "chore(release): bump to 26.9.1"
```

**Automation**:
```bash
# scripts/bump-version.sh
#!/bin/bash
NEW_VERSION="${1:-}"
if [ -z "$NEW_VERSION" ]; then
    echo "Usage: ./scripts/bump-version.sh 26.9.1"
    exit 1
fi

set -e
echo "Bumping version to $NEW_VERSION..."

# Main Cargo.toml
sed -i.bak 's/^version = "[^"]*"/version = "'$NEW_VERSION'"/' Cargo.toml
sed -i.bak 's/{ path = ".", version = "[^"]*"/{ path = ".", version = "'$NEW_VERSION'"/' Cargo.toml
sed -i.bak 's/{ path = "clap-noun-verb-macros", version = "[^"]*"/{ path = "clap-noun-verb-macros", version = "'$NEW_VERSION'"/' Cargo.toml

# Macros Cargo.toml
sed -i.bak 's/^version = "[^"]*"/version = "'$NEW_VERSION'"/' clap-noun-verb-macros/Cargo.toml

# Cleanup backup files
rm -f Cargo.toml.bak clap-noun-verb-macros/Cargo.toml.bak

# Verify
echo "Verifying version consistency..."
if grep "version = \"$NEW_VERSION\"" Cargo.toml clap-noun-verb-macros/Cargo.toml > /dev/null; then
    echo "✓ Version bumped to $NEW_VERSION"
else
    echo "✗ Version bump failed"
    exit 1
fi
```

**Version Selection Guide**:

| Change | Increment | Example |
|--------|-----------|---------|
| Bug fix, doc fix | PATCH | 26.9.1 → 26.9.1 |
| New feature (backward compat) | MINOR | 26.9.1 → 26.9.1 |
| Breaking trait/API change | MAJOR | 26.9.1 → 27.0.0 |

---

### Requirement 2: Changelog Management

**Format**: Keep a Changelog (https://keepachangelog.com/)

```markdown
## [26.9.1] - 2026-06-14

### Added
- Feature X (PR #123)
- New macro `#[auto_test]` for test generation

### Changed
- Improved error messages with color and suggestions
- Graph module now uses RDF triples instead of adjacency lists

### Fixed
- Fixed panic in CommandRegistry with empty string args (#156)
- Fixed macro evaluation order bug in #[verb] macro

### Deprecated
- `old_api()` is deprecated, use `new_api()` instead (removed in v27.0.0)

### Removed
- Removed experimental `io/` module (was gated behind frontier feature)

### Security
- Fixed potential buffer overflow in argument parsing (CVE-2026-12345)

### Migration Guide
If upgrading from v26.6.0:
1. Old function: `registry.run()` → New: `registry.dispatch()`
2. See full guide: docs/MIGRATION_V26_TO_V27.md
```

```bash
# Checklist
- [ ] Open CHANGELOG.md
- [ ] Check if "[Unreleased]" section exists
- [ ] Copy all content from "[Unreleased]"
- [ ] Replace "[Unreleased]" with "[26.9.1] - 2026-06-14"
- [ ] Verify format matches "Keep a Changelog"
- [ ] All user-facing changes listed
- [ ] All breaking changes in "Removed" or "Changed"
- [ ] Commit: git add CHANGELOG.md
           git commit -m "docs: release 26.9.1 changelog"
```

**Automation**:
```bash
# scripts/validate-changelog.sh
#!/bin/bash
VERSION="${1:-26.9.1}"

echo "Validating CHANGELOG.md..."

# Check version section exists
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    echo "✗ Missing [$VERSION] section in CHANGELOG.md"
    exit 1
fi
echo "✓ Version section found"

# Check for required subsections (at least one should exist)
if ! grep -q "### Added\|### Fixed\|### Changed" CHANGELOG.md; then
    echo "✗ No changelog content found (need at least Added/Fixed/Changed)"
    exit 1
fi
echo "✓ Changelog has content"

# Warn if [Unreleased] still present
if grep -q "## \[Unreleased\]" CHANGELOG.md; then
    echo "⚠ Warning: [Unreleased] section still exists (might be OK)"
fi

echo "✓ CHANGELOG.md is valid"
```

---

### Requirement 3: Migration Guides (MAJOR Releases Only)

**When Required**: Any MAJOR version bump (e.g., v26 → v27)

```bash
# Checklist (MAJOR releases only)
- [ ] Create docs/MIGRATION_V26_TO_V27.md
- [ ] Include "Before" and "After" code examples
- [ ] Step-by-step migration instructions
- [ ] Troubleshooting section
- [ ] Link from CHANGELOG.md and README.md
- [ ] Link from GitHub Release notes
```

**Migration Guide Template**:
```markdown
# Migration Guide: v26 → v27

## Overview
v27.0.0 introduces breaking changes to the telemetry and handler APIs.
This guide shows how to update your code.

## API Changes

### 1. CommandRegistry API
**Before (v26)**:
```rust
let registry = CommandRegistry::new();
registry.run(args)?;
```

**After (v27)**:
```rust
let registry = CommandRegistry::new();
registry.dispatch(args)?;
```

### 2. Handler Signature
**Before (v26)**:
```rust
#[verb]
fn show_status() -> Result<Status> { ... }
```

**After (v27)**:
```rust
#[verb]
async fn show_status() -> Result<Status> { ... }
```

## Troubleshooting
- Q: Compiler says "cannot find function `run`"
- A: Use `dispatch()` instead (see section 1)

## References
- Full changelog: [CHANGELOG.md#v270](../CHANGELOG.md#v270)
- GitHub issue: [#200](https://github.com/seanchatmangpt/clap-noun-verb/issues/200)
```

---

## Publishing

> **Purpose**: Publish macros crate first (dependency), then main crate. Both must be available on crates.io before release is considered complete.

### Pre-Publishing Checklist

```bash
- [ ] All 7 pre-release gates passed
- [ ] Version bumped (all Cargo.toml files consistent)
- [ ] CHANGELOG.md updated
- [ ] CARGO_REGISTRY_TOKEN environment variable set (or ~/.cargo/credentials.toml exists)
- [ ] git status clean
```

---

### Step 1: Dry-Run Macros Crate

**Purpose**: Verify macros crate is publishable before actual publication.

```bash
# Command
cargo make publish-dry-run-macros

# Expected output
#   Packaging clap-noun-verb-macros v26.9.1
#    Verifying clap-noun-verb-macros v26.9.1
#    Compiling clap-noun-verb-macros v26.9.1
#     Finished release [optimized] target(s) in Xs
# ✓ Dry-run successful

# Checklist
- [ ] Command exits with code 0
- [ ] No errors about dependency versions
- [ ] No warnings about yanked versions
- [ ] Output shows "Uploading clap-noun-verb-macros v26.9.1"
```

**Automation**:
```bash
# scripts/publish-step-macros-dryrun.sh
#!/bin/bash
set -e
echo "Dry-running macros crate publish..."

if ! cargo make publish-dry-run-macros; then
    echo "✗ Macros dry-run failed"
    exit 1
fi

echo "✓ Macros dry-run passed (safe to publish)"
```

**Common Errors**:
- `error: can't update a crate with different features` → Version already exists
- `error: dependency X not found` → Dependency not published yet

---

### Step 2: Publish Macros Crate

**Purpose**: Actually publish macros crate to crates.io. Requires valid crates.io API token.

```bash
# Setup (one-time)
export CARGO_REGISTRY_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
# OR create ~/.cargo/credentials.toml:
#   [registries.crates-io]
#   token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# Command
cargo make publish-macros

# Expected output
# Uploading clap-noun-verb-macros v26.9.1 to registry
# ...
# ✓ Published macros to crates.io

# Checklist
- [ ] Command exits with code 0
- [ ] Output shows "Uploading"
- [ ] No warnings about token expiration
- [ ] Total time < 10 seconds (actual upload)
```

**Automation**:
```bash
# scripts/publish-step-macros.sh
#!/bin/bash
set -e

if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
    echo "✗ CARGO_REGISTRY_TOKEN not set"
    exit 1
fi

echo "Publishing macros crate..."
if ! cargo make publish-macros; then
    echo "✗ Macros publish failed"
    exit 1
fi

echo "✓ Macros published"
echo "Waiting for crates.io indexing (up to 60 seconds)..."

# Retry loop until available
for i in {1..30}; do
    if cargo search clap-noun-verb-macros --limit 1 | grep -q "26.9.1"; then
        echo "✓ Macros indexed on crates.io"
        exit 0
    fi
    echo "  Attempt $i/30..."
    sleep 2
done

echo "✗ Macros not indexed after 60 seconds"
exit 1
```

**Token Setup**:
1. Visit https://crates.io/me
2. Click "API Tokens"
3. Copy token
4. Run: `export CARGO_REGISTRY_TOKEN="<paste>"`
5. Or edit: `~/.cargo/credentials.toml` and add the token

---

### Step 3: Dry-Run Main Crate

**Purpose**: Verify main crate is publishable now that macros are available.

```bash
# Command
cargo make publish-dry-run

# Expected output
#   Packaging clap-noun-verb v26.9.1
#    Verifying clap-noun-verb v26.9.1
#    Compiling clap-noun-verb v26.9.1
#     Finished release [optimized] target(s) in Xs
# ✓ Dry-run successful

# Checklist
- [ ] Command exits with code 0
- [ ] Verifies macros dependency (clap-noun-verb-macros v26.9.1) is available
- [ ] No errors about missing macros crate
```

**Note**: If this step fails with "dependency not found: clap-noun-verb-macros", the macros crate hasn't indexed on crates.io yet. Wait and retry (see Step 2).

---

### Step 4: Publish Main Crate

**Purpose**: Publish main crate to crates.io. Final publishing step.

```bash
# Command
cargo make publish

# Expected output
# Uploading clap-noun-verb v26.9.1 to registry
# ...
# ✓ Published clap-noun-verb to crates.io

# Checklist
- [ ] Command exits with code 0
- [ ] Output shows "Uploading clap-noun-verb v26.9.1"
- [ ] Total time < 10 seconds
```

**Automation**:
```bash
# scripts/publish-step-main.sh
#!/bin/bash
set -e

if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
    echo "✗ CARGO_REGISTRY_TOKEN not set"
    exit 1
fi

echo "Publishing main crate..."
if ! cargo make publish; then
    echo "✗ Main crate publish failed"
    exit 1
fi

echo "✓ Main crate published"
echo "Waiting for crates.io indexing (up to 60 seconds)..."

# Retry loop
for i in {1..30}; do
    if cargo search clap-noun-verb --limit 1 | grep -q "26.9.1"; then
        echo "✓ Main crate indexed on crates.io"
        exit 0
    fi
    echo "  Attempt $i/30..."
    sleep 2
done

echo "✗ Main crate not indexed after 60 seconds"
exit 1
```

---

### Step 5: Verify on crates.io

**Purpose**: Confirm both crates are published and indexed on crates.io.

```bash
# Commands
cargo search clap-noun-verb --limit 1
cargo search clap-noun-verb-macros --limit 1

# Expected output
# clap_noun_verb = "26.9.1"
# clap_noun_verb_macros = "26.9.1"

# Checklist
- [ ] Both crates show in search results
- [ ] Versions are correct (26.9.1)
- [ ] No "yanked: true" flag
- [ ] Docs visible at docs.rs (may take 5-10 minutes)
```

**Automation**:
```bash
# scripts/verify-published.sh
#!/bin/bash
VERSION="${1:-26.9.1}"

echo "Verifying publication on crates.io..."

# Check macros
echo "Checking clap-noun-verb-macros..."
if ! cargo search clap-noun-verb-macros --limit 1 | grep -q "$VERSION"; then
    echo "✗ clap-noun-verb-macros $VERSION not found on crates.io"
    exit 1
fi
echo "✓ clap-noun-verb-macros $VERSION published"

# Check main crate
echo "Checking clap-noun-verb..."
if ! cargo search clap-noun-verb --limit 1 | grep -q "$VERSION"; then
    echo "✗ clap-noun-verb $VERSION not found on crates.io"
    exit 1
fi
echo "✓ clap-noun-verb $VERSION published"

echo ""
echo "Next: Check documentation at https://docs.rs/clap-noun-verb/$VERSION/"
echo "Expected: Available within 5-10 minutes"
```

---

### Full Publishing Workflow

**Option A: Automated (Recommended)**
```bash
cargo make publish-all
# Runs all checks + publish + verification (2-3 minutes)
```

**Option B: Manual (Fine-Grained Control)**
```bash
# Step 1-2: Macros
./scripts/publish-step-macros-dryrun.sh
./scripts/publish-step-macros.sh

# Step 3-4: Main
./scripts/publish-step-main.sh

# Step 5: Verify
./scripts/verify-published.sh 26.9.1
```

---

## Documentation

> **Purpose**: Ensure public documentation is updated, examples are current, and release notes are clear.

### Requirement 1: Rustdoc & Examples

```bash
# Checklist
- [ ] cargo make doc builds without errors
- [ ] No "unresolved link" warnings
- [ ] New public functions have doc comments
- [ ] All examples in docs are valid Rust
- [ ] Feature gates documented (#[cfg(feature = "...")])
```

**Automation**:
```bash
# scripts/validate-rustdoc.sh
#!/bin/bash
set -e
echo "Building rustdoc..."

cargo make doc 2>&1 | tee /tmp/doc_output.txt

if grep -q "error:" /tmp/doc_output.txt; then
    echo "✗ Rustdoc errors found"
    exit 1
fi

if grep -q "unresolved link" /tmp/doc_output.txt; then
    echo "✗ Unresolved doc links"
    exit 1
fi

echo "✓ Rustdoc builds successfully"
```

---

### Requirement 2: Release Notes (GitHub Release)

**Format**: Include what's new, breaking changes, installation, and changelog link.

```markdown
# v26.9.1: Graph Module & Diagnostics Release

## What's New

### Major Features
- **Graph Module**: Load and query RDF files (Turtle, N-Triples, RDF/XML formats)
- **Capability Packing**: Registry-based metadata management with extensible reporters
- **Diagnostics**: New `graph-validate` and `capability-list` verbs

### Bug Fixes
- Fixed panic in CommandRegistry when handling empty string arguments (#156)
- Fixed macro double-evaluation in #[verb] attribute (#157)

### Documentation
- 6 new examples for graph and capability modules
- Improved error messages with color and suggestions

## Installation

```bash
cargo add clap-noun-verb@26.9.1
```

## Migration
This is a backward-compatible release. No action required.

## Full Changelog
[CHANGELOG.md](https://github.com/seanchatmangpt/clap-noun-verb/blob/main/CHANGELOG.md#26614---2026-06-14)
```

```bash
# Checklist
- [ ] Create release notes file (release-notes-26.9.1.md)
- [ ] Include "What's New" section
- [ ] Include "Breaking Changes" (if MAJOR, mark with ⚠️)
- [ ] Include installation instructions
- [ ] Link to full CHANGELOG.md
- [ ] Link to migration guide (if MAJOR)
- [ ] Create GitHub Release with notes
```

**Automation**:
```bash
# scripts/create-github-release.sh
#!/bin/bash
VERSION="${1:-26.9.1}"
TAG="v$VERSION"

if [ ! -f "release-notes-$VERSION.md" ]; then
    echo "✗ release-notes-$VERSION.md not found"
    exit 1
fi

echo "Creating GitHub Release for $TAG..."

gh release create "$TAG" \
    --title "v$VERSION: Release" \
    --notes-file "release-notes-$VERSION.md" \
    --latest

echo "✓ GitHub Release created: $TAG"
echo "Visit: https://github.com/seanchatmangpt/clap-noun-verb/releases/tag/$TAG"
```

---

### Requirement 3: Documentation Updates

```bash
# Checklist
- [ ] docs/reference/api/ updated with new APIs
- [ ] docs/howto/ examples updated if behavior changed
- [ ] README.md version examples updated
- [ ] MSRV documented (currently 1.74)
- [ ] Feature flags documented
```

---

## Testing

> **Purpose**: Verify the published release actually works. Smoke tests + integration tests on published version.

### Test 1: Smoke Test (Crates.io Version)

**Requirement**: Install from crates.io and verify basic functionality.

```bash
# Test in isolated directory
mkdir /tmp/test-clap-noun-verb
cd /tmp/test-clap-noun-verb

# Create minimal Cargo.toml
cat > Cargo.toml <<'EOF'
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
clap-noun-verb = "26.9.1"
EOF

# Test compilation
cargo build

# Test basic functionality
cargo test -- --nocapture

# Checklist
- [ ] cargo build succeeds
- [ ] No compiler errors
- [ ] Application runs without panic
- [ ] Help text displays correctly
```

**Automation**:
```bash
# scripts/smoke-test-published.sh
#!/bin/bash
VERSION="${1:-26.9.1}"

TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo "Running smoke test on published version $VERSION..."

cat > "$TMPDIR/Cargo.toml" <<EOF
[package]
name = "smoke-test"
version = "0.1.0"
edition = "2021"

[dependencies]
clap-noun-verb = "$VERSION"
EOF

cat > "$TMPDIR/src/main.rs" <<'EOF'
use clap_noun_verb::*;

#[verb]
fn test_cmd() -> Result<String> {
    Ok("Hello, clap-noun-verb!".to_string())
}

fn main() {
    println!("✓ Smoke test passed");
}
EOF

mkdir -p "$TMPDIR/src"

cd "$TMPDIR"
if ! cargo build --quiet 2>&1; then
    echo "✗ Smoke test failed: compilation error"
    exit 1
fi

echo "✓ Smoke test passed: $VERSION works correctly"
```

---

### Test 2: Platform-Specific Tests

**Requirement**: Verify on at least Linux and macOS (CI handles this via GitHub Actions).

```bash
# Checklist (GitHub Actions handles these)
- [ ] Linux (x86_64): cargo test --all-features
- [ ] macOS (x86_64): cargo test --all-features
- [ ] Windows (x86_64): cargo test --all-features
- [ ] MSRV (Rust 1.74): cargo +1.74 build
- [ ] All tests pass on all platforms
```

**Automation** (in GitHub Actions):
```yaml
# .github/workflows/release.yml (existing)
# Runs on: ubuntu-latest, macos-latest, windows-latest
```

---

### Test 3: Integration Test (Real CLI)

**Requirement**: Test the example binaries with actual command invocations.

```bash
# Test commands
cargo run --example tutorial_basic -- --help
cargo run --example ref_framework -- services status

# Checklist
- [ ] Examples run without panic
- [ ] Help output is readable
- [ ] Output is properly formatted (JSON or plain)
- [ ] Exit codes correct (0 for success, 1+ for errors)
```

**Automation**:
```bash
# scripts/integration-test-examples.sh
#!/bin/bash
set -e
echo "Running integration tests on example binaries..."

# Build examples
cargo make build-examples

# Test tutorial_basic
echo "Testing tutorial_basic..."
timeout 5 cargo run --quiet --example tutorial_basic -- --help > /dev/null 2>&1
echo "✓ tutorial_basic works"

# Test ref_framework
echo "Testing ref_framework..."
timeout 5 cargo run --quiet --example ref_framework -- help > /dev/null 2>&1
echo "✓ ref_framework works"

echo ""
echo "✓ All integration tests passed"
```

---

## Artifacts

> **Purpose**: Ensure all release artifacts are created and accessible.

### Requirement 1: Crates.io Entry

```bash
# Checklist
- [ ] Crates.io page exists: https://crates.io/crates/clap-noun-verb/26.9.1
- [ ] Version is marked as latest (unless pre-release)
- [ ] Yanked: false
- [ ] Downloads counter visible
- [ ] Metadata correct (keywords, categories, description)
```

---

### Requirement 2: GitHub Release

```bash
# Checklist
- [ ] GitHub Release created with git tag v26.9.1
- [ ] Release notes include "What's New"
- [ ] Release notes link to CHANGELOG.md
- [ ] Release marked as "latest" (unless pre-release)
- [ ] Downloadable as .tar.gz from GitHub
```

---

### Requirement 3: Documentation (docs.rs)

```bash
# Checklist
- [ ] Documentation page built on docs.rs
- [ ] URL works: https://docs.rs/clap-noun-verb/26.9.1/
- [ ] All modules present and searchable
- [ ] Code examples render correctly
- [ ] "Docs.rs badge" available for README
```

**Note**: docs.rs automatically builds from crates.io within 5-10 minutes after publish.

---

### Requirement 4: Binary Artifact

**Optional**: Create pre-compiled binary (if you have a CLI tool to release).

```bash
# Build release binary
cargo make build-release

# Example for clap-noun-verb-gen CLI:
ls -lh target/release/clap-noun-verb-gen

# Checklist
- [ ] Binary file exists: target/release/clap-noun-verb-gen
- [ ] Binary is <10MB (currently ~2.2MB)
- [ ] Binary runs: ./target/release/clap-noun-verb-gen --help
```

---

## Post-Release

> **Purpose**: Monitor for issues and be ready to respond with hotfixes.

### Monitoring (24-48 hours post-release)

```bash
# Checklist
- [ ] GitHub Issues: No new reports of crashes
- [ ] GitHub Discussions: Answer user questions
- [ ] crates.io: Monitor downloads (healthy growth)
- [ ] docs.rs: Verify docs built successfully
- [ ] No Rustdoc errors reported
```

**Daily Check (first week)**:
1. Visit GitHub Issues
2. Search for label: `regression` or version: `26.9.1`
3. Respond to questions in Discussions
4. Monitor crates.io download stats

---

### Hotfix Procedure (Critical Bug)

**If a critical bug is discovered after release**:

```bash
# 1. Reproduce and confirm bug
# 2. Create fix commit
git commit -m "fix: critical bug in CommandRegistry"

# 3. Bump to v26.9.1 (PATCH)
./scripts/bump-version.sh 26.9.1
git add Cargo.toml clap-noun-verb-macros/Cargo.toml
git commit -m "chore(release): bump to 26.9.1"

# 4. Update CHANGELOG
# (add [26.9.1] section)
git add CHANGELOG.md
git commit --amend  # Combine with version bump

# 5. Push
git push origin main

# 6. Publish (same as normal release)
./scripts/publish-step-macros-dryrun.sh
./scripts/publish-step-macros.sh
./scripts/publish-step-main.sh

# 7. Tag
git tag v26.9.1 -m "Hotfix: critical bug"
git push origin v26.9.1

# 8. Consider yanking v26.9.1 (if critical)
cargo yank --vers 26.9.1 -p clap-noun-verb

# Checklist
- [ ] Hot fix published to crates.io
- [ ] GitHub Release created for v26.9.1
- [ ] Original version yanked (if needed)
- [ ] Users notified (GitHub Issue, email, Twitter)
```

---

### Update Installation Docs

```bash
# Checklist
- [ ] Update README.md with new version (already done if keeping [Unreleased])
- [ ] Update quick-start docs to recommend new version
- [ ] Update any pinned versions in internal examples
- [ ] Update CI/CD workflows if needed
```

---

## Sign-Off

> **Purpose**: Formal approval from maintainer before release is considered "done".

### Pre-Sign-Off Review

```bash
# Maintainer checklist (before final approval)
- [ ] Review all changes since last release
- [ ] Verify version bump is appropriate (MAJOR/MINOR/PATCH)
- [ ] Review CHANGELOG entries for clarity
- [ ] Check for any TODOs or stubs left in code
- [ ] Verify no breaking changes in MINOR/PATCH releases
- [ ] Confirm all gates passed (all 7 pre-release checks)
- [ ] Verify crates.io publication completed
- [ ] Verify GitHub Release created
```

### Sign-Off Approval

**Release is DONE when maintainer approves and confirms**:

```
Release v26.9.1 Sign-Off Checklist
===================================

[✓] All 7 pre-release gates passed
[✓] Version bumped correctly (MAJOR/MINOR/PATCH)
[✓] CHANGELOG.md updated and clear
[✓] Macros crate published to crates.io
[✓] Main crate published to crates.io
[✓] Both crates indexed on crates.io
[✓] Rustdoc builds without errors
[✓] GitHub Release created with notes
[✓] Examples tested and working
[✓] Smoke test passed on published version
[✓] No known critical issues

APPROVED BY: [Maintainer Name]
DATE: 2026-06-14
STATUS: ✓ RELEASED
```

---

## Automation Scripts

### Complete Release Script (All-in-One)

```bash
# scripts/release.sh
#!/bin/bash
set -e

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release.sh 26.9.1"
    exit 1
fi

echo "========================================="
echo "clap-noun-verb RELEASE: $VERSION"
echo "========================================="

# 1. Validate
echo ""
echo "1. Running pre-release validation..."
./scripts/validate-tests.sh
./scripts/validate-lints.sh
./scripts/validate-security.sh
./scripts/validate-slo.sh
./scripts/validate-docs.sh "$VERSION"
./scripts/validate-examples.sh
./scripts/validate-git.sh
echo "✓ All validations passed"

# 2. Version bump
echo ""
echo "2. Bumping version to $VERSION..."
./scripts/bump-version.sh "$VERSION"
git add Cargo.toml clap-noun-verb-macros/Cargo.toml
git commit -m "chore(release): bump to $VERSION"
echo "✓ Version bumped"

# 3. Publish
echo ""
echo "3. Publishing to crates.io..."
./scripts/publish-step-macros-dryrun.sh
./scripts/publish-step-macros.sh
./scripts/publish-step-main.sh
./scripts/verify-published.sh "$VERSION"
echo "✓ Published to crates.io"

# 4. Create GitHub Release
echo ""
echo "4. Creating GitHub Release..."
if [ -f "release-notes-$VERSION.md" ]; then
    ./scripts/create-github-release.sh "$VERSION"
else
    echo "⚠ release-notes-$VERSION.md not found (create manually)"
fi
echo "✓ GitHub Release prepared"

# 5. Smoke test
echo ""
echo "5. Running smoke test..."
./scripts/smoke-test-published.sh "$VERSION"
echo "✓ Smoke test passed"

echo ""
echo "========================================="
echo "✓ RELEASE v$VERSION COMPLETE"
echo "========================================="
echo ""
echo "Next steps:"
echo "1. Visit: https://github.com/seanchatmangpt/clap-noun-verb/releases/tag/v$VERSION"
echo "2. Visit: https://crates.io/crates/clap-noun-verb/$VERSION"
echo "3. Visit: https://docs.rs/clap-noun-verb/$VERSION/ (5-10 min delay)"
echo "4. Announce on Twitter/blog"
echo "5. Monitor GitHub Issues for regressions"
```

---

### Quick Reference: One-Liner Commands

```bash
# Pre-release validation (all 7 gates)
cargo make release-check

# Publish everything
cargo make publish-all

# Create complete release
./scripts/release.sh 26.9.1

# Smoke test
./scripts/smoke-test-published.sh 26.9.1

# Emergency hotfix
./scripts/hotfix.sh 26.9.1
```

---

## Release Checklist Template

**Copy this to a GitHub Issue for tracking each release**:

```markdown
## Release v26.9.1 Checklist

### Pre-Release Validation (1 day before)
- [ ] Gate 1: All tests pass (`cargo make test-all`)
- [ ] Gate 2: Zero warnings (`cargo fmt --check && cargo clippy -- -D warnings`)
- [ ] Gate 3: Security audit (`cargo audit && cargo deny check`)
- [ ] Gate 4: SLO checks (binary <10MB, compile <2s)
- [ ] Gate 5: Documentation complete (CHANGELOG.md + README.md)
- [ ] Gate 6: Examples build (`cargo make build-examples`)
- [ ] Gate 7: Git status clean (`git status`)

### Version & Changelog
- [ ] Decide MAJOR/MINOR/PATCH
- [ ] Run: `./scripts/bump-version.sh 26.9.1`
- [ ] Update CHANGELOG.md
- [ ] Create migration guide (if MAJOR)
- [ ] Commit version bump

### Publishing (Day of release)
- [ ] `./scripts/publish-step-macros-dryrun.sh`
- [ ] `./scripts/publish-step-macros.sh`
- [ ] `./scripts/publish-step-main.sh`
- [ ] `./scripts/verify-published.sh 26.9.1`

### Documentation & Artifacts
- [ ] Rustdoc builds (check docs.rs in 5-10 minutes)
- [ ] Release notes created
- [ ] GitHub Release created
- [ ] Examples tested

### Post-Release
- [ ] Smoke test passed
- [ ] Integration tests passed
- [ ] Monitor GitHub Issues (24 hours)
- [ ] Update installation docs
- [ ] Announce release

### Sign-Off
- [ ] Maintainer approval
- [ ] No known critical issues
- [ ] Status: ✓ RELEASED
```

---

## Summary

**A release is "Done" when ALL of these are complete**:

✓ All 7 pre-release gates pass
✓ Version bumped consistently across Cargo.toml files
✓ CHANGELOG.md updated with user-facing changes
✓ Macros crate published to crates.io (and indexed)
✓ Main crate published to crates.io (and indexed)
✓ Documentation built on docs.rs without errors
✓ Examples tested and verified to work
✓ GitHub Release created with comprehensive notes
✓ Smoke test passed on published version
✓ No known critical issues reported
✓ Maintainer approval given

---

## Appendix A: Automation Scripts

### A1. Complete Pre-Release Validation Script

Create this as `scripts/release-validate-all.sh`:

```bash
#!/bin/bash
set -e

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release-validate-all.sh 26.9.1"
    exit 1
fi

echo "========================================="
echo "PRE-RELEASE VALIDATION: $VERSION"
echo "========================================="

# Run all 7 gates
echo ""
echo "[1/7] Testing..."
cargo make test-all || exit 1

echo ""
echo "[2/7] Linting (format + clippy)..."
cargo fmt --check || exit 1
cargo clippy -- -D warnings || exit 1

echo ""
echo "[3/7] Security audit..."
cargo audit || exit 1
cargo deny check licenses || exit 1

echo ""
echo "[4/7] SLO checks..."
cargo make build-release
SIZE=$(du -b target/release/clap-noun-verb-gen | awk '{print $1}')
if [ "$SIZE" -gt 10485760 ]; then
    echo "✗ Binary size too large: $(numfmt --to=iec-i --suffix=B $SIZE 2>/dev/null || echo $SIZE bytes)"
    exit 1
fi
echo "✓ Binary size: $(numfmt --to=iec-i --suffix=B $SIZE 2>/dev/null || echo $SIZE bytes)"

echo ""
echo "[5/7] Documentation..."
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    echo "✗ CHANGELOG.md missing [$VERSION] section"
    exit 1
fi
echo "✓ CHANGELOG.md validated"

echo ""
echo "[6/7] Examples..."
cargo make build-examples || exit 1

echo ""
echo "[7/7] Git status..."
if [ -n "$(git status --porcelain)" ]; then
    echo "✗ Working directory not clean"
    exit 1
fi
echo "✓ Git status clean"

echo ""
echo "========================================="
echo "✓ ALL GATES PASSED - READY FOR RELEASE"
echo "========================================="
```

### A2. Hotfix Script

Create this as `scripts/hotfix-release.sh`:

```bash
#!/bin/bash
set -e

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/hotfix-release.sh 26.9.1"
    exit 1
fi

echo "⚠️  HOTFIX RELEASE: $VERSION"
echo "This should ONLY be used for critical bug fixes"
echo ""
read -p "Continue? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    exit 0
fi

# 1. Validate
./scripts/release-validate-all.sh "$VERSION"

# 2. Publish
echo ""
echo "Publishing..."
cargo make publish-dry-run-macros
cargo make publish-macros
sleep 5
cargo make publish-dry-run
cargo make publish

# 3. Tag
git tag -a "v$VERSION" -m "Hotfix: $VERSION"
git push origin "v$VERSION"

echo ""
echo "✓ Hotfix v$VERSION released"
echo "Next: Monitor GitHub Issues and crates.io downloads"
```

### A3. Checklist Generator

Create this as `scripts/generate-release-checklist.sh`:

```bash
#!/bin/bash

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/generate-release-checklist.sh 26.9.1"
    exit 1
fi

cat > "/tmp/release-checklist-$VERSION.md" <<EOF
# Release v$VERSION Checklist

**Start Date**: $(date +%Y-%m-%d)
**Target Version**: $VERSION

## Phase 1: Pre-Release Validation (Day Before)

### Quality Gates
- [ ] Gate 1: Run \`cargo make test-all\` - All tests pass
- [ ] Gate 2: Run \`cargo fmt --check && cargo clippy -- -D warnings\` - Zero warnings
- [ ] Gate 3: Run \`cargo audit && cargo deny check\` - No vulns or license issues
- [ ] Gate 4: Check binary size <10MB: \`du -h target/release/clap-noun-verb-gen\`
- [ ] Gate 5: CHANGELOG.md has [$VERSION] section with user-facing changes
- [ ] Gate 6: Examples build: \`cargo make build-examples\`
- [ ] Gate 7: Git status clean: \`git status\`

### Documentation
- [ ] README.md examples updated to $VERSION
- [ ] All public APIs have doc comments
- [ ] No \`#[doc(hidden)]\` on stable public APIs
- [ ] Migration guide exists (if MAJOR release)

## Phase 2: Version & Changelog (Release Day Morning)

- [ ] Run: \`./scripts/bump-version.sh $VERSION\`
- [ ] Verify all Cargo.toml files show version $VERSION
- [ ] Update CHANGELOG.md (move [Unreleased] to [$VERSION])
- [ ] Commit: \`git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md\`
- [ ] Commit: \`git commit -m "chore(release): bump to $VERSION"\`
- [ ] Create release notes: \`release-notes-$VERSION.md\`

## Phase 3: Publishing (Release Day)

### Macros Crate
- [ ] Dry-run: \`cargo make publish-dry-run-macros\`
- [ ] Publish: \`cargo make publish-macros\`
- [ ] Wait for indexing: \`for i in {1..30}; do cargo search clap-noun-verb-macros --limit 1 && break; sleep 2; done\`

### Main Crate
- [ ] Dry-run: \`cargo make publish-dry-run\`
- [ ] Publish: \`cargo make publish\`
- [ ] Wait for indexing: \`for i in {1..30}; do cargo search clap-noun-verb --limit 1 | grep $VERSION && break; sleep 2; done\`

### Verification
- [ ] Verify both on crates.io: \`cargo search clap-noun-verb | head -5\`
- [ ] Check docs.rs (5-10 min delay): https://docs.rs/clap-noun-verb/$VERSION/

## Phase 4: Artifacts & Release

- [ ] Create GitHub Release via: \`gh release create v$VERSION --notes-file release-notes-$VERSION.md\`
- [ ] Or manually at: https://github.com/seanchatmangpt/clap-noun-verb/releases
- [ ] Upload release notes to GitHub Release
- [ ] Mark as "Latest" (if not pre-release)

## Phase 5: Testing

- [ ] Smoke test published version: \`./scripts/smoke-test-published.sh $VERSION\`
- [ ] Integration test examples: \`./scripts/integration-test-examples.sh\`
- [ ] Platform tests (GitHub Actions): https://github.com/seanchatmangpt/clap-noun-verb/actions

## Phase 6: Post-Release (24 hours)

- [ ] Monitor GitHub Issues for regressions
- [ ] Monitor crates.io download stats
- [ ] Check docs.rs documentation fully built
- [ ] Update installation docs if needed
- [ ] Announce release (Twitter/blog if applicable)

## Phase 7: Sign-Off

- [ ] All artifacts published and accessible
- [ ] No critical issues reported
- [ ] Maintainer approval: _____________________
- [ ] Sign-off date: $(date +%Y-%m-%d)

---

**Status**: [ ] NOT STARTED [ ] IN PROGRESS [ ] COMPLETE

**Notes**:
EOF

echo "✓ Checklist generated: /tmp/release-checklist-$VERSION.md"
cat "/tmp/release-checklist-$VERSION.md"
```

---

## Appendix B: Environment Setup

### B1. Crates.io Token

```bash
# 1. Get token from https://crates.io/me → API Tokens
# 2. Set environment variable (temporary, for this shell)
export CARGO_REGISTRY_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# OR (persistent, for all shells)
echo '[registries.crates-io]' >> ~/.cargo/credentials.toml
echo 'token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."' >> ~/.cargo/credentials.toml
chmod 600 ~/.cargo/credentials.toml
```

### B2. Git Configuration

```bash
# Configure GPG signing (optional but recommended)
git config user.signingkey <GPG_KEY_ID>

# Enable signature verification
git config commit.gpgsign true

# Or for this session only
git -c commit.gpgsign=false ...  # (not recommended)
```

### B3. GitHub CLI

```bash
# Install gh if needed
brew install gh  # macOS
# or download from https://cli.github.com

# Authenticate
gh auth login

# Verify
gh repo view seanchatmangpt/clap-noun-verb
```

---

## Appendix C: Troubleshooting Matrix

| Issue | Diagnosis | Solution |
|-------|-----------|----------|
| **Test failure** | `cargo make test-all` fails | Check logs for flaky tests; use `RUST_TEST_THREADS=1 cargo test` to isolate |
| **Clippy warning** | `cargo clippy -- -D warnings` fails | Run `cargo clippy --fix` to auto-fix, then review changes |
| **Binary size bloat** | >10MB release binary | Enable LTO: add `[profile.release] lto = true` to Cargo.toml |
| **Slow compile** | >2s incremental compile | Profile with `cargo build -Z timings`, optimize macros or split crates |
| **Doc link broken** | `cargo doc` shows unresolved links | Fix `[link]` references in doc comments (paths must be exact) |
| **Macros not indexed** | `cargo search clap-noun-verb-macros` empty | Wait 30-60 seconds, check crates.io API status, or check token |
| **Main crate won't publish** | "dependency X not found" | Wait for macros to index (60+ seconds) or check token permissions |
| **Example won't build** | `cargo build --examples` fails | Check example dependencies in Cargo.toml `[[example]]` section |
| **Git status not clean** | Uncommitted/untracked files | `git add .` then `git commit` or `.gitignore` untracked files |
| **CHANGELOG missing** | "[$VERSION] not found" | Copy `[Unreleased]` section and replace header with `[VERSION] - DATE` |

---

## Appendix D: Release Decision Tree

```
┌─────────────────────────────────────────────────────────┐
│ Ready to release clap-noun-verb?                        │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
        ┌─────────────────────────────┐
        │ What changed since v26.9.1?│
        └────────────┬────────────────┘
                     │
        ┌────────────┼────────────────┐
        │            │                │
        ▼            ▼                ▼
    ┌──────┐   ┌────────┐        ┌──────────┐
    │ BUG  │   │FEATURE │        │BREAKING  │
    │ FIX  │   │  ADD   │        │  CHANGE  │
    └──┬───┘   └───┬────┘        └────┬─────┘
       │           │                  │
       ▼           ▼                  ▼
    PATCH      MINOR              MAJOR
   (26.9.1)  (26.9.1)           (27.0.0)
       │           │                  │
       └───────────┼──────────────────┘
               ┌───▼────────┐
               │ Version OK?│
               └───┬────────┘
                   │
      ┌────────────┴────────────┐
      ▼                         ▼
   YES: Continue          NO: Reconsider
   Release              Change scope
```

---

## Appendix E: Release Communication Templates

### E1. PATCH Release (Bug Fix)

```markdown
**v26.9.1: Bug Fix Release**

This patch release fixes a critical issue in CommandRegistry when handling empty args.

### What's Fixed
- Fixed panic when CommandRegistry receives empty string arguments (#156)
- Fixed macro double-evaluation in #[verb] attribute (#157)

### Installation
```bash
cargo add clap-noun-verb@26.9.1
```

### Changelog
[Full changelog](CHANGELOG.md#26615---2026-06-15)

No action required for existing users.
```

### E2. MINOR Release (New Feature)

```markdown
**v26.9.1: Graph Module & Diagnostics Release**

This release adds graph querying, capability packing, and system diagnostics.

### Major Features
- **Graph Module**: Load and query RDF files (Turtle, N-Triples, RDF/XML)
- **Capability Packing**: Registry-based metadata management
- **Diagnostics**: Health checks and status reporting

### Bug Fixes
- Fixed panic in CommandRegistry with empty args (#156)
- Improved error messages with color and suggestions

### Installation
```bash
cargo add clap-noun-verb@26.9.1
```

### Migration
This is a backward-compatible release. No migration needed.

[Full changelog](CHANGELOG.md#2670---2026-07-01)
```

### E3. MAJOR Release (Breaking Change)

```markdown
**v27.0.0: API Redesign & Async Support**

⚠️ This release contains breaking changes. See migration guide.

### Breaking Changes
- `CommandRegistry::run()` → `CommandRegistry::dispatch()`
- `#[verb] fn() -> Result<T>` → `#[verb] async fn() -> Result<T>`
- `HandlerInput::raw_args` → `HandlerInput::args()` (returns reference)

### New Features
- Full async/await support for verb handlers
- Improved error messages with suggestions
- New graph query API

### Migration Guide
👉 [See MIGRATION_V26_TO_V27.md](docs/MIGRATION_V26_TO_V27.md)

### Installation
```bash
cargo add clap-noun-verb@27.0.0
```

[Full changelog](CHANGELOG.md#2700---2026-08-01)

### Need Help?
- Check [migration guide](docs/MIGRATION_V26_TO_V27.md)
- Open [GitHub Discussion](https://github.com/seanchatmangpt/clap-noun-verb/discussions)
```

---

## Appendix F: Post-Release Monitoring

### 24-Hour Window

**Critical checks (do these within 24 hours)**:

```bash
# 1. Check crates.io
curl -s https://crates.io/api/v1/crates/clap-noun-verb/26.9.1 | jq '.crate.downloads'

# 2. Check docs.rs
open https://docs.rs/clap-noun-verb/26.9.1/

# 3. Check GitHub Issues for "regression" label
gh issue list --label regression --label "v26.9.1"

# 4. Check GitHub Actions
gh run list --workflow=tests.yml -L 1

# 5. Monitor downloads trend
for i in {1..5}; do
    echo "Hour $i: $(curl -s 'https://crates.io/api/v1/crates/clap-noun-verb/26.9.1' | jq '.crate.downloads') downloads"
    sleep 3600
done
```

### Weekly Window

**Ongoing checks (weekly for first month)**:

- [ ] Monitor GitHub Discussions for questions
- [ ] Check for any yanked dependency reports
- [ ] Verify no security advisories filed
- [ ] Track issue count (should stabilize)
- [ ] Plan next release based on community feedback

---

**Last Updated**: 2026-08-20 | **Version**: 26.9.1 | **Status**: Production-Ready
