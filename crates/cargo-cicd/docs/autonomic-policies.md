# Autonomic policies

When you run `cargo cicd status`, the tool evaluates your workspace against policies and suggests actions. This guide explains each policy and how to configure them.

## Overview

Policies are rules that your workspace either satisfies or violates. Each policy:

- Checks a specific aspect of your workspace (target size, toolchain, git state)
- Either passes or fails
- Generates a recommendation if it fails
- Can be disabled if you don't need it

Run `cargo cicd status` to see which policies triggered.

## Policy: target_pressure

**What it checks:** Target directory size and artifact staleness.

**Fails if:**
- Target directory > configured maximum (default: 10 GB)
- Stale artifacts detected (unused compiled objects, caches)

**Recommendation:** Run `cargo cicd target prune --force` to clean up.

**Rationale:** Large target directories slow down builds, copy operations, and backups. Pruning removes only truly stale artifacts, preserving recent work.

**Configure:**

```toml
# .cicd/config.toml
[policies]
target_pressure = true

[target]
max_gb = 15.0
prune_stale_threshold_days = 30
```

**Disable:**

```toml
[policies]
target_pressure = false
```

**Examples:**

```bash
# Check target pressure
$ cargo cicd status | jq '.target_pressure'
true

# See details
$ cargo cicd target show
{
  "total_size_gb": 12.5,
  "configured_max_gb": 10.0,
  "verdict": "target size 12.5 GB exceeds limit 10.0 GB"
}

# Fix
$ cargo cicd target prune --force
```

## Policy: toolchain_mismatch

**What it checks:** Installed Rust version vs. project's MSRV (Minimum Supported Rust Version).

**Fails if:**
- `rustc --version` < MSRV in Cargo.toml
- Required components missing (rustfmt, clippy, etc.)

**Recommendation:** Upgrade Rust with `rustup update` or `rustup install`.

**Rationale:** Building with too old a Rust version may silently produce incorrect binaries. MSRV guarantees a version that is known to work.

**Configure:**

```toml
# Cargo.toml
[package]
rust-version = "1.74"

# .cicd/config.toml
[policies]
toolchain_mismatch = true
```

**Disable:**

```toml
[policies]
toolchain_mismatch = false
```

**Examples:**

```bash
# Check toolchain
$ cargo cicd status | jq '.toolchain_mismatch'
false  # Good, you have the right version

# If it fails:
$ rustup update
$ rustup install 1.74
```

## Policy: trybuild_changed

**What it checks:** Whether trybuild fixture snapshots match current test code.

**Fails if:**
- Test source code changed
- Trybuild fixtures exist but are stale (compiler output changed)

**Recommendation:** Run `TRYBUILD=overwrite cargo test` to update snapshots.

**Rationale:** Trybuild captures exact compiler output. When macros change, snapshots must be updated. Out-of-sync snapshots hide real changes to compiler behavior.

**Configure:**

```toml
[policies]
trybuild_changed = true
```

**Disable:**

```toml
[policies]
trybuild_changed = false
```

**Examples:**

```bash
# Check fixture status
$ cargo cicd status | jq '.trybuild_changed'
true  # Fixtures are out of date

$ cargo cicd trybuild changed
{
  "changed_fixtures": [
    "tests/fixtures/compile_fail_01.rs",
    "tests/fixtures/compile_fail_02.rs"
  ],
  "summary": "2 fixtures need updates"
}

# Fix
$ TRYBUILD=overwrite cargo test --test trybuild_tests
```

## Policy: git_phase_dirty

**What it checks:** Git repository cleanliness and push status.

**Fails if:**
- Uncommitted changes exist (modified files, untracked files)
- Unpushed commits exist (ahead of remote)

**Recommendation:**
- If dirty: Run `cargo cicd git close --message "..." --push`
- If advanced: Push with `git push`

**Rationale:** Dirty repositories indicate incomplete work. Pushing regularly prevents data loss and enables team collaboration. This policy encourages clean checkpoints.

