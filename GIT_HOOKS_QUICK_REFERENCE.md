# Git Hooks Quick Reference

Fast lookup guide for developers using git hooks in `clap-noun-verb`.

---

## Installation

**One-time setup:**
```bash
./.githooks/install.sh
```

**Verify:**
```bash
git config --local core.hooksPath
# Should output: .githooks
```

---

## Hook Lifecycle

```
git commit
    ↓
Pre-commit hook runs (1.2s) ← FORMAT, CLIPPY, DENY, COMPILE
    ↓
    ✅ PASS → Commit created
    ↓
Post-commit hook runs (50ms) ← HELPFUL REMINDERS, SUGGESTIONS
    ↓
    Developer makes more commits or runs:
    ↓
git push
    ↓
Pre-push hook runs (45s) ← FULL TEST SUITE, ALL FEATURES
    ↓
    ✅ PASS → Push to remote
    ✅ FAIL → Fix tests locally, retry push
```

---

## Quick Troubleshooting

| Problem | Solution |
|---------|----------|
| Hook doesn't run | `./.githooks/install.sh` |
| Formatting fails | `cargo make format` |
| Clippy complains | `cargo make clippy` |
| Tests fail on push | `cargo make test` locally first |
| Need to skip hook | `git commit --no-verify` (not recommended) |
| Uninstall hooks | `./.githooks/uninstall.sh` |

---

## Pre-Commit Hook

**When**: Before commit is created  
**Speed**: ~1.2s  
**Blocks**: Yes (if failures)  

**What it checks**:
1. Code formatting (`cargo fmt`)
2. Linting (`cargo clippy`)
3. License/security (`cargo deny`)
4. Compilation (`cargo check`)
5. Project-specific policies

**If it fails**:
```
Follow the suggestions shown (e.g., "cargo make format")
git add <fixed files>
git commit  # Try again
```

---

## Commit-Msg Hook

**When**: Before commit message is finalized  
**Speed**: <50ms  
**Blocks**: Yes (if failures)  

**What it validates**:
- ✅ Message is not empty
- ✅ First line ≤72 characters
- ✅ First character is capitalized
- ℹ️ Conventional commit format (recommended, not required)
- ℹ️ Issue/PR reference (best practice)

**Message examples** (all valid):
```
Fix router deadlock on shutdown
feat(router): handle concurrent shutdown requests
fix(cli): Resolve race condition in command parsing
```

**If it fails**:
```bash
git commit --amend  # Edit message
```

---

## Pre-Push Hook

**When**: Before push to remote  
**Speed**: 30-60 seconds  
**Blocks**: Yes (if failures)  

**What it validates**:
1. Code compiles
2. Library tests pass (single-threaded)
3. Integration tests pass (single-threaded)
4. All features work together
5. Critical feature combinations work

**If tests fail locally**:
```bash
cargo make test              # Run tests (quick)
cargo make test-all          # Run with all features
cargo test test_name         # Run single test
cargo test test_name --lib   # Run single test with output
```

**If you must push anyway** (not recommended):
```bash
git push --no-verify
```

---

## Post-Commit Hook

**When**: After successful commit  
**Speed**: ~50ms  
**Blocks**: Never (informational only)  

**What it shows**:
- ✅ Commit created (hash + message)
- 📋 Commit type (feature, fix, test, etc.)
- 📍 Current branch
- 📊 Commits ahead of main
- 💡 Type-specific suggestions
- 📈 Performance SLO status

**Example output**:
```
✅ Commit created: a1b2c3d

ℹ️  Type: Bug fix
📝 Message: Fix router deadlock on shutdown

📍 Branch: feat/fix-deadlock
📊 Commits ahead of main: 3

💡 Suggestion: Verify fix with: cargo test test_name

📋 Next steps:
  1. Continue development or push to remote
  2. When ready: git push (pre-push hook validates tests)
  3. Create PR from: gh pr create or GitHub UI

🎉 Great work! Keep going.
```

---

## Common Commands

### Commit workflow
```bash
# Make changes
git add .

# Try to commit (pre-commit runs automatically)
git commit -m "Fix router deadlock"

# If it fails, fix issues and retry
cargo make format
git add .
git commit -m "Fix router deadlock"

# Push (pre-push runs automatically)
git push

# If tests fail, fix locally and retry
cargo test --lib
git add .
git commit --amend  # Add fix to previous commit
git push
```

### Bypassing hooks (emergency only)
```bash
# Skip pre-commit
git commit --no-verify -m "message"

# Skip pre-push
git push --no-verify

# Both (⚠️ really not recommended)
git commit --no-verify && git push --no-verify
```

### Testing hooks manually
```bash
# Force pre-commit check
cargo fmt --check
cargo clippy -- -D warnings
cargo deny check

# Force pre-push check
cargo test --lib -- --test-threads=1
cargo test --all-features

# Test commit-msg hook
git commit --allow-empty -m "Test message"

# Test post-commit hook
git commit --allow-empty -m "Test" && echo "(check output above)"
```

---

## Performance Expectations

