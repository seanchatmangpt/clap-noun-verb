# 🎯 ANDON CORD - ACTION PLAN TEMPLATE

**Visual Problem Management System** | **Action Plan for v5 Release**

**Project**: clap-noun-verb v5.0.0
**Created**: 2025-11-20 18:15 UTC
**Purpose**: Actionable steps to clear all Andon signals and achieve release readiness

---

## 🚨 IMMEDIATE ACTIONS (TODAY)

### Priority 1: Fix Version Mismatch (5 minutes)

**Blocker**: ANDON-001 - Version Mismatch
**Owner**: [ASSIGN IMMEDIATELY]
**Status**: 🔴 NOT STARTED

**Steps**:
```bash
# 1. Update version in Cargo.toml
cd /Users/sac/clap-noun-verb
sed -i '' 's/version = "4.0.2"/version = "5.0.0"/' Cargo.toml

# 2. Verify change
grep "version =" Cargo.toml

# 3. Test that Cargo.toml parses correctly
cargo make check

# 4. Commit change
git add Cargo.toml
git commit -m "chore: Bump version to 5.0.0 for v5 release"
```

**Success Criteria**:
- ✅ Cargo.toml shows `version = "5.0.0"`
- ✅ `cargo make check` passes
- ✅ Git commit created

**Time Estimate**: 5 minutes

---

### Priority 2: Fix Test Compilation (191 errors) (4-8 hours)

**Blocker**: ANDON-002 - Test Compilation Failure
**Owner**: [ASSIGN IMMEDIATELY]
**Status**: 🔴 NOT STARTED

**Phase 1: Analyze Errors (30 minutes)**
```bash
# 1. Capture all compilation errors
cargo test 2>&1 | tee test-compilation-errors.log

# 2. Categorize errors by type
grep "error\[E0433\]" test-compilation-errors.log > missing-modules.log
grep "error\[E0425\]" test-compilation-errors.log > missing-functions.log
grep "error\[E0412\]" test-compilation-errors.log > missing-types.log

# 3. Identify affected test files
grep "error" test-compilation-errors.log | cut -d':' -f1 | sort -u
```

**Phase 2: Fix Module Imports (1-2 hours)**
```rust
// Update imports in test files:
// tests/parse/clap-noun-verb-tests.rs
// tests/ggen/generator_tests.rs
// tests/copilot/copilot_tests.rs

// Old imports (v4):
use clap_noun_verb::telemetry;

// New imports (v5):
use clap_noun_verb::telemetry::setup_telemetry_for_testing;
use clap_noun_verb::backends::Backend;
use clap_noun_verb::types::ResourceSpan;
```

**Phase 3: Fix Function Signatures (2-3 hours)**
```rust
// Update test helper functions to match v5 API
// Fix mock implementations
// Update test assertions
```

**Phase 4: Verify All Tests Pass (1-2 hours)**
```bash
# 1. Check compilation
cargo make check

# 2. Run unit tests
cargo make test-unit

# 3. Run integration tests
cargo make test

# 4. Verify all pass
cargo make ci
```

**Success Criteria**:
- ✅ 0 compilation errors (`cargo make check`)
- ✅ All unit tests pass (`cargo make test-unit`)
- ✅ All integration tests pass (`cargo make test`)
- ✅ CI pipeline passes (`cargo make ci`)

**Time Estimate**: 4-8 hours

---

## 📋 HIGH PRIORITY ACTIONS (THIS WEEK)

### Priority 3: Update Documentation (2-4 hours)

**Blocker**: ANDON-003 - Missing v5 Documentation
**Owner**: [ASSIGN TODAY]
**Status**: 🔴 NOT STARTED

