# cargo cicd status

Show workspace status and autonomic policy recommendations.

## Usage

```bash
cargo cicd status
```

No options.

## Description

Evaluates your workspace against autonomic policies and reports:

- Overall health status (nominal, caution, critical)
- Whether target pressure thresholds are exceeded
- Whether toolchain version matches requirements
- Whether trybuild fixtures are stale
- Whether git repository is in a valid state

Returns recommendations for each policy that triggered.

## Output format

```json
{
  "workspace_health": "nominal",
  "target_pressure": false,
  "toolchain_mismatch": false,
  "trybuild_changed": false,
  "git_phase_dirty": false,
  "policy_recommendations": [
    "target size is within configured limit"
  ],
  "timestamp": "2026-06-02T12:34:56Z"
}
```

### Field meanings

- `workspace_health` — Overall status: "nominal", "caution", or "critical"
- `target_pressure` — Target directory exceeds configured size or has stale artifacts
- `toolchain_mismatch` — Rust version or components don't match project requirements
- `trybuild_changed` — Test code changed but fixtures not updated
- `git_phase_dirty` — Repository has uncommitted changes or unpushed commits
- `policy_recommendations` — Array of suggested actions
- `timestamp` — When check was performed (UTC)

## Examples

### Basic status check

```bash
$ cargo cicd status
{
  "workspace_health": "nominal",
  "target_pressure": false,
  "toolchain_mismatch": false,
  "trybuild_changed": false,
  "git_phase_dirty": false,
  "policy_recommendations": [
    "target size is within configured limit"
  ],
  "timestamp": "2026-06-02T12:34:56Z"
}
```

### With caution status

```bash
$ cargo cicd status
{
  "workspace_health": "caution",
  "target_pressure": true,
  "toolchain_mismatch": false,
  "trybuild_changed": false,
  "git_phase_dirty": false,
  "policy_recommendations": [
    "target size 12.5 GB exceeds limit of 10.0 GB - run `cargo cicd target prune --force`",
    "14 stale artifacts found in target/debug/incremental"
  ],
  "timestamp": "2026-06-02T12:34:56Z"
}
```

### In CI/CD

Parse with `jq`:

```bash
$ cargo cicd status | jq '.workspace_health'
"nominal"

$ cargo cicd status | jq '.policy_recommendations[]'
"target size is within configured limit"

$ cargo cicd status | jq '.target_pressure'
false
```

Fail CI if workspace is critical:

```bash
#!/bin/bash
health=$(cargo cicd status | jq -r '.workspace_health')
if [[ "$health" == "critical" ]]; then
  echo "Workspace is in critical state"
  exit 1
fi
```

## Disabling policies

To disable a policy, add to `.cicd/config.toml`:

```toml
[policies]
target_pressure = false
toolchain_mismatch = false
trybuild_changed = true
git_phase_dirty = false
```

## See also

- [`cargo cicd target show`](target-show.md) — Detailed target directory metrics
- [`cargo cicd workspace doctor`](workspace-doctor.md) — Comprehensive diagnostics
- [Autonomic Policies](../autonomic-policies.md) — Full policy reference
