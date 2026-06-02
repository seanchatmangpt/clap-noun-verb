# cargo-cicd

Local-first CI/CD helpers for Rust workspaces: clean target dirs, run changed tests, check git state, and publish process state.

## What it does

`cargo-cicd` provides a small set of commands to keep your Rust workspace healthy during development and CI. It answers questions like:

- How much disk space is the target directory using?
- Which tests should I run based on my changes?
- What fixtures does trybuild need to update?
- Is my git repository in a valid state for publishing?
- What does my workspace actually look like?

All commands output JSON, making them easy to parse in scripts, CI systems, or local tools. Everything runs locally without network calls.

## Installation

```bash
cargo install cargo-cicd
```

Or from the repository:

```bash
cargo install --path crates/cargo-cicd
```

## Quick start

Check workspace status:

```bash
$ cargo cicd status
```

Show target directory metrics:

```bash
$ cargo cicd target show
$ cargo cicd target show --max-gb 15.0
```

Remove stale build artifacts:

```bash
$ cargo cicd target prune --force
```

Show tests affected by your changes:

```bash
$ cargo cicd test changed
$ cargo cicd test changed --base-ref origin/develop
```

Check git state before publishing:

```bash
$ cargo cicd git status
```

Stage and commit changes:

```bash
$ cargo cicd git close --message "feat(core): add validation"
$ cargo cicd git close --message "chore: bump version" --push
```

Show comprehensive workspace diagnostics:

```bash
$ cargo cicd workspace doctor
```

Emit process state to cicd.toml:

```bash
$ cargo cicd publish
```

## Commands overview

### `cargo cicd status`

Show workspace status and autonomic policy recommendations.

**Output:**
```json
{
  "workspace_health": "nominal",
  "target_pressure": false,
  "policy_recommendations": [
    "target size is within configured limit"
  ],
  "timestamp": "2026-06-02T12:34:56Z"
}
```

### `cargo cicd target show`

Display target directory statistics.

**Options:**
- `--target-dir <PATH>` — Path to target directory (default: `./target`)
- `--max-gb <GB>` — Maximum configured size in GB (default: `10.0`)

**Output:**
```json
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

### `cargo cicd target prune`

Remove stale build artifacts from the target directory.

**Options:**
- `--force` — Prune without confirmation (required on first run)

**Output:**
```json
{
  "candidates_found": 12,
  "force_required": true,
  "event_recorded": false,
  "summary": "Found 12 stale candidates. Use --force to prune."
}
```

After running with `--force`:

```json
{
  "candidates_found": 12,
  "force_required": false,
  "event_recorded": true,
  "summary": "Pruned 12 artifacts"
}
```

### `cargo cicd test changed`

Show which tests should run based on files changed since a base ref.

**Options:**
- `--base-ref <REF>` — Base ref for comparison (default: `origin/main`)

**Output:**
```json
{
  "test_plan": [
    ["unit", "src/core/validation.rs"],
    ["integration", "src/cli/handler.rs"],
    ["doc", "src/lib.rs"]
  ],
  "is_conservative": false,
  "summary": "Run 3 test suites"
}
```

### `cargo cicd trybuild changed`

Show which trybuild fixtures need snapshot updates based on test code changes.

**Output:**
```json
{
  "changed_fixtures": [
    "tests/fixtures/compile_fail_01.rs",
    "tests/fixtures/compile_fail_02.rs"
  ],
  "snapshot_path": "tests/fixtures/",
  "summary": "2 fixtures need updates"
}
```

### `cargo cicd git status`

Show git repository status: dirty files, unpushed commits, branch info.

**Output:**
```json
{
  "is_clean": false,
  "branch": "feat/new-feature",
  "dirty_files": [
    "src/main.rs",
    "Cargo.toml"
  ],
  "unpushed_commits": 3,
  "phase": "dirty"
}
```

### `cargo cicd git close`

Stage, commit, and optionally push changes.

**Options:**
- `--message <MSG>` — Commit message (default: `"feat(cicd): phase boundary close"`)
- `--files <FILES>` — Comma-separated files to stage (if omitted, stages all changes)
- `--push` — Push after commit

**Output:**
```json
{
  "commit_hash": "abc1234",
  "files_staged": 2,
  "pushed": true,
  "summary": "Committed 2 files and pushed to origin"
}
```

### `cargo cicd workspace doctor`

Run comprehensive workspace diagnostics: toolchain, dependencies, build issues, test failures.

**Output:**
```json
{
  "workspace_healthy": true,
  "checks": [
    {
      "name": "rust_version",
      "status": "ok",
      "detail": "1.74+"
    },
    {
      "name": "dependencies",
      "status": "ok",
      "detail": "All locked"
    }
  ],
  "summary": "Workspace is healthy"
}
```

### `cargo cicd publish`

Emit cicd.toml with process state and timestamps.

**Output:**
```json
{
  "version": "26.6.2",
  "workspace_id": "clap-noun-verb",
  "events": [
    {
      "timestamp": "2026-06-02T12:34:56Z",
      "event_type": "status_check",
      "health": "nominal"
    }
  ],
  "summary": "Published process state to cicd.toml"
}
```

## cicd.toml schema

The `cicd.toml` file records your workspace state over time.

### File location

By default, `cicd.toml` lives in your workspace root.

### Schema

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
```

