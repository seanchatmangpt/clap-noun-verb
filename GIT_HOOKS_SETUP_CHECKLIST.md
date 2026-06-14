# Git Hooks Setup Checklist

Complete verification checklist for git hooks installation and configuration.

---

## Pre-Installation Checklist

- [ ] Git version 2.9+ installed
  ```bash
  git --version
  # Should output: git version 2.9.0 or later
  ```

- [ ] Cargo toolchain installed
  ```bash
  cargo --version
  # Should output: cargo 1.70.0 or later
  ```

- [ ] In correct repository directory
  ```bash
  pwd
  # Should end with: /clap-noun-verb
  ```

- [ ] Repository is clean (no uncommitted changes)
  ```bash
  git status
  # Should show: "nothing to commit, working tree clean"
  ```

---

## Installation Steps

### Step 1: Run Installation Script

- [ ] Execute install script
  ```bash
  ./.githooks/install.sh
  ```

- [ ] Script completes without errors
  ```
  Expected output:
  ✅ Installation complete!
  ```

### Step 2: Verify Git Configuration

- [ ] Check hooks path is set
  ```bash
  git config --local core.hooksPath
  ```
  Expected output: `.githooks`

- [ ] Hooks are readable
  ```bash
  ls -la .githooks/
  ```
  Expected: All files present with `-rw-r--r--` permissions

- [ ] Hook files are executable
  ```bash
  test -x .githooks/pre-commit && echo "✅" || echo "❌"
  test -x .githooks/commit-msg && echo "✅" || echo "❌"
  test -x .githooks/pre-push && echo "✅" || echo "❌"
  test -x .githooks/post-commit && echo "✅" || echo "❌"
  ```
  Expected: All show `✅`

---

## Functional Testing

### Test 1: Pre-Commit Hook (Formatting Check)

- [ ] Create a formatting violation
  ```bash
  echo "fn main() {  " >> /tmp/test.rs
  cp /tmp/test.rs src/lib.rs
  git add src/lib.rs
  ```

- [ ] Attempt to commit
  ```bash
  git commit -m "Test formatting check"
  ```
  Expected: Commit rejected with formatting error

- [ ] Fix formatting
  ```bash
  cargo make format
  git add src/lib.rs
  git commit -m "Test formatting check"
  ```
  Expected: Commit succeeds

- [ ] Revert test
  ```bash
  git reset --soft HEAD~1
  git checkout src/lib.rs
  ```

### Test 2: Commit-Msg Hook (Message Validation)

- [ ] Test long commit message (>72 chars)
  ```bash
  git commit --allow-empty -m "This is a very long commit message that exceeds the 72 character limit on purpose"
  ```
  Expected: Rejected with length error

- [ ] Test non-capitalized message
  ```bash
  git commit --allow-empty -m "this is lowercase"
  ```
  Expected: Rejected with capitalization error

- [ ] Test valid conventional commit
  ```bash
  git commit --allow-empty -m "feat(test): add something"
  ```
  Expected: Accepted, shown as valid conventional commit

- [ ] Clean up test commits
  ```bash
  git reset --hard HEAD
  ```

### Test 3: Post-Commit Hook (Guidance)

- [ ] Make a test commit
  ```bash
  git commit --allow-empty -m "Test post-commit hook"
  ```
  Expected: See post-commit output with suggestions

- [ ] Clean up
  ```bash
  git reset --hard HEAD~1
  ```

### Test 4: Pre-Push Hook (Optional, Requires Remote)

- [ ] If you have a feature branch, try pushing
  ```bash
  git push
  ```
  Expected: Pre-push hook runs, tests execute

- [ ] Observe test output
  Expected: See steps 1-5 with test progress

---

## Dependency Checks

### Required Tools

- [ ] rustfmt (included with Rust)
  ```bash
  cargo fmt --version
  # Should output: rustfmt X.Y.Z
  ```

- [ ] Clippy (included with Rust)
  ```bash
  cargo clippy --version
  # Should output: clippy X.Y.Z
  ```

- [ ] Cargo (included with Rust)
  ```bash
  cargo --version
  ```

### Optional Tools

- [ ] cargo-deny (improves security checks)
  ```bash
  cargo-deny --version
  # If not installed, hook shows warning but continues
  ```

  To install:
  ```bash
  cargo install cargo-deny
  ```

