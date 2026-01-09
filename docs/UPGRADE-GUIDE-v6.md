# v6.0.0 Upgrade Guide
## Step-by-Step Instructions for Dependency Upgrade

---

## Table of Contents
1. Prerequisites
2. Phase 1: Core Dependency Updates
3. Phase 2: Frontier Feature Testing
4. Phase 3: Breaking Change Handling
5. Phase 4: Supply Chain Resolution
6. Phase 5: MSRV Update & Release
7. Troubleshooting
8. Rollback Procedures

---

## Prerequisites

### Required Tools
```bash
# Install rust 1.80 (target MSRV)
rustup install 1.80

# Ensure cargo-make is installed
cargo install cargo-make

# Optional but recommended
cargo install cargo-audit
cargo install cargo-tree
cargo install cargo-deny
```

### Environment Setup
```bash
# Clone and navigate to repo
git clone https://github.com/seanchatmangpt/clap-noun-verb
cd clap-noun-verb

# Create feature branch for v6.0.0 work
git checkout -b feature/v6.0.0-dependency-upgrade

# Verify current state
cargo --version    # Should be >= 1.74
rustc --version    # Should be >= 1.74
cargo make --version
```

### Baseline Verification
```bash
# Run all checks to establish baseline
cargo make check
cargo make test
cargo make lint

# All should PASS before proceeding
```

---

## PHASE 1: Core Dependency Updates (2 hours) 🟢

### Step 1.1: Update Core Dependencies in Cargo.toml

**Location**: `/Cargo.toml` - `[dependencies]` section

```toml
# BEFORE
clap = { version = "4.5", features = ["derive", "env", "suggestions"] }

# AFTER
clap = { version = "4.6", features = ["derive", "env", "suggestions"] }
```

**Dependencies to Update**:

```toml
# 1. Core CLI
clap = { version = "4.6", features = ["derive", "env", "suggestions"] }

# 2. UUID support (if used)
uuid = { version = "1.19", features = ["v4", "serde"], optional = true }

# 3. TOML config
toml = { version = "0.8.24", optional = true }

# 4. YAML config
serde_yaml = { version = "0.9.34", optional = true }

# 5. RDF support
oxrdf = { version = "0.3.1", optional = true }

# 6. Template engine
handlebars = { version = "6.4.0", optional = true }

# 7. Caching
lru = { version = "0.16.3", optional = true }

# Keep everything else at current versions
```

### Step 1.2: Verify Changes

```bash
# Check syntax
cargo make check

# Run tests
cargo make test-unit

# Lint
cargo make lint

# Expected: All pass, no warnings
```

### Step 1.3: Commit Phase 1

```bash
git add Cargo.toml
git commit -m "feat: Update core dependencies to patch/minor versions

- clap: 4.5 → 4.6
- uuid: 1.0 → 1.19
- toml: 0.8.23 → 0.8.24
- serde_yaml: 0.9 → 0.9.34
- oxrdf: 0.2.4 → 0.3.1
- handlebars: 5.1.2 → 6.4.0
- lru: 0.12.5 → 0.16.3

All tests passing, no API changes required."
```

---

## PHASE 2: Frontier Feature Testing (6 hours) 🟡

### Step 2.1: Update Frontier Packages

**Location**: `/Cargo.toml` - frontier dependencies section

```toml
# RDF Composition (stable)
libp2p = { version = "0.56", features = ["tokio", "tcp", "noise", "yamux", "gossipsub", "kad", "request-response", "mdns"], optional = true }

# Graph structures
petgraph = { version = "0.8.3", optional = true }
daggy = { version = "0.9.0", optional = true }

# ECS system
bevy_ecs = { version = "0.17.3", default-features = false, optional = true }

# ML features
smartcore = { version = "0.4.8", optional = true }
ndarray = { version = "0.17.1", optional = true }
linfa = { version = "0.8.1", optional = true }

# Testing
cucumber = { version = "0.22.1", optional = true }
gherkin = { version = "0.15.0", optional = true }

# Floats for economic simulation
ordered-float = { version = "5.1.0", optional = true }
```

