# Release Management Skills & Automation Guide

**clap-noun-verb** - Semantic Versioning Release Framework

**Current Version**: 26.6.14 | **Last Updated**: 2026-06-14

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Semantic Versioning Strategy](#semantic-versioning-strategy)
3. [Release Workflow](#release-workflow)
4. [Version Bumping](#version-bumping)
5. [Changelog Management](#changelog-management)
6. [Publishing Process](#publishing-process)
7. [Crates.io Metadata](#cratesio-metadata)
8. [GitHub Release Management](#github-release-management)
9. [Breaking Changes Communication](#breaking-changes-communication)
10. [Yanking & Emergency Procedures](#yanking--emergency-procedures)
11. [Automation Scripts](#automation-scripts)
12. [Release Checklist](#release-checklist)

---

## Quick Start

For an experienced release operator:

```bash
# 1. Ensure clean state
git status
git log origin/main..HEAD  # Should be empty

# 2. Determine version (MAJOR, MINOR, or PATCH)
# Review changes and Semantic Versioning rules below

# 3. Bump version across all files
./scripts/bump-version.sh 26.6.15

# 4. Update CHANGELOG.md
vim CHANGELOG.md  # Move [Unreleased] → [26.6.15] - YYYY-MM-DD

# 5. Commit and verify
git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md
git commit -m "chore(release): bump to 26.6.15"
./scripts/pre-release-check.sh 26.6.15

# 6. Publish
cargo make publish

# 7. Tag and trigger CI/CD
git tag v26.6.15 -m "Release v26.6.15: <description>"
git push origin v26.6.15

# Total time: ~5 minutes hands-on, ~3 minutes automated
```

---

## Semantic Versioning Strategy

**Format**: `MAJOR.MINOR.PATCH` (e.g., `26.6.14`)

### Version Increment Decision Tree

```
Is this a breaking change?
├─ YES → MAJOR version bump
│   Examples:
│   - Trait API redesign
│   - Removal of stable features
│   - Output format changes (JSON schema breaking)
│   - Type signature changes in public APIs
│
├─ NO, is this a new feature?
│   ├─ YES → MINOR version bump
│   │   Examples:
│   │   - New macro attributes
│   │   - New output formats
│   │   - New modules/functions
│   │   - New frontier features (gated behind flags)
│   │
│   └─ NO → PATCH version bump
│       Examples:
│       - Bug fixes
│       - Documentation improvements
│       - New trait implementations
│       - Error message improvements
```

### Real-World Examples

| Change | Type | Version | Rationale |
|--------|------|---------|-----------|
| Add `#[arg(new_attr)]` | MINOR | 26.7.0 | New feature, backward compatible |
| Fix panic in `registry::run()` | PATCH | 26.6.15 | Bug fix, no new features |
| Redesign `HandlerInput` trait | MAJOR | 27.0.0 | Breaking change, impl updates required |
| New `diagnostic` module | MINOR | 26.7.0 | Additive feature |
| Update docs/examples | PATCH | 26.6.15 | Documentation only |
| Add frontier feature `meta-framework` | MINOR | 26.7.0 | Gated behind `frontier-meta-framework` flag |

### Frontier Features

Frontier features (unstable, experimental):
- Always gated behind `frontier-*` or `meta-feature` Cargo flags
- New frontier feature = **MINOR** bump
- Frontier feature bug fix = **PATCH** bump
- Can change in MINOR releases without MAJOR bump (unstable API)

---

## Release Workflow

### Phase 1: Planning (1-2 days before)

1. Review commits since last release
2. Determine version bump type (MAJOR/MINOR/PATCH)
3. Collect changelog entries
4. Draft migration guide (if MAJOR breaking changes)

### Phase 2: Preparation (morning of release)

1. Update CHANGELOG.md
2. Bump version across Cargo.toml files
3. Update README.md version examples
4. Run quality gates
5. Commit version bump

### Phase 3: Publishing (release day)

1. Dry-run macros publish
2. Publish macros crate
3. Dry-run main publish
4. Publish main crate
5. Verify on crates.io

### Phase 4: Post-Release

1. Create git tag
2. Push tag (triggers GitHub Actions)
3. Monitor CI/CD pipeline
4. GitHub Release created automatically
5. Announce release (optional)

---

## Version Bumping

### Automated Bumping Script

**Location**: `scripts/bump-version.sh`

```bash
./scripts/bump-version.sh 26.6.15
```

**What it does**:
- Updates `Cargo.toml` (main crate version)
- Updates `Cargo.toml` workspace dependencies (macros version)
- Updates `clap-noun-verb-macros/Cargo.toml`
- Updates README.md version references
- Verifies all changes applied correctly

**Dry-run first**:
```bash
# Check what would be changed
git diff --no-index /dev/null Cargo.toml | grep version || true
grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml
```

### Manual Verification

After bumping, verify version consistency:

```bash
# All should match
grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml

# Expected output:
# Cargo.toml:version = "26.6.15"
# Cargo.toml:[workspace.dependencies]
# Cargo.toml:clap-noun-verb = { ... version = "26.6.15" }
# Cargo.toml:clap-noun-verb-macros = { ... version = "26.6.15" }
# clap-noun-verb-macros/Cargo.toml:version = "26.6.15"
```

### Testing Version Changes

```bash
cargo check          # Verify compilation
cargo make test      # Run full test suite
cargo doc --no-deps  # Build documentation
```

---

## Changelog Management

### Format: Keep a Changelog

**File**: `CHANGELOG.md`

**Structure**:
```markdown
## [Unreleased]

### Added
- New features

### Changed
- Enhancements

### Deprecated
- Deprecation notices

### Removed
- Breaking removals

### Fixed
- Bug fixes

### Security
- Security fixes

### Technical Details
- Internal notes (not user-facing)

## [26.6.14] - 2026-06-14

### Added
- Feature X
...
```

### Adding Entries During Development

Maintain an `[Unreleased]` section in CHANGELOG.md:

```markdown
## [Unreleased]

### Added
- Graph query performance improvements (PR #456)
- New `#[arg(validate)]` macro attribute
- Support for custom output formatters

### Fixed
- Fixed panic when parsing empty argument lists (issue #455)

### Technical Details
- Optimized linkme distributed slice traversal
```

### At Release Time

1. Update entry header:
   ```markdown
   ## [26.6.15] - 2026-06-14
   ```
   Replace `[Unreleased]` with `[VERSION] - YYYY-MM-DD`

2. Add new Unreleased section:
   ```markdown
   ## [Unreleased]

   ### Added

   ### Changed

   ### Fixed
   ```

### Writing Good Changelog Entries

**Bad Examples** (avoid):
- "Fixed bug"
- "Updated code"
- "Made improvements"
- "Various fixes"

**Good Examples**:
- "Fixed panic in `CommandRegistry::run()` when handling empty argument strings (issue #456)"
- "Changed `#[verb]` macro to auto-detect noun from module filename, reducing boilerplate"
- "Added support for custom output formatters via `#[format]` attribute"
- "Improved error messages with color-coded suggestions for invalid argument combinations"

**Include Details**:
- Which function/module is affected
- What the user-visible change is
- Issue/PR reference (if applicable)
- Migration path (if breaking)

### Changelog Tools

**Option 1: Manual (recommended, full control)**
```bash
# 1. Review commits since last release
git log v26.6.13..HEAD --oneline

# 2. Edit CHANGELOG.md directly
vim CHANGELOG.md

# 3. Organize by category (Added, Changed, Fixed, etc.)
# 4. Write clear, user-facing descriptions
```

**Option 2: Git-based (semi-automated)**
```bash
# Install git-cliff (optional)
cargo install git-cliff

# Generate changelog automatically
git-cliff --latest
```

---

## Publishing Process

### Overview: Dual-Crate Publishing

The framework uses two crates:
1. **clap-noun-verb-macros** (published first)
2. **clap-noun-verb** (depends on macros, published second)

**Why order matters**: The main crate depends on the macros crate. The macros must be published and indexed on crates.io before the main crate can successfully publish.

### Step-by-Step Publishing

#### Step 1: Dry-Run Macros Publish

```bash
cargo make publish-dry-run-macros
```

**What it checks**:
- Cargo.toml is valid
- All dependencies are published
- No yanked versions are used
- **Does NOT actually publish**

**Success output**:
```
   Packaging clap-noun-verb-macros v26.6.15
    Verifying clap-noun-verb-macros v26.6.15
    Compiling clap-noun-verb-macros v26.6.15
     Finished release [optimized] target(s)
     Uploading clap-noun-verb-macros v26.6.15
✓ Dry-run successful
```

**Common issues**:
| Error | Cause | Fix |
|-------|-------|-----|
| `error: can't update crate with different features` | Crate already published with different features | Bump PATCH version and try again |
| `error: dependency not found: linkme` | Dependency not published yet | Wait for dependency to be available on crates.io, then retry |

#### Step 2: Publish Macros Crate

```bash
cargo make publish-macros
```

**Requires**: `CARGO_REGISTRY_TOKEN` environment variable

**Setup token**:
```bash
# Option 1: Export to environment
export CARGO_REGISTRY_TOKEN="eyJhbGciOiJIUzI1NiJ9..."

# Option 2: Create ~/.cargo/credentials.toml
cat > ~/.cargo/credentials.toml << 'EOF'
[registries.crates-io]
token = "eyJhbGciOiJIUzI1NiJ9..."
EOF
chmod 600 ~/.cargo/credentials.toml

# Get token from: https://crates.io/me → API Tokens
```

**Post-publish check**:
```bash
# crates.io takes 2-60 seconds to index
cargo search clap-noun-verb-macros --limit 1

# Look for output like:
# clap_noun_verb_macros = "26.6.15"
```

#### Step 3: Wait for Indexing

```bash
# Automated waiting loop
for i in {1..30}; do
    if cargo search clap-noun-verb-macros --limit 1 | grep -q "26.6.15"; then
        echo "✓ Macros published"
        break
    fi
    echo "Waiting... ($i/30)"
    sleep 2
done
```

#### Step 4: Dry-Run Main Crate Publish

```bash
cargo make publish-dry-run
```

**What it verifies**:
- Main crate dependencies (including macros) are available
- clap-noun-verb-macros v26.6.15 is published
- All dependencies are available

#### Step 5: Publish Main Crate

```bash
cargo make publish
```

**This includes macros publishing** (via Makefile dependencies):
- Automatically depends on `publish-macros`
- Runs all quality gates first
- Publishes both crates in correct order

#### Step 6: Verify on crates.io

```bash
# Check both crates published
cargo search clap-noun-verb --limit 1
cargo search clap-noun-verb-macros --limit 1

# Expected:
# clap_noun_verb = "26.6.15"
# clap_noun_verb_macros = "26.6.15"

# Check documentation
# Visit: https://docs.rs/clap-noun-verb/26.6.15/
# Visit: https://docs.rs/clap-noun-verb-macros/26.6.15/
```

### Automated Publishing: GitHub Actions

**Triggered automatically on tag push**:

```bash
git tag v26.6.15 -m "Release v26.6.15"
git push origin v26.6.15
```

**Pipeline runs**:
1. Validate quality gates (format, clippy, tests, docs)
2. MSRV check (Rust 1.74)
3. Security audit (cargo-audit)
4. License check (cargo-deny)
5. Publish macros
6. Wait for indexing
7. Publish main crate
8. Create GitHub Release

**Monitor progress**:
- GitHub Actions tab: https://github.com/seanchatmangpt/clap-noun-verb/actions
- Click on "Release" workflow run
- View each job status

---

## Crates.io Metadata

### Cargo.toml Metadata

Located in `[package]` section:

```toml
[package]
name = "clap-noun-verb"
version = "26.6.15"
edition = "2021"
rust-version = "1.74"

# User-facing metadata
license = "MIT OR Apache-2.0"
authors = ["Sean Chatman <seanchatmangpt@gmail.com>"]
repository = "https://github.com/seanchatmangpt/clap-noun-verb"
documentation = "https://docs.rs/clap-noun-verb"
homepage = "https://github.com/seanchatmangpt/clap-noun-verb"

# Search keywords (max 5, comma-separated)
keywords = ["cli", "clap", "noun-verb", "command-line", "typer"]

# Categories (helps discoverability)
categories = ["command-line-utilities", "development-tools"]

# Long description (max 300 chars)
description = "Rust CLI framework with noun-verb patterns, graph operations, and capability packing"

# README (shown on crates.io page)
readme = "README.md"
```

### Metadata Update Checklist

When adding major features:

```bash
# Update description to reflect new capabilities
description = "..."  # Max 300 chars

# Add/update keywords if features are related
keywords = ["cli", "clap", "noun-verb", "command-line", "NEW"]

# Add categories if new domain
categories = ["command-line-utilities", "development-tools", "NEW"]

# Verify README has:
# - Feature overview
# - Quick start example
# - Links to documentation
# - MSRV policy
# - License
```

### Documentation Build

docs.rs automatically builds documentation from:
- Rustdoc comments (`/// ...`)
- README.md
- examples/

**Ensure no warnings**:
```bash
cargo doc --all-features --no-deps
# Should complete without RUSTDOCFLAGS=-D warnings errors
```

---

## GitHub Release Management

### Automatic Release Creation (Recommended)

When you push a git tag, GitHub Actions automatically creates a release:

```bash
git tag v26.6.15 -m "Release v26.6.15"
git push origin v26.6.15
```

**Automatically generates**:
- Release notes with changelog
- Marks as "latest" (unless pre-release)
- Links to published crates

### Manual Release Creation

For custom release notes:

```bash
# Create release with custom notes
gh release create v26.6.15 \
  --title "v26.6.15: Performance & Graph Improvements" \
  --notes-file release-notes.md

# Create pre-release (for RCs)
gh release create v26.6.15-rc.1 \
  --prerelease \
  --title "v26.6.15-rc.1: Release Candidate"

# Upload additional artifacts
gh release upload v26.6.15 ./target/release/my-binary
```

### Release Notes Format

Structure for maximum user value:

```markdown
## What's Changed

### Major Features
- **Graph Performance**: 3x faster SPARQL queries with optimized indexing
- **New Diagnostics**: Real-time system health monitoring via `doctor` verb

### Enhancements
- Improved error messages with color-coded suggestions
- Support for custom output formatters

### Bug Fixes
- Fixed panic when parsing empty argument lists (issue #456)
- Fixed memory leak in distributed slice traversal (#457)

### Breaking Changes
⚠️ **None** — This is a fully backward-compatible release.

## Installation

```bash
cargo add clap-noun-verb@26.6.15
```

## Full Changelog

See [CHANGELOG.md](https://github.com/seanchatmangpt/clap-noun-verb/blob/main/CHANGELOG.md#26615---2026-06-14) for complete details.

## Contributors

Thanks to: @user1, @user2, @user3
```

### Release Notes Checklist

- [ ] Title clearly indicates what's new
- [ ] Mentions breaking changes prominently (if MAJOR)
- [ ] Includes migration guide (if breaking)
- [ ] Shows installation command
- [ ] Links to full CHANGELOG
- [ ] Marks as pre-release (if RC/alpha/beta)
- [ ] Tag matches version (e.g., `v26.6.15`)
- [ ] Tag matches Cargo.toml version

---

## Breaking Changes Communication

### MAJOR Version Release Process

Breaking changes require careful communication:

#### 1. Announce in Release Notes

```markdown
## ⚠️ Breaking Changes in v27.0.0

This is a MAJOR version release with breaking changes.
All users must upgrade and follow the migration guide below.

### What's Changing
- HandlerInput trait redesigned
- Output serialization format updated
- CommandRegistry API refactored
```

#### 2. Provide Migration Guide

Create `docs/MIGRATION_V26_TO_V27.md`:

```markdown
# Migration Guide: v26.x → v27.0

## 1. HandlerInput Type Changes

### Before (v26)
```rust
pub struct HandlerInput {
    pub args: Vec<String>,
    pub config: Config,
}
```

### After (v27)
```rust
pub struct HandlerInput {
    pub parsed: ParsedArgs,
    pub context: ExecutionContext,
}
```

### Your Code
```rust
// OLD (v26)
#[verb]
fn my_command(input: HandlerInput) -> Result<String> {
    let arg = input.args.get(0)?;
    ...
}

// NEW (v27)
#[verb]
fn my_command(input: HandlerInput) -> Result<String> {
    let arg = input.parsed.positional(0)?;
    ...
}
```

## 2. Output Serialization Changes

### Before (v26)
```json
{"status": "ok", "data": {...}}
```

### After (v27)
```json
{"result": "success", "value": {...}}
```

### Migration
Update any JSON parsing code that depends on the old format.

## 3. CommandRegistry Changes

See full guide for remaining breaking changes...
```

#### 3. Deprecation Period (Recommended for API changes)

Release MINOR versions with deprecation warnings first:

```rust
#[deprecated(
    since = "26.8",
    note = "use TelemetryManager::instance() instead"
)]
pub fn create_span(name: &str) -> Span {
    ...
}
```

Then remove in next MAJOR version.

#### 4. Communication Strategy

1. **CHANGELOG.md**: Technical details
2. **GitHub Release Notes**: High-level summary + migration guide
3. **Migration Guide Doc**: Step-by-step upgrade instructions
4. **Twitter/Blog**: High-visibility breaking changes
5. **Discord/Community**: Answer user questions

---

## Yanking & Emergency Procedures

### What is Yanking?

Yanking removes a published version from crates.io's default results:
- Still downloadable (useful for CI/CD pinning)
- Don't show in `cargo update` or `cargo add` suggestions
- Display warning: `yanked: true`

### When to Yank

**Critical Security Vulnerabilities**:
```bash
# If v26.6.14 has CVE
cargo yank --vers 26.6.14 -p clap-noun-verb
cargo yank --vers 26.6.14 -p clap-noun-verb-macros
```

**Catastrophic Regressions** (breaks core functionality):
```bash
# If v27.0.0 doesn't compile on some platforms
cargo yank --vers 27.0.0 -p clap-noun-verb
```

**Publish Errors** (wrong binary, missing files):
```bash
# If macros were published incorrectly
cargo yank --vers 26.6.14 -p clap-noun-verb-macros
```

### How to Yank

```bash
# Install cargo-edit if needed
cargo install cargo-edit

# Yank a version
cargo yank --vers 26.6.14 -p clap-noun-verb
cargo yank --vers 26.6.14 -p clap-noun-verb-macros

# Undo a yank (if you yanked wrong version)
cargo yank --vers 26.6.14 -p clap-noun-verb --undo
```

### Yank Communication Template

```markdown
## Security Alert: v26.6.14 Yanked

**Affected**: clap-noun-verb v26.6.14 (released June 14, 2026)

**Issue**: [CVE-2026-12345](https://nvd.nist.gov/...) - Potential panic on untrusted input

**Required Action**:
- Upgrade immediately: `cargo update clap-noun-verb`
- If pinned to v26.6.14, update Cargo.toml: `clap-noun-verb = "26.6.15"`

**Status**:
- ✓ v26.6.15 released with fix
- ✓ v26.6.14 yanked
- ✓ Fix committed to main branch
```

### Emergency Patch Release

If critical bug discovered post-release:

```bash
# 1. Create fix commit
git checkout main
git pull origin main
git commit -m "fix: critical bug in CommandRegistry (issue #999)"

# 2. Bump PATCH version
./scripts/bump-version.sh 26.6.16

# 3. Update CHANGELOG
vim CHANGELOG.md  # Add [26.6.16] section

# 4. Verify and publish
./scripts/pre-release-check.sh 26.6.16
git push origin main
cargo make publish

# 5. Tag and trigger CI/CD
git tag v26.6.16 -m "Emergency patch: critical bug fix"
git push origin v26.6.16

# Total time: ~5 minutes
```

---

## Automation Scripts

### 1. Version Bumping Script

**Location**: `scripts/bump-version.sh`

```bash
./scripts/bump-version.sh 26.6.15
```

**Features**:
- Updates main Cargo.toml (package version)
- Updates workspace dependencies (macros reference)
- Updates macros Cargo.toml
- Updates README.md version references
- Verifies all changes applied

**Output**:
```
✓ Updated to version 26.6.15

Verify with:
  grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml
```

### 2. Pre-Release Check Script

**Location**: `scripts/pre-release-check.sh`

```bash
./scripts/pre-release-check.sh 26.6.15
```

**Runs 7 automated quality gates**:

| Gate | Purpose | Critical |
|------|---------|----------|
| Version Consistency | All versions match | YES |
| Compilation | Code compiles | YES |
| Tests | All tests pass | YES |
| Warnings | Zero clippy warnings | YES |
| Documentation | CHANGELOG + README updated | YES |
| Examples | Examples build | MEDIUM |
| Git Status | Clean working directory | MEDIUM |

**Output on success**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ ALL GATES PASSED - Ready for Release!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Next steps:
  1. cargo make publish-dry-run-macros
  2. cargo make publish-macros
  3. cargo make publish-dry-run
  4. cargo make publish
  5. Verify on crates.io
  6. Create git tag: git tag v26.6.15
  7. Push tag: git push origin v26.6.15
```

### 3. Makefile Release Tasks

```bash
# Dry-run macros publish
cargo make publish-dry-run-macros

# Publish macros (after dry-run)
cargo make publish-macros

# Dry-run main publish
cargo make publish-dry-run

# Publish main (includes macros via dependencies)
cargo make publish

# Full release workflow (all checks + publish)
cargo make publish-all

# Pre-release validation (7 gates)
cargo make release-check

# Verify published
cargo make verify-publish
```

### 4. Release Checklist Script

**Location**: `scripts/release-checklist.sh`

Interactive checklist for release operators:

```bash
./scripts/release-checklist.sh 26.6.15
```

---

## Release Checklist

Use this for each release:

### Pre-Release (1-2 days before)

```
□ Review all commits since last release
  git log v26.6.13..HEAD --oneline

□ Determine MAJOR/MINOR/PATCH bump
  (Review Semantic Versioning strategy above)

□ Collect all changelog entries
  (Review git commits and PRs)

□ Draft migration guide (if MAJOR breaking changes)
  vim docs/MIGRATION_V26_TO_V27.md

□ Get team approval (if MAJOR version)
```

### Version Bumping (morning of release)

```
□ Update version across files
  ./scripts/bump-version.sh 26.6.15

□ Verify all versions match
  grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml

□ Update CHANGELOG.md
  - Move [Unreleased] → [26.6.15] - 2026-06-15
  - Add new [Unreleased] section

□ Update README.md version examples
  sed -i 's/"26.6.14"/"26.6.15"/g' README.md

□ Compile and test
  cargo check
  cargo make test
  cargo make lint

□ Commit version bump
  git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md
  git commit -m "chore(release): bump to 26.6.15"
```

### Quality Gates

```
□ Run automated quality check
  ./scripts/pre-release-check.sh 26.6.15

□ All 7 gates should pass:
  ✓ Version Consistency
  ✓ Compilation
  ✓ Tests (100% pass)
  ✓ Warnings (0 count)
  ✓ Documentation
  ✓ Examples Build
  ✓ Git Status
```

### Publishing

```
□ Push to main
  git push origin main

□ Dry-run macros publish
  cargo make publish-dry-run-macros

□ Publish macros
  cargo make publish-macros

□ Wait for indexing (~30 seconds)
  cargo search clap-noun-verb-macros --limit 1

□ Dry-run main publish
  cargo make publish-dry-run

□ Publish main
  cargo make publish

□ Verify both published on crates.io
  cargo search clap-noun-verb --limit 1
  cargo search clap-noun-verb-macros --limit 1
```

### GitHub Release

```
□ Create git tag
  git tag v26.6.15 -m "Release v26.6.15"

□ Push tag (triggers GitHub Actions)
  git push origin v26.6.15

□ Monitor GitHub Actions
  https://github.com/seanchatmangpt/clap-noun-verb/actions

□ Verify GitHub Release created
  https://github.com/seanchatmangpt/clap-noun-verb/releases/tag/v26.6.15
```

### Post-Release Verification (next day)

```
□ GitHub Release created with notes
□ docs.rs documentation built and live
□ No GitHub Issues about release problems
□ Check for user migration questions
□ Announce on Twitter/blog (if major feature)
```

---

## Common Release Tasks

### Task: Create PATCH Release (Bug Fix)

**Scenario**: Critical bug fix released to production

**Time**: ~10 minutes

```bash
# 1. Create fix commit (2 min)
git checkout main
git pull origin main
git commit -m "fix: critical bug in CommandRegistry (issue #999)"

# 2. Bump PATCH version (1 min)
./scripts/bump-version.sh 26.6.15

# 3. Update CHANGELOG (2 min)
vim CHANGELOG.md
# Add: ## [26.6.15] - 2026-06-15
#      ### Fixed
#      - Critical bug fix description

# 4. Verify and commit (2 min)
./scripts/pre-release-check.sh 26.6.15
git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md
git commit --amend  # Amend to include CHANGELOG

# 5. Publish (3 min, mostly waiting)
git push origin main
cargo make publish

# 6. Tag and trigger CI/CD (1 min)
git tag v26.6.15 -m "Patch release: critical bug fix"
git push origin v26.6.15
# GitHub Actions handles rest automatically
```

### Task: Create MINOR Release (New Features)

**Scenario**: New features to release

**Time**: ~15 minutes

```bash
# 1. Ensure main branch is up to date
git checkout main
git pull origin main

# 2. Bump MINOR version
./scripts/bump-version.sh 26.7.0

# 3. Update CHANGELOG thoroughly
vim CHANGELOG.md
# Add: ## [26.7.0] - 2026-06-15
#      ### Added
#      - Feature 1 with detailed description
#      - Feature 2 with detailed description
#      ### Changed
#      - Enhancement with rationale

# 4. Update README.md if major new features
vim README.md  # Update feature list, examples

# 5. Run quality gates
./scripts/pre-release-check.sh 26.7.0

# 6. Commit
git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md README.md
git commit -m "chore(release): bump to 26.7.0 - new features"

# 7. Publish
git push origin main
cargo make publish

# 8. Tag
git tag v26.7.0 -m "Release v26.7.0: new features"
git push origin v26.7.0
```

### Task: Create MAJOR Release (Breaking Changes)

**Scenario**: Major breaking API changes

**Time**: ~30 minutes (more manual work)

```bash
# 1. Ensure migration guide is complete
ls docs/MIGRATION_V26_TO_V27.md

# 2. Bump MAJOR version
./scripts/bump-version.sh 27.0.0

# 3. Update CHANGELOG with breaking changes section
vim CHANGELOG.md
# Add: ## [27.0.0] - 2026-06-15
#      ### Breaking Changes
#      - Detailed description of each breaking change
#      - Rationale and migration path
#      ### Removed
#      - Old API that was deprecated in v26.x
#      ### Added
#      - New APIs that replace removed ones

# 4. Update README with migration guide link
vim README.md
# Add: "See [MIGRATION_V26_TO_V27.md](docs/MIGRATION_V26_TO_V27.md) for upgrade guide"

# 5. Run quality gates
./scripts/pre-release-check.sh 27.0.0

# 6. Commit
git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md README.md
git commit -m "chore(release): bump to 27.0.0 - breaking changes"

# 7. Publish
git push origin main
cargo make publish

# 8. Tag
git tag v27.0.0 -m "Release v27.0.0: breaking changes"
git push origin v27.0.0

# 9. Post-release communication
# - Create blog post about breaking changes
# - Pin GitHub issue with migration guide
# - Announce on Twitter/community
```

---

## Release Troubleshooting

### Problem: Tests failing before release

```bash
# Get more details
cargo test --all-features --lib -- --nocapture

# Check specific test
cargo test test_name --lib -- --nocapture

# Run single-threaded for deterministic results
RUST_TEST_THREADS=1 cargo test
```

### Problem: Clippy warnings block release

```bash
# See all warnings
cargo clippy --all-features -- -D warnings

# Fix warnings
# Option 1: Fix the code
# Option 2: Add allow attribute (if false positive)
#   #[allow(clippy::...)
//   fn my_function() { ... }

# Verify
cargo clippy --all-features -- -D warnings
```

### Problem: Macros crate takes too long to index

```bash
# Check indexing status
cargo search clap-noun-verb-macros --limit 1

# Expected output when indexed:
# clap_noun_verb_macros = "26.6.15"

# If not indexed yet, wait and retry
# Retry loop (GitHub Actions does this automatically)
for i in {1..30}; do
    if cargo search clap-noun-verb-macros --limit 1 | grep -q "26.6.15"; then
        echo "Indexed!"
        break
    fi
    sleep 2
done
```

### Problem: docs.rs documentation not building

```bash
# Check what docs.rs sees
cargo doc --all-features --no-deps

# Verify RUSTDOCFLAGS compatibility
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

# Check docs.rs build configuration
grep -A 5 '\[package.metadata.docs.rs\]' Cargo.toml
```

### Problem: Version mismatch detected

```bash
# See all versions in workspace
grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml

# Fix with bump script
./scripts/bump-version.sh <correct-version>

# Verify all match
grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml
# Should all be identical
```

---

## References

- **Semantic Versioning 2.0.0**: https://semver.org/
- **Keep a Changelog**: https://keepachangelog.com/
- **Cargo Publishing Guide**: https://doc.rust-lang.org/cargo/reference/publishing.html
- **Crates.io Metadata**: https://crates.io/me
- **docs.rs Build Configuration**: https://docs.rs/
- **GitHub Releases**: https://github.com/seanchatmangpt/clap-noun-verb/releases
- **Security Advisories**: https://rustsec.org/

---

**Last Updated**: 2026-06-14  
**Author**: Sean Chatman  
**Document Version**: 1.0  
**Rust MSRV**: 1.74 (main), 1.70 (macros)