- [ ] cargo-make (used by Makefile.toml)
  ```bash
  cargo make --version
  # If not installed, need to use cargo directly
  ```

  To install:
  ```bash
  cargo install cargo-make
  ```

---

## Performance Verification

### Pre-Commit Timing

- [ ] Measure pre-commit execution
  ```bash
  time cargo fmt -- --check
  # Expected: ~500ms
  ```

  ```bash
  time cargo clippy --quiet -- -D warnings
  # Expected: ~1s
  ```

  ```bash
  time cargo check --quiet
  # Expected: ~800ms
  ```

- [ ] Total time estimate
  Expected: <2 seconds

### Pre-Push Timing

- [ ] Note current time
  ```bash
  date
  ```

- [ ] Run test suite locally
  ```bash
  cargo test --lib -- --test-threads=1
  cargo test --all-features
  ```

- [ ] Note end time
  Expected: 30-60 seconds total

---

## Configuration Verification

### Git Hooks Directory

- [ ] Directory exists and is readable
  ```bash
  [ -d .githooks ] && echo "✅ .githooks exists" || echo "❌ Missing"
  ```

- [ ] All required files present
  ```bash
  [ -f .githooks/pre-commit ] && echo "✅ pre-commit" || echo "❌ Missing"
  [ -f .githooks/commit-msg ] && echo "✅ commit-msg" || echo "❌ Missing"
  [ -f .githooks/pre-push ] && echo "✅ pre-push" || echo "❌ Missing"
  [ -f .githooks/post-commit ] && echo "✅ post-commit" || echo "❌ Missing"
  [ -f .githooks/install.sh ] && echo "✅ install.sh" || echo "❌ Missing"
  [ -f .githooks/uninstall.sh ] && echo "✅ uninstall.sh" || echo "❌ Missing"
  ```

### Git Configuration

- [ ] Local configuration set
  ```bash
  git config --local core.hooksPath
  # Should output: .githooks
  ```

- [ ] No global hooks interference
  ```bash
  git config --global --get-regexp 'hooks.*'
  # Should output nothing (or be acceptable)
  ```

### File Permissions

- [ ] Scripts are executable
  ```bash
  ls -la .githooks/ | grep -E "^-.*x.*pre-commit"
  ls -la .githooks/ | grep -E "^-.*x.*commit-msg"
  ls -la .githooks/ | grep -E "^-.*x.*pre-push"
  ls -la .githooks/ | grep -E "^-.*x.*post-commit"
  ```
  Expected: All should show `rwx` in permissions

---

## Edge Case Testing

### Test 1: Merge Commit (Should Skip Validation)

- [ ] Create a merge scenario (if applicable)
  ```bash
  git checkout -b test-branch
  git commit --allow-empty -m "test"
  git checkout main
  git merge test-branch
  ```
  Expected: commit-msg hook skips validation for merge commit

### Test 2: Detached HEAD (Should Handle Gracefully)

- [ ] Enter detached HEAD state
  ```bash
  git checkout HEAD~1
  ```

- [ ] Try to commit (if applicable)
  ```bash
  git commit --allow-empty -m "test" || true
  ```
  Expected: Hooks handle gracefully (no crash)

- [ ] Return to branch
  ```bash
  git checkout -
  ```

### Test 3: No Remote Upstream

- [ ] On branch without upstream
  ```bash
  git checkout -b test-no-upstream
  ```

- [ ] Commit something
  ```bash
  git commit --allow-empty -m "test"
  ```

- [ ] Post-commit should handle gracefully
  Expected: No errors shown, branch status shows "?"

- [ ] Clean up
  ```bash
  git checkout main
  git branch -D test-no-upstream
  ```

---

## Team Setup Verification

### Shared Repository

- [ ] Other developers run install script
  ```bash
  # Each developer in their clone:
  ./.githooks/install.sh
  ```

- [ ] Verify all have same configuration
  ```bash
  # All should show:
  git config --local core.hooksPath
  # Output: .githooks
  ```

- [ ] Test workflow across team
  ```bash
  # Developer A:
  git commit -m "feat: test change"
  git push
  
  # Developer B:
  git pull
  git commit -m "fix: build on it"
  git push
  ```
  Expected: All hooks run consistently

