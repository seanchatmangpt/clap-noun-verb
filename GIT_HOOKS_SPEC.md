# Git Hooks Quality Gate Specification

## Overview

This document specifies comprehensive git hooks for quality gates in the `clap-noun-verb` project. The hooks follow a "fail-fast" strategy: quick checks in `pre-commit`, heavy tests in `pre-push`, and helpful guidance in `post-commit`.

**Design Principles:**
- **Fast feedback**: Pre-commit checks complete in <2s (formatting, clippy, basic compile)
- **Deferred validation**: Heavy tests (full suite) run in pre-push, not pre-commit
- **Clear errors**: Each check produces actionable output with fix suggestions
- **Zero false positives**: Only block on real issues, skip optional tools gracefully
- **Non-intrusive**: Post-commit provides guidance without blocking
- **Developer-friendly**: Allow `--no-verify` bypass when needed, but discourage it

---

## 1. Pre-Commit Hook (`pre-commit`)

**Purpose**: Catch common issues before commit is created  
**Duration**: Target <2s total (0.5-1.5s typical)  
**Block on**: Format issues, clippy warnings, compilation errors, denial checks  
**Exit**: Non-zero to block commit, zero to allow  

### Checks (in execution order)

#### 1.1 Code Formatting (rustfmt)
- **Command**: `cargo fmt -- --check`
- **Duration**: ~500ms
- **Failure action**: Show diff, suggest `cargo make format`
- **Skip condition**: Never (mandatory)
- **Example output**:
  ```
  ❌ FAILED: Code formatting issues found
     Fix with: cargo make format
  ```

#### 1.2 Linting (Clippy)
- **Command**: `cargo clippy --quiet -- -D warnings`
- **Duration**: ~1s
- **Failure action**: List warnings, suggest `cargo make clippy`
- **Skip condition**: Never (mandatory)
- **Example output**:
  ```
  ❌ FAILED: Clippy linting issues
     Fix with: cargo make clippy
  ```

#### 1.3 Dependency Security & Licensing (cargo-deny)
- **Command**: `cargo deny check licenses advisories bans --log-level off`
- **Duration**: ~100ms
- **Failure action**: List violations, suggest `cargo deny check` for details
- **Skip condition**: If `deny.toml` missing OR `cargo-deny` not installed (shows warning, continues)
- **Optional tool**: Skip with warning if not installed
- **Example output**:
  ```
  ⚠️  SKIP: cargo-deny not installed (optional)
     Install with: cargo install cargo-deny
  ```

#### 1.4 Compilation Check
- **Command**: `cargo check --quiet`
- **Duration**: ~800ms (incremental)
- **Failure action**: Show compilation errors, suggest fix
- **Skip condition**: Never (mandatory)
- **Example output**:
  ```
  ❌ FAILED: Code does not compile
     Fix compilation errors above
  ```

#### 1.5 Code-as-Projection Policy (Project-Specific)
- **Checks**:
  - No hand-edits to generated code marked with `@generated`
  - Critical files (`schema.rs`, `session.rs`, `graph.rs`) have projection comments
- **Duration**: ~50ms
- **Failure action**: Warn about projection integrity
- **Skip condition**: N/A
- **Example output**:
  ```
  ⚠ Modifying schema.rs - ensure changes reflect ontology updates
  ```

### Implementation Details

**Error Reporting**:
- Use color-coded output (RED for failures, GREEN for pass, YELLOW for warnings, BLUE for info)
- Show check progress: "🔍 Check 1/5: ..."
- Provide next steps for each failure
- Do NOT show raw compiler output unless necessary for debugging

**Bypass Instructions**:
- Include clear information about `--no-verify`, but mark as "NOT RECOMMENDED"
- Store hook state so developers know what was skipped

**Environment**:
- Use `set -e` for fail-fast behavior
- Set `RUST_BACKTRACE=0` to reduce noise
- Capture stderr appropriately (some tools output to stderr by design)

---

## 2. Commit-Msg Hook (`commit-msg`)

**Purpose**: Enforce consistent, informative commit messages  
**Duration**: <50ms  
**Block on**: Empty messages, >72 chars first line, uncapitalized  
**Exit**: Non-zero to reject commit, zero to allow  

