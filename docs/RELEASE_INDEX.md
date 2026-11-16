# Release Management Index

**Complete Release Management Framework for clap-noun-verb**

---

## Overview

This directory contains a comprehensive release management system for clap-noun-verb, covering semantic versioning, changelog maintenance, publishing workflows, and emergency procedures.

**Key Assets**:
- 📚 **4 documentation guides** (1,200+ lines of guidance)
- 🚀 **Automated release scripts** (pre-release checks, version bumping, guided workflow)
- 📋 **Quick-reference card** (one-page cheat sheet)
- ✅ **7 automated quality gates** (version, compilation, tests, warnings, docs, examples, git)

---

## Documents

### 1. **RELEASE_SKILLS.md** (Comprehensive Guide)
**Best for**: Learning the complete release process

- ✅ Quick start (5-minute release for experienced operators)
- ✅ Semantic versioning decision tree
- ✅ Release workflow phases (planning, prep, publishing, post-release)
- ✅ Version bumping with automation script
- ✅ Changelog management (Keep a Changelog format)
- ✅ Publishing process (dual-crate model)
- ✅ Crates.io metadata
- ✅ GitHub release creation
- ✅ Breaking change communication
- ✅ Yanking & emergency procedures
- ✅ Automation scripts overview
- ✅ Release checklist

**Read this first** to understand the complete framework.

**File**: `/docs/RELEASE_SKILLS.md` (29 KB)

---

### 2. **RELEASE_MANAGEMENT.md** (Existing Detailed Guide)
**Best for**: Reference during release day

- ✅ Pre-release checklist
- ✅ Semantic versioning strategy with rationale
- ✅ Version bumping workflow
- ✅ Changelog management with tools
- ✅ Publishing workflow (step-by-step)
- ✅ Crates.io metadata optimization
- ✅ GitHub release creation (automatic & manual)
- ✅ Breaking change communication
- ✅ Yanking strategies for bad releases
- ✅ Release automation & GitHub Actions
- ✅ Emergency release procedures
- ✅ Comprehensive release checklist
- ✅ Quick summary process

**Reference this** during actual releases for detailed procedures.

**File**: `/docs/RELEASE_MANAGEMENT.md` (32 KB)

---

### 3. **RELEASE_BEST_PRACTICES.md** (Excellence Patterns)
**Best for**: Improving release quality & predictability

- ✅ Pre-release rituals (48h, 24h, day-of)
- ✅ Semantic versioning deep dive with real examples
- ✅ Changelog excellence patterns
- ✅ Publishing excellence checklist
- ✅ Breaking change management (deprecation lifecycle)
- ✅ Performance & quality SLOs
- ✅ Release communication templates (PATCH/MINOR/MAJOR)
- ✅ Post-release verification
- ✅ Release retrospective template

**Use this** to level up release quality and consistency.

**File**: `/docs/RELEASE_BEST_PRACTICES.md` (16 KB)

---

### 4. **RELEASE_QUICK_REFERENCE.md** (One-Page Cheat Sheet)
**Best for**: Quick lookup during release

- ✅ Version bump decision table
- ✅ Release workflow (11 minutes)
- ✅ 7 quality gates checklist
- ✅ Publishing sequence (critical order)
- ✅ CHANGELOG template
- ✅ MSRV verification
- ✅ Breaking changes (MAJOR only)
- ✅ Troubleshooting table
- ✅ Post-release verification
- ✅ Emergency: yank bad release
- ✅ Environment setup
- ✅ Documentation links
- ✅ Time estimates
- ✅ Success criteria

**Print this** or keep in a terminal window during release.

**File**: `/docs/RELEASE_QUICK_REFERENCE.md` (4.8 KB)

---

## Scripts

### 1. **release-automation.sh** (Guided Workflow)
**Best for**: Your first few releases and stress-free releases

```bash
./scripts/release-automation.sh           # Interactive mode
./scripts/release-automation.sh 26.9.1   # Release v26.9.1
```

**Features**:
- Interactive guidance through entire workflow
- Validates prerequisites (cargo, git)
- Determines version number
- Reviews changes since last release
- Runs pre-release checks
- Bumps version automatically
- Updates CHANGELOG (with editor prompt)
- Commits version bump
- Runs quality gates
- Publishes to crates.io (with token check)
- Creates git tag
- Pushes to remote
- Shows next steps

**Time**: ~12 minutes with full automation

**File**: `/scripts/release-automation.sh` (14 KB, executable)

---

### 2. **pre-release-check.sh** (Quality Gates)
**Best for**: Validating readiness before release

