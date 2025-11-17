# v4.0.0 Release - Priority Action Items

**Status**: CONDITIONAL - DO NOT RELEASE until P0 blockers fixed
**Estimated Time to Release**: 2-3 days (P0 only) or 1-2 weeks (P0+P1)

---

## P0 BLOCKERS (Must Fix Before Release)

### 1. Fix Cargo.toml Lint Violations
- **Issue**: 50+ unwrap/expect/panic violations despite deny-level lints
- **Effort**: 8-12 hours
- **Owner**: TBD
- **Files**: See full report for list of 20+ files
- **Fix**: Add `#[allow(clippy::unwrap_used)]` to test modules
- **Test**: `cargo clippy --all-targets --all-features -- -D warnings`

**Quick Fix Script**:
```bash
# Add to all test modules
find src -name "*.rs" -exec sed -i '' '/#\[cfg(test)\]/a\
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]\
' {} \;
```

### 2. Fix Example Compilation Failures
- **Issue**: io_advanced.rs and autonomic_example.rs don't compile
- **Effort**: 2-3 hours
- **Owner**: TBD
- **Errors**:
  - `io_advanced.rs`: borrow of moved value `inputs`
  - `autonomic_example.rs`: missing `LoggingMiddleware`, `ReadOnlyFS`
- **Fix**: Review ownership or remove incomplete examples
- **Test**: `cargo build --examples --all-features`

### 3. Fix Doc Test Failures
- **Issue**: 3 doc tests fail (simd, const_caps, typestate)
- **Effort**: 3-4 hours
- **Owner**: TBD
- **Files**:
  - src/kernel/simd.rs line 84
  - src/kernel/const_caps.rs line 255
  - src/kernel/typestate.rs line 43
- **Fix**: Fix examples or mark as `ignore`/`no_run`
- **Test**: `cargo test --doc`

**Total P0 Effort**: 13-19 hours (2-3 days)

---

## P1 HIGH PRIORITY (Strongly Recommended)

### 4. Fix Vec<String> Parsing in Proc Macro
- **Issue**: Documented feature doesn't work
- **Effort**: 6-8 hours
- **Owner**: TBD
- **Details**: See docs/VEC_STRING_PARSING_ISSUE.md
- **Impact**: Users must use workaround
- **Fix**: Enhance proc macro parser in clap-noun-verb-macros

### 5. Remove Dead Code in Macros
- **Issue**: Entire io_detection module unused (10 warnings)
- **Effort**: 1-2 hours (removal) or 8-12 hours (completion)
- **Owner**: TBD
- **Fix**: Remove or complete feature
- **Test**: `cargo build -p clap-noun-verb-macros`

### 6. Document Kani Configuration
- **Issue**: 10+ warnings about unexpected cfg(kani)
- **Effort**: 1 hour
- **Owner**: TBD
- **Fix**: Add to Cargo.toml:
```toml
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(kani)'] }
```

**Total P1 Effort**: 8-11 hours (1-2 days)

---

## P2 MEDIUM PRIORITY (Improvements)

### 7. Clean Up Unused Imports
- **Effort**: 1-2 hours
- **Fix**: `cargo fix --allow-dirty --allow-staged`
- **Test**: `cargo check --all-targets`

### 8. Fix cfg(feature = "tracing") Warnings
- **Effort**: 30 minutes
- **Fix**: Add tracing feature to Cargo.toml or remove cfg

---

## QUICK WINS (Do These First)

1. **Remove unused imports** (15 minutes)
   ```bash
   cargo fix --lib --allow-dirty
   ```

2. **Add Kani cfg** (10 minutes)
   ```toml
   # Add to Cargo.toml [lints.rust]
   unexpected_cfgs = { level = "warn", check-cfg = ['cfg(kani)'] }
   ```

3. **Fix simple lint violations** (30 minutes)
   - Add #[allow] attributes to test code

---

## RELEASE CHECKLIST

### Before Release:
- [ ] All P0 blockers fixed
- [ ] All examples compile: `cargo build --examples`
- [ ] All tests pass: `cargo test --all`
- [ ] Doc tests pass: `cargo test --doc`
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Docs build: `cargo doc --no-deps`
- [ ] Package builds: `cargo package --allow-dirty`

### Recommended Before Release:
- [ ] P1 issues fixed (Vec<String>, dead code, Kani)
- [ ] P2 issues fixed (unused imports, cfg warnings)
- [ ] CHANGELOG.md updated
- [ ] Version bumped to 4.0.0
- [ ] Migration guide written

### After Release:
- [ ] Publish to crates.io
- [ ] Create GitHub release
- [ ] Update documentation site
- [ ] Announce on social media
- [ ] Monitor for early adopter issues

---

## TESTING COMMANDS

```bash
# Full validation suite
cargo clean
cargo build --all-features
cargo build --examples --all-features
cargo test --all
cargo test --doc
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps

# Quick validation
cargo check --all-targets
cargo test --lib
cargo build --examples
```

---

## RISK ASSESSMENT

**If Released Now (Without Fixes)**:
- HIGH RISK: Builds fail in CI, examples broken, trust damage
- Impact: Support burden, credibility loss, rollback needed

**After P0 Fixes Only**:
- MEDIUM RISK: Some rough edges (Vec<String>, warnings)
- Impact: Manageable, can fix in v4.0.1

**After P0 + P1 Fixes**:
- LOW RISK: Production-ready, minor polish needed
- Impact: Strong foundation for v4.x series

---

## ESTIMATED TIMELINE

### Minimum (P0 Only):
- Day 1: Fix lint violations (8h)
- Day 2: Fix examples (3h) + doc tests (4h)
- Day 3: Final testing and release prep (4h)
- **Total**: 2-3 days

### Recommended (P0 + P1):
- Week 1: P0 blockers (2-3 days)
- Week 2: P1 issues (2-3 days)
- Week 3: Testing and release (1-2 days)
- **Total**: 1-2 weeks

### Ideal (All Issues):
- Weeks 1-2: Critical and high priority
- Week 3: Medium priority and polish
- Week 4: Final testing and release
- **Total**: 3-4 weeks

---

## DECISION MATRIX

| Timeline Goal | Fixes Required | Risk Level | Recommendation |
|---------------|----------------|------------|----------------|
| Release ASAP | None | HIGH | DO NOT DO |
| Release in 3 days | P0 only | MEDIUM | Acceptable |
| Release in 2 weeks | P0 + P1 | LOW | Recommended |
| Release in 1 month | All issues | VERY LOW | Ideal |

---

## CONTACT & TRACKING

- **Full Report**: docs/v4_0_0_VALIDATION_REPORT.md
- **Known Issues**: docs/VEC_STRING_PARSING_ISSUE.md
- **GitHub Issues**: (Create issues for each action item)
- **Project Board**: (Track progress on board)

---

**Last Updated**: 2025-11-16
**Next Review**: After P0 blockers fixed