### Step 2.2: Test Individual Features

```bash
# Test RDF composition feature
echo "Testing rdf-composition..."
cargo make test --features rdf-composition
# Expected: ALL TESTS PASS

# Test economic simulation
echo "Testing economic-sim..."
cargo make test --features economic-sim
# Expected: ALL TESTS PASS

# Test federated network
echo "Testing federated-network..."
cargo make test --features federated-network
# Expected: ALL TESTS PASS (or fails with bft-rs issue - expected)

# Test reflexive (property-based) testing
echo "Testing reflexive-testing..."
cargo make test --features reflexive-testing
# Expected: ALL TESTS PASS

# Test learning trajectories (ML)
echo "Testing learning-trajectories..."
cargo make test --features learning-trajectories
# Expected: ALL TESTS PASS
```

### Step 2.3: Full Feature Integration Test

```bash
# Test ALL features together
echo "Testing all features together..."
cargo make test --all-features
# Expected: ALL TESTS PASS

# Run linter on all features
cargo make lint --all-features
```

### Step 2.4: If Tests Fail

**Common Issues & Solutions**:

#### Issue: Test failure in economic-sim
```
Error: unresolved import `simrs`
```
**Solution**: This is EXPECTED - simrs is not available
- Document in CHANGELOG
- Continue to Phase 3

#### Issue: Test failure in federated-network
```
Error: unresolved import `bft_rs`
```
**Solution**: This is EXPECTED - bft-rs is not available
- Document in CHANGELOG
- Continue to Phase 3

#### Issue: Other test failures
**Solution**: Stop and debug:
```bash
# Get detailed error
cargo make test --features [feature] -- --nocapture

# Check what changed
git diff HEAD~1 Cargo.lock

# Revert and investigate
git checkout -- Cargo.lock
```

### Step 2.5: Commit Phase 2

```bash
git add Cargo.toml Cargo.lock
git commit -m "feat: Update frontier feature dependencies

- libp2p: 0.54.1 → 0.56.0
- petgraph: 0.6.5 → 0.8.3
- daggy: 0.8.1 → 0.9.0
- bevy_ecs: 0.14.2 → 0.17.3
- smartcore: 0.3.2 → 0.4.8
- cucumber: 0.21 → 0.22.1
- gherkin: 0.14 → 0.15.0
- ordered-float: 4.2 → 5.1.0

All frontier features tested, all tests passing."
```

---

## PHASE 3: Breaking Change Handling (9 hours) 🔴

### Step 3.1: Update Breaking Changes in Cargo.toml

**Location**: `/Cargo.toml` - critical dependencies section

```toml
# ⚠️ MAJOR UPDATE - Requires Code Changes
thiserror = { version = "2.0", optional = false }

# ⚠️ MAJOR UPDATE - Requires RDF feature testing
json-ld = { version = "0.21.2", optional = true }

# ⚠️ MAJOR UPDATE - Requires RDF feature testing
rmcp = { version = "0.12.0", features = ["server", "macros"], optional = true }
```

### Step 3.2: Audit Error Types (thiserror 2.0)

**Location**: Search all error definitions

```bash
# Find all error types
grep -r "thiserror" src/ --include="*.rs"
grep -r "#\[error" src/ --include="*.rs"
grep -r "#\[from\]" src/ --include="*.rs"

# Expected output: List of all error types using thiserror
```

**Check for API Changes**:

thiserror 2.0 maintains backward compatibility for most cases. Check:

```rust
// BEFORE (v1.0)
#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// AFTER (v2.0) - Should still work with no changes
#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Test Error Handling**:

```bash
# Run all error tests
cargo make test --lib
# Expected: All error-related tests pass

# Check error message formatting
cargo make test --test '*'
# Expected: All integration tests pass
```

### Step 3.3: Test RDF Feature Updates

**For `json-ld` 0.21.2 and `rmcp` 0.12.0**:

```bash
# Test RDF feature
echo "Testing RDF feature..."
cargo make test --features rdf
# Expected: ALL TESTS PASS

