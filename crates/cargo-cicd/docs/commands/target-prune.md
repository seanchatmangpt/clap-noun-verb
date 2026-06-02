# cargo cicd target prune

Remove stale build artifacts from the target directory.

## Usage

```bash
cargo cicd target prune [OPTIONS]
```

## Options

- `--force` — Perform pruning without confirmation (required)

## Description

Scans the target directory for artifacts that are safe to remove:

- Old incremental compilation caches
- Stale dependency artifacts
- Fingerprints for removed dependencies

The first run shows candidates; use `--force` to actually delete.

This is safer than `rm -rf target` because it preserves recent work and only removes clearly stale data.

## Output format

### First run (without --force)

```json
{
  "candidates_found": 12,
  "force_required": true,
  "event_recorded": false,
  "summary": "Found 12 stale candidates. Use --force to prune."
}
```

### After --force

```json
{
  "candidates_found": 12,
  "force_required": false,
  "event_recorded": true,
  "summary": "Pruned 12 artifacts"
}
```

### Field meanings

- `candidates_found` — Number of artifacts marked for removal
- `force_required` — Whether --force is needed (always true on first run)
- `event_recorded` — Whether the action was recorded to cicd.toml
- `summary` — Human-readable status

## Examples

### Check candidates

```bash
$ cargo cicd target prune
{
  "candidates_found": 12,
  "force_required": true,
  "event_recorded": false,
  "summary": "Found 12 stale candidates. Use --force to prune."
}
```

### Actually prune

```bash
$ cargo cicd target prune --force
{
  "candidates_found": 12,
  "force_required": false,
  "event_recorded": true,
  "summary": "Pruned 12 artifacts"
}
```

### No artifacts to prune

```bash
$ cargo cicd target prune
{
  "candidates_found": 0,
  "force_required": false,
  "event_recorded": true,
  "summary": "No stale artifacts found"
}
```

### In CI/CD

Check if pruning is needed:

```bash
$ cargo cicd target prune | jq '.candidates_found'
12

$ cargo cicd target prune | jq '.force_required'
true
```

Prune in cleanup step:

```yaml
- name: Prune target directory
  run: cargo cicd target prune --force
```

## Workflow tips

### Before pushing

Clean up target directory before committing to avoid bloating the repo:

```bash
cargo clean          # Full clean (slow)
cargo cicd target prune --force  # Selective clean (fast)
```

### In long-running builds

Prune periodically to control disk usage:

```bash
# In a build matrix job
cargo cicd target prune --force
cargo test
```

### Safe alternative to `cargo clean`

Instead of:

```bash
cargo clean  # Removes EVERYTHING, next build is slow
```

Use:

```bash
cargo cicd target prune --force  # Only removes truly stale items
```

## See also

- [`cargo cicd target show`](target-show.md) — View target directory metrics
- [Autonomic Policies](../autonomic-policies.md) — Configure target pressure thresholds
