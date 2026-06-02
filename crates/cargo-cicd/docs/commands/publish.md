# cargo cicd publish

Emit workspace state to cicd.toml with process event record.

## Usage

```bash
cargo cicd publish
```

No options.

## Description

Records current workspace state to `cicd.toml`:

- Workspace metadata (id, version, created_at)
- Process events (timestamps, types, metrics)
- Action history (what happened and when)

Creates a simple, queryable log of your build process. Useful for:

- Understanding build history
- Debugging workflow issues
- Detecting patterns in build time/failures
- Integration with external monitoring

The file grows append-only; events are never deleted.

## Output format

```json
{
  "version": "26.6.2",
  "workspace_id": "clap-noun-verb",
  "events_recorded": 5,
  "timestamp": "2026-06-02T12:34:56Z",
  "summary": "Published process state to cicd.toml"
}
```

### Field meanings

- `version` — cargo-cicd version used to publish
- `workspace_id` — Name of workspace (from Cargo.toml)
- `events_recorded` — Total events now in cicd.toml
- `timestamp` — When publish occurred (UTC)
- `summary` — Human-readable status

## cicd.toml format

Created automatically in workspace root. Example:

```toml
[workspace]
id = "my-workspace"
version = "26.6.2"
created_at = "2026-06-01T12:00:00Z"

[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "target_prune"
size_before_gb = 12.5
size_after_gb = 8.2
artifacts_removed = 142

[[events]]
timestamp = "2026-06-02T12:35:00Z"
event_type = "test_run"
test_count = 156
passed = 156
failed = 0

[[events]]
timestamp = "2026-06-02T12:36:00Z"
event_type = "git_commit"
commit_hash = "abc1234"
files_changed = 3

[[events]]
timestamp = "2026-06-02T12:37:00Z"
event_type = "status_check"
health = "nominal"
```

## Examples

### First publish

```bash
$ cargo cicd publish
{
  "version": "26.6.2",
  "workspace_id": "clap-noun-verb",
  "events_recorded": 1,
  "timestamp": "2026-06-02T12:34:56Z",
  "summary": "Published process state to cicd.toml"
}
```

Created `cicd.toml`:

```toml
[workspace]
id = "clap-noun-verb"
version = "26.6.2"
created_at = "2026-06-02T12:34:56Z"

[[events]]
timestamp = "2026-06-02T12:34:56Z"
event_type = "publish"
```

### Subsequent publishes

```bash
$ cargo cicd publish
{
  "version": "26.6.2",
  "workspace_id": "clap-noun-verb",
  "events_recorded": 5,
  "timestamp": "2026-06-02T12:45:00Z",
  "summary": "Published process state to cicd.toml"
}
```

### In CI/CD

Record build stages:

```yaml
- name: Run tests
  run: cargo test

- name: Record test stage completion
  run: cargo cicd publish

- name: Build release
  run: cargo build --release

- name: Record build completion
  run: cargo cicd publish
```

Use as audit trail:

```bash
#!/bin/bash
# Before publishing, record state
cargo cicd publish

# Check cicd.toml for history
cat cicd.toml
```

## Workflow tips

### After major operations

Publish after key milestones:

```bash
cargo test && cargo cicd publish
cargo build --release && cargo cicd publish
cargo doc && cargo cicd publish
```

### In release scripts

Record release process in cicd.toml:

```bash
#!/bin/bash
set -e

echo "=== Release v1.1.0 ==="

# Update version
sed -i 's/version = "1.0.0"/version = "1.1.0"/' Cargo.toml
cargo cicd publish

# Test
cargo test
cargo cicd publish

# Build release
cargo build --release
cargo cicd publish

# Commit
cargo cicd git close --message "chore: release v1.1.0" --push
cargo cicd publish

echo "=== Release complete ==="
```

### Historical analysis

Query cicd.toml for trends:

```bash
# See all events
cat cicd.toml | grep 'event_type'

# Check test history
grep -A3 'event_type = "test_run"' cicd.toml

# Track target size over time
grep -B1 'size_after_gb' cicd.toml
```

## File location

`cicd.toml` is created in the workspace root (same level as root `Cargo.toml`).

To use a custom location, set `CICD_TOML_PATH`:

```bash
CICD_TOML_PATH=/var/log/builds/cicd.toml cargo cicd publish
```

## See also

- [cicd.toml schema](../cicd-toml.md) — Full reference
- [`cargo cicd status`](status.md) — Check current state
