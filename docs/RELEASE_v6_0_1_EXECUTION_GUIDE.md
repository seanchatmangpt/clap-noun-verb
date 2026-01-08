# v6.0.1 Release - Quick Execution Guide

**Purpose**: Step-by-step instructions for executing v6.0.1 patch release
**Status**: Ready for execution when bug fixes complete
**Estimated Duration**: 30-45 minutes

---

## Pre-Flight Checklist

Before executing release, verify all prerequisites:

```
[ ] All bug fix agents have completed their work
[ ] Working tree is clean (git status shows nothing to commit)
[ ] On branch: claude/launch-agents-production-release-q0r6w
[ ] Latest main branch code available
[ ] Crates.io credentials configured (.cargo/credentials)
[ ] GitHub credentials configured (for tag push)
[ ] Verified: No breaking changes in code
[ ] Verified: All tests pass locally
```

---

## Quick Version Reference

| File | Line | Current | Target |
|------|------|---------|--------|
| `/Cargo.toml` | 7 | 5.5.0 | 6.0.1 |
| `/Cargo.toml` | 153 | 5.4.0 | 6.0.1 |
| `/clap-noun-verb-macros/Cargo.toml` | 3 | 5.5.0 | 6.0.1 |

---

## Step 1: Validation (5 minutes)

Run comprehensive validation to ensure release readiness:

```bash
# Quick check
cd /home/user/clap-noun-verb
cargo make check

# Full test suite
cargo make test

# Linting
cargo make lint

# Performance validation
cargo make slo-check

# Release validation
cargo make release-validate
```

**Success Criteria**: All commands pass with zero errors/warnings

If any failures detected:
- STOP - Do not proceed
- Root cause analysis required
- Fix issues, re-run validation
- Only proceed when all pass

---

## Step 2: Update Versions (5 minutes)

### Edit 1: Main Cargo.toml - Primary Version
```bash
# File: /home/user/clap-noun-verb/Cargo.toml
# Line 7
OLD: version = "5.5.0"
NEW: version = "6.0.1"
```

### Edit 2: Main Cargo.toml - Macros Dependency
```bash
# File: /home/user/clap-noun-verb/Cargo.toml
# Line 153
OLD: clap-noun-verb-macros = { version = "5.4.0", path = "clap-noun-verb-macros" }
NEW: clap-noun-verb-macros = { version = "6.0.1", path = "clap-noun-verb-macros" }
```

### Edit 3: Macros Cargo.toml - Macros Version
```bash
# File: /home/user/clap-noun-verb/clap-noun-verb-macros/Cargo.toml
# Line 3
OLD: version = "5.5.0"
NEW: version = "6.0.1"
```

### Edit 4: Update Cargo.lock
```bash
cargo update -p clap-noun-verb -p clap-noun-verb-macros
```

---

## Step 3: Update CHANGELOG (5 minutes)

File: `/home/user/clap-noun-verb/CHANGELOG.md`

**Location**: Right after line 8 (after v6.0.0 section starts)

**Add**:
```markdown
## [6.0.1] - 2026-01-08

### Fixed
- Fixed macros dependency version inconsistency (pinned to 5.4.0, now 6.0.1)
- [List actual bug fixes from agents]

### Changed
- Synchronized all version numbers to 6.0.1

### Security
- [List security updates if any]
```

---

## Step 4: Verify Changes (2 minutes)

```bash
# Verify file changes
git status

# Show diffs
git diff /home/user/clap-noun-verb/Cargo.toml
git diff /home/user/clap-noun-verb/clap-noun-verb-macros/Cargo.toml
git diff /home/user/clap-noun-verb/CHANGELOG.md

# Verify Cargo.lock was updated
git status Cargo.lock
```

**Expected**: All 4 files modified (2 Cargo.toml + 1 macros Cargo.toml + CHANGELOG + Cargo.lock)

---

## Step 5: Create Atomic Commit (2 minutes)

