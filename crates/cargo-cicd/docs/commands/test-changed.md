# cargo cicd test changed

Show which tests should run based on files changed since a base reference.

## Usage

```bash
cargo cicd test changed [OPTIONS]
```

## Options

- `--base-ref <REF>` — Git reference to compare against (default: `origin/main`)

## Description

Analyzes your git diff and determines which test suites are affected by your changes:

- Unit tests for modified files
- Integration tests for changed interfaces
- Doc tests for changed examples
- Trybuild tests for changed fixtures

Useful for:

- Running only relevant tests locally (faster feedback)
- Setting up CI test matrices (parallelize testing)
- Avoiding false negatives in selective test runs

## Output format

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

### Field meanings

- `test_plan` — Array of [test type, affected file] pairs
- `is_conservative` — If true, recommend running all tests (due to uncertainty)
- `summary` — Human-readable count and recommendation

## Examples

### Default (compare to origin/main)

```bash
$ cargo cicd test changed
{
  "test_plan": [
    ["unit", "src/core/validation.rs"],
    ["integration", "src/cli/handler.rs"]
  ],
  "is_conservative": false,
  "summary": "Run 2 test suites"
}
```

### Compare to different branch

```bash
$ cargo cicd test changed --base-ref origin/develop
{
  "test_plan": [
    ["unit", "src/core/validation.rs"],
    ["unit", "src/core/registry.rs"],
    ["integration", "src/cli/handler.rs"],
    ["integration", "src/cli/router.rs"],
    ["doc", "src/lib.rs"]
  ],
  "is_conservative": false,
  "summary": "Run 5 test suites"
}
```

### Conservative mode (fallback to all tests)

```bash
$ cargo cicd test changed --base-ref some/obscure/ref
{
  "test_plan": [
    ["all", "entire workspace"]
  ],
  "is_conservative": true,
  "summary": "Run all tests (conservative due to ref not found)"
}
```

### No changes

```bash
$ cargo cicd test changed  # (when on main with no staged changes)
{
  "test_plan": [],
  "is_conservative": false,
  "summary": "No changes detected"
}
```

### In CI/CD

Parse test plan:

```bash
$ cargo cicd test changed --base-ref origin/main | jq '.test_plan'
[
  ["unit", "src/core/validation.rs"],
  ["integration", "src/cli/handler.rs"]
]

$ cargo cicd test changed | jq '.is_conservative'
false

$ cargo cicd test changed | jq '.summary'
"Run 2 test suites"
```

Run only affected tests:

```bash
#!/bin/bash
plan=$(cargo cicd test changed --base-ref origin/main | jq -r '.test_plan[] | .[0]' | sort | uniq)

for test_type in $plan; do
  case "$test_type" in
    unit)
      cargo test --lib
      ;;
    integration)
      cargo test --test '*'
      ;;
    doc)
      cargo test --doc
      ;;
    all)
      cargo test
      ;;
  esac
done
```

### In GitHub Actions

```yaml
- name: Determine test plan
  id: test-plan
  run: |
    plan=$(cargo cicd test changed | jq -r '.test_plan[] | .[0]' | sort | uniq | tr '\n' ',' | sed 's/,$//') 
    echo "test-types=$plan" >> $GITHUB_OUTPUT
    echo "Conservative: $(cargo cicd test changed | jq '.is_conservative')"

- name: Run unit tests
  if: contains(steps.test-plan.outputs.test-types, 'unit')
  run: cargo test --lib

- name: Run integration tests
  if: contains(steps.test-plan.outputs.test-types, 'integration')
  run: cargo test --test '*'
```

## See also

- [`cargo cicd trybuild changed`](trybuild-changed.md) — Detect changed trybuild fixtures
- [Autonomic Policies](../autonomic-policies.md) — Configure test selection policies