**Configure:**

```toml
[policies]
git_phase_dirty = true

[git]
require_signed_commits = false
require_pr_from_main = true
```

**Disable:**

```toml
[policies]
git_phase_dirty = false
```

**Examples:**

```bash
# Check git state
$ cargo cicd status | jq '.git_phase_dirty'
true  # Repository has changes

$ cargo cicd git status
{
  "is_clean": false,
  "branch": "feat/new-feature",
  "dirty_files": ["src/main.rs", "Cargo.toml"],
  "unpushed_commits": 0,
  "phase": "dirty"
}

# Fix: commit and push
$ cargo cicd git close --message "feat: add validation" --push
```

## Combined example

Check all policies:

```bash
$ cargo cicd status
{
  "workspace_health": "caution",
  "target_pressure": true,
  "toolchain_mismatch": false,
  "trybuild_changed": true,
  "git_phase_dirty": true,
  "policy_recommendations": [
    "target size 12.5 GB exceeds limit 10.0 GB - run `cargo cicd target prune --force`",
    "2 trybuild fixtures changed - run `TRYBUILD=overwrite cargo test`",
    "Repository dirty with 2 files - run `cargo cicd git close --push`"
  ]
}
```

Address all issues:

```bash
# 1. Prune target
cargo cicd target prune --force

# 2. Update trybuild fixtures
TRYBUILD=overwrite cargo test --test trybuild_tests

# 3. Commit and push
cargo cicd git close --message "chore: workspace cleanup" --push

# Verify
$ cargo cicd status
{
  "workspace_health": "nominal",
  "target_pressure": false,
  "toolchain_mismatch": false,
  "trybuild_changed": false,
  "git_phase_dirty": false,
  "policy_recommendations": [
    "All policies pass. Workspace is ready."
  ]
}
```

## Configuration hierarchy

Policies use this configuration priority (highest to lowest):

1. **Environment variables**: `CICD_TARGET_MAX_GB=20.0`
2. **Project config**: `.cicd/config.toml` in workspace root
3. **User config**: `~/.cargo/cicd.toml`
4. **Defaults**: Built-in defaults

### Default configuration

```toml
[target]
max_gb = 10.0
prune_stale_threshold_days = 7

[test]
base_ref = "origin/main"
conservative_mode = false

[git]
require_signed_commits = false
require_pr_from_main = false

[policies]
target_pressure = true
toolchain_mismatch = true
trybuild_changed = true
git_phase_dirty = true
```

### Project-level override

Create `.cicd/config.toml`:

```toml
[target]
max_gb = 20.0

[policies]
target_pressure = true
toolchain_mismatch = false
```

### Environment variable override

```bash
CICD_TARGET_MAX_GB=25.0 cargo cicd status
CICD_TOOLCHAIN_CHECK=false cargo cicd status
```

## Workflow integration

### Pre-commit hook

Use policies in git hook to prevent committing broken code:

```bash
#!/bin/bash
# .git/hooks/pre-commit
health=$(cargo cicd status | jq -r '.workspace_health')
if [[ "$health" == "critical" ]]; then
  echo "Workspace is critical. Fix issues before committing."
  exit 1
fi
```

### CI gate

Fail CI if workspace is unhealthy:

```yaml
# .github/workflows/ci.yml
- name: Check workspace health
  run: |
    result=$(cargo cicd status | jq '.workspace_health')
    if [[ "$result" == "critical" ]]; then
      echo "::error::Workspace critical"
      exit 1
    fi
```

### Local development

Auto-suggest fixes:

```bash
#!/bin/bash
# In your shell init file
if ! cargo cicd status | jq -e '.policy_recommendations[0]' &>/dev/null; then
  recommendations=$(cargo cicd status | jq -r '.policy_recommendations[]')
  echo "Policy suggestions:"
  echo "$recommendations"
fi
```

## See also

- [`cargo cicd status`](commands/status.md) — Check policy status
- [cicd.toml](cicd-toml.md) — Configuration file reference
