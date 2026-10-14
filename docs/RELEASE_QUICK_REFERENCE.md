# Release Quick Reference Card

**Print this or keep it handy for release day**

---

## Version Bump Decision (2 minutes)

| Change Type | Bump | Example |
|-------------|------|---------|
| Bug fix | PATCH | `26.9.1 → 26.9.1` |
| New feature | MINOR | `26.9.1 → 26.9.1` |
| Breaking change | MAJOR | `26.9.1 → 27.0.0` |

---

## Release Workflow (11 minutes)

### Automated (Recommended)
```bash
./scripts/release-automation.sh 26.9.1
# Guided workflow - follows all steps automatically
```

### Manual Step-by-Step
```bash
# 1. Bump version (1 min)
./scripts/bump-version.sh 26.9.1
vim CHANGELOG.md
git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md
git commit -m "chore(release): bump to 26.9.1"

# 2. Quality check (3 min)
./scripts/pre-release-check.sh 26.9.1

# 3. Publish (5 min + waiting)
cargo make publish

# 4. Tag (1 min)
git tag v26.9.1
git push origin v26.9.1
```

---

## Quality Gates (7 checks)

**All must pass before release**:

```
✓ Version consistency (Cargo.toml files match)
✓ Compilation (cargo check succeeds)
✓ Tests (100% pass rate)
✓ Warnings (0 clippy/rustfmt warnings)
✓ Documentation (CHANGELOG + README updated)
✓ Examples (build without errors)
✓ Git status (clean working directory)
```

**Run**: `./scripts/pre-release-check.sh VERSION`

---

## Publishing Sequence

**CRITICAL: Correct order required**

1. Dry-run macros: `cargo make publish-dry-run-macros`
2. Publish macros: `cargo make publish-macros`
3. Wait for index (~30 sec): `cargo search clap-noun-verb-macros`
4. Dry-run main: `cargo make publish-dry-run`
5. Publish main: `cargo make publish`

**All-in-one**: `cargo make publish` (includes macros)

---

## CHANGELOG Template

```markdown
## [26.9.1] - 2026-06-15

### Added
- New feature description with example

### Changed
- Enhancement description with rationale

### Fixed
- Bug fix with issue number (#456)

### Security
- Security fix with CVE (if applicable)
```

**Move `[Unreleased]` → `[VERSION] - YYYY-MM-DD`**

---

## MSRV Verification

**Before release, test minimum Rust version**:

```bash
rustup install 1.74          # Main crate MSRV
cargo build --all-features   # Must succeed
rustup default stable        # Switch back
```

---

## Breaking Changes (MAJOR only)

**Create migration guide**:
```bash
cat > docs/MIGRATION_V26_TO_V27.md << 'EOF'
# Migration Guide: v26 → v27

## 1. Breaking Change Name
Before: ...
After: ...
EOF
```

**In CHANGELOG**:
```markdown
## [27.0.0] - 2026-06-15

### ⚠️ Breaking Changes
- List each breaking change
- Include migration path
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Tests fail | `cargo make test` - fix failures |
| Clippy errors | `cargo make clippy` - fix warnings |
| Version mismatch | `./scripts/bump-version.sh VERSION` |
| Macros not indexed | Wait 30-60 sec, then retry |
| `CARGO_REGISTRY_TOKEN` not set | `export CARGO_REGISTRY_TOKEN=...` |
| docs.rs build fails | `cargo doc --all-features --no-deps` |

---

## Verification (After Release)

```bash
# 1. Check crates.io
cargo search clap-noun-verb --limit 1

# 2. Check docs.rs
# Visit: https://docs.rs/clap-noun-verb/26.9.1/

# 3. Check GitHub Release
# Visit: https://github.com/seanchatmangpt/clap-noun-verb/releases/tag/v26.9.1

# 4. Monitor CI/CD
# Visit: https://github.com/seanchatmangpt/clap-noun-verb/actions
```

---

## Emergency: Yank Bad Release

```bash
# Yank both crates
cargo yank --vers 26.9.1 -p clap-noun-verb
cargo yank --vers 26.9.1 -p clap-noun-verb-macros

# Fix and publish new version
./scripts/bump-version.sh 26.9.1
cargo make publish
```

---

## Environment Setup

**Required before first release**:

```bash
# 1. Get token from https://crates.io/me
# 2. Set environment variable
export CARGO_REGISTRY_TOKEN="eyJhbGciOiJIUzI1NiJ9..."

# 3. Or create ~/.cargo/credentials.toml
[registries.crates-io]
token = "eyJhbGciOiJIUzI1NiJ9..."
```

---

## Documentation Links

| Doc | Purpose |
|-----|---------|
| `docs/RELEASE_SKILLS.md` | Complete release guide |
| `docs/RELEASE_BEST_PRACTICES.md` | Best practices & patterns |
| `docs/RELEASE_MANAGEMENT.md` | Detailed procedures |
| `scripts/release-automation.sh` | Guided workflow |
| `scripts/pre-release-check.sh` | Quality gates |
| `Makefile.toml` | Build/publish tasks |

---

## Time Estimate

| Phase | Time |
|-------|------|
| Version bump + CHANGELOG | 5 min |
| Quality checks | 3 min |
| Publishing | 3 min + wait |
| Tagging + push | 1 min |
| **Total** | **~12 min** |

(First release might take 2-3x longer due to setup)

---

## Success Criteria

Release is successful when:

- ✅ All quality gates pass
- ✅ Both crates published on crates.io
- ✅ docs.rs documentation live
- ✅ GitHub Release created
- ✅ No error reports in issues

---

**Version**: 1.0  
**Last Updated**: 2026-08-20  
**Timezone**: UTC  
**Maintainer**: Sean Chatman
