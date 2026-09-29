# Deployment and Release Standards for clap-noun-verb

**Version**: 26.9.1 | **Last Updated**: 2026-08-20 | **Status**: Production-Ready

## Table of Contents

1. [Release Types & Decision Criteria](#release-types--decision-criteria)
2. [Semantic Versioning & Frontier Features](#semantic-versioning--frontier-features)
3. [Changelog Management](#changelog-management)
4. [Breaking Changes & Deprecation](#breaking-changes--deprecation)
5. [Publishing Workflow](#publishing-workflow)
6. [Post-Release Operations](#post-release-operations)
7. [Hotfix Process](#hotfix-process)
8. [Yanking & Version Retirement](#yanking--version-retirement)
9. [SLO Targets & Compliance](#slo-targets--compliance)
10. [CI/CD Automation](#cicd-automation)
11. [Decision Trees & Checklists](#decision-trees--checklists)
12. [Emergency Procedures](#emergency-procedures)

---

## 1. Release Types & Decision Criteria

### Release Type Decision Tree

```
START: Change is ready
  │
  ├─ Does it break existing APIs or remove documented features?
  │   YES → MAJOR release (e.g., 25.0.0 → 26.0.0)
  │   NO  → Continue
  │
  ├─ Does it add new features (verbs, nouns, modules)?
  │   YES → MINOR release (e.g., 26.6.0 → 26.9.1)
  │   NO  → Continue
  │
  ├─ Does it fix bugs or improve documentation?
  │   YES → PATCH release (e.g., 26.9.1 → 26.9.1)
  │   NO  → Continue
  │
  └─ Is it experimental/frontier functionality?
      YES → PRE-RELEASE (e.g., 26.9.1-alpha.1, 26.9.1-rc.1)
      NO  → Error: Classify the change
```

### 1.1 MAJOR Release (Breaking Changes)

**When to use**: Incompatible changes to stable, documented APIs.

**Examples**:
- Redesign of trait APIs (`NounCommand`, `VerbCommand` signatures)
- Removal of previously stable modules
- Changes to `#[verb]` or `#[noun]` macro behavior
- Breaking changes to `HandlerInput`/`HandlerOutput` types
- JSON schema changes (output format incompatible with v25 parsers)

**Procedure**:
1. Update main version in `Cargo.toml` and `clap-noun-verb-macros/Cargo.toml`
2. Create `BREAKING_CHANGES.md` in `docs/` documenting migration path
3. Add detailed deprecation guide in `CHANGELOG.md`
4. Run full test suite and CI
5. Publish with migration announcement

**Recent Examples**:
- v26.0.0 (Nov 2025): Minimalist refactor, removal of non-core optional modules
- v5.0.0 (Nov 2025): Telemetry API refactor

### 1.2 MINOR Release (New Features)

**When to use**: Backward-compatible additions (new features, new verbs/nouns, new traits).

**Examples**:
- New macro attributes: `#[arg(new_param)]`
- New output formats or serialization options
- New modules/traits/functions (additive only)
- New frontier features with `frontier-*` flag
- MSRV bumps (with careful consideration)

**Procedure**:
1. Update minor version in both `Cargo.toml` files
2. Add feature descriptions to `CHANGELOG.md` under `### Added`
3. Ensure backward compatibility (no breaking changes)
4. Write examples or documentation for new features
5. Test with all feature combinations

**Recent Examples**:
- v26.9.1 (June 2026): Graph module, capability packing, diagnostics
- v5.4.0 (Jan 2026): ggen integration, frontier features

### 1.3 PATCH Release (Bug Fixes)

**When to use**: Bug fixes, documentation improvements, no new features.

**Examples**:
- Fixing incorrect behavior in existing verbs
- Trait implementations (new impl blocks)
- Documentation fixes
- Clippy warning resolution
- Error message improvements

**Procedure**:
1. Update patch version only
2. Add fixes to `CHANGELOG.md` under `### Fixed`
3. Include test cases demonstrating the fix
4. Quick release cycle (less extensive testing than MINOR/MAJOR)

### 1.4 Pre-Release Versions

**Format**: `MAJOR.MINOR.PATCH-<type>.<number>` (e.g., `26.9.1-alpha.1`, `26.9.1-rc.2`)

**When to use**:
- Alpha releases: Early feature preview, may have breaking changes
- Beta releases: Feature-complete, testing for regressions
- Release candidates (RC): Final validation before stable release

**Procedure**:
1. Increment pre-release suffix for each iteration
2. Tag as pre-release on GitHub (not picked up by default `cargo update`)
3. Announce in community channels with "EXPERIMENTAL" warning
4. Track issues and feedback explicitly

**Incrementing Pre-Release**:
```bash
# Alpha series
26.9.1-alpha.1 → 26.9.1-alpha.2 → 26.9.1-alpha.3

# Beta series (after alpha is complete)
26.9.1-beta.1 → 26.9.1-beta.2

# RC series (after beta is complete)
26.9.1-rc.1 → 26.9.1-rc.2

# Stable (final release)
26.9.1
```

---

## 2. Semantic Versioning & Frontier Features

### 2.1 Semantic Versioning Format

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
  26   .  6   .  14
```

- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible feature additions
- **PATCH**: Backward-compatible bug fixes
- **PRERELEASE**: Optional alpha/beta/rc suffix
- **BUILD**: Metadata (rarely used)

### 2.2 Frontier Feature Versioning

**Frontier features** are experimental, unstable features behind feature flags (e.g., `meta-framework`, `fractal-patterns`, `executable-specs`).

**Versioning Strategy**:
1. Frontier features can have **breaking changes within MINOR versions** (semantic exception)
2. Breaking changes to frontier features documented in `CHANGELOG.md` with `[FRONTIER]` prefix
3. Users explicitly opt into frontier features via `Cargo.toml`
4. When frontier features stabilize, transition to standard semver rules

**Example**:
```toml
# In CHANGELOG.md
## [26.9.1] - 2026-07-01

### Added (Frontier)
- [FRONTIER] `#[executable-specs]` macro for specification-based testing
- [FRONTIER] `economic-sim` module for token economics modeling

### Changed (Frontier)
- [FRONTIER] `#[fractal-patterns]` attribute syntax changed (backward incompatible)
  Users: Update `#[fractal(depth=N)]` to `#[fractal_iter(n = N)]`
```

### 2.3 MSRV (Minimum Supported Rust Version)

**Current MSRV**: Rust 1.74 (main crate), Rust 1.70 (macros crate)

**Bumping MSRV**:
- Treat MSRV bump as potential MAJOR (some ecosystems require this)
- Or MINOR if coordinated with careful dependency management
- Announce clearly in release notes
- Do NOT bump MSRV in PATCH releases

**MSRV Testing**:
```bash
# CI tests against declared MSRV
cargo +1.74 build
cargo +1.74 test --all-features
```

---

## 3. Changelog Management

### 3.1 Changelog Format

Follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

```markdown
# Changelog

All notable changes to clap-noun-verb will be documented in this file.

## [Unreleased]

### Added
- New `#[verb(priority = N)]` attribute for verb ordering

### Changed
- Improved error messages in CommandRouter

### Fixed
- Bug: Verbs with no args were not dispatching correctly
- Security: Dependency version bump for potential CVE

### Deprecated
- `old_function()` - use `new_function()` instead (will be removed in v27.0.0)

### Removed
- Removed deprecated `legacy_module` (announced in v26.0.0)

### Security
- Updated `dependency-x` to patch security issue

## [26.9.1] - 2026-06-14

### Added
- New graph query functionality for RDF validation

### Fixed
- Fixed panic in registry validation on empty verb set

## [26.9.1] - 2026-06-13

### Changed
- Minimalist refactor: removed non-core optional modules
```

### 3.2 Section Guidelines

| Section | Purpose | Example |
|---------|---------|---------|
| **Added** | New features | "New `pack add` verb for capability management" |
| **Changed** | Enhancements to existing features | "Improved performance of CommandRegistry lookup" |
| **Fixed** | Bug fixes | "Fixed panic when verb has no arguments" |
| **Deprecated** | Features marked for future removal | "`old_api()` deprecated; use `new_api()` instead" |
| **Removed** | Deletion of previously deprecated items | "Removed `wizard` module (deprecated in v26.0.0)" |
| **Security** | Dependency/code security updates | "Updated `serde` to patch CVE-2024-1234" |

### 3.3 Detail Level Guidance

**Too vague** (❌):
```
- Fixed various bugs
- Updated dependencies
```

**Good** (✓):
```
- Fixed: Verb handlers with no arguments failed to dispatch
- Updated: `tokio` from 1.39 to 1.40
```

**Too detailed** (❌):
```
- Changed `CommandRouter::route()` to use `Result<Output>` instead of `Result<(Output, Metadata)>`
  Internal refactoring in router.rs line 142 to eliminate unnecessary tuple unwrapping
  which was identified during profiling as a bottleneck
```

**Better** (✓):
```
- Changed: `CommandRouter::route()` return type for cleaner error handling
```

### 3.4 Generating Changelog Automatically

**Conventional Commits Strategy**:

Commit messages follow conventional commit format:
```
<type>(<scope>): <subject>

<body>
```

Types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `style`, `chore`

**Changelog Script** (in development):
```bash
#!/bin/bash
# Generate changelog from git log since last tag

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "HEAD")
VERSION="$1"

echo "## [$VERSION] - $(date +%Y-%m-%d)"
echo ""

# Features
if git log "$LAST_TAG..HEAD" --oneline | grep -q "feat:"; then
    echo "### Added"
    git log "$LAST_TAG..HEAD" --grep="^feat:" --oneline | \
        sed 's/^[a-f0-9]* feat(\([^)]*\)): /- /'
    echo ""
fi

# Fixes
if git log "$LAST_TAG..HEAD" --oneline | grep -q "fix:"; then
    echo "### Fixed"
    git log "$LAST_TAG..HEAD" --grep="^fix:" --oneline | \
        sed 's/^[a-f0-9]* fix(\([^)]*\)): /- /'
fi
```

### 3.5 Migration Guides for Major Releases

For MAJOR releases with breaking changes, create `docs/MIGRATION_GUIDES.md`:

```markdown
# Migration Guide: v25 → v26

## Removed: wizard Module

**Announcement**: v25.5.0  
**Removal**: v26.0.0

### Impact
If you use the `wizard` module for interactive CLI generation, migration is required.

### Before (v25)
```rust
use clap_noun_verb::wizard::CliWizard;

let wizard = CliWizard::new();
let app = wizard.generate_from_config("config.yml")?;
```

### After (v26)
Use `CliBuilder` directly or the new agent-based generation:
```rust
use clap_noun_verb::CliBuilder;

let app = CliBuilder::new("myapp")
    .version("1.0.0")
    .build_from_registry(&registry)?;
```

### Migration Steps
1. Replace `CliWizard::new()` with `CliBuilder::new()`
2. Remove `wizard` feature from `Cargo.toml` (if present)
3. Update configuration to match CliBuilder API
4. Test with `cargo make test-all`

## Changed: HandlerOutput Serialization

**Announcement**: v25.8.0  
**Breaking**: v26.0.0

### Impact
JSON output structure has changed for nested results.

### Before (v25)
```json
{
  "status": "ok",
  "data": { "items": [...] }
}
```

### After (v26)
```json
{
  "result": {
    "status": "ok",
    "data": { "items": [...] }
  }
}
```

### Workaround
For JSON parsing, update your jq filters:
```bash
# Before
myapp verb noun | jq '.status'

# After
myapp verb noun | jq '.result.status'
```
```

---

## 4. Breaking Changes & Deprecation

### 4.1 Deprecation Timeline

**Standard Deprecation Cycle**: 3 release cycles (approximately 3-6 months)

```
Cycle 1: Announce deprecation in v26.5.0
  ├─ Add `#[deprecated]` attribute in code
  ├─ Document in CHANGELOG under "Deprecated"
  └─ Update docs with migration guidance

Cycle 2-3: Use in v26.6.0, v26.9.1
  ├─ Monitor usage via GitHub issues
  └─ Answer migration questions

Cycle 4: Remove in v27.0.0 (MAJOR)
  ├─ Remove code
  ├─ Document removal in CHANGELOG under "Removed"
  └─ Cross-reference deprecation announcement
```

### 4.2 Deprecation Announcement Template

**In Code**:
```rust
/// Runs a verb command (deprecated).
///
/// # Deprecated
/// Use [`CommandRouter::route()`] instead (v26.0.0+).
/// This function will be removed in v27.0.0.
///
/// # Examples
/// ```
/// # use clap_noun_verb::CommandRouter;
/// let router = CommandRouter::from_registry(&registry);
/// let output = router.route(&matches).await?;
/// ```
#[deprecated(
    since = "26.5.0",
    note = "use `CommandRouter::route()` instead"
)]
pub async fn dispatch_legacy(verb: &str) -> Result<HandlerOutput> {
    // Implementation
}
```

**In CHANGELOG**:
```markdown
## [26.5.0] - 2026-05-01

### Deprecated
- `dispatch_legacy()` function - use `CommandRouter::route()` instead
  (will be removed in v27.0.0)
```

**In MIGRATION_GUIDE**:
```markdown
## Deprecation: dispatch_legacy()

### Timeline
- **Announced**: v26.5.0 (May 2026)
- **Removal**: v27.0.0 (est. Nov 2026)

### Migration
Replace:
```rust
dispatch_legacy("services status").await?
```

With:
```rust
let router = CommandRouter::from_registry(&registry);
router.route(&matches).await?
```
```

### 4.3 Communication Strategy

1. **Announcement Release**: Update docs, CHANGELOG, send email
2. **Follow-up Releases**: Mention deprecation in release notes
3. **Removal Release**: Cross-reference deprecation announcement
4. **Post-Removal**: Link to migration guide in GitHub issues

---

## 5. Publishing Workflow

### 5.1 Pre-Publishing Checklist

**Use the automated pre-release check**:
```bash
scripts/pre-release-check.sh 26.9.1
```

This validates:
- ✓ Version consistency across `Cargo.toml` files
- ✓ Code compiles without errors
- ✓ All tests pass (100% pass rate)
- ✓ Zero compiler warnings
- ✓ CHANGELOG updated
- ✓ README version references updated
- ✓ Examples build successfully
- ✓ Git working directory clean

**Manual Checklist**:
```
Before Publishing:
☐ All 7 quality gates pass (run pre-release-check.sh)
☐ CHANGELOG has entry for new version
☐ Version numbers match across Cargo.toml files
☐ MSRV is documented (rust-version field)
☐ New features have examples
☐ Breaking changes have migration guide
☐ No security issues in dependencies (cargo audit)
☐ License compliance checked (cargo deny check)
☐ Git history is clean and linear (no rebase conflicts)
```

### 5.2 Publishing Order (Dual-Crate Model)

**CRITICAL**: Macros crate must be published BEFORE main crate (dependency order).

#### Step 1: Dry-run Macros Publish
```bash
cargo make publish-dry-run-macros
```
Validates metadata, dependencies, doc examples without pushing to registry.

#### Step 2: Publish Macros Crate
```bash
cargo make publish-macros
```
Publishing to crates.io typically takes 2-60 seconds to index.

#### Step 3: Wait for Indexing
```bash
# Poll until macros appear on crates.io
cargo search clap-noun-verb-macros --limit 1
```
Macros crate must be indexed before main crate can depend on it.

#### Step 4: Dry-run Main Crate
```bash
cargo make publish-dry-run
```

#### Step 5: Publish Main Crate
```bash
cargo make publish
```

#### Step 6: Verify Publication
```bash
cargo make verify-publish
```
Checks both crates appear on crates.io and docs.rs.

### 5.3 Publishing Workflow Script

Use the automated release script for interactive guidance:
```bash
./scripts/release-automation.sh [VERSION]
```

**Features**:
- Interactive version selection
- Git history review
- Pre-release quality gates
- Automatic version bumping
- CHANGELOG editing
- Dual-crate publishing with retry logic
- Git tag creation
- Post-release verification

**Full Release Workflow**:
```bash
# Interactive mode (prompts for version)
./scripts/release-automation.sh

# Direct mode (specify version)
./scripts/release-automation.sh 26.9.1
```

### 5.4 Publishing Permissions & Tokens

**Requirements**:
- crates.io account with publish permissions on `clap-noun-verb` and `clap-noun-verb-macros`
- API token set in `CARGO_REGISTRY_TOKEN` environment variable

**Setup**:
```bash
# Get token from https://crates.io/me → API Tokens
export CARGO_REGISTRY_TOKEN="<your-token>"

# Verify token works
cargo login
```

**Security**:
- Store token in `~/.cargo/credentials.toml` (not in git!)
- Or use environment variable with CI/CD secrets
- Rotate token annually

---

## 6. Post-Release Operations

### 6.1 Post-Release Checklist

```
After Publishing:
☐ Both crates indexed on crates.io (verify with cargo search)
☐ Documentation built on docs.rs
☐ GitHub Release created with release notes
☐ GitHub tag pushed (v26.9.1)
☐ Release announcement posted (if MAJOR/MINOR)
☐ Dependents updated (internal projects using this crate)
☐ Examples repo updated with new version
```

### 6.2 Monitoring & Validation

**Verify crates.io**:
```bash
# Check macros crate
curl https://crates.io/api/v1/crates/clap-noun-verb-macros/26.9.1

# Check main crate
curl https://crates.io/api/v1/crates/clap-noun-verb/26.9.1
```

**Verify docs.rs**:
- Navigate to https://docs.rs/clap-noun-verb/26.9.1
- Check all modules are documented
- Verify examples compile
- Check feature gating is correct

**Verify downloads**:
```bash
# Monitor download stats (24-48 hour delay)
curl https://crates.io/api/v1/crates/clap-noun-verb/downloads
```

### 6.3 Documentation Updates

**Post-Release Documentation Tasks**:
1. **Update README.md** with new version references
2. **Update quickstart guide** with new features
3. **Add examples** for new verbs/nouns
4. **Update API reference** if breaking changes
5. **Update feature matrix** in README
6. **Archive old release notes** to `docs/releases/`

**Example Documentation Structure**:
```
docs/
├── releases/
│   ├── 26.9.1.md          # Latest
│   ├── 26.9.1.md
│   ├── 26.9.1.md
│   └── v26-milestone.md
├── MIGRATION_GUIDES.md      # Breaking change migrations
├── DEPLOYMENT_STANDARDS.md  # This file
└── RELEASE_MANAGEMENT.md    # Operational details
```

### 6.4 Community Communication

**For MAJOR/MINOR releases**:

**1. Announcement Email** (to mailing list/community channels)
```markdown
Subject: clap-noun-verb v26.9.1 Released

Hi community,

We're excited to announce clap-noun-verb v26.9.1 with the following improvements:

**New Features**:
- Graph module for RDF validation (closes #123)
- Capability packing system (closes #145)

**Bug Fixes**:
- Fixed panic in registry validation (closes #156)

**Upgrade**:
cargo add clap-noun-verb@26.9.1
# or update Cargo.toml manually

**Documentation**:
- Release notes: https://github.com/seanchatmangpt/clap-noun-verb/releases/tag/v26.9.1
- Docs: https://docs.rs/clap-noun-verb/26.9.1/

**Breaking Changes**: None in this release

Thanks for using clap-noun-verb!
```

**2. GitHub Release Notes**:
- Use same format as CHANGELOG
- Link to migration guides for MAJOR releases
- Include download/install instructions
- Tag release appropriately (e.g., `release`, `bugfix`)

**3. Social Media** (optional, for major releases):
- Tweet/Mastodon announcement
- Cross-post to Rust forums/Reddit if significant

---

## 7. Hotfix Process

### 7.1 Emergency Release Criteria

Use hotfix process when:
- **Security vulnerability** in current release
- **Data loss bug** affecting users
- **Critical functionality broken** in production
- **Regression** introduced in recent release

Do NOT use for:
- Minor bugs (wait for next scheduled release)
- Feature requests (next MINOR release)
- Low-impact issues

### 7.2 Hotfix Workflow

**Step 1: Create Hotfix Branch**
```bash
# From main (must be on current release tag)
git checkout v26.9.1
git checkout -b hotfix/critical-bug-description

# Or from a local tag
git tag -l              # Find the tag
git checkout v26.9.1
git checkout -b hotfix/critical-bug
```

**Step 2: Apply Fix**
```bash
# Make minimal code changes
git add src/the_file.rs
git commit -m "fix: critical bug description"
```

**Step 3: Bump Patch Version**
```bash
# PATCH version only (26.9.1 → 26.9.1)
sed -i 's/version = "26.9.1"/version = "26.9.1"/' Cargo.toml
sed -i 's/version = "26.9.1"/version = "26.9.1"/' clap-noun-verb-macros/Cargo.toml
git add Cargo.toml clap-noun-verb-macros/Cargo.toml
git commit -m "chore(hotfix): bump to 26.9.1"
```

**Step 4: Update CHANGELOG**
```markdown
## [26.9.1] - 2026-06-15

### Fixed
- **CRITICAL**: Fixed panic in graph query when handling cycles
  (Security: prevents DoS attack via malicious RDF input)
```

**Step 5: Fast-track Validation**
```bash
# Run focused tests only
cargo test graph::tests
cargo make format-check
cargo make clippy
```

**Step 6: Publish Immediately**
```bash
cargo make publish-dry-run-macros
cargo make publish-macros
# Wait for indexing ~30 seconds
cargo make publish-dry-run
cargo make publish
```

**Step 7: Tag & Announce**
```bash
git tag -a v26.9.1 -m "Hotfix: Critical bug fix"
git push origin hotfix/critical-bug
git push origin v26.9.1

# Create GitHub release with "HOTFIX" label
# Email community immediately with CVE info if applicable
```

**Step 8: Merge Back to Main**
```bash
# After hotfix is live
git checkout main
git merge hotfix/critical-bug
git push origin main

# Delete hotfix branch
git branch -d hotfix/critical-bug
git push origin --delete hotfix/critical-bug
```

### 7.3 Hotfix SLOs

- **Detection**: Issue reported
- **Triage**: <1 hour (assess severity)
- **Fix**: <4 hours (from triage to PR)
- **Validation**: <30 minutes (testing)
- **Publish**: <1 hour (from validated to crates.io)
- **Announcement**: <2 hours (from publish to community)

**Total SLO**: <8 hours from detection to live

---

## 8. Yanking & Version Retirement

### 8.1 When to Yank a Version

**YANK** (remove from crate index, users cannot download) when:
- **Security vulnerability** with no fix in older version
- **Critical data loss bug**
- **Completely broken release** (fails to compile for users)

**DO NOT YANK** for:
- Minor bugs (publish a patch version instead)
- Feature requests
- User misunderstanding
- To "clean up" the version history

### 8.2 Yanking Procedure

**Via crates.io Web UI**:
1. Go to https://crates.io/crates/clap-noun-verb/26.9.1
2. Click "Edit metadata" (requires permissions)
3. Check "This version has been yanked"
4. Save

**Via Cargo CLI**:
```bash
cargo yank --vers 26.9.1

# Undo yank if mistake
cargo yank --vers 26.9.1 --undo
```

**Announce Yank**:
```markdown
## Version 26.9.1 Yanked

**Reason**: Critical bug in CommandRegistry causes panic on empty verb set

**Affected Users**: Anyone using v26.9.1

**Action Required**:
- Upgrade to v26.9.1 immediately
- Fix includes: Proper handling of edge cases in registry

**Why Yank?**: 
- v26.9.1 cannot start a CLI with no verbs registered
- v26.9.1 fixes the panic with graceful error handling
- This version is unusable for most users

**Migration**:
cargo update clap-noun-verb
# Updates to v26.9.1 automatically
```

### 8.3 Alternative Version Recommendations

When yanking, always recommend upgrade path:

```bash
# Query recent non-yanked versions
curl https://crates.io/api/v1/crates/clap-noun-verb/versions | \
  jq '.versions[] | select(.yanked==false) | .num' | head -5
```

**In Release Notes**:
```
If you're using v26.9.1 (YANKED):
  Upgrade to v26.9.1 (or v26.9.1 for latest features)
  cargo update clap-noun-verb
```

### 8.4 Yanking vs. Deprecation

| Action | When | Visibility | User Impact |
|--------|------|-----------|------------|
| **Deprecate** | Feature will be removed in future | Clear in CHANGELOG | Warnings in code |
| **Yank** | Version has critical bug/security issue | Marked on crates.io | Cannot be downloaded |
| **Minor Patch** | Bug fix available | New version released | Upgrade recommended |

---

## 9. SLO Targets & Compliance

### 9.1 Performance SLOs

| Target | Current | SLO | Status |
|--------|---------|-----|--------|
| Incremental Compilation | 0.66s | ≤2.0s | ✓ PASS |
| Binary Size (release) | 2.2MB | ≤10MB | ✓ PASS |
| Test Suite (parallel) | <0.5s | ≤1.0s | ✓ PASS |
| Doc Build Time | <5s | ≤10s | ✓ PASS |

### 9.2 Quality SLOs

| Target | Current | SLO | Status |
|--------|---------|-----|--------|
| Test Coverage | ~85% | ≥80% | ✓ PASS |
| Doc Coverage | ~95% | ≥90% | ✓ PASS |
| Compiler Warnings | 0 | 0 | ✓ PASS |
| Security Audit | Clean | No CVEs | ✓ PASS |

### 9.3 SLO Monitoring

**In CI Pipeline** (`Makefile.toml`):
```toml
[tasks.slo-check]
description = "Verify all SLO targets"
dependencies = ["slo-compile-time", "slo-binary-size", "slo-test-time"]

[tasks.slo-compile-time]
script = '''
echo "Checking incremental compile time..."
touch src/lib.rs
time cargo build 2>&1 | grep "real\|user" | tail -1
# Must be <2.0s
'''

[tasks.slo-binary-size]
script = '''
echo "Checking binary size..."
cargo build --release 2>&1
SIZE=$(ls -lh target/release/libclap_noun_verb.rlib | awk '{print $5}')
echo "Size: $SIZE (SLO: ≤10MB)"
'''

[tasks.slo-test-time]
script = '''
echo "Checking test execution time..."
time cargo test --quiet 2>&1 | grep "real"
# Must be <1.0s
'''
```

**Benchmark Tests** (in `benches/`):
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use clap_noun_verb::CommandRegistry;

fn bench_registry_creation(c: &mut Criterion) {
    c.bench_function("registry_new_empty", |b| {
        b.iter(|| {
            CommandRegistry::new()
        })
    });
}

criterion_group!(benches, bench_registry_creation);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo make bench
# Results in target/criterion/
```

### 9.4 Regression Response

If SLO breached:
1. **Investigate** the regression (git bisect)
2. **Revert** if unfixable in short term
3. **Fix forward** with optimization PR
4. **Document** the issue and workaround

---

## 10. CI/CD Automation

### 10.1 CI/CD Pipeline Stages

#### Stage 1: Syntax & Format (60s)
```bash
cargo make format-check
cargo make clippy
```
Fails fast if code doesn't meet standards.

#### Stage 2: Compilation (45s)
```bash
cargo make check
cargo make check-all  # all features
```
Validates code compiles against declared MSRV.

#### Stage 3: Unit Tests (90s)
```bash
cargo make test
cargo make test-lib-deterministic  # single-threaded
```
100% pass rate required.

#### Stage 4: Integration Tests (120s)
```bash
cargo make test-all     # all feature combinations
cargo make test-frontier  # frontier features
```

#### Stage 5: Security & Dependencies (30s)
```bash
cargo audit
cargo deny check
```
No known CVEs, license compliance checked.

#### Stage 6: Documentation (45s)
```bash
cargo make doc
cargo make build-examples
```
Documentation must build without warnings.

#### Stage 7: Release Validation (60s)
```bash
./scripts/pre-release-check.sh 26.9.1
```
Only on `v*` tags (before publishing).

### 10.2 GitHub Actions Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release CI

on:
  push:
    tags:
      - 'v*'

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  quality-gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-gnu
      - uses: Swatinem/rust-cache@v2

      - name: Format check
        run: cargo make format-check

      - name: Clippy
        run: cargo make clippy

      - name: Test all features
        run: cargo make test-all

      - name: Build release
        run: cargo make build-release

      - name: Build documentation
        run: cargo make doc

      - name: Run SLO checks
        run: cargo make slo-check

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Security audit
        run: cargo audit

      - name: License check
        run: cargo deny check

  publish:
    needs: [quality-gates, security]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Publish macros crate
        run: cargo publish --manifest-path clap-noun-verb-macros/Cargo.toml
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}

      - name: Wait for indexing
        run: |
          for i in {1..30}; do
            if cargo search clap-noun-verb-macros --limit 1; then
              echo "Macros indexed"
              break
            fi
            echo "Waiting... ($i/30)"
            sleep 2
          done

      - name: Publish main crate
        run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            CHANGELOG.md
            README.md
          body_path: CHANGELOG.md
          draft: false
          prerelease: ${{ contains(github.ref, '-alpha') || contains(github.ref, '-beta') || contains(github.ref, '-rc') }}
```

### 10.3 Branch Protection Rules

**Main Branch Protection** (`Settings → Branches`):

```
- Require pull request reviews before merging
- Require approval from 1+ code owners
- Require status checks to pass:
  ✓ quality-gates
  ✓ security
- Require branches to be up to date before merging
- Require code review dismissal stale PRs
- Allow auto-merge (enabled)
```

**Release Tag Automation**:
- CI automatically publishes on `v*` tags
- Only maintainers can create release tags
- Tag must pass all quality gates before publishing

---

## 11. Decision Trees & Checklists

### 11.1 Release Type Decision Tree (Interactive)

```
┌─────────────────────────────────────────────────────────┐
│  WHAT TYPE OF CHANGE ARE YOU RELEASING?                │
└─────────────────────────────────────────────────────────┘

1. Is it a breaking change (API redesign, removal of feature)?
   YES → Go to MAJOR RELEASE (Section 1.1)
   NO  → Continue to #2

2. Are you adding new features (verbs, nouns, modules)?
   YES → Go to MINOR RELEASE (Section 1.2)
   NO  → Continue to #3

3. Are you fixing bugs or improving docs?
   YES → Go to PATCH RELEASE (Section 1.3)
   NO  → Go to PRE-RELEASE (Section 1.4)

MAJOR RELEASE Checklist
├─ Version: 26.0.0 → 27.0.0 (increment MAJOR)
├─ Document: Create BREAKING_CHANGES.md
├─ Migrate: Provide code examples in CHANGELOG
├─ Announce: Email community, post on forums
├─ Timeline: 2-4 week deprecation cycle if possible
└─ Tests: Run all features, all platforms

MINOR RELEASE Checklist
├─ Version: 26.6.0 → 26.9.1 (increment MINOR, reset PATCH)
├─ Features: Document in CHANGELOG under ### Added
├─ Compat: Ensure backward compatibility
├─ Examples: Provide examples for new features
├─ Tests: Test all feature combinations
└─ Timeline: ~1 week development before release

PATCH RELEASE Checklist
├─ Version: 26.9.1 → 26.9.1 (increment PATCH)
├─ Fixes: Document in CHANGELOG under ### Fixed
├─ Tests: Regression tests for fixed bugs
├─ Timeline: Can be immediate for critical bugs
└─ SLOs: Verify performance unchanged
```

### 11.2 Pre-Release Validation Checklist

**BEFORE** running `cargo make publish-macros`:

```
Code Quality
  ☐ cargo make format-check passes
  ☐ cargo make clippy passes
  ☐ cargo make test-all passes
  ☐ cargo make doc builds without warnings
  ☐ Examples build (cargo make build-examples)

Documentation
  ☐ CHANGELOG.md has [VERSION] entry
  ☐ README.md references correct version
  ☐ New features have examples
  ☐ Migration guide (if MAJOR)
  ☐ Deprecation notices removed (if removal)

Configuration
  ☐ Cargo.toml version matches across both crates
  ☐ clap-noun-verb-macros/Cargo.toml version matches
  ☐ workspace.dependencies versions consistent
  ☐ rust-version (MSRV) is current

Repository
  ☐ git status clean (no uncommitted changes)
  ☐ All commits pushed to origin
  ☐ Main branch is stable (CI passing)
  ☐ CARGO_REGISTRY_TOKEN is set
  ☐ Credentials file has crates.io token

Security
  ☐ cargo audit passes (no CVEs)
  ☐ cargo deny check passes (licenses OK)
  ☐ Dependencies updated if needed
```

### 11.3 Post-Release Verification Checklist

**AFTER** both crates published:

```
Verification
  ☐ cargo search clap-noun-verb-macros shows 26.9.1
  ☐ cargo search clap-noun-verb shows 26.9.1
  ☐ https://docs.rs/clap-noun-verb/26.9.1/ builds
  ☐ Documentation shows all modules
  ☐ Crate size reasonable (<50MB)

GitHub
  ☐ v26.9.1 tag created
  ☐ GitHub Release has notes
  ☐ Release marked as "Latest"
  ☐ Changelog links correct

Communication
  ☐ Email sent to users (if MAJOR/MINOR)
  ☐ Documentation updated
  ☐ Example projects updated
  ☐ Release notes posted

Monitoring (next 24h)
  ☐ Check download stats
  ☐ Monitor GitHub issues for regression reports
  ☐ Verify no urgent issues filed
```

### 11.4 Hotfix Decision Tree

```
┌──────────────────────────────────┐
│  SHOULD THIS BE A HOTFIX?       │
└──────────────────────────────────┘

Does it affect production users?
  NO → Use normal release cycle
  YES → Continue

Is it security-critical?
  YES → Go to HOTFIX (Section 7)
  
Is it a data-loss bug?
  YES → Go to HOTFIX
  
Is it a complete functionality break?
  YES → Go to HOTFIX

Otherwise → Use PATCH release cycle

HOTFIX CHECKLIST
  ☐ Issue: Describe the problem
  ☐ Impact: How many users affected?
  ☐ Severity: P1 (critical), P2 (high), P3 (medium)
  ☐ Fix: Minimal code change (not refactoring)
  ☐ Test: Regression test included
  ☐ SLO: <8 hours from detection to live
```

---

## 12. Emergency Procedures

### 12.1 Major Security Vulnerability Response

**Scenario**: Critical CVE or security bug discovered in released version

**Response Timeline**:
```
T+0min: Report received
  → Escalate to maintainers
  → Assess severity/impact
  → Gather affected versions

T+15min: Triage decision
  → Is fix possible in <30min?
    YES → Fast-track to hotfix
    NO  → Prepare advisory

T+45min: Fix development
  → Write minimal fix
  → Comprehensive test case
  → No refactoring/improvements

T+75min: Validation
  → Tests pass 100%
  → Manual testing
  → Run SLO checks

T+90min: Publish
  → cargo make publish
  → Verify on crates.io
  → Create security advisory

T+2hr: Announce
  → Email users
  → GitHub Security Advisory
  → Recommend upgrade urgently
```

### 12.2 Critical Bug Response (Data Loss)

**Scenario**: Bug that corrupts data or causes data loss

**Immediate Actions**:
1. **Yank affected version** immediately (don't wait for fix)
2. **Determine workaround** for affected users
3. **Publish patch** with fix
4. **Post-incident review** (what allowed this through?)

**Yank Command**:
```bash
cargo yank --vers 26.9.1
# Notifies: Cannot download this version anymore
```

**Patch Release** (same day):
```bash
# Bump to 26.9.1 with fix
./scripts/release-automation.sh 26.9.1
```

**User Communication**:
```
URGENT: Version 26.9.1 Yanked

A critical bug in clap-noun-verb v26.9.1 can cause data loss.

IMMEDIATE ACTION REQUIRED:
1. Do NOT use v26.9.1
2. Upgrade to v26.9.1 immediately
3. Verify your data integrity

Affected: Only users running v26.9.1
Safe versions: v26.9.1-v26.9.1, v26.9.1+

For support: File issue on GitHub
```

### 12.3 Publish Failure Recovery

**Scenario**: Publishing to crates.io fails mid-way

**If Macros Published, Main Failed**:
```bash
# Macros is already on crates.io
# Retry main publish
cargo make publish

# If it still fails, investigate:
# - Token expired? Get new token from crates.io
# - Metadata invalid? Run 'cargo make publish-dry-run'
# - Network issue? Wait 5min and retry
```

**If Both Failed**:
```bash
# Rollback changes
git reset --hard HEAD~1

# Start over after fixing issue
./scripts/release-automation.sh <version>
```

**If Only Dry-run Failed**:
```bash
# Fix issues before attempting publish
cargo make publish-dry-run
# Fix reported issues (missing CHANGELOG, etc)
cargo make publish-dry-run  # verify fix
cargo make publish
```

### 12.4 Rollback Procedure

**If Release is Broken**:

```bash
# Step 1: Identify broken version
VERSION="26.9.1"

# Step 2: Yank the broken version
cargo yank --vers $VERSION

# Step 3: Announce rollback
# Email: "v26.9.1 yanked due to [issue]"
# Recommend: "Use v26.9.1 until fix is published"

# Step 4: Fix the issue
git revert <broken-commit>
# or create new fix commit

# Step 5: Bump version and re-release
./scripts/release-automation.sh 26.9.1
```

**Communication Template**:
```
We've yanked v26.9.1 due to [issue].

If you're using v26.9.1:
1. Downgrade: cargo update clap-noun-verb --precise 26.9.1
2. Monitor: We're releasing v26.9.1 with the fix

If you're using v26.9.1:
No action needed; you're safe.

We apologize for the inconvenience.
```

### 12.5 Emergency Communication

**Channels** (in priority order):
1. GitHub Release / Yanked version warning
2. GitHub Security Advisory (if applicable)
3. Crates.io Yanked warning
4. Email to registered maintainer contacts
5. Project issue tracker (pin issue)
6. Social media (for critical security)

**Template**:
```
Subject: [URGENT] clap-noun-verb Security Fix / Data Loss Issue

Affected Versions: v26.9.1
Safe Versions: v26.9.1, v26.9.1+
Severity: CRITICAL [Security | Data Loss]

The issue:
[1-sentence description]

Impact:
- Affects: [specific users / all users]
- Risk: [potential damage]

Action:
1. Upgrade to v26.9.1 (recommended)
2. Or downgrade to v26.9.1 (temporary)

Questions? File issue: [link]
```

---

## Appendix A: Version History Reference

| Version | Release Date | Type | Major Changes |
|---------|--------------|------|----------------|
| 26.9.1 | 2026-06-14 | PATCH | Graph queries, capability packing |
| 26.9.1 | 2026-06-13 | PATCH | Eliminated stubs, fixed doctests |
| 26.9.1 | 2026-06-01 | MINOR | Graph module, diagnostics |
| 26.0.0 | 2025-11-15 | MAJOR | Minimalist refactor |
| 5.5.0 | 2026-01-15 | MINOR | Agent CLI builder |
| 5.4.0 | 2026-01-01 | MINOR | Frontier features, ggen |
| 5.0.0 | 2025-11-01 | MAJOR | Telemetry refactor |

---

## Appendix B: Useful Commands

```bash
# View current version
grep '^version' Cargo.toml

# Bump versions
sed -i 's/version = "26.9.1"/version = "26.9.1"/' Cargo.toml
sed -i 's/version = "26.9.1"/version = "26.9.1"/' clap-noun-verb-macros/Cargo.toml

# Pre-release checks
./scripts/pre-release-check.sh 26.9.1

# Dry-run publishing
cargo make publish-dry-run-macros
cargo make publish-dry-run

# Real publishing
cargo make publish-macros
cargo make publish

# Verify published
cargo search clap-noun-verb --limit 1

# Create tag
git tag -a v26.9.1 -m "Release v26.9.1"

# Push tag
git push origin v26.9.1

# Yank a version
cargo yank --vers 26.9.1

# Check for CVEs
cargo audit

# Check license compliance
cargo deny check

# Run full CI
cargo make ci

# Benchmark
cargo make bench
```

---

## Appendix C: Release Responsibilities

### Release Manager Role
- Owns the release process
- Runs scripts and validates outputs
- Communicates with community
- Handles emergency rollbacks

### Code Reviewer Role (pre-release)
- Approves all changes in PR
- Verifies tests pass
- Checks documentation completeness

### Security Reviewer Role (security releases)
- Assesses CVE severity
- Verifies fix adequacy
- Recommends upgrade path

### Community Manager Role
- Announces release
- Gathers feedback
- Files follow-up issues

---

## Appendix D: Glossary

- **SLO**: Service Level Objective (performance target)
- **CVE**: Common Vulnerabilities and Exposures (security issue ID)
- **MSRV**: Minimum Supported Rust Version
- **Pre-release**: Alpha/Beta/RC version (not yet stable)
- **Yanked**: Version marked as broken, cannot be downloaded
- **Hotfix**: Emergency release outside normal cycle
- **Dry-run**: Test publish without actually uploading

---

**For questions or process improvements**: Create an issue at https://github.com/seanchatmangpt/clap-noun-verb/issues with label `release-process`

**Last updated**: 2026-08-20  
**Next review**: 2026-11-20 (quarterly)
