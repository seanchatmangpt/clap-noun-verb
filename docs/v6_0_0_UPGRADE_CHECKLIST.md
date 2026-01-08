# clap-noun-verb v6.0.0 Upgrade Checklist

**For**: End Users and Application Developers
**Time Required**: 30 minutes - 4 hours (depending on codebase size)
**Difficulty Level**: Intermediate

---

## Pre-Upgrade Assessment (5-10 minutes)

Determine your upgrade complexity before starting:

- [ ] **Project Size**
  - [ ] Small (< 10 handlers): ~30 minutes
  - [ ] Medium (10-50 handlers): ~1-2 hours
  - [ ] Large (50+ handlers): ~3-4 hours

- [ ] **Feature Usage**
  - [ ] Using custom telemetry? (adds 15-30 min)
  - [ ] Using custom handlers? (adds 20-40 min)
  - [ ] Using frontier features? (adds 5-10 min)
  - [ ] Using plugins? (adds 10-20 min)

- [ ] **Test Coverage**
  - [ ] Has test suite? (makes upgrade safer)
  - [ ] >80% coverage? (easier to verify)
  - [ ] < 50% coverage? (needs extra validation)

---

## Phase 1: Preparation (10 minutes)

### 1.1 Create Feature Branch

```bash
git checkout -b upgrade/v6.0.0
```

### 1.2 Backup Current State

```bash
git stash
# or
git tag pre-v6-upgrade
```

### 1.3 Review Breaking Changes

- [ ] Read [v6_0_0_RELEASE_NOTES.md](./v6_0_0_RELEASE_NOTES.md) - 5 min
- [ ] Skim [v6_0_0_MIGRATION_GUIDE.md](./v6_0_0_MIGRATION_GUIDE.md) - 5 min
- [ ] Identify which breaking changes affect your code:
  - [ ] Telemetry API (check `grep -r "TelemetryManager" src/`)
  - [ ] Custom handlers (check `impl.*Handler`)
  - [ ] Macro syntax (check `#[arg(...)]` patterns)
  - [ ] Feature flags (check `Cargo.toml` features)
  - [ ] Error handling (check `match.*Error`)

---

## Phase 2: Dependency Update (5 minutes)

### 2.1 Update Cargo.toml

```toml
[dependencies]
# Before
clap-noun-verb = "5.5"
clap-noun-verb-macros = "5.5"

# After
clap-noun-verb = "6.0"
clap-noun-verb-macros = "6.0"
```

### 2.2 Update Features

If you're using frontier features:

```toml
# Before
clap-noun-verb = { version = "5.5", features = [
    "frontier-learning",
    "frontier-discovery",
    "frontier-federated"
] }

# After
clap-noun-verb = { version = "6.0", features = ["frontier"] }
```

### 2.3 Verify Dependencies

```bash
cargo check
# Note: Will show compiler errors for breaking changes (this is expected)
```

---

## Phase 3: API Migration (varies)

Choose your migration path based on what features you use:

### 3A: Using Telemetry (15-30 min)

**Complexity**: Medium

If your code contains telemetry calls:

```bash
grep -r "TelemetryManager" src/
grep -r "span!" src/
grep -r ".record_span" src/
```

**Migration Steps**:

1. [ ] Identify all telemetry usage in your codebase
2. [ ] For each telemetry call, apply pattern from [Telemetry Migration](./v6_0_0_MIGRATION_GUIDE.md#telemetry-api-migration)
3. [ ] Replace `TelemetryManager::instance()` with `TelemetryManager::v2()`
4. [ ] Update span recording to builder pattern
5. [ ] Test telemetry with `cargo test`

**Quick Reference**:

```rust
// Old pattern (v5.5.0)
use clap_noun_verb::TelemetryManager;
let tm = TelemetryManager::instance();
tm.record_span("operation", duration);

// New pattern (v6.0.0)
use clap_noun_verb::telemetry::TelemetryManager;
TelemetryManager::v2()
    .span_builder("operation")
    .with_duration(duration)
    .record()?;
```

**Validation**: `cargo test --lib` should pass

### 3B: Using Custom Handlers (20-40 min)

**Complexity**: Medium-High

If you have custom trait implementations:

```bash
grep -r "impl.*Handler" src/
grep -r "HandlerInput" src/
grep -r "HandlerOutput" src/
```

**Migration Steps**:

1. [ ] Identify all custom handler implementations
2. [ ] Replace `VerbHandler` trait with `CommandHandler`
3. [ ] Update method signature: `handle(HandlerInput)` → `execute(&CommandArgs)`
4. [ ] Convert `HandlerOutput` to `CommandOutput`
5. [ ] Update registration code to use `AgentCliBuilder`
6. [ ] Test handlers with `cargo test`

**Quick Reference**:

```rust
// Old pattern (v5.5.0)
impl VerbHandler for MyHandler {
    fn handle(&self, input: HandlerInput) -> Result<HandlerOutput> {
        Ok(HandlerOutput::json(data))
    }
}

// New pattern (v6.0.0)
impl CommandHandler for MyHandler {
    fn execute(&self, args: &CommandArgs) -> Result<CommandOutput> {
        Ok(CommandOutput::json(data))
    }
}
```

**Validation**: `cargo test --lib` should pass

### 3C: Using Macro Constraints (10-20 min)

**Complexity**: Low-Medium

If you have constraint attributes in macros:

```bash
grep -r '#\[arg(' src/
grep -r 'group =' src/
grep -r 'requires =' src/
grep -r 'conflicts_with' src/
```

**Migration Steps**:

1. [ ] Identify all constraint attributes
2. [ ] Replace `#[arg(...)]` with doc comment tags
3. [ ] Convert constraint table (see [Macro Signature Updates](./v6_0_0_MIGRATION_GUIDE.md#macro-signature-updates))
4. [ ] Test macros with `cargo check`

**Quick Reference**:

```rust
// Old pattern (v5.5.0)
#[arg(group = "format", exclusive = true)]

// New pattern (v6.0.0)
/// Output format [group: format exclusive]
```

**Validation**: `cargo check` should succeed

### 3D: Using Error Handling (5-10 min)

**Complexity**: Low

If you match on error types:

```bash
grep -r "match.*Error" src/
grep -r "Error::" src/
```

**Migration Steps**:

1. [ ] Find all error pattern matches
2. [ ] Update variant names (ParsingFailed → Parsing, etc.)
3. [ ] Add new error variants if needed (Plugin, Configuration)
4. [ ] Test with `cargo test`

**Quick Reference**:

```rust
// Old patterns (v5.5.0)
Error::ParsingFailed(msg)
Error::ExecutionFailed(msg)

// New patterns (v6.0.0)
Error::Parsing(msg)
Error::Execution(msg)
Error::Plugin(err)
Error::Configuration(msg)
```

**Validation**: `cargo test` should pass

### 3E: No Breaking Changes Needed

If your code only uses basic noun/verb attributes:

```bash
grep -r '#\[noun' src/
grep -r '#\[verb' src/
```

And you're not using any advanced features above, you might be done!

**Validation**: `cargo check && cargo test`

---

## Phase 4: Compilation & Testing (15-30 min)

### 4.1 Initial Compilation Check

```bash
cargo check
```

**If you see errors**:
- [ ] Review error messages carefully
- [ ] Match error to one of Phase 3 categories above
- [ ] Apply appropriate migration from [Migration Guide](./v6_0_0_MIGRATION_GUIDE.md)
- [ ] Repeat until `cargo check` passes

### 4.2 Run Full Test Suite

```bash
cargo test
```

**If tests fail**:
- [ ] Review test failures
- [ ] Update test code for new APIs if needed
- [ ] Re-run until all tests pass

**Expected result**: ✅ All tests pass

### 4.3 Run Build System (if using cargo make)

```bash
cargo make check
cargo make test
cargo make lint
```

**Expected results**:
- ✅ `cargo make check` - No compiler errors/warnings
- ✅ `cargo make test` - All tests pass
- ✅ `cargo make lint` - No clippy warnings

---

## Phase 5: Verification (10-15 min)

### 5.1 Manual Testing

- [ ] Run your application with v6
- [ ] Test primary user workflows
- [ ] Verify command-line help still works
- [ ] Check error messages are clear
- [ ] Verify JSON output format (if applicable)

### 5.2 Performance Check

```bash
time cargo build --release
# Compare build time to v5.5.0 (should be faster)
```

Expected improvement: 30-50% faster builds

### 5.3 Telemetry Verification (if enabled)

If using telemetry:

- [ ] Verify telemetry spans are recorded
- [ ] Check telemetry output format
- [ ] Confirm no telemetry errors in logs

### 5.4 Documentation Check

- [ ] Update any internal docs mentioning v5.5.0
- [ ] Update CHANGELOG if maintaining one
- [ ] Document any custom migration notes

---

## Phase 6: Production Readiness (varies)

### 6.1 Release Planning

- [ ] Tag commit for internal release
- [ ] Update version in application
- [ ] Document v6 upgrade in release notes

### 6.2 Deployment Strategy

Choose your deployment approach:

**Option A: Full Cutover** (safest for small projects)
- [ ] Thoroughly test v6 build
- [ ] Deploy v6 to production
- [ ] Monitor for issues
- [ ] Keep v5.5.0 available for rollback

**Option B: Canary Deployment** (safest for large projects)
- [ ] Deploy v6 to subset of servers (10%)
- [ ] Monitor metrics (errors, latency, throughput)
- [ ] Gradually increase rollout (25% → 50% → 100%)
- [ ] Keep v5.5.0 available for rollback

**Option C: Blue-Green Deployment** (best for zero-downtime)
- [ ] Set up parallel v6 environment
- [ ] Route traffic to v6 (using load balancer)
- [ ] Monitor v6 deployment
- [ ] Switch all traffic when confident
- [ ] Keep v5.5.0 environment ready for quick rollback

### 6.3 Monitoring & Observability

- [ ] Set up alerts for error rate increases
- [ ] Monitor latency percentiles (p50, p95, p99)
- [ ] Track command execution times
- [ ] Monitor resource usage (CPU, memory)
- [ ] Set rollback triggers (e.g., error rate > 1%)

### 6.4 Post-Deployment

- [ ] Monitor for 24 hours
- [ ] Check application logs for issues
- [ ] Verify telemetry data
- [ ] Get team feedback
- [ ] Celebrate successful upgrade!

---

## Rollback Plan

If you encounter critical issues:

### Quick Rollback

```bash
# Revert to v5.5.0
git checkout pre-v6-upgrade
cargo update clap-noun-verb --precise 5.5.0
cargo build --release
# Redeploy v5.5.0
```

### Known Issues Before Rollback

Check [Release Notes](./v6_0_0_RELEASE_NOTES.md#known-issues--workarounds) for workarounds before rolling back.

---

## Post-Upgrade Validation

### Quick Smoke Test

```bash
# Run this checklist after deploying v6

# 1. Application starts
./myapp --help
# Expected: Displays help correctly

# 2. Basic command works
./myapp noun verb --arg value
# Expected: Returns valid JSON/output

# 3. Error handling
./myapp noun verb --bad-arg
# Expected: Clear error message

# 4. Telemetry (if enabled)
# Check your telemetry backend for spans
# Expected: Spans recorded with v6 format
```

### Metrics to Watch

- **Error Rate**: Should stay < 0.1%
- **Latency**: Should be same or faster
- **Success Rate**: Should stay > 99.9%
- **CPU Usage**: May decrease (more efficient)
- **Memory Usage**: May decrease (smaller binary)

---

## Support & Help

### If Upgrade Fails

1. Check [Troubleshooting](./v6_0_0_MIGRATION_GUIDE.md#troubleshooting) section
2. Review [COMMON_MISTAKES.md](../COMMON_MISTAKES.md)
3. Search [GitHub Issues](https://github.com/seanchatmangpt/clap-noun-verb/issues)
4. Ask in [GitHub Discussions](https://github.com/seanchatmangpt/clap-noun-verb/discussions)

### Additional Resources

- [v6.0.0 Release Notes](./v6_0_0_RELEASE_NOTES.md) - Feature details
- [v6.0.0 Migration Guide](./v6_0_0_MIGRATION_GUIDE.md) - Detailed migration
- [Common Mistakes](../COMMON_MISTAKES.md) - Error patterns
- [API Reference](https://docs.rs/clap-noun-verb/6.0) - Full API docs

---

## Completion Checklist

Once all phases complete, verify:

- [ ] `cargo check` passes with no errors
- [ ] `cargo test` passes with 100% success
- [ ] `cargo make lint` passes with no warnings
- [ ] Manual testing of key workflows succeeds
- [ ] Performance is same or better
- [ ] Telemetry (if used) is working
- [ ] No regression in error handling
- [ ] Documentation updated
- [ ] Team is trained on new APIs
- [ ] Rollback plan documented

**Final Status**: ✅ **Ready for Production**

---

## Summary

| Phase | Duration | Key Tasks |
|-------|----------|-----------|
| **1. Preparation** | 10 min | Assessment, branching, review |
| **2. Dependency Update** | 5 min | Update Cargo.toml, features |
| **3. API Migration** | Varies | Telemetry, handlers, macros, errors |
| **4. Compilation & Testing** | 15-30 min | Fix errors, run tests |
| **5. Verification** | 10-15 min | Manual testing, performance, telemetry |
| **6. Production Readiness** | Varies | Deploy, monitor, prepare rollback |

**Total Time**: 1-4 hours depending on project complexity

**Questions?** Open an issue or check the troubleshooting guide!
