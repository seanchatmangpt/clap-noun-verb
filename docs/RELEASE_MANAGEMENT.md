# Release Management Guide for clap-noun-verb

A comprehensive guide to managing releases for the clap-noun-verb project, including version bumping, changelog management, publishing to crates.io, and GitHub release creation.

**Version**: 26.6.14 | **Current Status**: Production-Ready

## Table of Contents

1. [Overview](#overview)
2. [Semantic Versioning Strategy](#semantic-versioning-strategy)
3. [Pre-Release Checklist](#pre-release-checklist)
4. [Version Bumping Workflow](#version-bumping-workflow)
5. [Changelog Management](#changelog-management)
6. [Publishing Workflow](#publishing-workflow)
7. [Crates.io Metadata](#cratesio-metadata)
8. [GitHub Release Creation](#github-release-creation)
9. [Breaking Change Communication](#breaking-change-communication)
10. [Yanking Strategies](#yanking-strategies)
11. [Release Automation](#release-automation)
12. [Emergency Release Procedures](#emergency-release-procedures)

---

## Overview

The clap-noun-verb project follows a **dual-crate publishing model**:

- **clap-noun-verb-macros** (published first)
  - Proc-macro crate providing `#[noun]`, `#[verb]`, `#[arg]` and frontier feature macros
  - Located in `clap-noun-verb-macros/`
  - MSRV: Rust 1.70

- **clap-noun-verb** (published after macros)
  - Main library crate with core CLI framework
  - Located in project root `src/`
  - MSRV: Rust 1.74
  - Depends on macros crate

**Why this order matters**: The main crate depends on the macros crate, so macros must be published and indexed on crates.io before the main crate. crates.io takes ~2-60 seconds to index each crate.

### Current Infrastructure

**Quality Gates**: 7 automated gates in `scripts/pre-release-check.sh`
- Version consistency across Cargo.toml files
- Compilation check
- Test pass rate (100% required)
- Zero compiler warnings
- Documentation completeness (CHANGELOG + README)
- Example builds
- Git status validation

**CI/CD Pipeline**: `.github/workflows/release.yml`
- Triggered on: `git tag v*` (e.g., `v26.6.14`)
- Validates: Format, clippy, build, tests, docs
- MSRV: Tested against Rust 1.74
- Security: cargo-audit, cargo-deny
- Publishing: Automatic to crates.io with retry logic

---

## Semantic Versioning Strategy

The clap-noun-verb project follows **Semantic Versioning 2.0.0** with the format:

```
MAJOR.MINOR.PATCH
 26   .  6   .  14
```

### Version Increment Rules

#### MAJOR (breaking changes)
- Trait API redesigns requiring `impl` updates
- Removal of previously stable, documented features
- Incompatible changes to `#[verb]` or `#[noun]` macro behavior
- Changes to `HandlerInput`/`HandlerOutput` type signatures
- Output serialization format changes (JSON schema breaking)

**Recent MAJOR bumps:**
- v5.0.0 (Nov 2025): Telemetry API refactor + MCP integration
- v4.0.0 (Nov 2025): Autonomic CLI layer, production validation

#### MINOR (new features, backward compatible)
- New macro attributes: `#[arg(new_attr)]`
- New output formats or serialization options
- New modules/traits/functions (additive only)
- New frontier features with `frontier-*` flag
- MSRV bumps (rare, requires MAJOR in some ecosystems)

**Recent MINOR bumps:**
- v26.6.1 (June 2026): Graph module, capability packing, diagnostics
- v5.5.0 (Jan 2026): Agent CLI Builder, runtime generation
- v5.4.0 (Jan 2026): ggen integration, frontier features

#### PATCH (bug fixes, no new features)
- Trait implementations (new impl blocks)
- Documentation fixes
- Clippy warning resolution
- Error message improvements
- Test infrastructure enhancements

### Special Versioning Cases

**Frontier Features**:
- Frontier features are always gated behind `frontier-*` Cargo flags
- New frontier feature = MINOR bump (new additive feature)
- Frontier feature bug fix = PATCH bump

**Experimental/Unstable APIs**:
- Mark with `#[doc(hidden)]` or `#[unstable]` attributes
- Can change in MINOR releases without MAJOR bump
- Document stability status in rustdoc

**Deprecations**:
- Follow standard deprecation path (warn → remove in MINOR or MAJOR)
- Provide `#[deprecated(since = "X.Y.Z", note = "use X instead")]`
- Support deprecated APIs for at least 2-3 releases

### Example Version Progression

```
26.5.0  →  26.5.1 (doc fix)
        →  26.6.0 (new graph module - MINOR)
        →  26.6.1 (graph bug fix - PATCH)
        →  26.7.0 (new frontier features - MINOR)
        →  27.0.0 (breaking trait redesign - MAJOR)
```

---

## Pre-Release Checklist

### 1. Decide on Version Number
- Analyze changes since last release
- Determine if MAJOR, MINOR, or PATCH
- Verify against SemVer rules above
- Get team consensus if MAJOR

### 2. Run Automated Quality Gates

```bash
# Run all 7 pre-release gates (recommended approach)
./scripts/pre-release-check.sh 26.6.14

# Or run gates individually:
cargo make release-check      # Comprehensive check
cargo make verify-frontier    # Test frontier features
cargo make security-scan      # Security audit
cargo make slo-check          # Performance SLOs
```

**Must-Pass Gates**:
- ✓ Version consistency (main + macros + dependencies)
- ✓ Compilation check (zero errors)
- ✓ Test pass rate (100%, no failures)
- ✓ Zero compiler warnings (Clippy + rustfmt)
- ✓ Documentation (CHANGELOG + README updated)

### 3. Manual Quality Review

- [ ] Read all new commits since last release
- [ ] Verify no stubs/TODOs merged to main
- [ ] Check for unwrap/panic in production code (allowed: dev code, examples)
- [ ] Review CHANGELOG for clarity and accuracy
- [ ] Verify examples build and run correctly
- [ ] Check MSRV still accurate (Rust 1.74 for main, 1.70 for macros)

### 4. Git Status Validation

```bash
# Clean working directory required
git status                    # No uncommitted changes
git log origin/main..HEAD     # No unpushed commits

# Last commit should be CHANGELOG update
git log --oneline -1
# Example: "docs: release v26.6.14 - graph module, diagnostics"
```

### 5. Dependency Audit

```bash
cargo audit
cargo deny check licenses
cargo deny check advisories
cargo outdated --root-deps-only
```

---

## Version Bumping Workflow

### Step 1: Update Version in Cargo.toml

Versions must be synchronized across two crates:

**Main crate** (`Cargo.toml`):
```toml
[package]
name = "clap-noun-verb"
version = "26.6.14"  # ← Update here
edition = "2021"
...

[workspace.dependencies]
clap-noun-verb = { path = ".", version = "26.6.14" }  # ← And here
clap-noun-verb-macros = { path = "clap-noun-verb-macros", version = "26.6.14" }  # ← And here
```

**Macros crate** (`clap-noun-verb-macros/Cargo.toml`):
```toml
[package]
name = "clap-noun-verb-macros"
version = "26.6.14"  # ← Update here
...
```

### Step 2: Automated Bump with Script

Create a simple bump script:

```bash
#!/bin/bash
# scripts/bump-version.sh - Bump version across all files

NEW_VERSION="${1:-}"
if [ -z "$NEW_VERSION" ]; then
    echo "Usage: ./scripts/bump-version.sh 26.6.15"
    exit 1
fi

set -e

# Update main Cargo.toml
sed -i.bak "s/^version = \"[^\"]*\"/version = \"$NEW_VERSION\"/" Cargo.toml
sed -i.bak "s/{ path = \".\", version = \"[^\"]*\"/{ path = \".\", version = \"$NEW_VERSION\"/" Cargo.toml
sed -i.bak "s/{ path = \"clap-noun-verb-macros\", version = \"[^\"]*\"/{ path = \"clap-noun-verb-macros\", version = \"$NEW_VERSION\"/" Cargo.toml

# Update macros Cargo.toml
sed -i.bak "s/^version = \"[^\"]*\"/version = \"$NEW_VERSION\"/" clap-noun-verb-macros/Cargo.toml

# Update README if it has version examples
sed -i.bak "s/\"26\.[0-9]\+\.[0-9]\+\"/\"$NEW_VERSION\"/" README.md

# Verify
echo "✓ Updated to version $NEW_VERSION"
echo ""
echo "Verify with:"
echo "  grep 'version = \"' Cargo.toml clap-noun-verb-macros/Cargo.toml"
```

Usage:
```bash
./scripts/bump-version.sh 26.6.15
cargo check  # Verify compilation
git add Cargo.toml clap-noun-verb-macros/Cargo.toml README.md
git commit -m "chore(release): bump version to 26.6.15"
```

### Step 3: Verify Version Consistency

```bash
# Check all version strings match
grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml
# Output should be: version = "26.6.15" (all identical)

# Run pre-release check
./scripts/pre-release-check.sh 26.6.15
```

### Dry-Run Before Committing

```bash
cargo check
cargo make test
cargo make lint
```

---

## Changelog Management

### Format

The project uses **Keep a Changelog** format (https://keepachangelog.com/en/1.0.0/):

```markdown
## [VERSION] - YYYY-MM-DD

### Added
- Feature descriptions (user-facing changes)

### Changed
- Enhancement descriptions
- Behavior modifications (backward compatible)

### Deprecated
- APIs that will be removed in future releases
- Migration path required

### Removed
- Deleted features (breaking change, requires MAJOR)
- Removed APIs

### Fixed
- Bug fixes with observable impact
- Security fixes always noted as `Fixed`

### Security
- Security vulnerabilities (critical, always in CHANGELOG)

### Technical Details
- Internal implementation notes
- Dependency updates
- Build system changes
- NOT user-facing

### Migration Guide
- Step-by-step upgrade instructions for breaking changes
- Code examples showing old vs new patterns
```

### Current CHANGELOG Status

**Latest Release**: 26.6.13 (June 13, 2026)

Key sections in current release:
- **Added**: Graph module, capability packing, diagnostics (6 new verbs)
- **Changed**: Minimalist refactor, no default features
- **Fixed**: Eliminated stubs/cheats, 18 doctests refactored
- **Technical Details**: RDF loading, metadata storage, extensible reporters

### Adding Entries During Development

**DO NOT** add entries commit-by-commit. Instead, maintain an **Unreleased** section:

```markdown
## [Unreleased]

### Added
- Feature X (PR #123)
- Feature Y (PR #125)

### Fixed
- Bug Z (PR #126)

## [26.6.13] - 2026-06-13
...
```

Then, at release time, replace `[Unreleased]` with `[VERSION] - YYYY-MM-DD`.

### Writing Good Changelog Entries

**❌ Bad**:
- "Fixed bug"
- "Updated code"
- "Made improvements"

**✅ Good**:
- "Fixed panic in `CommandRegistry::run()` when handling empty string args"
- "Changed `#[verb]` macro to auto-detect noun from module filename (closes #156)"
- "Improved error messages for invalid argument combinations with color + suggestions"

### Tools for Changelog Generation

**Option 1: Manual (recommended for project control)**
```bash
# Edit CHANGELOG.md directly
# - Review git log since last release
# - Group changes by category
# - Write clear, user-facing descriptions
git log v26.6.13..HEAD --oneline | sort
```

**Option 2: Semi-automated with git-cliff**
```bash
# Install: cargo install git-cliff
git-cliff --tag v26.6.14 --output CHANGELOG.md
# Then review and clean up
```

**Option 3: GitHub Release Notes (as fallback)**
```bash
# Requires GitHub CLI: gh
gh release create v26.6.14 --generate-release-notes
```

### CHANGELOG Quality Checklist

- [ ] All breaking changes clearly marked (in MAJOR releases)
- [ ] All migration paths documented (in `### Migration Guide`)
- [ ] Examples include version constraints (`clap-noun-verb = "26.6.14"`)
- [ ] All new public APIs documented
- [ ] All deprecated APIs have `#[deprecated]` marker in code
- [ ] Security issues clearly labeled `### Security` section
- [ ] Technical details don't leak implementation (users don't care)
- [ ] Links to issues/PRs where relevant (GitHub friendly)

---

## Publishing Workflow

### Overview: Two-Step Publish

The dual-crate model requires publishing in order with retry logic:

1. **Dry-run both crates** (catch errors early)
2. **Publish macros crate** (wait for indexing)
3. **Dry-run main crate** (verify macros available)
4. **Publish main crate** (final step)
5. **Verify on crates.io** (manual check)

### Step 1: Dry-Run Macros Crate

```bash
cargo make publish-dry-run-macros
```

**What it does**:
- Checks `clap-noun-verb-macros/Cargo.toml` is valid
- Verifies all dependencies are published
- Confirms no yanked versions used
- **Does NOT publish** (dry-run only)

**Expected output**:
```
   Packaging clap-noun-verb-macros v26.6.14 (/path/to/clap-noun-verb-macros)
    Verifying clap-noun-verb-macros v26.6.14 (/path/to/clap-noun-verb-macros)
    Compiling clap-noun-verb-macros v26.6.14
     Finished release [optimized] target(s) in Xs
     Uploading clap-noun-verb-macros v26.6.14
✓ Dry-run successful
```

**Common errors**:
- `error: can't update a crate with a different set of features`
  → Means the crate was already published with different features
  → Publish a new version instead (increment PATCH)

- `error: dependency not found: linkme`
  → Dependency not published yet
  → Wait for dependency to be available, then retry

### Step 2: Publish Macros Crate

```bash
cargo make publish-macros
```

**What it does**:
- Actually publishes to crates.io
- Requires `CARGO_REGISTRY_TOKEN` env var set
- Blocks until upload completes (typically <10 seconds)

**Setup `CARGO_REGISTRY_TOKEN`**:
1. Get token from https://crates.io/me
2. Set environment variable:
   ```bash
   export CARGO_REGISTRY_TOKEN="eyJhbGciOiJIUzI1..."
   ```
3. Or create `~/.cargo/credentials.toml`:
   ```toml
   [registries.crates-io]
   token = "eyJhbGciOiJIUzI1..."
   ```

**Post-publish verification**:
```bash
# crates.io takes 2-60 seconds to index
cargo search clap-noun-verb-macros --limit 1

# Should show:
# clap_noun_verb_macros = "26.6.14"
```

**Wait for indexing** (automatic in GitHub Actions, manual otherwise):
```bash
# Retry loop until published
for i in {1..30}; do
    if cargo search clap-noun-verb-macros --limit 1 | grep -q "26.6.14"; then
        echo "✓ Macros published to crates.io"
        break
    fi
    echo "Waiting for indexing... ($i/30)"
    sleep 2
done
```

### Step 3: Dry-Run Main Crate

```bash
cargo make publish-dry-run
```

**What it does**:
- Verifies main crate dependencies
- Confirms macros crate dependency version available
- **Does NOT publish**

**Key difference from Step 1**:
- Also checks that `clap-noun-verb-macros = "26.6.14"` is published (from Step 2)

### Step 4: Publish Main Crate

```bash
cargo make publish
```

**What it does**:
- Publishes main crate to crates.io
- Includes automatic macros publish (via Makefile dependencies)
- Blocks until complete

**This command is idempotent** (safe to re-run if it fails):
```bash
# If publish fails, the crate is already partially registered
# Re-running will complete the upload
cargo make publish
```

### Step 5: Verify on crates.io

```bash
# Check both crates are published
cargo search clap-noun-verb --limit 1
cargo search clap-noun-verb-macros --limit 1

# Expected output:
# clap_noun_verb = "26.6.14"
# clap_noun_verb_macros = "26.6.14"

# Verify documentation built successfully
# Visit: https://docs.rs/clap-noun-verb/26.6.14/
# Visit: https://docs.rs/clap-noun-verb-macros/26.6.14/
```

### Automated Publishing: GitHub Actions

The `.github/workflows/release.yml` automates everything on tag push:

```bash
# Tag the release
git tag v26.6.14 -m "Release v26.6.14"

# Push to GitHub (triggers CI/CD)
git push origin v26.6.14
```

**GitHub Actions flow**:
1. Validate quality gates (format, clippy, tests, docs)
2. MSRV check (Rust 1.74)
3. Security check (cargo-audit)
4. License check (cargo-deny)
5. Publish macros crate
6. Wait for indexing (2-60 seconds, retries 30x)
7. Publish main crate
8. Create GitHub Release (with auto-generated notes)

**Monitor GitHub Actions**:
- GitHub: https://github.com/seanchatmangpt/clap-noun-verb/actions
- Click on the `Release` workflow run
- Check each job status
- View logs if any job fails

---

## Crates.io Metadata

### Cargo.toml Metadata (User-Facing)

Located in `Cargo.toml` `[package]` section:

```toml
[package]
name = "clap-noun-verb"
version = "26.6.14"
edition = "2021"
rust-version = "1.74"

# User-facing metadata
license = "MIT OR Apache-2.0"
authors = ["Sean Chatman <seanchatmangpt@gmail.com>"]
repository = "https://github.com/seanchatmangpt/clap-noun-verb"
documentation = "https://docs.rs/clap-noun-verb"
homepage = "https://github.com/seanchatmangpt/clap-noun-verb"

# Search keywords (max 5)
keywords = ["cli", "clap", "noun-verb", "command-line", "typer"]

# Categories (helps discoverability)
categories = ["command-line-utilities", "development-tools"]

# README and descriptions
readme = "README.md"
description = "Rust CLI framework with noun-verb patterns, graph operations, and capability packing"
```

### docs.rs Build Configuration

Located in `Cargo.toml` `[package.metadata.docs.rs]`:

```toml
[package.metadata.docs.rs]
rustdoc-args = ["--cfg", "docsrs"]
# Enables conditional docs for unstable features
# Reference in docstrings: #[cfg(docsrs)]
```

### Keeping Metadata Accurate

**Description (max 300 chars)**:
- Concise 1-2 sentence overview
- Update with major features
- Example: `"Rust CLI framework with noun-verb patterns, graph operations, and capability packing"`

**Keywords** (max 5, comma-separated):
- Add new keywords when major features added
- Example: added `"graph"` when graph module released

**Categories**:
- https://crates.io/categories
- Current: `command-line-utilities`, `development-tools`
- Add `asynchronous` if async features added

### README.md Discoverability

The README is the front page on crates.io:

- First 300 chars are shown as summary
- Should have:
  - Quick overview (1 paragraph)
  - Feature highlights (bullet list)
  - Quick start example
  - Link to documentation
  - MSRV policy
  - License

**Current README highlights**:
```markdown
# clap-noun-verb

Rust CLI framework with noun-verb command patterns (e.g., `myapp services status`).
Uses proc-macros for declarative command registration with linkme for compile-time auto-discovery.

## Features
- Zero-boilerplate noun-verb CLIs
- Macro-driven auto-discovery
- Graph operations
- Capability packing
- Diagnostics system

## Quick Start
```rust
#[verb]
fn show_status() -> Result<Status> { ... }
```

## Minimum Supported Rust Version
1.74 (same as Rust 2021 edition requirement)

## License
MIT OR Apache-2.0
```

---

## GitHub Release Creation

### Automatic Release (Recommended)

When you push a git tag matching `v*`, GitHub Actions automatically:

1. Publishes both crates to crates.io
2. Creates GitHub Release with auto-generated release notes
3. Sets release as "latest" (unless pre-release)

```bash
# Trigger automatic release creation
git tag v26.6.14 -m "Release v26.6.14"
git push origin v26.6.14
```

### Manual Release Creation

Use GitHub CLI for fine-grained control:

```bash
# Create release with custom notes
gh release create v26.6.14 \
  --title "v26.6.14: Graph Module Release" \
  --notes-file release-notes.md

# Create pre-release (for RCs)
gh release create v26.6.14-rc.1 \
  --prerelease \
  --title "v26.6.14-rc.1: Release Candidate" \
  --notes "Testing frontier features before final release"

# Upload artifacts (binaries, etc.)
gh release upload v26.6.14 ./target/release/clap-noun-verb-gen
```

### Release Notes Format

Structure release notes with:

```markdown
## What's Changed

### Major Features
- **Graph Module**: Load, query, and validate RDF files (Turtle, N-Triples, RDF/XML)
- **Capability Packing**: Registry-based capability management with metadata

### Bug Fixes
- Fixed panic in CommandRegistry when handling empty args (#456)
- Fixed macro E0434 error due to double evaluation (#457)

### Documentation
- Added 6 comprehensive examples for graph and capability modules
- Improved error messages with color and suggestions

### Breaking Changes
⚠️ **None** - This is a backward-compatible release.

## Installation

```bash
cargo add clap-noun-verb@26.6.14
```

## Full Changelog
See [CHANGELOG.md](https://github.com/seanchatmangpt/clap-noun-verb/blob/main/CHANGELOG.md#26614---2026-06-14) for full details.
```

### GitHub Release Checklist

- [ ] Title clearly indicates what's new (e.g., "v26.6.14: Graph Module")
- [ ] Notes mention breaking changes prominently (if MAJOR)
- [ ] Notes include migration guide for breaking changes
- [ ] Installation instructions included
- [ ] Link to full CHANGELOG
- [ ] Mark as pre-release if RC/alpha/beta version
- [ ] Tag is correct format: `v26.6.14` (with `v` prefix)
- [ ] Tag matches version in Cargo.toml

---

## Breaking Change Communication

Breaking changes require careful communication to minimize user pain:

### MAJOR Version Policy

When releasing a MAJOR version with breaking changes:

1. **Announce in Release Notes**
   ```markdown
   ## ⚠️ Breaking Changes in v27.0.0

   This is a MAJOR version release with breaking changes...
   ```

2. **Provide Migration Guide**
   ```markdown
   ## Migration Guide: v26 → v27

   ### 1. Telemetry API Changes
   
   **Before (v26):**
   ```rust
   telemetry::create_span("my-span", "my-trace-id");
   ```
   
   **After (v27):**
   ```rust
   TelemetryManager::instance().create_span("my-span", "my-trace-id");
   ```

   ### 2. HandlerInput Type Changes
   ```
   // See full migration guide in MIGRATION_V26_TO_V27.md
   ```

3. **Create Migration Guide Document**
   - Location: `docs/MIGRATION_V26_TO_V27.md`
   - Include: Before/after code examples, rationale, troubleshooting
   - Link from: README, CHANGELOG, release notes

4. **Deprecation Period (Recommended)**
   - Release MINOR versions with deprecation warnings first
   - Keep old API working for 2-3 releases
   - Then remove in MAJOR version
   - Example: `#[deprecated(since = "26.8", note = "use TelemetryManager instead")]`

### Handling Major Breaks

**Monolithic Breaks** (required for architectural changes):
- If multiple interconnected APIs break together (trait redesign)
- Release as single MAJOR version with comprehensive guide
- Example: v5.0.0 (November 2025) telemetry redesign

**Staggered Deprecations** (preferred for single API changes):
- Release deprecation warning in MINOR version
- Keep old API 2-3 releases
- Remove in next MAJOR version

### Communication Channels

1. **CHANGELOG.md**: Detailed technical changes
2. **GitHub Release Notes**: What users should care about
3. **Migration Guide**: Step-by-step upgrade instructions
4. **Twitter/Blog**: High-visibility breaking changes
5. **Discord/Community**: Answer user migration questions

---

## Yanking Strategies

"Yanking" removes a published version from crates.io's default results. Yanked versions:
- Still downloadable (useful for CI/CD pinning)
- Don't show in `cargo update` or `cargo add` recommendations
- Display warning: `yanked: true` on crates.io

### When to Yank

**CRITICAL Security Vulnerabilities**:
```bash
# If v26.6.13 has critical security hole
cargo yank --vers 26.6.13

# Users on v26.6.13 see warning + recommended upgrade path
```

**Catastrophic Regressions** (breaks core functionality):
```bash
# If v27.0.0 doesn't compile on some platforms
cargo yank --vers 27.0.0
# Users revert to v26.6.14 automatically
```

**Publish Errors** (wrong binary, missing files):
```bash
# If macros were published without proc-macro lib setting
cargo yank --vers 26.6.12
# Publish corrected version as v26.6.13
```

### How to Yank

```bash
# Install cargo-edit (if not already installed)
cargo install cargo-edit

# Yank a specific version
cargo yank --vers 26.6.13 -p clap-noun-verb

# Yank macros crate
cargo yank --vers 26.6.13 -p clap-noun-verb-macros

# Undo yank (if you yanked wrong version)
cargo yank --vers 26.6.13 -p clap-noun-verb --undo
```

### Yank Communication

When you yank, notify users:

```markdown
## Security Alert: v26.6.13 Yanked

**Affected**: clap-noun-verb v26.6.13 (released June 13, 2026)

**Issue**: [CVE-2026-12345](https://nvd.nist.gov/...) - Potential panic on untrusted input

**Action Required**:
- Upgrade immediately: `cargo update clap-noun-verb`
- This will install v26.6.14 (patched)
- If pinned to v26.6.13, update Cargo.toml: `clap-noun-verb = "26.6.14"`

**Status**: 
- ✓ v26.6.14 released with fix
- ✓ v26.6.13 yanked
- ✓ Patch committed to main branch
```

### Yank Checklist

Before yanking a version:

- [ ] Verify the issue exists (reproduce, test)
- [ ] Prepare fixed version (v26.6.14 if yanking v26.6.13)
- [ ] Get team approval (breaking user installations)
- [ ] Prepare user communication (blog post, email, etc.)
- [ ] Yank the version
- [ ] Publish communication immediately
- [ ] Publish fixed version if not already available
- [ ] Monitor GitHub Issues/Discussions for user questions

### Security Yank Example

```bash
#!/bin/bash
# scripts/yank-security-patch.sh

YANKED_VERSION="26.6.13"
FIXED_VERSION="26.6.14"

echo "Yanking $YANKED_VERSION for security reasons..."

# Verify fixed version is published
if ! cargo search clap-noun-verb --limit 1 | grep -q "$FIXED_VERSION"; then
    echo "ERROR: Fixed version $FIXED_VERSION not published yet"
    echo "Publish fixed version first: cargo make publish"
    exit 1
fi

# Yank macros
cargo yank --vers "$YANKED_VERSION" -p clap-noun-verb-macros
echo "✓ Yanked clap-noun-verb-macros $YANKED_VERSION"

# Yank main crate
cargo yank --vers "$YANKED_VERSION" -p clap-noun-verb
echo "✓ Yanked clap-noun-verb $YANKED_VERSION"

echo ""
echo "Post-yank checklist:"
echo "[ ] Publish security advisory"
echo "[ ] Post on GitHub Discussions"
echo "[ ] Email to mailing list (if applicable)"
echo "[ ] Tweet/blog post about fix"
```

---

## Release Automation

### Makefile Tasks

Key tasks in `Makefile.toml`:

| Task | Purpose | Flags |
|------|---------|-------|
| `publish-dry-run-macros` | Test macros publish | None |
| `publish-macros` | Publish macros crate | None |
| `publish-dry-run` | Test main publish | None |
| `publish` | Publish main crate | Depends on publish-macros |
| `publish-all` | Full workflow | Includes all checks |
| `release-check` | Pre-release validation | 7 gates (6 require PASS) |
| `verify-publish` | Confirm on crates.io | None |

### Full Release Command (Recommended)

```bash
# This runs all checks and publishes
cargo make publish-all
```

**What it does** (in order):
1. ✓ Format check
2. ✓ Clippy lint
3. ✓ Test all features
4. ✓ Build examples
5. ✓ Check all features
6. ✓ Build docs
7. ✓ Dry-run macros publish
8. ✓ Publish macros
9. ✓ Dry-run main publish
10. ✓ Publish main
11. ✓ Verify on crates.io

**Total time**: ~2-3 minutes

### Step-by-Step Manual Release

For more control, run steps individually:

```bash
# 1. Pre-release checks (7 gates)
./scripts/pre-release-check.sh 26.6.14

# 2. Dry-run macros
cargo make publish-dry-run-macros

# 3. Publish macros
cargo make publish-macros

# 4. Wait for indexing (check manually)
cargo search clap-noun-verb-macros --limit 1

# 5. Dry-run main
cargo make publish-dry-run

# 6. Publish main
cargo make publish

# 7. Verify both published
cargo search clap-noun-verb --limit 1
cargo search clap-noun-verb-macros --limit 1

# 8. Create GitHub tag (triggers CI/CD)
git tag v26.6.14
git push origin v26.6.14
```

### GitHub Actions Release Workflow

Triggered automatically on tag push:

```yaml
# .github/workflows/release.yml
on:
  push:
    tags:
      - 'v*'

jobs:
  validate:     # Format, clippy, tests, docs
  msrv-check:   # Rust 1.74 compatibility
  security-check: # cargo-audit
  license-check:  # cargo-deny
  publish:      # Publish both crates + GitHub Release
```

**To trigger:**
```bash
git tag v26.6.14 -m "Release v26.6.14"
git push origin v26.6.14
```

**Monitor:**
- GitHub Actions tab → Release workflow
- Check each job status
- View logs if failure

---

## Emergency Release Procedures

### Patch Release for Critical Bug

If a critical bug is discovered in production:

```bash
# 1. Backport fix to main branch
git checkout main
git pull origin main

# 2. Create fix commit
git commit -m "fix: critical bug in CommandRegistry (closes #999)"

# 3. Increment PATCH version
./scripts/bump-version.sh 26.6.15  # From 26.6.14

# 4. Update CHANGELOG
# Edit CHANGELOG.md, add [26.6.15] section

# 5. Run pre-release checks
./scripts/pre-release-check.sh 26.6.15

# 6. Commit version bump + CHANGELOG
git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md
git commit -m "chore(release): bump to 26.6.15"

# 7. Push and tag
git push origin main
git tag v26.6.15 -m "Emergency patch: fix critical bug"
git push origin v26.6.15

# 8. Monitor GitHub Actions publish
```

### Reverting a Released Version

If a published version has unfixable bugs:

```bash
# 1. Yank the bad version
cargo yank --vers 26.6.14 -p clap-noun-verb
cargo yank --vers 26.6.14 -p clap-noun-verb-macros

# 2. Publish fixed version
./scripts/bump-version.sh 26.6.15
# ... (same as patch release above)

# 3. Notify users (GitHub issue, release notes)
# 4. Update docs with migration guide if needed
```

### MSRV Incompatibility Discovered

If MSRV isn't actually supported after publishing:

```bash
# 1. Yank affected versions
cargo yank --vers 26.6.14 -p clap-noun-verb

# 2. Fix MSRV in code
# - Update Cargo.toml: rust-version = "1.75"  (if needed)
# - Fix syntax/feature usage for supported version

# 3. Test on actual MSRV
rustup install 1.74
rustup default 1.74
cargo build --all-features

# 4. Publish fix
./scripts/bump-version.sh 26.6.15
cargo make publish

# 5. Update CHANGELOG noting MSRV bump (if applicable)
```

---

## Release Checklist Template

Use this checklist for each release:

```bash
# Copy this to a GitHub Issue for tracking

## Release v26.6.14 - Checklist

### Pre-Release (1-2 days before)
- [ ] Review all commits since v26.6.13
- [ ] Determine MAJOR/MINOR/PATCH bump
- [ ] Update CHANGELOG.md with all changes
- [ ] Update README.md version examples
- [ ] Code review for any stubs/TODOs

### Version Bumping (day of release)
- [ ] Run ./scripts/bump-version.sh 26.6.14
- [ ] Verify Cargo.toml versions match
- [ ] Commit: "chore(release): bump to 26.6.14"
- [ ] Push to main branch

### Quality Gates
- [ ] ./scripts/pre-release-check.sh 26.6.14 (all gates pass)
- [ ] cargo make release-check (comprehensive validation)
- [ ] cargo make verify-frontier (frontier features)
- [ ] cargo make security-scan (audit + deny + outdated)

### Publishing
- [ ] cargo make publish-dry-run-macros
- [ ] cargo make publish-macros
- [ ] Wait ~30 seconds for crates.io indexing
- [ ] Verify: cargo search clap-noun-verb-macros
- [ ] cargo make publish-dry-run
- [ ] cargo make publish

### Post-Release
- [ ] Verify on crates.io (both crates)
- [ ] Visit docs.rs (v26.6.14 should be available)
- [ ] git tag v26.6.14
- [ ] git push origin v26.6.14 (trigger GitHub Actions)
- [ ] Monitor GitHub Actions publish job
- [ ] Create GitHub Release with notes

### Verification (next day)
- [ ] GitHub Release created successfully
- [ ] docs.rs documentation built
- [ ] No GitHub Issues about release problems
- [ ] Announce on Twitter/blog (if major feature)

### Post-Release Communication
- [ ] Add version to any installation docs
- [ ] Update examples to use new version
- [ ] Close any "ready for release" milestones
- [ ] Plan next release
```

---

## Summary: Quick Release Process

For a typical PATCH release:

```bash
# 1. Update version (3 minutes)
./scripts/bump-version.sh 26.6.15
git add Cargo.toml clap-noun-verb-macros/Cargo.toml README.md
git commit -m "chore(release): bump to 26.6.15"

# 2. Update CHANGELOG.md manually
vim CHANGELOG.md  # Add [26.6.15] section with changes
git add CHANGELOG.md
git commit --amend  # Amend to include CHANGELOG

# 3. Run all checks (3-5 minutes)
./scripts/pre-release-check.sh 26.6.15

# 4. Publish (2 minutes, mostly waiting)
git push origin main
cargo make publish  # Waits for macros indexing automatically

# 5. Tag and verify (1 minute)
git tag v26.6.15
git push origin v26.6.15
# GitHub Actions handles rest automatically

# Total: ~10 minutes hands-on, ~3 minutes automated

# Optional: Monitor GitHub Actions
open https://github.com/seanchatmangpt/clap-noun-verb/actions
```

For a MAJOR release with breaking changes:

```bash
# Same as above, plus:
# - Write comprehensive MIGRATION_V26_TO_V27.md
# - Add "### Breaking Changes" section to CHANGELOG
# - Link migration guide from release notes
# - Consider staggered deprecation in MINOR releases first
```

---

## References

- **Semantic Versioning**: https://semver.org/
- **Keep a Changelog**: https://keepachangelog.com/en/1.0.0/
- **Cargo Docs**: https://doc.rust-lang.org/cargo/
- **crates.io Badge**: https://crates.io/crates/clap-noun-verb
- **docs.rs**: https://docs.rs/clap-noun-verb/
- **GitHub Releases**: https://github.com/seanchatmangpt/clap-noun-verb/releases
- **Security Advisories**: https://rustsec.org/

---

**Document Version**: 26.6.14 | **Last Updated**: 2026-06-14