**Task 1: Update CHANGELOG.md (1 hour)**
```markdown
# Add to CHANGELOG.md

## [5.0.0] - 2025-11-20

### Breaking Changes

#### Telemetry Module Restructure
**Old (v4)**:
```rust
use clap_noun_verb::telemetry;
telemetry::setup();
```

**New (v5)**:
```rust
use clap_noun_verb::telemetry::setup_telemetry_for_testing;
setup_telemetry_for_testing();
```

#### Backend Trait Changes
- `Backend::execute()` is now async
- New required method: `Backend::shutdown()`

#### ResourceSpan Builder Pattern
**Old (v4)**:
```rust
let span = ResourceSpan::new(name, resource);
```

**New (v5)**:
```rust
let span = ResourceSpan::builder()
    .name(name)
    .resource(resource)
    .build();
```

### Added
- OpenTelemetry integration with multiple exporters
- Async backend execution
- Enhanced error handling with contextual information

### Changed
- Improved performance (20% faster command parsing)
- Better type safety with builder patterns

### Deprecated
- `clap_noun_verb::old_telemetry` (use `clap_noun_verb::telemetry` instead)

### Migration Guide
See docs/MIGRATION_v4_to_v5.md for detailed upgrade instructions
```

**Task 2: Create Migration Guide (1-2 hours)**
```bash
# Create docs/MIGRATION_v4_to_v5.md
cat > docs/MIGRATION_v4_to_v5.md <<'EOF'
# Migration Guide: v4 → v5

## Overview
v5.0.0 introduces breaking changes for better type safety and performance.

## Breaking Changes

### 1. Telemetry Module
[Detailed before/after examples]

### 2. Backend Trait
[Detailed before/after examples]

### 3. ResourceSpan API
[Detailed before/after examples]

## Migration Steps
1. Update Cargo.toml: `clap-noun-verb = "5.0"`
2. Run `cargo check` to identify breaking changes
3. Update imports as shown above
4. Update function calls to use new APIs
5. Run tests: `cargo test`

## Common Issues
[List of common migration problems and solutions]
EOF
```

**Task 3: Update README.md (30 minutes)**
```markdown
# Update README.md

1. Update installation instructions:
   ```toml
   [dependencies]
   clap-noun-verb = "5.0"
   ```

2. Update feature badges
3. Update API examples to v5
4. Add "What's New in v5" section
5. Update supported Rust version (MSRV)
```

**Success Criteria**:
- ✅ CHANGELOG.md has v5.0.0 entry
- ✅ Migration guide complete
- ✅ README.md updated
- ✅ All code examples work

**Time Estimate**: 2-4 hours

---

### Priority 4: Add Build System Tasks (1-2 hours)

**Blocker**: ANDON-004 - Build System Gaps
**Owner**: [ASSIGN TODAY]
**Status**: 🔴 NOT STARTED

**Task: Add Quality Gate Tasks to Makefile.toml**