### Customization

Create a `.cicd/config.toml` in your workspace root to override defaults:

```toml
[target]
max_gb = 15.0
prune_stale_threshold_days = 30

[test]
base_ref = "origin/develop"
conservative_mode = false

[git]
require_signed_commits = false
```

## Autonomic policies

When you run `cargo cicd status`, the tool evaluates your workspace against several policies and suggests actions.

### target_pressure

**What it checks:**
- Target directory size vs. configured maximum
- Presence of stale artifacts

**Suggests pruning if:**
- Target > max_gb
- Stale artifacts detected

**Disable:**
```toml
[policies]
target_pressure = false
```

### toolchain_mismatch

**What it checks:**
- Installed Rust version vs. MSRV in Cargo.toml
- Required components (rustfmt, clippy)

**Suggests install if:**
- Rust version < MSRV
- Missing components

### trybuild_changed

**What it checks:**
- Changed test source files
- Stale trybuild fixtures

**Suggests snapshot update if:**
- Test code changed and fixtures are outdated

### git_phase_dirty

**What it checks:**
- Uncommitted changes
- Unpushed commits

**Suggests git close if:**
- Repository is dirty
- Changes are ready to commit

## FAQ

**Q: Can I use this in CI?**

Yes. All commands output JSON, so you can parse results in shell scripts, GitHub Actions, GitLab CI, etc.

Example GitHub Actions:

```yaml
- name: Check workspace status
  run: cargo cicd status | jq '.workspace_health'
```

**Q: Does this require network access?**

No. Everything runs locally. No calls to crates.io, GitHub, or any external service.

**Q: Can I customize commands?**

Yes. Create `.cicd/config.toml` in your workspace root for policy settings, thresholds, and defaults.

**Q: What Rust versions are supported?**

Rust 1.74+. See the MSRV in Cargo.toml.

**Q: How fast is it?**

- `status` — <100ms
- `target show` — 100-500ms (depends on target directory size)
- `test changed` — 200-800ms (depends on git diff size)
- `git status` — <100ms

**Q: Can I use this as a library?**

Yes. `cargo-cicd` is split into:

- **Library** (`src/lib.rs`, `src/adapters/`) — Public API for programmatic use
- **Binary** (`src/main.rs`) — CLI entrypoint

```rust
use cargo_cicd::{TargetScanning, TestPlan};

let info = TargetScanning::scan(PathBuf::from("target"))?;
println!("Total size: {} GB", info.total_size_gb);

let plan = TestPlan::discover()?;
println!("Tests to run: {:?}", plan);
```

## License

Dual-licensed under MIT or Apache-2.0.
