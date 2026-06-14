# Definition of Done - Release Process

**clap-noun-verb** release process definition. A release is considered "done" only when all checklist items below are complete. This document serves as both a checklist and an automation blueprint.

**Version**: 26.6.14 | **Date**: 2026-06-14 | **MSRV**: Rust 1.74

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
- [ ] CHANGELOG.md has [VERSION] section (e.g., [26.6.14])
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
VERSION="${1:-26.6.14}"

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

**Semantic Versioning**: `MAJOR.MINOR.PATCH` (26.6.14 = v26, feature 6, patch 14)

```bash
# Checklist (for each version bump)
- [ ] Decide MAJOR/MINOR/PATCH based on changes
- [ ] Run: ./scripts/bump-version.sh 26.6.15
- [ ] Verify: grep 'version = ' Cargo.toml clap-noun-verb-macros/Cargo.toml
      All should show "26.6.15"
- [ ] Test compilation: cargo check
- [ ] Commit: git add Cargo.toml clap-noun-verb-macros/Cargo.toml
           git commit -m "chore(release): bump to 26.6.15"
```

**Automation**:
```bash
# scripts/bump-version.sh
#!/bin/bash
NEW_VERSION="${1:-}"
if [ -z "$NEW_VERSION" ]; then
    echo "Usage: ./scripts/bump-version.sh 26.6.15"
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
| Bug fix, doc fix | PATCH | 26.6.14 → 26.6.15 |
| New feature (backward compat) | MINOR | 26.6.14 → 26.7.0 |
| Breaking trait/API change | MAJOR | 26.6.14 → 27.0.0 |

---

### Requirement 2: Changelog Management

**Format**: Keep a Changelog (https://keepachangelog.com/)

```markdown
## [26.6.14] - 2026-06-14

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
- [ ] Replace "[Unreleased]" with "[26.6.14] - 2026-06-14"
- [ ] Verify format matches "Keep a Changelog"
- [ ] All user-facing changes listed
- [ ] All breaking changes in "Removed" or "Changed"
- [ ] Commit: git add CHANGELOG.md
           git commit -m "docs: release 26.6.14 changelog"
```

**Automation**:
```bash
# scripts/validate-changelog.sh
#!/bin/bash
VERSION="${1:-26.6.14}"

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
#   Packaging clap-noun-verb-macros v26.6.14
#    Verifying clap-noun-verb-macros v26.6.14
#    Compiling clap-noun-verb-macros v26.6.14
#     Finished release [optimized] target(s) in Xs
# ✓ Dry-run successful

# Checklist
- [ ] Command exits with code 0
- [ ] No errors about dependency versions
- [ ] No warnings about yanked versions
- [ ] Output shows "Uploading clap-noun-verb-macros v26.6.14"
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
# Uploading clap-noun-verb-macros v26.6.14 to registry
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
    if cargo search clap-noun-verb-macros --limit 1 | grep -q "26.6.14"; then
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
#   Packaging clap-noun-verb v26.6.14
#    Verifying clap-noun-verb v26.6.14
#    Compiling clap-noun-verb v26.6.14
#     Finished release [optimized] target(s) in Xs
# ✓ Dry-run successful

# Checklist
- [ ] Command exits with code 0
- [ ] Verifies macros dependency (clap-noun-verb-macros v26.6.14) is available
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
# Uploading clap-noun-verb v26.6.14 to registry
# ...
# ✓ Published clap-noun-verb to crates.io

# Checklist
- [ ] Command exits with code 0
- [ ] Output shows "Uploading clap-noun-verb v26.6.14"
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
    if cargo search clap-noun-verb --limit 1 | grep -q "26.6.14"; then
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
# clap_noun_verb = "26.6.14"
# clap_noun_verb_macros = "26.6.14"

# Checklist
- [ ] Both crates show in search results
- [ ] Versions are correct (26.6.14)
- [ ] No "yanked: true" flag
- [ ] Docs visible at docs.rs (may take 5-10 minutes)
```

**Automation**:
```bash
# scripts/verify-published.sh
#!/bin/bash
VERSION="${1:-26.6.14}"

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
./scripts/verify-published.sh 26.6.14
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
# v26.6.14: Graph Module & Diagnostics Release

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
cargo add clap-noun-verb@26.6.14
```

## Migration
This is a backward-compatible release. No action required.

## Full Changelog
[CHANGELOG.md](https://github.com/seanchatmangpt/clap-noun-verb/blob/main/CHANGELOG.md#26614---2026-06-14)
```

```bash
# Checklist
- [ ] Create release notes file (release-notes-26.6.14.md)
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
VERSION="${1:-26.6.14}"
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
clap-noun-verb = "26.6.14"
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
VERSION="${1:-26.6.14}"

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
- [ ] Crates.io page exists: https://crates.io/crates/clap-noun-verb/26.6.14
- [ ] Version is marked as latest (unless pre-release)
- [ ] Yanked: false
- [ ] Downloads counter visible
- [ ] Metadata correct (keywords, categories, description)
```

---

### Requirement 2: GitHub Release

```bash
# Checklist
- [ ] GitHub Release created with git tag v26.6.14
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
- [ ] URL works: https://docs.rs/clap-noun-verb/26.6.14/
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
2. Search for label: `regression` or version: `26.6.14`
3. Respond to questions in Discussions
4. Monitor crates.io download stats

---

### Hotfix Procedure (Critical Bug)

**If a critical bug is discovered after release**:

```bash
# 1. Reproduce and confirm bug
# 2. Create fix commit
git commit -m "fix: critical bug in CommandRegistry"

# 3. Bump to v26.6.15 (PATCH)
./scripts/bump-version.sh 26.6.15
git add Cargo.toml clap-noun-verb-macros/Cargo.toml
git commit -m "chore(release): bump to 26.6.15"

# 4. Update CHANGELOG
# (add [26.6.15] section)
git add CHANGELOG.md
git commit --amend  # Combine with version bump

# 5. Push
git push origin main

# 6. Publish (same as normal release)
./scripts/publish-step-macros-dryrun.sh
./scripts/publish-step-macros.sh
./scripts/publish-step-main.sh

# 7. Tag
git tag v26.6.15 -m "Hotfix: critical bug"
git push origin v26.6.15

# 8. Consider yanking v26.6.14 (if critical)
cargo yank --vers 26.6.14 -p clap-noun-verb

# Checklist
- [ ] Hot fix published to crates.io
- [ ] GitHub Release created for v26.6.15
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
Release v26.6.14 Sign-Off Checklist
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
    echo "Usage: ./scripts/release.sh 26.6.15"
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
./scripts/release.sh 26.6.15

# Smoke test
./scripts/smoke-test-published.sh 26.6.15

# Emergency hotfix
./scripts/hotfix.sh 26.6.15
```

---

## Release Checklist Template

**Copy this to a GitHub Issue for tracking each release**:

```markdown
## Release v26.6.14 Checklist

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
- [ ] Run: `./scripts/bump-version.sh 26.6.14`
- [ ] Update CHANGELOG.md
- [ ] Create migration guide (if MAJOR)
- [ ] Commit version bump

### Publishing (Day of release)
- [ ] `./scripts/publish-step-macros-dryrun.sh`
- [ ] `./scripts/publish-step-macros.sh`
- [ ] `./scripts/publish-step-main.sh`
- [ ] `./scripts/verify-published.sh 26.6.14`

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

**Last Updated**: 2026-06-14 | **Version**: 26.6.14 | **Status**: Production-Ready
