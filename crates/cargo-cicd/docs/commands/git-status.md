# cargo cicd git status

Show git repository status: branch, dirty files, unpushed commits.

## Usage

```bash
cargo cicd git status
```

No options.

## Description

Reports current git state without making any changes:

- Current branch name
- Dirty/clean status
- List of modified/untracked files
- Number of unpushed commits
- Repository phase (clean, dirty, advanced, etc.)

Useful for:

- Pre-publish validation
- CI gate checks
- Understanding what's in your working tree before operations

## Output format

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

### Field meanings

- `is_clean` — True if no changes and all commits are pushed
- `branch` — Current branch name
- `dirty_files` — Array of modified/untracked files
- `unpushed_commits` — Number of commits not yet pushed
- `phase` — Repository state: "clean", "dirty", "advanced"

## Examples

### Clean repository

```bash
$ cargo cicd git status
{
  "is_clean": true,
  "branch": "main",
  "dirty_files": [],
  "unpushed_commits": 0,
  "phase": "clean"
}
```

### With uncommitted changes

```bash
$ cargo cicd git status
{
  "is_clean": false,
  "branch": "feat/new-feature",
  "dirty_files": [
    "src/core/handler.rs",
    "tests/integration.rs",
    "Cargo.toml"
  ],
  "unpushed_commits": 0,
  "phase": "dirty"
}
```

### With unpushed commits

```bash
$ cargo cicd git status
{
  "is_clean": false,
  "branch": "feat/new-feature",
  "dirty_files": [],
  "unpushed_commits": 3,
  "phase": "advanced"
}
```

### Mixed state

```bash
$ cargo cicd git status
{
  "is_clean": false,
  "branch": "feat/new-feature",
  "dirty_files": [
    "src/main.rs",
    "README.md"
  ],
  "unpushed_commits": 2,
  "phase": "dirty"
}
```

### In CI/CD

Check if clean:

```bash
$ cargo cicd git status | jq '.is_clean'
false

$ cargo cicd git status | jq '.branch'
"feat/new-feature"

$ cargo cicd git status | jq '.dirty_files'
["src/main.rs", "Cargo.toml"]
```

Fail if dirty:

```bash
#!/bin/bash
is_clean=$(cargo cicd git status | jq '.is_clean')
if [[ "$is_clean" == "false" ]]; then
  echo "Repository is dirty. Please commit changes."
  exit 1
fi
```

Only allow publishing from main:

```bash
#!/bin/bash
branch=$(cargo cicd git status | jq -r '.branch')
is_clean=$(cargo cicd git status | jq '.is_clean')

if [[ "$branch" != "main" ]]; then
  echo "Can only publish from main branch"
  exit 1
fi

if [[ "$is_clean" == "false" ]]; then
  echo "Cannot publish: repository is dirty"
  exit 1
fi

# OK to publish
```

## See also

- [`cargo cicd git close`](git-close.md) — Stage, commit, and push changes
- [Autonomic Policies](../autonomic-policies.md) — Configure git phase policies
