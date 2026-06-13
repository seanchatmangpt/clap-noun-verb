# cargo cicd git close

Stage, commit, and optionally push changes in one step.

## Usage

```bash
cargo cicd git close [OPTIONS]
```

## Options

- `--message <MSG>` — Commit message (default: `"feat(cicd): phase boundary close"`)
- `--files <FILES>` — Comma-separated files to stage (if omitted, stages all changes)
- `--push` — Push to origin after commit

## Description

Automates a common workflow:

1. Stage specified files (or all if none specified)
2. Create a commit with provided message
3. Optionally push to origin

Safer than manual `git add . && git commit && git push` because:

- You can specify exactly which files to stage
- Requires explicit `--push` to push
- Returns full commit details in JSON

## Output format

### After commit (without push)

```json
{
  "commit_hash": "abc1234def5678",
  "files_staged": 2,
  "pushed": false,
  "summary": "Committed 2 files"
}
```

### After push

```json
{
  "commit_hash": "abc1234def5678",
  "files_staged": 2,
  "pushed": true,
  "summary": "Committed 2 files and pushed to origin"
}
```

### Field meanings

- `commit_hash` — Full git commit hash
- `files_staged` — Number of files included in commit
- `pushed` — Whether changes were pushed to origin
- `summary` — Human-readable action description

## Examples

### Commit all changes

```bash
$ cargo cicd git close --message "feat(core): add validation"
{
  "commit_hash": "abc1234def5678",
  "files_staged": 3,
  "pushed": false,
  "summary": "Committed 3 files"
}
```

### Commit specific files

```bash
$ cargo cicd git close --message "docs: update README" --files "README.md,docs/guide.md"
{
  "commit_hash": "xyz9876abc5432",
  "files_staged": 2,
  "pushed": false,
  "summary": "Committed 2 files"
}
```

### Commit and push

```bash
$ cargo cicd git close --message "chore: bump version" --push
{
  "commit_hash": "def3456ghi7890",
  "files_staged": 1,
  "pushed": true,
  "summary": "Committed 1 file and pushed to origin"
}
```

### Using default message

```bash
$ cargo cicd git close
{
  "commit_hash": "jkl0123mno4567",
  "files_staged": 5,
  "pushed": false,
  "summary": "Committed 5 files"
}
```

### In CI/CD

Commit build artifacts:

```bash
$ cargo cicd git close \
  --message "chore(ci): publish artifacts" \
  --files "dist/,build/" \
  --push
```

Commit test results:

```yaml
- name: Commit test results
  run: |
    cargo cicd git close \
      --message "test: record coverage" \
      --files "coverage/,reports/" \
      --push
```

Conditional push based on branch:

```bash
#!/bin/bash
branch=$(git rev-parse --abbrev-ref HEAD)
push_flag=""

if [[ "$branch" == "main" || "$branch" == "develop" ]]; then
  push_flag="--push"
fi

cargo cicd git close \
  --message "chore: automated update" \
  $push_flag
```

## Workflow tips

### Release process

Automated release with single command:

```bash
# Update version in Cargo.toml
sed -i 's/version = "1.0.0"/version = "1.1.0"/' Cargo.toml

# Commit and push
cargo cicd git close \
  --message "chore: release v1.1.0" \
  --files "Cargo.toml,Cargo.lock" \
  --push
```

### GitHub Actions automation

```yaml
- name: Auto-fix and commit
  run: |
    cargo fmt
    cargo cicd git close \
      --message "style: automated formatting" \
      --push
```

### Staging groups

Commit different groups separately:

```bash
# Commit source changes
cargo cicd git close \
  --message "feat: add validation" \
  --files "src/*.rs" \
  --push

# Commit tests separately
cargo cicd git close \
  --message "test: add validation tests" \
  --files "tests/*.rs" \
  --push
```

## Safety notes

- **--files is exclusive**: Only listed files are staged. Unmentioned changes stay in working tree.
- **No --push is safe**: Commits locally without pushing. You can review before pushing manually.
- **Message is required (unless using default)**: Explicit message helps traceability.
- **Commits are immutable**: Once pushed, you cannot delete. Plan your message carefully.

## See also

- [`cargo cicd git status`](git-status.md) — Check repository state before closing
- [Autonomic Policies](../autonomic-policies.md) — Configure commit policies
