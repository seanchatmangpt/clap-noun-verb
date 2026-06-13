# cargo cicd trybuild changed

Show which trybuild fixtures need snapshot updates based on test code changes.

## Usage

```bash
cargo cicd trybuild changed
```

No options.

## Description

Scans your test code for changes and identifies trybuild fixtures that may need snapshot updates:

- Detects changes to files in `tests/` that might generate new macro compiler output
- Lists fixture files (`.rs` compile-fail tests) that match changed test code
- Helps you stay in sync before committing

Trybuild captures exact compiler output; when your macro changes, fixtures must be updated to match.

## Output format

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

### Field meanings

- `changed_fixtures` — Fixture files detected as stale
- `snapshot_path` — Path where trybuild stores snapshots
- `summary` — Human-readable status

## Examples

### Fixtures need updating

```bash
$ cargo cicd trybuild changed
{
  "changed_fixtures": [
    "tests/fixtures/compile_fail_01.rs",
    "tests/fixtures/compile_fail_02.rs"
  ],
  "snapshot_path": "tests/fixtures/",
  "summary": "2 fixtures need updates"
}
```

### All up to date

```bash
$ cargo cicd trybuild changed
{
  "changed_fixtures": [],
  "snapshot_path": "tests/fixtures/",
  "summary": "No stale fixtures"
}
```

### In CI/CD

Check if fixtures are out of sync:

```bash
$ cargo cicd trybuild changed | jq '.changed_fixtures | length'
2

$ cargo cicd trybuild changed | jq '.summary'
"2 fixtures need updates"
```

Update fixtures:

```bash
#!/bin/bash
count=$(cargo cicd trybuild changed | jq '.changed_fixtures | length')
if [[ $count -gt 0 ]]; then
  echo "Updating $count trybuild fixtures..."
  TRYBUILD=overwrite cargo test --test trybuild_tests
fi
```

### In GitHub Actions

```yaml
- name: Check trybuild fixtures
  id: trybuild
  run: |
    stale=$(cargo cicd trybuild changed | jq '.changed_fixtures | length')
    echo "stale-fixtures=$stale" >> $GITHUB_OUTPUT

- name: Update trybuild fixtures
  if: steps.trybuild.outputs.stale-fixtures > 0
  run: TRYBUILD=overwrite cargo test --test trybuild_tests
```

## Workflow tips

### Before committing

Always check for stale fixtures:

```bash
cargo cicd trybuild changed
if [ $? -ne 0 ]; then
  echo "Update fixtures and try again"
  TRYBUILD=overwrite cargo test --test trybuild_tests
fi
```

### In your test suite

Update all fixtures at once:

```bash
TRYBUILD=overwrite cargo test
```

Update only specific tests:

```bash
TRYBUILD=overwrite cargo test --test macros_ui
```

## See also

- [`cargo cicd test changed`](test-changed.md) — Detect other test changes
- [`cargo cicd status`](status.md) — Check trybuild_changed policy
