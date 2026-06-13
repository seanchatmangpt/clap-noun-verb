# cargo cicd target show

Display target directory information and usage metrics.

## Usage

```bash
cargo cicd target show [OPTIONS]
```

## Options

- `--target-dir <PATH>` — Path to target directory (default: `./target`)
- `--max-gb <GB>` — Maximum configured size in GB (default: `10.0`)

## Description

Scans the target directory and reports:

- Total size in GB
- Size per profile (debug, release, etc.)
- Candidates for pruning (stale artifacts)
- Verdict comparing actual vs. configured maximum

Useful for understanding disk usage before running `target prune` or for setting CI limits.

## Output format

```json
{
  "target_path": "target",
  "total_size_gb": 8.5,
  "profiles": [
    ["debug", 5.2],
    ["release", 3.3]
  ],
  "stale_candidates": [
    "target/debug/incremental/my-crate-abc123/s/...",
    "target/release/.fingerprint/old-version-xyz/..."
  ],
  "configured_max_gb": 10.0,
  "verdict": "target size is within limit"
}
```

### Field meanings

- `target_path` — Path to the target directory
- `total_size_gb` — Total disk space used
- `profiles` — Array of [profile name, size in GB] pairs
- `stale_candidates` — Paths to artifacts likely safe to remove
- `configured_max_gb` — Maximum size threshold (from `--max-gb` or config)
- `verdict` — Human-readable summary of size status

## Examples

### Default scan

```bash
$ cargo cicd target show
{
  "target_path": "target",
  "total_size_gb": 8.5,
  "profiles": [
    ["debug", 5.2],
    ["release", 3.3]
  ],
  "stale_candidates": [
    "target/debug/incremental/..."
  ],
  "configured_max_gb": 10.0,
  "verdict": "target size is within limit"
}
```

### With custom max size

```bash
$ cargo cicd target show --max-gb 15.0
{
  "target_path": "target",
  "total_size_gb": 8.5,
  "profiles": [
    ["debug", 5.2],
    ["release", 3.3]
  ],
  "stale_candidates": [],
  "configured_max_gb": 15.0,
  "verdict": "target size is within limit"
}
```

### Over limit

```bash
$ cargo cicd target show --max-gb 5.0
{
  "target_path": "target",
  "total_size_gb": 8.5,
  "profiles": [
    ["debug", 5.2],
    ["release", 3.3]
  ],
  "stale_candidates": [
    "target/debug/incremental/...",
    "target/debug/deps/old-*.d",
    "target/release/.fingerprint/..."
  ],
  "configured_max_gb": 5.0,
  "verdict": "target size 8.5 GB exceeds limit 5.0 GB - consider pruning"
}
```

### Custom target directory

```bash
$ cargo cicd target show --target-dir /var/cache/my-build/target
{
  "target_path": "/var/cache/my-build/target",
  "total_size_gb": 12.3,
  "profiles": [
    ["debug", 8.1],
    ["release", 4.2]
  ],
  "stale_candidates": [...],
  "configured_max_gb": 10.0,
  "verdict": "target size 12.3 GB exceeds limit 10.0 GB - consider pruning"
}
```

### In CI/CD

Get total size:

```bash
$ cargo cicd target show | jq '.total_size_gb'
8.5

$ cargo cicd target show | jq '.profiles'
[["debug", 5.2], ["release", 3.3]]

$ cargo cicd target show | jq '.verdict'
"target size is within limit"
```

Fail if over limit:

```bash
#!/bin/bash
size=$(cargo cicd target show | jq '.total_size_gb')
max=10.0
if (( $(echo "$size > $max" | bc -l) )); then
  echo "Target directory too large: $size GB > $max GB"
  exit 1
fi
```

## See also

- [`cargo cicd target prune`](target-prune.md) — Remove stale artifacts
- [cicd.toml schema](../cicd-toml.md) — Configure target limits