# Test RDF composition (more intensive)
echo "Testing RDF composition..."
cargo make test --features rdf-composition
# Expected: ALL TESTS PASS

# Verify SPARQL queries still work (if applicable)
echo "Checking SPARQL compatibility..."
grep -r "sparql" tests/ --include="*.rs"
# If found, verify they execute correctly
```

### Step 3.4: Comprehensive Regression Testing

```bash
# Full test suite
echo "Running full test suite..."
cargo make test
# Expected: 100% tests passing

# Check for any warnings
echo "Checking for warnings..."
cargo make lint
# Expected: No new warnings

# Run benchmarks to ensure performance unchanged
echo "Running benchmarks..."
cargo make bench
# Expected: Performance within 5% of baseline
```

### Step 3.5: Error Type Documentation

Create/update error handling guide:

```bash
# Create error documentation
cat > docs/ERROR-HANDLING.md << 'EOF'
# Error Handling in clap-noun-verb v6.0.0

## Overview
This version uses thiserror 2.0 for ergonomic error handling.

## Error Types
[Document all error types and their meanings]

## Migration from v5.5.0
[Document any API changes users need to handle]
EOF

git add docs/ERROR-HANDLING.md
```

### Step 3.6: Commit Phase 3

```bash
git add Cargo.toml Cargo.lock
git commit -m "feat: Update to thiserror 2.0, json-ld 0.21.2, rmcp 0.12.0

BREAKING CHANGES:
- thiserror upgraded to 2.0
- json-ld upgraded to 0.21.2 (RDF composition)
- rmcp upgraded to 0.12.0 (MCP integration)

All error types audited and verified compatible.
RDF features tested and working.
Full test suite passing.

Migration guide: docs/ERROR-HANDLING.md"
```

---

## PHASE 4: Supply Chain Resolution (40 hours) 🔴 CRITICAL

### Step 4.1: Resolve bft-rs Alternative

**Issue**: `bft-rs` not available on crates.io

**Decision Tree**:

```
Option 1: Use alternative Byzantine consensus library
  → Recommended: hotstuff or tendermint
  → Effort: 1-2 weeks
  → Impact: Update federated-network feature to use new lib

Option 2: Implement minimal BFT layer
  → Effort: 2-3 weeks
  → Impact: New module in src/bft/

Option 3: Disable federated-network feature
  → Effort: 1-2 hours
  → Impact: Feature removed from v6.0.0
```

**Chosen Approach**: [DECISION NEEDED FROM LEADERSHIP]

#### If Option 1 (Alternative Library): hotstuff Example

```toml
# Update Cargo.toml
hotstuff = { version = "0.2", optional = true }

# Update feature
federated-network = ["async", "dep:libp2p", "dep:quinn", "dep:rustls", "dep:hotstuff"]
```

```bash
# Implement hotstuff integration
# Edit: src/federated/bft.rs

# Test it
cargo make test --features federated-network

# Document it
cat > docs/FEDERATED-NETWORK.md << 'EOF'
# Federated Network Feature

This feature uses hotstuff for Byzantine Fault Tolerance...
EOF
```

#### If Option 3 (Remove Feature)

```toml
# Remove from features
# Remove: federated-network line

# Keep dependency optional but commented
# bft-rs = { version = "0.3", optional = true }  # Not on crates.io - disabled
```

```bash
# Document removal
cat >> CHANGELOG.md << 'EOF'

## Removed Features (v6.0.0)
- federated-network: Byzantine consensus library (bft-rs) no longer available on crates.io
  Future versions may restore with alternative implementation
EOF
```

### Step 4.2: Resolve simrs Alternative

**Issue**: `simrs` not available on crates.io

**Decision Tree**:

```
Option 1: Use alternative simulation framework
  → Recommended: simulate-rs or discrete-event
  → Effort: 1-2 weeks
  → Impact: Update economic-sim feature to use new lib

Option 2: Implement minimal simulation layer
  → Effort: 2-3 weeks
  → Impact: New module in src/simulation/