---

## Troubleshooting Verification

### If Hooks Don't Run

- [ ] Reinstall hooks
  ```bash
  ./.githooks/install.sh
  ```

- [ ] Verify permissions
  ```bash
  chmod +x .githooks/*
  ```

- [ ] Check git version
  ```bash
  git --version  # Must be 2.9+
  ```

### If Hooks Are Too Slow

- [ ] Check disk I/O
  ```bash
  iostat 1 5
  # Look for high %util
  ```

- [ ] Check cargo cache
  ```bash
  du -sh ~/.cargo/registry ~/.cargo/git
  ```

- [ ] Clear and rebuild
  ```bash
  cargo clean
  cargo build
  ```

### If cargo-deny Fails

- [ ] Check deny.toml exists
  ```bash
  [ -f deny.toml ] && echo "✅ Found" || echo "❌ Missing"
  ```

- [ ] Install cargo-deny
  ```bash
  cargo install cargo-deny
  ```

- [ ] Run manually
  ```bash
  cargo deny check
  ```

---

## Documentation Verification

- [ ] GIT_HOOKS_SPEC.md exists and is readable
  ```bash
  [ -f GIT_HOOKS_SPEC.md ] && echo "✅" || echo "❌"
  ```

- [ ] GIT_HOOKS_IMPLEMENTATION.md exists and is readable
  ```bash
  [ -f GIT_HOOKS_IMPLEMENTATION.md ] && echo "✅" || echo "❌"
  ```

- [ ] GIT_HOOKS_QUICK_REFERENCE.md exists and is readable
  ```bash
  [ -f GIT_HOOKS_QUICK_REFERENCE.md ] && echo "✅" || echo "❌"
  ```

- [ ] Team has read the quick reference
  Expected: Positive feedback from team

---

## Final Sign-Off

### Installation Checklist

- [ ] All 4 hooks installed and executable
- [ ] Git configuration correct (core.hooksPath)
- [ ] All dependencies available or optional tools skipped
- [ ] Pre-commit completes in <2 seconds
- [ ] Commit-msg validates correctly
- [ ] Post-commit provides helpful output
- [ ] Pre-push runs full test suite

### Testing Checklist

- [ ] Tested formatting check (pre-commit)
- [ ] Tested message validation (commit-msg)
- [ ] Tested post-commit output
- [ ] Tested pre-push (optional, if applicable)
- [ ] Tested edge cases (merge, detached HEAD, etc.)

### Team Checklist

- [ ] Installation script works for all team members
- [ ] Documentation is accessible and clear
- [ ] Team has read quick reference guide
- [ ] Common workflows tested
- [ ] Support process documented

### Go/No-Go Decision

**All items complete?** → ✅ Go!

**Missing items?** → Review and fix before proceeding

---

## Regular Maintenance

### Weekly

- [ ] Check for hook failures in team
  ```bash
  # Ask team: "Any hook issues this week?"
  ```

### Monthly

- [ ] Review hook performance
  ```bash
  # Track timing over time
  ```

- [ ] Check for updates to tools
  ```bash
  rustup update
  cargo install cargo-deny --force
  ```

### Quarterly

- [ ] Review hook rules and policies
- [ ] Update documentation if needed
- [ ] Solicit feedback from team
- [ ] Consider adding new checks

---

## Quick Reset

If something goes wrong, reset to known good state:

```bash
# Backup current hooks (just in case)
cp -r .githooks .githooks.backup

# Reinstall fresh
./.githooks/install.sh

# Verify
git config --local core.hooksPath
# Should output: .githooks
```

---

## Support Contacts

For issues:

1. **Check quick reference**: `GIT_HOOKS_QUICK_REFERENCE.md`
2. **Check full spec**: `GIT_HOOKS_SPEC.md`
3. **Check implementation**: `GIT_HOOKS_IMPLEMENTATION.md`
4. **Ask team lead** or **project maintainer**

---

## Sign-Off

- **Installed by**: ________________
- **Date**: ________________
- **Verified by**: ________________
- **Date**: ________________
- **Team notified**: ________________
- **Date**: ________________

---

**Last updated**: 2026-06-14  
**For repo**: clap-noun-verb (v26.6.1)  
**Status**: ✅ Ready for Production