```bash
# Stage all changes
git add /home/user/clap-noun-verb/Cargo.toml
git add /home/user/clap-noun-verb/clap-noun-verb-macros/Cargo.toml
git add /home/user/clap-noun-verb/CHANGELOG.md
git add /home/user/clap-noun-verb/Cargo.lock

# Create commit with detailed message
git commit -m "release: Bump v6.0.1 - patch release with bug fixes and dependency sync

- Update main crate: 5.5.0 → 6.0.1
- Update macros crate: 5.5.0 → 6.0.1
- Fix macros dependency: 5.4.0 → 6.0.1 (was inconsistent)
- Update CHANGELOG with v6.0.1 release notes
- Regenerate Cargo.lock

This patch release addresses bug fixes and fixes the version
inconsistency in macros dependency (was pinned to 5.4.0 while
macros crate was 5.5.0). No breaking changes, fully backward compatible.

Type: Patch Release
Breaking Changes: None
Backward Compatible: Yes"

# Verify commit
git log -1 --stat
```

---

## Step 6: Pre-Publish Validation (5 minutes)

```bash
# Verify versions updated correctly
cargo --version
cargo metadata --format-version 1 | grep '"version"' | head -5

# Verify cargo can resolve dependencies
cargo build --lib

# Run final test suite
cargo make test

# Verify lock file
cargo update --dry-run
```

**Success**: No errors, all tests pass

---

## Step 7: Publish Main Crate (5 minutes)

```bash
# Publish clap-noun-verb@6.0.1
cargo publish -p clap-noun-verb

# Wait for crates.io to process (typically 1-2 minutes)
sleep 60

# Verify on crates.io
curl https://crates.io/api/v1/crates/clap-noun-verb | jq '.crate.max_version'
# Expected: "6.0.1"
```

**Success**: Crates.io shows clap-noun-verb 6.0.1

---

## Step 8: Publish Macros Crate (5 minutes)

```bash
# Publish clap-noun-verb-macros@6.0.1
cargo publish -p clap-noun-verb-macros

# Wait for crates.io to process
sleep 60

# Verify on crates.io
curl https://crates.io/api/v1/crates/clap-noun-verb-macros | jq '.crate.max_version'
# Expected: "6.0.1"
```

**Success**: Crates.io shows clap-noun-verb-macros 6.0.1

---

## Step 9: Create Git Tag (3 minutes)

```bash
# Create annotated tag
git tag -a v6.0.1 -m "Release v6.0.1 - Patch release with bug fixes and version sync

This patch release (6.0.1) follows the major v6.0.0 release and
addresses critical bug fixes without introducing breaking changes.

Key Changes:
- Bug fixes from v6.0.0 work
- Fixed version inconsistency in macros dependency
- All changes backward compatible
- Fully tested against SLOs

Published:
- clap-noun-verb@6.0.1 on crates.io
- clap-noun-verb-macros@6.0.1 on crates.io"

# Push tag to GitHub
git push origin v6.0.1

# Verify tag
git tag -l v6.0.1 -n
```

**Success**: Git tag appears in github repo

---

## Step 10: Create GitHub Release (2 minutes)

```bash
# Create GitHub release with release notes
gh release create v6.0.1 \
  --title "v6.0.1 - Patch Release" \
  --notes "## v6.0.1 Patch Release

This is a patch release following v6.0.0 major release.

### Fixed
- Fixed macros dependency version inconsistency (5.4.0 → 6.0.1)
- [List actual bug fixes from agents]

### Changed
- Synchronized all version numbers to 6.0.1

### Type
- Patch Release (SemVer: 6.0.0 → 6.0.1)
- Breaking Changes: None
- Backward Compatible: Yes

### Publication Status
- clap-noun-verb@6.0.1 ✓ Published to crates.io
- clap-noun-verb-macros@6.0.1 ✓ Published to crates.io

### Downloads
- [crates.io clap-noun-verb](https://crates.io/crates/clap-noun-verb)
- [crates.io macros](https://crates.io/crates/clap-noun-verb-macros)"

# Verify release
gh release view v6.0.1
```

**Success**: GitHub release published with release notes

---

## Step 11: Post-Release Verification (3 minutes)

```bash
# Verify both crates are published and discoverable
cargo search clap-noun-verb --limit 1

# Check documentation on docs.rs (may take a few minutes)
# https://docs.rs/clap-noun-verb/6.0.1

# Verify git tag in repository
git ls-remote origin 'refs/tags/v6.0.1'

# Show GitHub release
gh release view v6.0.1 --web
```