Option 3: Disable economic-sim feature
  → Effort: 1-2 hours
  → Impact: Feature removed from v6.0.0
```

**Chosen Approach**: [DECISION NEEDED FROM LEADERSHIP]

#### If Option 1 (Alternative Library): discrete-event Example

```toml
# Update Cargo.toml
discrete-event = { version = "0.1", optional = true }

# Update feature
economic-sim = ["dep:priority-queue", "dep:ordered-float", "dep:bevy_ecs", "dep:discrete-event"]
```

```bash
# Implement discrete-event integration
# Edit: src/economic/simulation.rs

# Test it
cargo make test --features economic-sim

# Document it
cat > docs/ECONOMIC-SIMULATION.md << 'EOF'
# Economic Simulation Feature

This feature uses discrete-event simulation for agent economies...
EOF
```

#### If Option 3 (Remove Feature)

```toml
# Remove from features
# Remove: economic-sim line

# Keep dependency optional but commented
# simrs = { version = "0.1", optional = true }  # Not on crates.io - disabled
```

```bash
# Document removal
cat >> CHANGELOG.md << 'EOF'

## Removed Features (v6.0.0)
- economic-sim: Simulation library (simrs) no longer available on crates.io
  Future versions may restore with alternative implementation
EOF
```

### Step 4.3: Full Feature Testing After Resolution

```bash
# Test all features again with resolved dependencies
echo "Final feature testing..."
cargo make test --all-features
# Expected: ALL TESTS PASS

# Lint entire codebase
cargo make lint --all-features
# Expected: No warnings
```

### Step 4.4: Commit Phase 4

```bash
git add Cargo.toml Cargo.lock docs/

if [ "$BFT_CHOICE" = "alternative" ]; then
  git commit -m "feat: Integrate hotstuff for Byzantine consensus (federated-network)

- Replaced unavailable bft-rs with hotstuff 0.2
- Updated federated-network feature
- All tests passing
- Documentation: docs/FEDERATED-NETWORK.md"
else
  git commit -m "feat: Remove federated-network feature (supply chain risk)

- bft-rs library no longer available on crates.io
- Disabled federated-network feature in v6.0.0
- Can be restored with alternative implementation in v6.1+
- Documentation: CHANGELOG.md"
fi

if [ "$SIM_CHOICE" = "alternative" ]; then
  git commit -m "feat: Integrate discrete-event for agent simulation (economic-sim)

- Replaced unavailable simrs with discrete-event
- Updated economic-sim feature
- All tests passing
- Documentation: docs/ECONOMIC-SIMULATION.md"
else
  git commit -m "feat: Remove economic-sim feature (supply chain risk)

- simrs library no longer available on crates.io
- Disabled economic-sim feature in v6.0.0
- Can be restored with alternative implementation in v6.1+
- Documentation: CHANGELOG.md"
fi
```

---

## PHASE 5: MSRV Update & Release (4 hours) 🟢

### Step 5.1: Update MSRV in Cargo.toml

**Location**: `/Cargo.toml` - package section

```toml
# BEFORE
rust-version = "1.74"

# AFTER
rust-version = "1.80"
```

### Step 5.2: Verify with Rust 1.80

```bash
# Install and test with target MSRV
rustup install 1.80

# Verify compilation
cargo +1.80 check
# Expected: SUCCESS

# Verify tests
cargo +1.80 test --all-features
# Expected: ALL PASS

# Verify benchmarks
cargo +1.80 build --release
# Expected: SUCCESS
```

### Step 5.3: Update Documentation

```bash
# Update README.md
sed -i 's/rust-version = "1.74"/rust-version = "1.80"/' README.md

# Create migration guide
cat > docs/MIGRATION-v5.5-to-v6.0.md << 'EOF'
# Migration Guide: v5.5.0 → v6.0.0

## Breaking Changes

### 1. MSRV Requirement
- Minimum Rust version is now 1.80 (was 1.74)
- Update your toolchain: `rustup update`

### 2. thiserror 2.0
- Error handling improved but mostly backward compatible
- If you implement custom error types using thiserror, verify they compile

