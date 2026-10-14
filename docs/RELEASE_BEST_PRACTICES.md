# Release Best Practices & Patterns

**clap-noun-verb** Release Excellence Guide

---

## Pre-Release Rituals

### 48 Hours Before Release

**Goal**: Prepare the release without rushing

```bash
# 1. Create a release branch for final polishing
git checkout -b release/v26.9.1
git pull origin main

# 2. List all changes since last release
git log v26.9.1..HEAD --oneline --graph

# 3. Draft CHANGELOG.md entries
# Group commits by category and write user-facing descriptions

# 4. Review each commit for quality
# Look for: stubs, TODOs, unwrap(), panic(), print!()
git log v26.9.1..HEAD -p | less

# 5. Create migration guide (if MAJOR version)
# Document breaking changes with before/after examples
cat > docs/MIGRATION_V26_TO_V27.md << 'EOF'
# Migration Guide: v26 → v27

## Breaking Changes
...
EOF
```

### 24 Hours Before Release

**Goal**: Final validation and dry-runs

```bash
# 1. Final quality check
./scripts/pre-release-check.sh 26.9.1

# 2. Dry-run publishing
cargo make publish-dry-run-macros
cargo make publish-dry-run

# 3. Verify documentation build
cargo doc --all-features --no-deps

# 4. Final CHANGELOG review
# Ensure all entries are:
# - User-facing (not implementation details)
# - Specific and descriptive
# - Properly categorized
vim CHANGELOG.md

# 5. Get team approval (if MAJOR)
# Tag stakeholders for final review
```

### Release Day Morning

**Goal**: Execute release with focus

```bash
# 1. Ensure fresh state
git status
git log origin/main..HEAD  # Should be empty or clean

# 2. Use automated helper for guided workflow
./scripts/release-automation.sh 26.9.1

# Or manual step-by-step if preferred
```

---

## Semantic Versioning Deep Dive

### MAJOR Version Rules

**Increment MAJOR when**:
1. Removing previously stable, documented traits/functions
2. Changing trait method signatures (return types, parameters)
3. Changing public type definitions (struct fields, enum variants)
4. Breaking output format (JSON schema changes)
5. Changing behavior in incompatible way

**Example MAJOR Releases**:
- v5.0.0 (Nov 2025): Telemetry API redesign
- v4.0.0 (Nov 2025): Autonomic layer introduction

**MAJOR Release Checklist**:
```
□ Deprecation period completed (warn → remove cycle)
□ Migration guide written with code examples
□ CHANGELOG has "### Breaking Changes" section
□ README updated with migration notice
□ Release notes clearly state breaking changes
□ All old APIs have proper #[deprecated] markers
```

### MINOR Version Rules

**Increment MINOR when**:
1. Adding new macro attributes: `#[arg(new_feature)]`
2. Adding new modules: `pub mod new_feature { ... }`
3. Adding new functions/traits (additive only)
4. Adding output formats: `--format toml`
5. Adding frontier features: `frontier-new-feature`

**Example MINOR Releases**:
- v26.9.1 (June 2026): Graph module, capability packing
- v5.5.0 (Jan 2026): Agent CLI Builder
- v5.4.0 (Jan 2026): ggen integration

**MINOR Release Checklist**:
```
□ All changes are backward compatible
□ No trait methods removed
□ No type signatures changed
□ No behavior changes (only additions)
□ New features are well-documented
□ Examples demonstrate new capabilities
```

### PATCH Version Rules

**Increment PATCH when**:
1. Fixing bugs (observable behavioral changes)
2. Improving error messages
3. Fixing panic/unwrap issues
4. Documentation improvements
5. Test infrastructure enhancements
6. Adding new trait implementations

**Example PATCH Releases**:
- v26.9.1 → v26.9.1: Critical bug fix
- v26.9.1 → v26.9.1: Documentation refresh + minor fixes

**PATCH Release Checklist**:
```
□ Only bug fixes and documentation changes
□ No new features added
□ No API changes
□ All tests pass
□ Minimal release time (high frequency OK)
```