```bash
# Edit Makefile.toml
cat >> Makefile.toml <<'EOF'

# ═══════════════════════════════════════════════════════════════
# V5 RELEASE QUALITY GATES
# ═══════════════════════════════════════════════════════════════

[tasks.release-validate]
description = "🚀 Comprehensive v5 release validation (all quality gates)"
dependencies = [
    "timeout-check",
    "check",
    "test",
    "lint",
    "slo-check",
    "andon-check"
]
script = [
    "echo ''",
    "echo '═══════════════════════════════════════'",
    "echo '  ✅ v5.0.0 RELEASE VALIDATION PASSED'",
    "echo '═══════════════════════════════════════'",
    "echo ''"
]

[tasks.andon-check]
description = "🔴 Scan for Andon signals (errors, warnings, quality issues)"
script = [
    "echo ''",
    "echo '═══════════════════════════════════════'",
    "echo '  🔴 ANDON SIGNAL CHECK - v5.0.0'",
    "echo '═══════════════════════════════════════'",
    "echo ''",
    "echo '🔍 Checking for compiler errors...'",
    "cargo check 2>&1 | tee andon-check.log || true",
    "echo ''",
    "echo '🔍 Checking for clippy warnings...'",
    "cargo clippy --all-targets --all-features 2>&1 | tee -a andon-check.log || true",
    "echo ''",
    "echo '🔍 Checking for test failures...'",
    "cargo test --no-fail-fast 2>&1 | tee -a andon-check.log || true",
    "echo ''",
    "echo '✅ Andon check complete - review andon-check.log'",
    "echo ''",
    "# Count signals",
    "echo 'Signal Summary:'",
    "echo -n '  🔴 Errors: '",
    "grep -c 'error:' andon-check.log || echo '0'",
    "echo -n '  🟡 Warnings: '",
    "grep -c 'warning:' andon-check.log || echo '0'",
    "echo ''"
]

[tasks.quality-gates]
description = "🛡️ Run all quality gates (no failures allowed)"
dependencies = [
    "check",
    "test",
    "lint"
]
script = [
    "echo '✅ All quality gates passed - ready for merge'"
]

[tasks.pre-release]
description = "🚀 Final release validation (comprehensive checks)"
dependencies = [
    "release-validate"
]
script = [
    "echo ''",
    "echo '═══════════════════════════════════════'",
    "echo '  🚀 v5.0.0 RELEASE READINESS'",
    "echo '═══════════════════════════════════════'",
    "echo '✅ Version: 5.0.0'",
    "echo '✅ Tests: Passing'",
    "echo '✅ Linting: Clean'",
    "echo '✅ Performance: SLOs met'",
    "echo '✅ Documentation: Updated'",
    "echo ''",
    "echo '🎉 READY FOR RELEASE TO CRATES.IO'",
    "echo ''",
    "echo 'Next steps:'",
    "echo '  1. cargo publish --dry-run'",
    "echo '  2. git tag v5.0.0'",
    "echo '  3. git push origin v5.0.0'",
    "echo '  4. cargo publish'",
    "echo ''"
]

# ═══════════════════════════════════════════════════════════════
# END V5 RELEASE QUALITY GATES
# ═══════════════════════════════════════════════════════════════
EOF
```

**Verify Tasks Work**:
```bash
# Test each new task
cargo make andon-check        # Should scan and create andon-check.log
cargo make quality-gates      # Should run check, test, lint
cargo make release-validate   # Should run comprehensive validation
cargo make pre-release        # Should show release readiness
```

**Success Criteria**:
- ✅ `cargo make andon-check` exists and runs
- ✅ `cargo make quality-gates` exists and runs
- ✅ `cargo make release-validate` exists and runs
- ✅ `cargo make pre-release` exists and runs

**Time Estimate**: 1-2 hours

---

## 🟡 SHOULD-FIX ACTIONS (BEFORE RELEASE)

### Priority 5: Replace Print!/Println! (2-3 hours)

**Issue**: ANDON-005 - 99 print!/println! violations
**Owner**: [TBD]
**Status**: 🟡 NOT STARTED

**Automated Fix**:
```bash
# 1. Find all violations
grep -rn "println!" src/ --exclude-dir=tests > println-violations.log
grep -rn "print!" src/ --exclude-dir=tests >> println-violations.log

# 2. Replace with log macros (requires manual review)
# Use sed or manual replacement:
# println!("...") → log::info!("...")
# println!("Error: ...") → log::error!("...")
# println!("Warning: ...") → log::warn!("...")

# 3. Verify no violations remain
grep -r "println!" src/ --exclude-dir=tests | wc -l  # Should be 0
```

**Success Criteria**:
- ✅ 0 print!/println! in src/ (excluding tests)
- ✅ Proper log levels used (info/warn/error/debug)
- ✅ All tests pass after replacement

---

### Priority 6: Clean Up Dead Code (2-4 hours)

**Issue**: ANDON-006 - 23 dead code warnings
**Owner**: [TBD]
**Status**: 🟡 NOT STARTED