### 3. RDF Features
- json-ld upgraded to 0.21.2
- rmcp upgraded to 0.12.0
- If you use RDF composition, test with new versions

### 4. Removed Features
- [List any features removed like federated-network or economic-sim]

## Migration Steps

1. Update your clap-noun-verb dependency:
```toml
[dependencies]
clap-noun-verb = "6.0"  # Updated from 5.5
```

2. Update Rust toolchain:
```bash
rustup update stable  # or rustup update 1.80
```

3. Run tests:
```bash
cargo test --all-features
```

4. Check for any breaking change impacts specific to your use case

## Support

If you encounter issues:
- Check this guide
- Review CHANGELOG.md
- File an issue on GitHub
EOF

git add docs/MIGRATION-v5.5-to-v6.0.md README.md
```

### Step 5.4: Update CHANGELOG

```bash
cat > CHANGELOG-v6.0.0-entry.md << 'EOF'
# Version 6.0.0 (Release Date: TBD)

## 🚀 Major Features
- Full frontier feature integration
- Enhanced RDF/Semantic composition
- Improved error handling with thiserror 2.0

## ⚠️ BREAKING CHANGES

### MSRV Bump: 1.74 → 1.80
- Minimum supported Rust version is now 1.80
- Update with: `rustup update`

### thiserror 2.0 Update
- Error types updated for better ergonomics
- Mostly backward compatible, verify error handling

### RDF Package Updates
- json-ld: 0.18 → 0.21.2
- rmcp: 0.9 → 0.12.0
- RDF composition feature users should test thoroughly

### Feature Changes
[Include any features added/removed/changed]

## 📦 Dependency Updates

### Core
- clap: 4.5 → 4.6
- thiserror: 1.0 → 2.0

### Features
- libp2p: 0.54.1 → 0.56.0
- bevy_ecs: 0.14.2 → 0.17.3
- handlebars: 5.1.2 → 6.4.0
[Full list...]

## 🔍 Security
- No new vulnerabilities
- All dependencies audited
- Cryptographic libraries verified

## 📚 Migration Guide
See [MIGRATION-v5.5-to-v6.0.md](docs/MIGRATION-v5.5-to-v6.0.md)

## 🐛 Bug Fixes
- [List any bug fixes]

## 📖 Documentation
- New: docs/MIGRATION-v5.5-to-v6.0.md
- Updated: docs/ERROR-HANDLING.md
- Updated: README.md (MSRV requirement)
EOF

# Append to CHANGELOG.md
cat CHANGELOG-v6.0.0-entry.md >> CHANGELOG.md
git add CHANGELOG.md
rm CHANGELOG-v6.0.0-entry.md
```

### Step 5.5: Final Validation

```bash
# Run full CI pipeline
cargo make ci
# Expected: ALL CHECKS PASS

# Run release validation
cargo make release-validate
# Expected: READY FOR RELEASE

# Check formatting
cargo make fmt
# Expected: No changes needed

# Generate documentation
cargo doc --no-deps --all-features --open
# Expected: Documentation builds successfully
```

### Step 5.6: Commit Phase 5

```bash
git add Cargo.toml CHANGELOG.md docs/MIGRATION-v5.5-to-v6.0.md
git commit -m "chore: Bump MSRV to 1.80 and prepare v6.0.0 release

- Update rust-version: 1.74 → 1.80
- Create migration guide for users
- Update CHANGELOG with all breaking changes
- Verify compatibility with Rust 1.80
- Full test suite passing
- CI pipeline green

Ready for release."
```

### Step 5.7: Create Release Tag

```bash
# Create annotated tag
git tag -a v6.0.0 -m "Release v6.0.0

Major Features:
- MSRV bump to 1.80
- thiserror 2.0 integration
- Frontier feature stability

Breaking Changes:
- MSRV requirement increased
- Error handling improvements
- Dependency updates

See CHANGELOG.md for full details."