---

## Changelog Excellence

### Structure Template

```markdown
## [26.9.1] - 2026-06-15

### Added
- **New Feature**: Detailed description with context
  - Rationale: Why this feature matters
  - Example: `myapp new-feature --option value`
- **Another Feature**: Description
  - Example usage

### Changed
- **Enhancement**: What changed and why
  - Before: Old behavior
  - After: New behavior
  - Impact: How it affects users (positive)
- **Another Enhancement**: Description

### Deprecated
- `old_api()` - Use `new_api()` instead (will be removed in v27.0)
- `OldTrait` - Implement `NewTrait` instead (deprecates in v26.8)

### Removed
- Removed `very_old_function()` (deprecated since v26.0)

### Fixed
- Fixed panic in `CommandRegistry::run()` when handling empty args (#456)
- Fixed memory leak in distributed slice traversal (#457)
- Fixed error message color codes on Windows

### Security
- **CVE-2026-12345**: Fixed potential DoS with malformed input
  - Impact: High (could cause application crash)
  - Affected versions: v26.9.1 and earlier
  - Mitigation: Update to v26.9.1+

### Technical Details
- Updated to Rust 1.74 MSRV
- Refactored linkme integration for better performance
- Updated dependencies: `syn 2.1`, `quote 1.0`
- Note: Does not affect users, internal implementation improvements

### Migration Guide

**No breaking changes** — All existing code continues to work without modification.

**To use new features**:
```rust
// New feature example
#[verb]
fn my_command(input: HandlerInput) -> Result<Output> {
    let result = new_feature::do_something()?;
    Ok(Output::Success(result))
}
```
```

### Common Entry Mistakes

**❌ Vague**:
```markdown
### Fixed
- Fixed bug
- Various improvements
- Code cleanup
```

**✅ Specific**:
```markdown
### Fixed
- Fixed panic in `registry::run()` when verb handler returns empty string
- Fixed incorrect error message in argument validation (issue #456)
```

**❌ Implementation Details**:
```markdown
### Technical Details
- Rewrote CommandRegistry to use HashMap instead of Vec
- Changed internal caching mechanism
```

**✅ User-Focused**:
```markdown
### Changed
- Improved performance of large CLI registries (3x faster dispatch)

### Technical Details
- Refactored CommandRegistry to use HashMap for O(1) lookup
- Updated caching mechanism for reduced memory usage
```

---

## Publishing Excellence

### Pre-Publish Checklist

**72 hours before**:
```bash
# Run full quality suite
cargo make ci                    # All CI checks
cargo make verify-frontier       # Frontier features
cargo make security-scan         # Security audit
cargo make coverage-report       # Code coverage

# Verify MSRV
rustup install 1.74
rustup default 1.74
cargo build --all-features
rustup default stable
```

**24 hours before**:
```bash
# Dry-run everything
cargo make publish-dry-run-macros
cargo make publish-dry-run
cargo make release-check

# Final docs check
cargo doc --all-features --no-deps
# Manually verify docs.rs metadata
```

**1 hour before** (if using manual process):
```bash
# Final state check
git status                # Clean?
git log origin/main..HEAD # Empty?

# Fresh test run
cargo make test-all       # All features
```

### Publishing Sequence

**Critical**: Must publish in correct order

```
1. ✓ Pre-release checks pass
2. ✓ Cargo.toml versions match (26.9.1)
3. ✓ Dry-run macros publish
4. ✓ Publish macros to crates.io
5. ⏳ Wait for macros indexing (2-60 seconds)
6. ✓ Dry-run main publish
7. ✓ Publish main to crates.io
8. ⏳ Wait for main indexing (2-60 seconds)
9. ✓ Verify both on crates.io
10. ✓ Create git tag
11. ✓ Push tag (triggers GitHub Actions)
12. ⏳ Monitor GitHub Actions
```

**Time breakdown**:
- Steps 1-7: ~5 minutes
- Steps 8-9: ~2 minutes waiting
- Steps 10-11: ~1 minute
- Step 12: ~3 minutes automated
- **Total**: ~11 minutes hands-on