| Operation | Time | What's happening |
|-----------|------|------------------|
| `git commit` | 1.2s | Format, clippy, deny, compile |
| `git push` | 45s | Library tests, integration tests, feature tests |
| `git commit --no-verify` | 0.1s | No checks (emergency only) |

---

## Commit Message Guide

### Format
```
<type>(<scope>): <description>

<body (optional)>

<footer (optional)>
```

### Types
- `feat` - New feature
- `fix` - Bug fix
- `refactor` - Code refactoring
- `perf` - Performance improvement
- `docs` - Documentation changes
- `test` - Test additions/changes
- `chore` - Build, CI, dependency updates
- `style` - Formatting (shouldn't trigger format hook)
- `ci` - CI/CD changes
- `revert` - Reverts a previous commit

### Examples

**Simple**:
```
Fix router deadlock on shutdown
```

**Conventional**:
```
fix(router): handle concurrent shutdown requests
```

**Detailed**:
```
fix(router): handle concurrent shutdown requests

The router was waiting for mutex acquisition while holding
another lock, causing deadlock. This fix reorders lock
acquisition to prevent the issue.

Fixes #123
```

### Rules
- ✅ Start with capital letter
- ✅ Use imperative mood ("Fix" not "Fixed")
- ✅ Keep first line ≤72 characters
- ✅ Add blank line between subject and body
- ✅ Reference issues: `Fixes #123`, `Related to #456`
- ❌ Don't end first line with period
- ❌ Don't use all caps

---

## Git Config Status

Check current configuration:

```bash
# Show hooks path
git config --local core.hooksPath

# List all hook files
ls -la .githooks/

# Verify hook executability
test -x .githooks/pre-commit && echo "pre-commit: executable ✅" || echo "pre-commit: not executable ❌"
test -x .githooks/commit-msg && echo "commit-msg: executable ✅" || echo "commit-msg: not executable ❌"
test -x .githooks/pre-push && echo "pre-push: executable ✅" || echo "pre-push: not executable ❌"
test -x .githooks/post-commit && echo "post-commit: executable ✅" || echo "post-commit: not executable ❌"
```

---

## Environment Variables

Hooks use these internally:

```bash
HOOKS_DIR         # Location of hook scripts
PROJECT_ROOT      # Location of Cargo.toml
RUST_BACKTRACE=0  # Reduce error output noise (pre-push)
RUST_TEST_THREADS=1  # Single-threaded testing (pre-push)
```

You typically don't need to set these; hooks handle it.

---

## Useful Make Commands

```bash
cargo make format       # Fix formatting
cargo make clippy       # Show linting issues
cargo make check        # Quick compilation check
cargo make test         # Run tests
cargo make test-all     # Run with all features
cargo make lint         # Run all linting checks
cargo make ci            # Run full CI locally
```

---

## Feature Compatibility

Hooks test these critical features:
- `federated-network` - Network federation features
- `otel` - OpenTelemetry observability

Add more features to test by editing `.githooks/pre-push`:

```bash
FEATURE_TESTS=(
    "federated-network"
    "otel"
    "new-feature"  # Add here
)
```

---

## When Things Go Wrong

### Hook hangs (seems stuck)

```bash
Ctrl+C              # Kill the process
git status          # Check what's happening
cargo check         # Manual check
```

### cargo-deny not installed

The hook shows a warning but continues. To use it:

```bash
cargo install cargo-deny
# Run hook again
git commit --allow-empty -m "test"
```

### Tests pass locally but fail in pre-push

```bash
# Exact command the hook runs:
cargo test --lib -- --test-threads=1
cargo test --all-features -- --test-threads=1

# Try these if above fails:
cargo test --lib
cargo test --all-features
```

### Forgot to install hooks

```bash
./.githooks/install.sh
```

---

## Branch Naming

While not enforced by hooks, recommended patterns:

```
feat/feature-name          # New feature
fix/bug-name              # Bug fix
refactor/improvement      # Code refactoring
docs/topic                # Documentation
perf/optimization         # Performance work
test/test-addition        # Test additions
claude/*                  # Claude Code work
```

---

## Integration with CI/CD

Hooks are local only. CI still runs all checks:
- Same format/clippy/deny checks as pre-commit
- Same test suite as pre-push
- Additional checks (code coverage, benchmarks, etc.)

**Goal**: Catch issues locally before pushing (fast feedback) and verify in CI (comprehensive verification).

---

## Further Reading

- **Full specification**: See `GIT_HOOKS_SPEC.md`
- **Implementation details**: See `GIT_HOOKS_IMPLEMENTATION.md`
- **Conventional Commits**: https://www.conventionalcommits.org/
- **Cargo Make**: https://sagiegurari.github.io/cargo-make/
- **Git Hooks**: https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks

---

## Support

Having hook issues? Check these in order:

1. **Run install script**: `./.githooks/install.sh`
2. **Check config**: `git config --local core.hooksPath` (should output `.githooks`)
3. **Check permissions**: `ls -la .githooks/` (should be executable)
4. **Run check manually**: `cargo make format-check`, `cargo test`, etc.
5. **Check CLAUDE.md**: Project-specific requirements
6. **Ask team**: Others may have solved the issue

---

**Last updated**: 2026-06-14  
**For repo**: clap-noun-verb (v26.9.1)