**Success**: Everything published and visible

---

## Step 12: Announcement (2 minutes)

```bash
# Option 1: GitHub Discussions
gh api repos/:owner/:repo/discussions \
  --input - << 'EOF'
{
  "title": "v6.0.1 Released",
  "body": "v6.0.1 patch release is now available on crates.io. This release addresses bug fixes and fixes version inconsistencies.",
  "category_id": "announcements"
}
EOF

# Option 2: Direct message to users
# Post in GitHub issues/discussions:
# "v6.0.1 is now available! Install with: cargo add clap-noun-verb@6.0.1"

# Option 3: Twitter/Social (if applicable)
# "Just released v6.0.1 on crates.io! Bug fixes and dependency sync."
```

---

## Troubleshooting

### If Cargo Publish Fails

```bash
# Clear local cargo cache
rm -rf ~/.cargo/registry/cache

# Verify credentials
cat ~/.cargo/credentials

# Retry publish
cargo publish -p clap-noun-verb --token <token>
```

### If Tests Fail During Release

```bash
# STOP - Do not proceed
cargo make test  # Identify failures
# Fix issues
git add <fixed-files>
git commit --amend  # Amend previous commit
# Re-run tests
cargo make test
# Verify commit is still one commit
git log -1
```

### If Git Tag Push Fails

```bash
# Verify GitHub credentials
gh auth status

# Retry push
git push origin v6.0.1

# Or create release directly
gh release create v6.0.1 --notes "See CHANGELOG"
```

### If Need to Rollback

```bash
# BEFORE publish:
git reset HEAD~1
git reset --hard HEAD

# AFTER publish:
cargo yank --vers 6.0.1
git tag -d v6.0.1
git push origin :v6.0.1
```

---

## Success Checklist

After completing all steps, verify:

```
[ ] All version numbers updated to 6.0.1
[ ] Cargo.toml shows 6.0.1
[ ] clap-noun-verb-macros/Cargo.toml shows 6.0.1
[ ] CHANGELOG.md updated with v6.0.1 section
[ ] Cargo.lock regenerated
[ ] All tests passing
[ ] Git commit created (atomic)
[ ] clap-noun-verb@6.0.1 published to crates.io
[ ] clap-noun-verb-macros@6.0.1 published to crates.io
[ ] Git tag v6.0.1 created and pushed
[ ] GitHub release created with notes
[ ] Announcement posted (optional)
```

---

## Time Estimates

| Phase | Duration | Cumulative |
|-------|----------|-----------|
| Validation | 5 min | 5 min |
| Version Updates | 5 min | 10 min |
| CHANGELOG Update | 5 min | 15 min |
| Verify Changes | 2 min | 17 min |
| Create Commit | 2 min | 19 min |
| Pre-Publish Test | 5 min | 24 min |
| Publish Main | 5 min | 29 min |
| Publish Macros | 5 min | 34 min |
| Create Tag | 3 min | 37 min |
| Create Release | 2 min | 39 min |
| Post-Release Verify | 3 min | 42 min |
| Announcement | 2 min | 44 min |

**Total**: ~45 minutes

---

## Emergency Contacts

If critical issues occur:
1. Stop the line immediately
2. Do NOT publish further if major issue found
3. Rollback using procedures above
4. Create incident report
5. Analyze root cause
6. Plan v6.0.2 if critical

---

## Document References

- **Comprehensive Plan**: `/home/user/clap-noun-verb/docs/RELEASE_v6_0_1_PLAN.md`
- **Manager Summary**: `/home/user/clap-noun-verb/docs/RELEASE_v6_0_1_MANAGER_SUMMARY.md`
- **Changelog**: `/home/user/clap-noun-verb/CHANGELOG.md`
- **Production Checklist**: `/home/user/clap-noun-verb/docs/PRODUCTION_RELEASE_CHECKLIST.md`

---

**Ready for Execution**: v6.0.1 release package complete
**Blocked On**: Bug fix agents completing work
**Time to Execute**: ~45 minutes once agents finish