### Publishing Failure Recovery

**If macros publish fails**:
```bash
# Check error message
cargo make publish-macros

# Common fixes:
# 1. Version already published?
#    → Increment version and retry
cargo make publish-dry-run-macros

# 2. Dependency missing?
#    → Wait for dependency to index, then retry
sleep 30
cargo make publish-macros

# 3. Token invalid?
#    → Verify CARGO_REGISTRY_TOKEN is set
echo $CARGO_REGISTRY_TOKEN
```

**If main publish fails**:
```bash
# Check if macros are indexed
cargo search clap-noun-verb-macros --limit 1

# If not indexed:
# → Wait longer and retry
sleep 30
cargo make publish-dry-run
cargo make publish

# If macros are indexed but main fails:
# → Check error message and fix
cargo make publish-dry-run  # Get detailed error
```

**If something published wrong**:
```bash
# 1. Immediately yank the broken version
cargo yank --vers 26.9.1 -p clap-noun-verb
cargo yank --vers 26.9.1 -p clap-noun-verb-macros

# 2. Fix the issue
# ... make changes ...

# 3. Bump version (don't reuse 26.9.1)
./scripts/bump-version.sh 26.9.1

# 4. Republish
cargo make publish
```

---

## Breaking Changes Management

### Deprecation Lifecycle

**Recommended pattern for non-breaking removal**:

```
Release v26.6: Add new API
┣ #[deprecated] markers added
┣ Migration guide written
┣ CHANGELOG mentions deprecation

Release v26.7-26.8: Stabilization
┣ Keep old API working
┣ Warnings printed when used
┣ Users have time to migrate

Release v27.0: Major version
┣ Remove deprecated APIs
┣ Update CHANGELOG with removal notice
┣ Provide migration guide in release notes
```

**Example deprecation marker**:
```rust
#[deprecated(
    since = "26.8",
    note = "use TelemetryManager::instance() instead"
)]
pub fn create_span(name: &str) -> Span {
    TelemetryManager::instance().create_span(name)
}
```

### Breaking Change Announcement

**CHANGELOG entry**:
```markdown
## [27.0.0] - 2026-07-15

### ⚠️ Breaking Changes

**This is a MAJOR version with breaking changes.**
See [Migration Guide](docs/MIGRATION_V26_TO_V27.md) for upgrade instructions.

#### Removed: Telemetry Module API
- Removed `telemetry::create_span()` - Use `TelemetryManager::instance().create_span()` instead
- Removed `telemetry::set_context()` - Use `TelemetryManager::instance().set_context()` instead
- Impact: Direct telemetry API calls need updating

#### Removed: CommandRegistry Methods
- Removed `CommandRegistry::register_verb_directly()` (deprecated since v26.8)
- Use `#[verb]` macro instead

#### Changed: HandlerInput Type
- `HandlerInput::args` changed from `Vec<String>` to `ParsedArgs`
- Access positional args via `input.parsed.positional(0)?` instead of `input.args.get(0)?`
- Rationale: Type-safe argument access, prevents panics

### Migration Guide

See [complete migration guide](docs/MIGRATION_V26_TO_V27.md) with code examples.
```

**GitHub Release notes**:
```markdown
# v27.0.0: Breaking Changes Release

⚠️ **This is a MAJOR version with breaking changes.**

## What's Changing

1. **Telemetry API** - Direct function calls → Manager pattern
2. **HandlerInput** - Vec<String> args → Type-safe ParsedArgs
3. **CommandRegistry** - Direct registration → Macro-only registration

## Required Action