# Push to remote
git push origin feature/v6.0.0-dependency-upgrade
git push origin v6.0.0
```

---

## Troubleshooting

### Issue: Compilation fails with "no such item: bft_rs"
```
error[E0433]: failed to resolve: use of undeclared type `bft_rs`
```

**Solution**:
```bash
# This is expected - bft-rs is not available
# Remove the feature or implement alternative

# Option 1: Update Cargo.toml to use alternative
# Option 2: Comment out bft-rs dependency
# Option 3: Disable federated-network feature
```

### Issue: Test failure in newly updated feature
```
test result: FAILED. failures:
```

**Solution**:
```bash
# Get detailed error output
cargo make test --features [feature] -- --nocapture --test-threads=1

# Check if it's a version-related API change
grep -r "VERSION CHANGE" [feature files]

# If API changed:
# 1. Update code to match new API
# 2. Verify with docs of new version
# 3. Retest

# If unrelated:
# 1. Check git diff to see what changed
# 2. Revert problematic change
# 3. File issue
```

### Issue: MSRV verification fails
```
error: package requires rustc 1.80.0, but you have 1.79.0
```

**Solution**:
```bash
# Update Rust to at least 1.80
rustup update stable

# Verify version
rustc --version

# Retry test
cargo +1.80 test
```

### Issue: Dependency conflict
```
error: the following packages have conflicting requirements:
```

**Solution**:
```bash
# Check dependency tree
cargo tree

# Look for conflicting version requirements
cargo tree --duplicates

# May need to pin a specific version
# Edit Cargo.toml to resolve conflict
```

---

## Rollback Procedures

### If Phase 1 fails
```bash
git reset --hard HEAD
git clean -fd

# Start over with different approach
cargo make check
```

### If Phase 2 fails
```bash
# Keep Phase 1, revert Phase 2
git revert HEAD

# Or reset to Phase 1 commit
git reset --hard <phase-1-commit-hash>
```

### If Phase 3 fails
```bash
# Investigate error thoroughly
cargo make test --all-features -- --nocapture

# May need to:
# 1. Fix code to match new API
# 2. Pin to different version
# 3. Revert update
```

### If Phase 4 fails
```bash
# Supply chain issues are critical
# May need to:
# 1. Choose different alternative
# 2. Implement custom layer
# 3. Remove feature entirely
```

### Full Rollback
```bash
# Go back to v5.5.0
git reset --hard <v5.5.0-tag>
git clean -fd

# Start over with different decisions
```

---

## Success Criteria

✅ **Phase 1 Complete** when:
- [ ] `cargo make check` passes
- [ ] `cargo make test` passes
- [ ] `cargo make lint` passes
- [ ] No new warnings

✅ **Phase 2 Complete** when:
- [ ] All frontier features tested
- [ ] `cargo make test --all-features` passes
- [ ] No regressions

✅ **Phase 3 Complete** when:
- [ ] thiserror 2.0 API audited
- [ ] All error types verified
- [ ] RDF features tested
- [ ] No breaking behavior changes

✅ **Phase 4 Complete** when:
- [ ] bft-rs alternative resolved
- [ ] simrs alternative resolved
- [ ] All features compile and test
- [ ] Documentation updated

✅ **Phase 5 Complete** when:
- [ ] MSRV verified with Rust 1.80
- [ ] `cargo make ci` passes
- [ ] `cargo make release-validate` passes
- [ ] Migration guide published
- [ ] CHANGELOG updated

✅ **Ready to Release** when:
- ALL success criteria met
- [ ] Security audit passed
- [ ] Benchmarks acceptable
- [ ] Documentation complete

---

## Timeline Estimates

| Phase | Duration | Critical Path | Blocker Risk |
|-------|----------|---------------|--------------|
| 1 | 2 hours | No | Low |
| 2 | 6 hours | No | Medium |
| 3 | 9 hours | No | Medium |
| 4 | 40 hours | YES | High |
| 5 | 4 hours | No | Low |
| **TOTAL** | **61+ hours** | - | - |

**Critical Path**: Phase 4 (bft-rs and simrs decisions)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-08
**Status**: Ready for Use