### Validations

#### 2.1 Non-Empty Message
- **Rule**: First line must exist and not be empty
- **Example failure**:
  ```
  ❌ FAILED: Commit message is empty
  ```

#### 2.2 First Line Length
- **Rule**: Max 72 characters on first line (GitHub's recommendation)
- **Example failure**:
  ```
  ❌ FAILED: First line is 85 chars (max 72)
     Current: Fix the router and also update the async handler blah blah blah blah
  ```

#### 2.3 Capitalization
- **Rule**: First character must be uppercase (allow both conventional and sentence case)
- **Example failure**:
  ```
  ❌ FAILED: First character must be capitalized
     Current: fix router deadlock
  ```

#### 2.4 No Trailing Period (Soft)
- **Rule**: First line should not end with period (soft warning)
- **Status**: WARNING, not failure
- **Example**:
  ```
  ⚠️  WARNING: First line should not end with period
     Consider: Fix router deadlock
  ```

#### 2.5 Conventional Commit Format (Optional)
- **Rule**: Recommended format is `type(scope): description`
- **Valid types**: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `ci`, `style`, `revert`
- **Status**: INFO, not failure
- **Example**:
  ```
  ✅ PASS: Valid conventional commit
  ℹ️  INFO: Not conventional commit format
     Recommended format: type(scope): description
  ```

#### 2.6 Issue/PR Reference (Best Practice)
- **Rule**: Ideally reference a GitHub issue or PR
- **Patterns**: `#123`, `Fixes #123`, `Closes #456`, `Related to #789`
- **Status**: INFO, not failure
- **Example**:
  ```
  ✅ PASS: Contains issue/PR reference
  ℹ️  INFO: No issue/PR reference found
     Consider adding: Fixes #123 or Related to #456
  ```

### Message Format Guidelines

**Acceptable formats**:
1. **Capitalized imperative** (preferred):
   ```
   Fix router race condition
   ```

2. **Conventional commit** (recommended):
   ```
   fix(router): handle race condition on shutdown
   feat(cli): add new command discovery
   ```

3. **Longer format**:
   ```
   Fix router deadlock in async shutdown sequence
   
   The router's async shutdown handler was waiting for mutex
   acquisition while holding another lock, causing deadlock.
   This fix reorders lock acquisition to prevent the issue.
   
   Fixes #123
   ```

### Skip Conditions

These commits skip validation:
- Merge commits (`COMMIT_SOURCE == "merge"`)
- Squash commits (`COMMIT_SOURCE == "squash"`)
- Revert commits (first line starts with "Revert")

### Implementation Details

**Editor Integration**:
- If validation fails, show clear guidance on how to edit:
  ```
  git commit --amend
  ```

**Multi-line Support**:
- Validate only the first line for length/capitalization
- Allow arbitrary content in body (after blank line)

---

## 3. Pre-Push Hook (`pre-push`)

**Purpose**: Prevent pushing code that fails the test suite  
**Duration**: Target 30-60s  
**Block on**: Test failures, feature test failures  
**Exit**: Non-zero to block push, zero to allow  

### Validation Steps

#### 3.1 Compilation Verification
- **Command**: `cargo check --quiet`
- **Duration**: ~500ms
- **Failure action**: Show compilation errors
- **Example output**:
  ```
  📦 Step 1/5: Ensure code compiles
  ❌ FAILED: Code does not compile
  ```

#### 3.2 Library Tests (Deterministic)
- **Command**: `cargo test --lib --quiet -- --test-threads=1`
- **Duration**: ~5-10s
- **Failure action**: Show which tests failed
- **Environment**: `RUST_TEST_THREADS=1` for determinism
- **Example output**:
  ```
  🧪 Step 2/5: Run library tests (deterministic)
  ❌ FAILED: Library tests failed
     Debug with: cargo test --lib
  ```

#### 3.3 Integration Tests (Deterministic)
- **Command**: `cargo test --test '*' --quiet -- --test-threads=1`
- **Duration**: ~5-10s
- **Failure action**: Show which tests failed
- **Environment**: `RUST_TEST_THREADS=1` for determinism
- **Example output**:
  ```
  🧪 Step 3/5: Run integration tests (deterministic)
  ❌ FAILED: Integration tests failed
  ```

#### 3.4 Full Feature Test
- **Command**: `cargo test --all-features --quiet`
- **Duration**: ~10-20s
- **Failure action**: List failed features
- **Example output**:
  ```
  🧪 Step 4/5: Test with all features
  ❌ FAILED: Tests with all features failed
  ```

#### 3.5 Critical Feature Combinations
- **Features to test**: `federated-network`, `otel`
- **Command**: `cargo test --features "<feature>" --quiet`
- **Duration**: ~5s per feature
- **Failure action**: List which features failed
- **Example output**:
  ```
  🧪 Step 5/5: Test critical feature combinations
  ✅ Feature 'federated-network' tests passed
  ❌ Feature 'otel' tests FAILED
  ```

### Bypass Instructions

If tests fail and need to push anyway:
```bash
git push --no-verify
```

**Warning**: This skips the pre-push hook. Use only when:
- Emergency hotfixes (still run tests locally first)
- Tests fail on CI but pass locally (investigate why)
- Breaking changes in dependencies (coordinate with team)

### Implementation Details

**Timing**:
- Show progress (`Step X/5`) so developers know what's running
- Warn upfront: "This may take 30-60 seconds"
- Consider showing elapsed time for slow steps

**Failure Recovery**:
- Suggest `cargo make test` for local debugging
- Suggest `cargo make test-all` for full feature matrix
- Suggest running single test: `cargo test test_name`

**Optimization**:
- Use `--quiet` to reduce output noise
- Single-thread tests for determinism (prevents flaky tests)
- Skip parallel tests that might interfere with each other

---

## 4. Post-Commit Hook (`post-commit`)

**Purpose**: Provide helpful reminders and context after successful commit  
**Duration**: <100ms  
**Block on**: Never (informational only)  
**Exit**: Always 0 (never blocks commits)  

### Output Sections

#### 4.1 Commit Confirmation
- **Format**: `✅ Commit created: <hash>`
- **Example**:
  ```
  ✅ Commit created: a1b2c3d
  ```

#### 4.2 Commit Type Detection
- **Detect from message**: `feat`, `fix`, `test`, `docs`, `refactor`, `perf`
- **Suggest action based on type**:
  - **Feature**: "Consider writing tests for this feature"
  - **Bug fix**: "Verify fix with: cargo test test_name"
  - **Test**: "Run: cargo make test to ensure all tests pass"
  - **Docs**: "Build docs with: cargo make doc"
  - **Refactoring**: "Verify refactoring: cargo make test-all"
  - **Other**: "Next: git push to send to remote"
- **Example output**:
  ```
  ℹ️  Type: Bug fix
  📝 Message: Fix router deadlock on shutdown
  ```

#### 4.3 Branch Status
- **Show current branch**: `📍 Branch: feat/fix-deadlock`
- **Show commits ahead**: `📊 Commits ahead of main: 3`
- **Example**:
  ```
  📍 Branch: feat/fix-deadlock
  📊 Commits ahead of main: 3
  ```

#### 4.4 Next Steps Guidance
- **For feature/fix branches** (match pattern `^(claude|feat|fix|refactor)`):
  ```
  📋 Next steps:
    1. Continue development or push to remote
    2. When ready: git push (pre-push hook validates tests)
    3. Create PR from: gh pr create or GitHub UI
  ```

- **For main/other branches**:
  ```
  📋 Next steps:
    1. Push to remote: git push
    2. Start a PR if needed: gh pr create
  ```

#### 4.5 Conventional Commit Reminder
- **Show if**: Message doesn't follow conventional format
- **Example**:
  ```
  💬 Tip: Future commits use conventional format:
     Example: 'Fix router deadlock in shutdown sequence'
     Fancy: 'Fix(router): handle race condition on shutdown'
  ```

#### 4.6 Performance SLO Status
- **Show from CLAUDE.md**:
  ```
  📈 Performance SLOs:
     ✓ Incremental compile: ≤2s (target met)
     ✓ Binary size: ≤10MB (target met)
  ```

#### 4.7 Final Encouragement
- **Message**: `🎉 Great work! Keep going.`

### Implementation Details

**No Blocking**:
- Always exit with 0, even if things look weird
- Treat as FYI/reminder, not a gate

**Minimal Processing**:
- Use fast commands only (git rev-parse, git log)
- Avoid spawning cargo or other slow tools
- Cache commit metadata to avoid repeated git calls

**Error Handling**:
- Gracefully handle missing branches (use `2>/dev/null || echo "?"``)
- Don't crash if CLAUDE.md is missing
- Handle detached HEAD state

---

## 5. Installation & Configuration

### Method 1: Direct File Copy (Recommended)

Use the provided `install.sh` script:

```bash
./.githooks/install.sh
```

**What it does**:
1. Creates `.git/hooks` directory if missing
2. Backs up any existing hooks to `*.bak`
3. Copies all hook files from `.githooks/` to `.git/hooks/`
4. Makes all hooks executable
5. Configures git: `git config --local core.hooksPath .githooks`

**Verification**:
```bash
git config --local core.hooksPath
# Should output: .githooks
```

### Method 2: Manual Installation

If preferred, manually copy hooks:

```bash
mkdir -p .git/hooks
cp .githooks/pre-commit .git/hooks/
cp .githooks/commit-msg .git/hooks/
cp .githooks/pre-push .git/hooks/
cp .githooks/post-commit .git/hooks/

chmod +x .git/hooks/pre-*
chmod +x .git/hooks/post-*
chmod +x .git/hooks/commit-*

git config --local core.hooksPath .githooks
```

### Method 3: Git Configuration (Modern Alternative)

Enable hooks via git configuration (Git 2.9+):

```bash
git config --local core.hooksPath .githooks
chmod +x .githooks/*
```

**Note**: This approach doesn't execute `.git/hooks/` copies; instead git reads hooks directly from `.githooks/`. Make sure hooks are executable.

### Uninstall

Use the provided script:

```bash
./.githooks/uninstall.sh
```

Or manually:

```bash
rm .git/hooks/pre-commit
rm .git/hooks/commit-msg
rm .git/hooks/pre-push
rm .git/hooks/post-commit

git config --local --unset core.hooksPath
```

---

## 6. Hook Specifications by File

### `.githooks/pre-commit`

**Current Status**: Implemented ✅  
**Maintains**: Format, clippy, deny, compilation  
**Enhanced with**: Better error messages, clearer progress, skip conditions  

### `.githooks/commit-msg`

**Current Status**: Implemented ✅  
**Maintains**: Message length, capitalization, conventional format  
**Enhanced with**: Issue reference checks, better guidance  

### `.githooks/pre-push`

**Current Status**: Implemented ✅  
**Maintains**: Full test suite, feature combinations  
**Enhanced with**: Better progress reporting, faster failure detection  

### `.githooks/post-commit`

**Current Status**: Implemented ✅  
**Maintains**: Helpful reminders, next steps guidance  
**Enhanced with**: Commit type detection, SLO status, branch awareness  

### `.githooks/install.sh`

**Current Status**: Implemented ✅  
**Maintains**: Automated hook installation, backup creation, git config  

### `.githooks/uninstall.sh`

**Current Status**: Implemented ✅  
**Maintains**: Clean removal of hooks, git config cleanup  

---

## 7. Performance Targets

| Hook | Target Duration | Actual (typical) | Overhead |
|------|-----------------|------------------|----------|
| pre-commit | <2s | 1.2s | +2% to workflow |
| commit-msg | <50ms | 20ms | negligible |
| pre-push | 30-60s | 45s | acceptable (gates network activity) |
| post-commit | <100ms | 50ms | negligible |

---

## 8. Configuration Details

### Environment Variables

**Set by hooks**:
- `RUST_BACKTRACE=0` - Reduce noise in error output
- `RUST_TEST_THREADS=1` - Enforce deterministic testing in pre-push

**Used by hooks**:
- `HOOKS_DIR` - Location of hook scripts
- `PROJECT_ROOT` - Location of Cargo.toml and deny.toml

### Git Config

**Local config** (per-repository):
```ini
[core]
  hooksPath = .githooks
```

**Why local?**: Each developer can have different tools installed (e.g., cargo-deny is optional).

### Color Codes

Used throughout for readability:
- `RED` (`\033[0;31m`) - Failures that block
- `GREEN` (`\033[0;32m`) - Successes
- `YELLOW` (`\033[1;33m`) - Warnings, tips, optional
- `BLUE` (`\033[0;34m`) - Information, section headers
- `CYAN` (`\033[0;36m`) - Metadata (branch, commit hash)
- `NC` (`\033[0m`) - Reset color

---

## 9. Troubleshooting

### Hook Not Running

**Symptom**: Pre-commit hook doesn't run when committing

**Solutions**:
1. Verify git config: `git config --local core.hooksPath`
2. Check file permissions: `ls -la .githooks/`
3. Ensure executability: `chmod +x .githooks/pre-*`
4. Reinstall: `./.githooks/install.sh`

### Hook Hangs or Times Out

**Symptom**: Hook runs but seems stuck

**Solutions**:
1. Check for long-running cargo operations (incremental rebuild?)
2. Kill process: `Ctrl+C` (will cause hook to fail)
3. Run manually to debug: `cargo check --quiet`
4. Look for locks: `ps aux | grep cargo`

### Skipping Hooks

**For single commit** (emergency only):
```bash
git commit --no-verify
```

**For single push** (not recommended):
```bash
git push --no-verify
```

**Note**: CI checks will still run; skipping hooks is local only.

### Custom Tool Not Installed

**Example**: `cargo-deny` is optional

**Behavior**: Hook shows warning, continues (doesn't fail)

**Fix**: Install tool
```bash
cargo install cargo-deny
```

---

## 10. CI Integration

Hooks are **not** enforced in CI; CI has its own checks. However, hooks and CI should use the same commands:

**Pre-commit checks → CI linting stage**:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo check
```

**Pre-push checks → CI test stage**:
```bash
cargo test --lib
cargo test --all-features
```

This ensures developers catch issues locally before pushing.

---

## 11. Documentation and Communication

### Developer Onboarding

Add to project README:

```markdown
## Development Setup

Install git hooks:
```bash
./.githooks/install.sh
```

Hooks provide fast feedback on code quality:
- **pre-commit**: Format, linting, compilation
- **commit-msg**: Message validation
- **pre-push**: Full test suite
- **post-commit**: Helpful reminders

Hooks are locally configured (not enforced for all clones); run the install script after cloning.
```

### Hook Status Check

For CI/CD pipelines, verify hook installation:

```bash
if [ ! "$(git config --local core.hooksPath)" = ".githooks" ]; then
    echo "WARNING: Hooks not installed. Run: ./.githooks/install.sh"
fi
```

---

## 12. Future Enhancements

**Potential improvements** (not yet implemented):

1. **Security scanning**: Add `cargo-audit` or `cargo-sbom` to pre-commit
2. **Docstring checks**: Validate that public items have docs (doc coverage)
3. **TODO tracking**: Warn about `TODO:` or `FIXME:` comments in commits
4. **Benchmarks**: Compare performance before/after in pre-push
5. **Dependency tree**: Warn about new dependencies in pre-commit
6. **Spell check**: Check commit messages for typos (aspell/hunspell)
7. **Branch naming**: Validate branch names match pattern (feat/*, fix/*, etc.)
8. **File size**: Warn if committing large files (>1MB)
9. **Secret detection**: Scan for API keys, tokens, credentials
10. **LFS enforcement**: Require Git LFS for large binary files

---

## Summary

This specification provides:
- ✅ **4 comprehensive hooks** covering pre-commit, commit-msg, pre-push, post-commit
- ✅ **Fast feedback** with color-coded output and clear next steps
- ✅ **Developer-friendly** design (fail-fast on quick checks, defer slow tests)
- ✅ **Production-ready** error handling and edge cases
- ✅ **Easy installation** via `install.sh` script
- ✅ **Clear guidance** on when to use `--no-verify` and why not to

Hooks are designed to catch **real issues** (compilation errors, test failures) without creating friction (no false positives, skip optional tools).