```bash
./scripts/pre-release-check.sh 26.9.1
```

**Runs 7 automated gates**:

| Gate | Checks | Requirement |
|------|--------|-------------|
| 1. Version Consistency | All Cargo.toml versions match | CRITICAL |
| 2. Compilation | Code compiles without errors | CRITICAL |
| 3. Test Pass Rate | 100% test pass rate, 0 failures | CRITICAL |
| 4. Compiler Warnings | Zero clippy/rustfmt warnings | CRITICAL |
| 5. Documentation | CHANGELOG & README updated | CRITICAL |
| 6. Build System | Examples build successfully | MEDIUM |
| 7. Git Status | Clean working directory, pushed | MEDIUM |

**Success Output**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ ALL GATES PASSED - Ready for Release!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Time**: ~3 minutes

**File**: `/scripts/pre-release-check.sh` (executable)

---

### 3. **bump-version.sh** (Version Bumping)
**Best for**: Atomic version updates across all files

```bash
./scripts/bump-version.sh 26.9.1
```

**Updates**:
- ✅ Main Cargo.toml package version
- ✅ Main Cargo.toml workspace dependencies
- ✅ Macros Cargo.toml version
- ✅ README.md version examples

**Verification**:
```bash
grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml
# All should show: version = "26.9.1"
```

**Time**: <1 minute

**File**: `/scripts/bump-version.sh` (executable)

---

## Makefile Tasks

**Key release tasks** (in `Makefile.toml`):

```bash
# Publishing
cargo make publish-dry-run-macros    # Test macros publish
cargo make publish-macros             # Publish macros crate
cargo make publish-dry-run            # Test main publish
cargo make publish                    # Publish main (includes macros)
cargo make publish-all                # Full workflow (checks + publish)

# Verification
cargo make release-check              # 7-gate quality check
cargo make verify-publish             # Confirm on crates.io
cargo make verify-frontier            # Frontier features

# Security
cargo make security-scan              # Audit + deny + outdated

# Performance
cargo make slo-check                  # SLO validation
```

---

## Quick Start: Your First Release

### 1. Read Documentation (30 minutes)
```bash
# Start here - overview & complete framework
cat docs/RELEASE_SKILLS.md | less

# Quick reference during release
cat docs/RELEASE_QUICK_REFERENCE.md
```

### 2. Understand Semantic Versioning (5 minutes)
```
Is it a breaking change?
├─ YES → MAJOR (27.0.0)
├─ NO, is it a new feature?
│   ├─ YES → MINOR (26.9.1)
│   └─ NO → PATCH (26.9.1)
```

### 3. Prepare Your First Release (30 minutes)
```bash
# Determine version (MAJOR/MINOR/PATCH)
# Update CHANGELOG.md
# Review all commits since last release

git log v26.9.1..HEAD --oneline
```

### 4. Execute Automated Workflow (15 minutes)
```bash
# Set environment variable first
export CARGO_REGISTRY_TOKEN="eyJhbGciOiJIUzI1NiJ9..."

# Run guided automation
./scripts/release-automation.sh 26.9.1
```

### 5. Verify Success (5 minutes)
```bash
# Check crates.io
cargo search clap-noun-verb --limit 1

# Visit docs.rs
open https://docs.rs/clap-noun-verb/26.9.1/

# Check GitHub Release
open https://github.com/seanchatmangpt/clap-noun-verb/releases
```

**Total time**: ~1.5 hours for first release (includes learning)

---

## Release Checklist Quick Copy

```markdown
## Release v26.9.1 Checklist

### Pre-Release (1-2 days before)
- [ ] Review commits since last release
- [ ] Determine MAJOR/MINOR/PATCH
- [ ] Update CHANGELOG.md
- [ ] Get team approval (if MAJOR)

### Version Bumping (day of)
- [ ] Run: ./scripts/bump-version.sh 26.9.1
- [ ] Update CHANGELOG.md (move [Unreleased])
- [ ] Update README.md version examples
- [ ] Verify: grep 'version = "' Cargo.toml clap-noun-verb-macros/Cargo.toml

### Quality Checks
- [ ] Run: ./scripts/pre-release-check.sh 26.9.1
- [ ] All 7 gates should PASS

### Publishing
- [ ] cargo make publish-dry-run-macros
- [ ] cargo make publish-macros
- [ ] Wait for crates.io indexing (~30 sec)
- [ ] cargo make publish-dry-run
- [ ] cargo make publish

### Post-Release
- [ ] git tag v26.9.1
- [ ] git push origin v26.9.1
- [ ] Monitor GitHub Actions
- [ ] Verify on crates.io
- [ ] Check docs.rs
```

