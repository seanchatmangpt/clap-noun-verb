# Git Hooks Directory

This directory contains git hooks for the `clap-noun-verb` project that enforce quality gates on code before commit and push.

## Quick Start

```bash
./install.sh
```

This installs all hooks and configures git to use them.

## Hook Files

### `pre-commit`
Runs fast quality checks before commit is created (~1.2s):
- Code formatting check (rustfmt)
- Linting check (clippy)
- License/security check (cargo-deny)
- Compilation check (cargo check)
- Project policy validation

**Blocks commit if**: Formatting issues, linting errors, compilation errors, or security issues

### `commit-msg`
Validates commit message format (<50ms):
- Checks message is not empty
- Checks first line ≤72 characters
- Checks first character is capitalized
- Validates conventional commit format (optional)
- Detects issue references (best practice)

**Blocks commit if**: Message is empty, >72 chars, or not capitalized

### `pre-push`
Runs comprehensive test validation before push (30-60s):
- Compilation sanity check
- Library tests (deterministic)
- Integration tests (deterministic)
- All features test
- Critical feature combination tests

**Blocks push if**: Any test fails

### `post-commit`
Provides helpful reminders after successful commit (<50ms):
- Confirms commit was created
- Detects commit type (feat/fix/test/docs/refactor/perf)
- Shows type-specific suggestions
- Displays branch status
- Provides context-aware next steps
- Shows performance SLO status

**Never blocks** (informational only)

### `install.sh`
Installation helper script that:
- Creates `.git/hooks` directory if missing
- Backs up existing hooks to `*.bak`
- Copies all hooks from `.githooks/` to `.git/hooks/`
- Makes all hooks executable
- Configures git: `core.hooksPath = .githooks`

### `uninstall.sh`
Removes all hooks and git configuration:
- Deletes hook files from `.git/hooks/`
- Unsets `core.hooksPath` configuration
- Safe to run if hooks are already removed

## Installation Methods

### Method 1: Automated (Recommended)
```bash
./install.sh
```

### Method 2: Manual
```bash
chmod +x pre-commit commit-msg pre-push post-commit
git config --local core.hooksPath .githooks
```

### Method 3: Direct Copy to .git/hooks
```bash
mkdir -p ../.git/hooks
cp pre-commit commit-msg pre-push post-commit ../.git/hooks/
chmod +x ../.git/hooks/{pre-commit,commit-msg,pre-push,post-commit}
git config --local core.hooksPath .githooks
```

## Verification

Check that hooks are installed:
```bash
git config --local core.hooksPath
# Should output: .githooks
```

Test a hook:
```bash
git commit --allow-empty -m "Test message"
# Should show post-commit hook output
```

## Documentation

For complete information, see:

- **`GIT_HOOKS_SPEC.md`** - Complete specification of all hooks and validations
- **`GIT_HOOKS_IMPLEMENTATION.md`** - Implementation details with code snippets
- **`GIT_HOOKS_QUICK_REFERENCE.md`** - Fast lookup guide for developers
- **`GIT_HOOKS_SETUP_CHECKLIST.md`** - Installation and verification checklist
- **`GIT_HOOKS_SUMMARY.md`** - Executive overview and status

These files are in the repository root directory.

## Troubleshooting

### Hooks not running
```bash
# Reinstall
./install.sh

# Verify
git config --local core.hooksPath
```

### Hooks running but failing
Follow the error message suggestions:
- Formatting: `cargo make format`
- Linting: `cargo make clippy`
- Tests: `cargo make test`

### Need to skip hooks (emergency only)
```bash
git commit --no-verify    # Skip pre-commit
git push --no-verify      # Skip pre-push
```

### Remove hooks
```bash
./uninstall.sh
```

## Hook Configuration

All hooks are configured via:
```bash
# Current configuration
git config --local core.hooksPath

# Change (if needed)
git config --local core.hooksPath .githooks
```

## Performance

| Hook | Duration | Purpose |
|------|----------|---------|
| pre-commit | <2s | Fast quality gates |
| commit-msg | <50ms | Message validation |
| pre-push | 30-60s | Full test suite |
| post-commit | <100ms | Helpful reminders |

## Requirements

- Git 2.9+ (for `core.hooksPath` support)
- Rust toolchain (rustfmt, clippy)
- Bash shell
- Optional: cargo-deny (for license/security checks)

## Support

For issues or questions:
1. Check **GIT_HOOKS_QUICK_REFERENCE.md**
2. Run `./.githooks/install.sh` to reinstall
3. Review **GIT_HOOKS_IMPLEMENTATION.md** for details
4. Ask your team lead or project maintainer

## Files

```
.githooks/
├── README.md                          ← You are here
├── pre-commit                         (146 lines)
├── commit-msg                         (133 lines)
├── pre-push                           (125 lines)
├── post-commit                        (99 lines)
├── install.sh                         (83 lines)
└── uninstall.sh                       (64 lines)
                                       --------
                                       650 lines total
```

## License

These hooks are part of the `clap-noun-verb` project and follow the same license terms.

## Last Updated

2026-06-14 - For clap-noun-verb v26.6.1

---

**Status**: ✅ Ready to use  
**Installation**: `./.githooks/install.sh`  
**For help**: Read `GIT_HOOKS_QUICK_REFERENCE.md`