1. Read [Migration Guide](https://github.com/seanchatmangpt/clap-noun-verb/blob/main/docs/MIGRATION_V26_TO_V27.md)
2. Update your code following the examples
3. Test thoroughly
4. Update your Cargo.toml: `clap-noun-verb = "27.0"`

## Why These Changes

These changes make the CLI framework more type-safe and less error-prone.
The new APIs are better integrated with Rust's type system and prevent common mistakes.

## Support

Questions? Open a discussion: https://github.com/seanchatmangpt/clap-noun-verb/discussions
```

---

## Performance & Quality SLOs

### Compile Time

**Target**: Incremental compilation ≤ 2 seconds

```bash
# Measure current performance
time cargo build

# Should complete in < 2 seconds for incremental changes
```

### Binary Size

**Target**: Release binary ≤ 10 MB

```bash
# Check current size
ls -lh target/release/clap-noun-verb-gen

# Should be < 10 MB
```

### Test Suite

**Target**: All tests ≤ 1 second total

```bash
# Run and time
time cargo test --quiet

# Should complete in < 1 second
```

### Documentation

**Target**: All docs build without warnings

```bash
# Check
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

# Should complete without any warning
```

---

## Release Communication

### For PATCH Releases

**Minimal announcement** (bug fixes are common):

```markdown
v26.9.1 released - bug fixes

- Fixed panic in registry handling (#456)
- Improved error messages

Update with: `cargo update`
```

### For MINOR Releases

**Feature announcement** (highlight new capabilities):

```markdown
v26.9.1 released - new features

## What's New
- New `#[arg(validate)]` macro for custom validation
- Graph module enhancements (3x faster queries)
- Support for YAML output format

## Migration
No breaking changes - existing code works as-is.
New features are optional and backward compatible.

Update with: `cargo update`
```

### For MAJOR Releases

**Comprehensive announcement** (breaking changes require attention):

```markdown
v27.0.0 released - breaking changes

## ⚠️ Action Required
This release has breaking changes. All users must update their code.
See the [migration guide](link) for step-by-step instructions.

## What Changed
- Telemetry API (direct functions → manager pattern)
- HandlerInput type (Vec<String> → type-safe ParsedArgs)
- CommandRegistry registration (direct methods → macros only)

## Why?
These changes improve type safety and prevent common mistakes.

## Help
- [Migration Guide](link)
- [GitHub Discussions](link)
- [Discord](link)
```

---

## Post-Release

### Verification Checklist (Next Day)

```bash
# 1. Verify crates.io
curl -s https://crates.io/api/v1/crates/clap-noun-verb | jq '.crate.max_version'
# Should output: "26.9.1"

# 2. Verify docs.rs
# Visit: https://docs.rs/clap-noun-verb/26.9.1/
# Should load without errors

# 3. Check GitHub Release
# Visit: https://github.com/seanchatmangpt/clap-noun-verb/releases/tag/v26.9.1
# Should have auto-generated release notes

# 4. Monitor Issues
# Check: GitHub Issues for any problems reported
```

### User Communication

```bash
# 1. Answer migration questions (if MAJOR)
# Monitor GitHub Discussions and Issues

# 2. Update external references
# - Update company blog
# - Update third-party package lists
# - Announce on Twitter/social media (if major feature)

# 3. Plan next release
# - Create milestone for next version
# - Label issues for next release
# - Add to project roadmap
```

---

## Release Retrospective

After each release (especially MAJOR), conduct a retrospective:

```markdown
# v26.9.1 Release Retrospective

## What Went Well
- All quality gates passed on first try
- Documentation was comprehensive
- Community feedback was positive

## What Could Improve
- CHANGELOG took longer to prepare than expected
- Should start drafting earlier

## Action Items
- [ ] Create CHANGELOG template
- [ ] Start documenting changes as they're made
- [ ] Schedule release planning meeting earlier

## Metrics
- Release time: 11 minutes
- Time to index on crates.io: 45 seconds
- User upgrades in first 24h: 234
```

---

## References

- **This Guide**: `docs/RELEASE_BEST_PRACTICES.md`
- **Release Skills**: `docs/RELEASE_SKILLS.md`
- **Full Management Guide**: `docs/RELEASE_MANAGEMENT.md`
- **Automation Helper**: `scripts/release-automation.sh`
- **Pre-Release Check**: `scripts/pre-release-check.sh`

**Document Version**: 1.0  
**Last Updated**: 2026-08-20  
**Authored by**: Sean Chatman