---

## File Organization

```
clap-noun-verb/
├── docs/
│   ├── RELEASE_INDEX.md              ← You are here
│   ├── RELEASE_SKILLS.md             ← Complete guide (read first)
│   ├── RELEASE_MANAGEMENT.md         ← Detailed procedures (reference)
│   ├── RELEASE_BEST_PRACTICES.md     ← Excellence patterns
│   ├── RELEASE_QUICK_REFERENCE.md    ← One-page cheat sheet (print this)
│   └── MIGRATION_V26_TO_V27.md       ← Breaking change guide (if MAJOR)
│
├── scripts/
│   ├── release-automation.sh         ← Guided workflow (use this)
│   ├── pre-release-check.sh          ← Quality gates (7 checks)
│   └── bump-version.sh               ← Version bumping
│
└── Makefile.toml                     ← Build & publish tasks
```

---

## Decision Trees

### "Which document should I read?"

```
Do you have 30 minutes?
├─ YES → RELEASE_SKILLS.md (complete framework)
├─ NO, I'm about to release
│   ├─ YES → RELEASE_QUICK_REFERENCE.md (one page)
│   └─ Want details? → RELEASE_MANAGEMENT.md (reference)
└─ Want to improve? → RELEASE_BEST_PRACTICES.md (patterns)
```

### "What version number?"

```
What changed since last release?
├─ API removed or changed incompatibly
│   └─ MAJOR → v27.0.0
├─ New feature added
│   └─ MINOR → v26.9.1
└─ Bug fix or docs
    └─ PATCH → v26.9.1
```

### "Which script should I run?"

```
I want to release version 26.9.1...
├─ Guidance through entire workflow?
│   └─ ./scripts/release-automation.sh 26.9.1
├─ Just validate I'm ready?
│   └─ ./scripts/pre-release-check.sh 26.9.1
├─ Bump version across files?
│   └─ ./scripts/bump-version.sh 26.9.1
└─ Publish to crates.io?
    └─ cargo make publish
```

---

## Emergency Procedures

### Problem: Tests failing
```bash
cargo make test --all-features
# Fix failures, then retry release
```

### Problem: Bad version released
```bash
# Yank both crates
cargo yank --vers 26.9.1 -p clap-noun-verb
cargo yank --vers 26.9.1 -p clap-noun-verb-macros

# Fix and publish 26.9.1
./scripts/bump-version.sh 26.9.1
cargo make publish
```

### Problem: Macros not indexed
```bash
# Wait and check status
for i in {1..30}; do
    cargo search clap-noun-verb-macros --limit 1 && break
    sleep 2
done
```

---

## Performance Metrics

**Release time breakdown** (experienced operator):

| Phase | Time |
|-------|------|
| Version bumping | 1 min |
| CHANGELOG update | 2 min |
| Quality checks | 3 min |
| Publishing | 3 min |
| Tagging + push | 1 min |
| CI/CD automation | 3 min |
| **Total** | **~13 min** |

**First release** (with learning): ~1.5-2 hours

---

## Success Criteria

Release is complete when:

✅ All 7 quality gates pass
✅ Both crates published on crates.io
✅ docs.rs documentation live
✅ GitHub Release created with notes
✅ No error reports in first 24 hours

---

## Maintenance

These documents are kept current with:
- Each release cycle (updates to procedures)
- New Makefile tasks (updates to task list)
- Changing crates.io policies (updates to metadata guidance)
- Community feedback (improvements to clarity)

**Last updated**: 2026-08-20
**Maintained by**: Sean Chatman
**Version**: 1.0

---

## Getting Help

**Questions about release process?**
1. Check RELEASE_QUICK_REFERENCE.md (1 min)
2. Search RELEASE_SKILLS.md (5 min)
3. Review RELEASE_BEST_PRACTICES.md for patterns (10 min)
4. Open GitHub Issue with question (link context)

**Want to improve this system?**
- Suggest improvements via GitHub Discussion
- Submit PRs for documentation clarity
- Share release retrospectives and lessons learned

---

## Related Documentation

- **CLAUDE.md**: Project overview, crate structure, critical rules
- **Makefile.toml**: Complete build and release task definitions
- **Cargo.toml**: Package metadata, dependencies, feature flags
- **CHANGELOG.md**: Historical release notes for reference

---

**Ready to release?** Start with `/docs/RELEASE_SKILLS.md` → Run `./scripts/release-automation.sh`