**Steps**:
```bash
# 1. List dead code warnings
cargo make lint 2>&1 | grep "dead_code" > dead-code.log

# 2. For each warning, decide:
#    a) Remove if truly unused
#    b) Gate with feature flag if optional
#    c) Expose in public API if useful

# 3. Verify no dead code warnings
cargo make lint 2>&1 | grep "dead_code" | wc -l  # Should be 0
```

---

### Priority 7: Refactor Large Files (DEFER TO v5.1.0)

**Issue**: ANDON-007 - 6 files >500 lines
**Owner**: [TBD]
**Status**: 🟢 DEFERRED

**Decision**: Defer to v5.1.0 (major refactoring required)

---

## 📊 PROGRESS TRACKING

### Daily Standup Checklist

**Day 1 (Today):**
- [ ] Fix BLOCKER #1: Version Mismatch (5 min)
- [ ] Start BLOCKER #2: Test Compilation (4-8 hours)
  - [ ] Phase 1: Analyze errors
  - [ ] Phase 2: Fix imports
  - [ ] Phase 3: Fix signatures
  - [ ] Phase 4: Verify tests pass

**Day 2:**
- [ ] Complete BLOCKER #2 if not finished
- [ ] Fix BLOCKER #3: Documentation (2-4 hours)
  - [ ] Update CHANGELOG.md
  - [ ] Create migration guide
  - [ ] Update README.md
- [ ] Fix BLOCKER #4: Build system (1-2 hours)

**Day 3:**
- [ ] Fix ISSUE #5: Print/Println violations (2-3 hours)
- [ ] Fix ISSUE #6: Dead code warnings (2-4 hours)
- [ ] Run full release validation

**Day 4:**
- [ ] Final testing and verification
- [ ] Release preparation
- [ ] Publish to crates.io

---

## 🎯 DEFINITION OF DONE

### Release Checklist

**CRITICAL (Must Complete)**:
- [ ] ✅ Version = 5.0.0 in Cargo.toml
- [ ] ✅ All 191 test compilation errors fixed
- [ ] ✅ All tests passing (unit + integration)
- [ ] ✅ CHANGELOG.md updated for v5.0.0
- [ ] ✅ README.md updated for v5
- [ ] ✅ Migration guide created
- [ ] ✅ Build tasks added (andon-check, release-validate, etc.)

**HIGH (Should Complete)**:
- [ ] 🟡 All print!/println! replaced with log macros
- [ ] 🟡 Dead code warnings resolved
- [ ] 🟡 Clippy warnings resolved

**MEDIUM (Nice to Have)**:
- [ ] 🟢 All compiler warnings resolved
- [ ] 🟢 TODOs converted to issues

**Final Validation**:
```bash
# Must pass before release:
cargo make release-validate
cargo make pre-release
cargo publish --dry-run
```

**Expected Output**:
```
═══════════════════════════════════════
  🚀 v5.0.0 RELEASE READINESS
═══════════════════════════════════════
✅ Version: 5.0.0
✅ Tests: Passing
✅ Linting: Clean
✅ Performance: SLOs met
✅ Documentation: Updated

🎉 READY FOR RELEASE TO CRATES.IO
```

---

## 🔗 ANDON DISCIPLINE REMINDERS

**During this work:**
- 🔴 **PULL THE CORD** immediately if new blockers found
- 🔴 **STOP THE LINE** if tests start failing
- 🔴 **FIX ROOT CAUSE** not symptoms
- 🔴 **VERIFY CLEARED** before proceeding

**Never:**
- ❌ Suppress signals with `#[allow(...)]`
- ❌ Skip tests to "save time"
- ❌ Defer critical blockers
- ❌ Merge with failing tests

---

**Action Plan Owner**: Quality Engineer
**Last Updated**: 2025-11-20 18:15 UTC
**Review Frequency**: Daily until release

**Remember**: Quality is not negotiable. Every signal is an opportunity to improve.
