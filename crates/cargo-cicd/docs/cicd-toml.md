# cicd.toml schema

The `cicd.toml` file is your workspace's CI/CD event log. It records what happened, when, and key metrics.

## Location

By default: `./cicd.toml` (same level as root `Cargo.toml`)

Override with environment variable:

```bash
CICD_TOML_PATH=/var/builds/cicd.toml cargo cicd publish
```

## File structure

### Workspace section

Metadata about your workspace.

```toml
[workspace]
id = "my-workspace"
version = "26.6.2"
created_at = "2026-06-01T12:00:00Z"
```

- `id` — Workspace name (from Cargo.toml's first crate)
- `version` — cargo-cicd version that created this file
- `created_at` — UTC timestamp of first publish

### Events array

Each event is a record of something that happened.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "target_prune"
size_before_gb = 12.5
size_after_gb = 8.2
artifacts_removed = 142
```

All events have `timestamp` and `event_type`. Other fields depend on the event.

## Event types

### publish

Recorded when you run `cargo cicd publish`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "publish"
```

### status_check

Result of `cargo cicd status`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "status_check"
health = "nominal"
target_pressure = false
toolchain_mismatch = false
trybuild_changed = false
git_phase_dirty = false
```

- `health` — "nominal", "caution", or "critical"
- `target_pressure` — true if target exceeds limit
- `toolchain_mismatch` — true if Rust version mismatch
- `trybuild_changed` — true if fixtures are stale
- `git_phase_dirty` — true if uncommitted changes or unpushed commits

### target_show

Result of `cargo cicd target show`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "target_show"
target_path = "target"
total_size_gb = 8.5
max_configured_gb = 10.0
```

- `total_size_gb` — Disk space used by target directory
- `max_configured_gb` — Configured limit

### target_prune

Result of `cargo cicd target prune --force`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "target_prune"
size_before_gb = 12.5
size_after_gb = 8.2
artifacts_removed = 142
```

- `size_before_gb` — Target size before pruning
- `size_after_gb` — Target size after pruning
- `artifacts_removed` — Number of files deleted

### test_changed

Result of `cargo cicd test changed`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "test_changed"
test_count = 3
test_types = ["unit", "integration", "doc"]
is_conservative = false
```

- `test_count` — Number of test suites to run
- `test_types` — Array of test type names
- `is_conservative` — true if falling back to all tests

### test_run

Custom event (you add this).

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "test_run"
test_count = 156
passed = 156
failed = 0
duration_secs = 42
```

Add this in your CI after running tests:

```bash
cargo test
# Parse output and append:
# [[events]]
# event_type = "test_run"
# passed = 156, failed = 0, etc.
```

### trybuild_changed

Result of `cargo cicd trybuild changed`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "trybuild_changed"
changed_fixtures = 2
fixture_names = ["compile_fail_01.rs", "compile_fail_02.rs"]
```

- `changed_fixtures` — Number of stale fixtures
- `fixture_names` — Array of fixture file names

### git_status

Result of `cargo cicd git status`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "git_status"
branch = "feat/new-feature"
is_clean = false
dirty_files = 2
unpushed_commits = 3
phase = "dirty"
```

- `branch` — Current branch name
- `is_clean` — true if repository is clean
- `dirty_files` — Number of modified/untracked files
- `unpushed_commits` — Number of local commits not pushed
- `phase` — "clean", "dirty", or "advanced"

### git_close

Result of `cargo cicd git close`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "git_close"
commit_hash = "abc1234def5678"
files_staged = 2
pushed = false
message = "feat(core): add validation"
```

- `commit_hash` — Full git commit hash
- `files_staged` — Number of files in commit
- `pushed` — true if pushed to origin
- `message` — Commit message

### workspace_doctor

Result of `cargo cicd workspace doctor`.

```toml
[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "workspace_doctor"
workspace_healthy = true
checks_passed = 6
checks_total = 6
```

- `workspace_healthy` — true if all checks pass
- `checks_passed` — Number of passing checks
- `checks_total` — Total checks run

## Example complete cicd.toml

```toml
[workspace]
id = "clap-noun-verb"
version = "26.6.2"
created_at = "2026-06-01T12:00:00Z"

[[events]]
timestamp = "2026-06-02T08:00:00Z"
event_type = "workspace_doctor"
workspace_healthy = true
checks_passed = 6
checks_total = 6

[[events]]
timestamp = "2026-06-02T08:01:00Z"
event_type = "test_run"
test_count = 156
passed = 156
failed = 0
duration_secs = 42

[[events]]
timestamp = "2026-06-02T08:05:00Z"
event_type = "target_show"
target_path = "target"
total_size_gb = 8.5
max_configured_gb = 10.0

[[events]]
timestamp = "2026-06-02T08:06:00Z"
event_type = "git_status"
branch = "feat/validation"
is_clean = false
dirty_files = 2
unpushed_commits = 0
phase = "dirty"

[[events]]
timestamp = "2026-06-02T08:07:00Z"
event_type = "git_close"
commit_hash = "abc1234def5678"
files_staged = 2
pushed = true
message = "feat: add validation"

[[events]]
timestamp = "2026-06-02T08:08:00Z"
event_type = "publish"
```

## Customization

Create `.cicd/config.toml` to override defaults:

```toml
# Target directory settings
[target]
max_gb = 15.0
prune_stale_threshold_days = 30

# Test selection settings
[test]
base_ref = "origin/develop"
conservative_mode = false

# Git settings
[git]
require_signed_commits = false
require_pr_from_main = true

# Policies (enable/disable checks)
[policies]
target_pressure = true
toolchain_mismatch = true
trybuild_changed = true
git_phase_dirty = true
```

All settings are optional. Defaults:

```toml
[target]
max_gb = 10.0
prune_stale_threshold_days = 7

[test]
base_ref = "origin/main"
conservative_mode = false

[git]
require_signed_commits = false

[policies]
target_pressure = true
toolchain_mismatch = true
trybuild_changed = true
git_phase_dirty = true
```

## Querying cicd.toml

### With `grep`

```bash
# See all events
grep 'event_type' cicd.toml

# Find test runs
grep -A3 'event_type = "test_run"' cicd.toml

# Track target size
grep -B1 'total_size_gb' cicd.toml
```

### With `toml-cli` (if installed)

```bash
# Get workspace ID
toml get cicd.toml workspace.id

# Count events
toml get cicd.toml events | wc -l

# Get last event
toml get cicd.toml 'events[0]'
```

### With shell parsing

```bash
#!/bin/bash
# Extract timestamps and event types
grep 'timestamp\|event_type' cicd.toml | paste - - | head -10
```

## Integrations

### With CI/CD systems

GitHub Actions:

```yaml
- name: Record build event
  run: |
    cargo build
    cargo cicd publish
```

GitLab CI:

```yaml
build:
  script:
    - cargo build
    - cargo cicd publish
  artifacts:
    paths:
      - cicd.toml
```

### With monitoring tools

Stream events to external system:

```bash
#!/bin/bash
# After running cargo cicd commands...
cargo cicd publish

# Parse and send to monitoring system
while IFS= read -r line; do
  if [[ $line == *"event_type"* ]]; then
    event_type=$(echo "$line" | cut -d'=' -f2 | tr -d ' "')
    # Send to monitoring system
    curl -X POST https://monitoring.example.com/events \
      -d "type=$event_type&timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  fi
done < cicd.toml
```

## Best practices

1. **Commit cicd.toml** — Keep it in git history for audit trail
2. **Archive periodically** — Move old cicd.toml to backup for long-running projects
3. **Parse regularly** — Analyze trends in target size, test times, build success rates
4. **Use in alerting** — Monitor health status and alert on degradation
5. **Clean log rotation** — Consider archiving/resetting after releases

## See also

- [`cargo cicd publish`](commands/publish.md) — Record state to cicd.toml
- [`cargo cicd status`](commands/status.md) — Check current workspace state
