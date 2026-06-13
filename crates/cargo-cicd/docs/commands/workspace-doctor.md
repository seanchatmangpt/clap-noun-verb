# cargo cicd workspace doctor

Run comprehensive workspace diagnostics and report health.

## Usage

```bash
cargo cicd workspace doctor
```

No options.

## Description

Performs a complete workspace health check, testing:

- Rust toolchain version and components
- Dependency consistency and audit status
- Build integrity (no stale `.rlib` files)
- Test suite health (compile and run)
- Git repository validity
- Documentation generation

Returns a summary status and detailed check results. Useful for:

- Detecting broken builds before pushing
- Verifying CI environment configuration
- Troubleshooting integration issues

## Output format

```json
{
  "workspace_healthy": true,
  "checks": [
    {
      "name": "rust_version",
      "status": "ok",
      "detail": "rustc 1.75.0 (stable)"
    },
    {
      "name": "dependencies",
      "status": "ok",
      "detail": "All locked, audit clean"
    },
    {
      "name": "build_integrity",
      "status": "ok",
      "detail": "No stale artifacts"
    },
    {
      "name": "test_suite",
      "status": "ok",
      "detail": "All 156 tests passed"
    },
    {
      "name": "git",
      "status": "ok",
      "detail": "Repository clean, on main"
    },
    {
      "name": "documentation",
      "status": "ok",
      "detail": "Doc generation successful"
    }
  ],
  "summary": "Workspace is healthy"
}
```

### Field meanings

- `workspace_healthy` — True if all checks pass
- `checks` — Array of [name, status, detail] triples
  - `name` — Check category
  - `status` — "ok", "warning", or "error"
  - `detail` — Human-readable explanation
- `summary` — Overall health assessment

## Examples

### Healthy workspace

```bash
$ cargo cicd workspace doctor
{
  "workspace_healthy": true,
  "checks": [
    {
      "name": "rust_version",
      "status": "ok",
      "detail": "rustc 1.75.0 (stable)"
    },
    {
      "name": "dependencies",
      "status": "ok",
      "detail": "All locked, audit clean"
    },
    {
      "name": "build_integrity",
      "status": "ok",
      "detail": "No stale artifacts"
    },
    {
      "name": "test_suite",
      "status": "ok",
      "detail": "All 156 tests passed"
    },
    {
      "name": "git",
      "status": "ok",
      "detail": "Repository clean, on main"
    }
  ],
  "summary": "Workspace is healthy"
}
```

### With warnings

```bash
$ cargo cicd workspace doctor
{
  "workspace_healthy": false,
  "checks": [
    {
      "name": "rust_version",
      "status": "warning",
      "detail": "rustc 1.73.0 (MSRV is 1.74)"
    },
    {
      "name": "dependencies",
      "status": "warning",
      "detail": "2 outdated packages"
    },
    {
      "name": "build_integrity",
      "status": "ok",
      "detail": "No stale artifacts"
    },
    {
      "name": "test_suite",
      "status": "ok",
      "detail": "All 156 tests passed"
    }
  ],
  "summary": "Workspace has warnings"
}
```

### With errors

```bash
$ cargo cicd workspace doctor
{
  "workspace_healthy": false,
  "checks": [
    {
      "name": "rust_version",
      "status": "error",
      "detail": "rustc 1.70.0, MSRV is 1.74 — upgrade required"
    },
    {
      "name": "dependencies",
      "status": "error",
      "detail": "Audit found 1 high-severity vulnerability"
    },
    {
      "name": "test_suite",
      "status": "error",
      "detail": "3 tests failed"
    }
  ],
  "summary": "Workspace has errors"
}
```

### In CI/CD

Check if healthy:

```bash
$ cargo cicd workspace doctor | jq '.workspace_healthy'
true

$ cargo cicd workspace doctor | jq '.summary'
"Workspace is healthy"

$ cargo cicd workspace doctor | jq '.checks[] | select(.status != "ok")'
{
  "name": "dependencies",
  "status": "warning",
  "detail": "2 outdated packages"
}
```

Fail CI on errors:

```bash
#!/bin/bash
healthy=$(cargo cicd workspace doctor | jq '.workspace_healthy')
if [[ "$healthy" != "true" ]]; then
  echo "Workspace diagnostics failed"
  cargo cicd workspace doctor | jq '.checks[]'
  exit 1
fi
```

### In GitHub Actions

```yaml
- name: Run workspace diagnostics
  run: |
    result=$(cargo cicd workspace doctor | jq '.workspace_healthy')
    if [[ "$result" != "true" ]]; then
      echo "::error::Workspace has issues"
      cargo cicd workspace doctor | jq '.checks[] | select(.status != "ok")'
      exit 1
    fi
```

## What each check does

### rust_version

Compares installed `rustc --version` against MSRV in Cargo.toml. Fails if too old.

### dependencies

Runs `cargo audit` to detect security vulnerabilities. Warns if packages are outdated.

### build_integrity

Scans target directory for stale compiled artifacts. Safe to delete those files.

### test_suite

Compiles and runs all tests. Reports pass/fail count. Fails if any test fails.

### git

Verifies repository is clean and on a recognized branch. Warns if unpushed commits.

### documentation

Attempts to generate documentation with `cargo doc`. Fails if doc generation has errors.

## See also

- [`cargo cicd status`](status.md) — Quick health overview
- [`cargo cicd target show`](target-show.md) — Detailed target directory analysis
