# Git Hooks CI/CD Integration Guide

Guide for integrating git hooks checks into CI/CD pipelines and ensuring consistency between local hooks and CI checks.

---

## Philosophy

**Hooks ≠ CI, but they should align**

- **Local hooks**: Fast feedback to developers (catch issues before push)
- **CI/CD checks**: Comprehensive verification (catch edge cases, environment differences)
- **Goal**: Developers catch issues locally; CI verifies before merge

---

## Command Parity

The same commands run locally and in CI, but with different configurations:

### Formatting Check

**Local (pre-commit hook)**:
```bash
cargo fmt -- --check
```

**CI/CD**:
```bash
cargo fmt -- --check
```

**Same command** ✅

### Linting Check

**Local (pre-commit hook)**:
```bash
cargo clippy --quiet -- -D warnings
```

**CI/CD** (more verbose):
```bash
cargo clippy -- -D warnings
```

**Nearly identical** ✅ (just remove `--quiet` for CI logs)

### License/Security Check

**Local (pre-commit hook)**:
```bash
cargo deny check licenses advisories bans --log-level off
```

**CI/CD** (with logging):
```bash
cargo deny check licenses advisories bans
```

**Same intent** ✅ (just more logging in CI)

### Compilation Check

**Local (pre-commit hook)**:
```bash
cargo check --quiet
```

**CI/CD**:
```bash
cargo check
cargo build
cargo build --release
```

**Local subset** ✅ (CI does more variations)

### Test Suite

**Local (pre-push hook)**:
```bash
cargo test --lib -- --test-threads=1
cargo test --test '*' -- --test-threads=1
cargo test --all-features -- --test-threads=1
cargo test --features federated-network
cargo test --features otel
```

**CI/CD** (comprehensive):
```bash
cargo test --lib
cargo test --all-features
cargo test --doc
cargo test --examples
cargo test --benches  # optional
```

**Local subset** ✅ (CI tests more, developers test enough)

---

## GitHub Actions Integration

### Minimal CI (mirrors hooks)

```yaml
name: Lint & Test
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Format check
        run: cargo fmt -- --check
      
      - name: Clippy
        run: cargo clippy -- -D warnings
      
      - name: Deny check
        run: cargo install cargo-deny && cargo deny check

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build
        run: cargo build --verbose
      
      - name: Tests
        run: cargo test --all-features --verbose
```

### Comprehensive CI (recommended)

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  # Stage 1: Fast checks (mirror pre-commit)
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Format check
        run: cargo fmt -- --check
      
      - name: Clippy
        run: cargo clippy -- -D warnings
      
      - name: Cargo deny
        run: cargo install cargo-deny && cargo deny check

  # Stage 2: Build & basic tests
  build:
    name: Build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Build
        run: cargo build --verbose
      
      - name: Build (release)
        run: cargo build --release --verbose
      
      - name: Build docs
        run: cargo doc --no-deps --verbose

  # Stage 3: Tests (mirror pre-push)
  test:
    name: Test
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Library tests
        run: cargo test --lib --verbose
      
      - name: Integration tests
        run: cargo test --test '*' --verbose
      
      - name: Doc tests
        run: cargo test --doc --verbose

  # Stage 4: Feature testing
  features:
    name: Features
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Test all features
        run: cargo test --all-features --verbose
      
      - name: Test federated-network
        run: cargo test --features federated-network --verbose
      
      - name: Test otel
        run: cargo test --features otel --verbose

  # Stage 5: Coverage & Quality (optional)
  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --verbose --workspace --timeout 120 --exclude-files benches tests
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  # Stage 6: Security (optional)
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: rustsec/audit-check-action@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### Matrix Testing

For comprehensive platform testing:

```yaml
test:
  name: Test
  runs-on: ${{ matrix.os }}
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      rust: [stable, beta, nightly]
  steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@${{ matrix.rust }}
    - uses: Swatinem/rust-cache@v2
    - run: cargo test --all-features
```

---

## GitLab CI Integration

### `.gitlab-ci.yml`

```yaml
image: rust:latest

stages:
  - lint
  - build
  - test

format:check:
  stage: lint
  script:
    - cargo fmt -- --check

clippy:
  stage: lint
  script:
    - cargo clippy -- -D warnings

deny:
  stage: lint
  script:
    - cargo install cargo-deny
    - cargo deny check

build:
  stage: build
  script:
    - cargo build --verbose
    - cargo build --release --verbose

test:lib:
  stage: test
  script:
    - cargo test --lib --verbose

test:all:
  stage: test
  script:
    - cargo test --all-features --verbose
```

---

## BitBucket Pipelines Integration

### `bitbucket-pipelines.yml`

```yaml
image: rust:latest

pipelines:
  default:
    - step:
        name: Format & Lint
        script:
          - cargo fmt -- --check
          - cargo clippy -- -D warnings
          - cargo install cargo-deny
          - cargo deny check
    
    - step:
        name: Build
        script:
          - cargo build --verbose
    
    - step:
        name: Test
        script:
          - cargo test --lib
          - cargo test --all-features
```

---

## Jenkins Integration

### `Jenkinsfile`

```groovy
pipeline {
    agent any
    
    environment {
        RUST_BACKTRACE = '1'
        CARGO_TERM_COLOR = 'always'
    }
    
    stages {
        stage('Lint') {
            steps {
                sh 'cargo fmt -- --check'
                sh 'cargo clippy -- -D warnings'
                sh 'cargo install cargo-deny && cargo deny check'
            }
        }
        
        stage('Build') {
            steps {
                sh 'cargo build --verbose'
            }
        }
        
        stage('Test') {
            steps {
                sh 'cargo test --lib --verbose'
                sh 'cargo test --all-features --verbose'
            }
        }
    }
    
    post {
        always {
            junit '**/target/test-results.xml'
        }
        failure {
            echo 'Pipeline failed'
        }
    }
}
```

---

## Local CI Testing

Developers can run the full CI pipeline locally using `cargo make`:

```bash
# Run what pre-commit would check
cargo make format-check
cargo make clippy
cargo make check

# Run what pre-push would check
cargo make test
cargo make test-all

# Run full CI locally
cargo make ci
```

Check `Makefile.toml` for all available tasks.

---

## Environment Configuration

### CI/CD Environment Variables

```bash
# Disable interactive prompts
CARGO_TERM_CI=true

# Show all output
CARGO_TERM_VERBOSE=true

# Use color
CARGO_TERM_COLOR=always

# Set test threads
RUST_TEST_THREADS=1  # For deterministic testing (optional in CI)

# Enable backtrace for debugging
RUST_BACKTRACE=1
```

### Docker/Container Setup

```dockerfile
FROM rust:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    cargo-deny \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Run hooks' equivalent checks
RUN cargo fmt -- --check
RUN cargo clippy -- -D warnings
RUN cargo deny check
RUN cargo build
RUN cargo test --all-features
```

---

## Consistency Matrix

| Check | Local Hook | CI/CD | Sync |
|-------|-----------|-------|------|
| rustfmt | ✅ | ✅ | ✅ |
| clippy | ✅ | ✅ | ✅ |
| cargo-deny | ✅ (optional) | ✅ | ✅ |
| cargo check | ✅ | ✅ | ✅ |
| cargo build | ❌ | ✅ | - |
| cargo build --release | ❌ | ✅ | - |
| cargo test --lib | ✅ (pre-push) | ✅ | ✅ |
| cargo test --all | ✅ (pre-push) | ✅ | ✅ |
| cargo test --doc | ❌ | ✅ | - |
| cargo test --examples | ❌ | ✅ | - |

**Rationale**: Local hooks test core functionality (fast). CI adds edge cases and documentation tests (comprehensive).

---

## Branch Protection Rules

### GitHub

Settings → Branches → Branch protection rules:

```
Branch name pattern: main

Require:
  ✅ Pull request reviews before merging (1+ reviewers)
  ✅ Dismiss stale pull request approvals when new commits are pushed
  ✅ Require status checks to pass before merging
      - lint (GitHub Actions)
      - build (GitHub Actions)
      - test (GitHub Actions)
      - features (GitHub Actions)
  ✅ Require branches to be up to date before merging
  ✅ Require code conversations to be resolved before merging
```

### GitLab

Settings → Merge requests:

```
Approval settings:
  • Approvals required: 1
  • Dismiss approvals when new commits added: Yes

Pipeline configuration:
  • Status checks must pass: All of [format, clippy, deny, build, test:lib, test:all]
```

---

## Merge Request/Pull Request Checks

### GitHub PR Checks

```yaml
# .github/pull_request_template.md
## Description

## Testing

- [ ] Pre-commit hooks passed locally
- [ ] Pre-push tests passed locally
- [ ] CI pipeline passed
- [ ] Manual testing completed

## Checklist

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings introduced
```

### GitLab Merge Request Checks

```markdown
<!-- .gitlab/merge_request_templates/default.md -->
## What does this MR do?

## Related issues

Fixes #(issue number)

## Testing

- [ ] Tested locally with `cargo make test`
- [ ] Pre-commit hooks passed
- [ ] Pre-push tests passed
- [ ] CI pipeline green

## Checklist

- [ ] Code review self-check
- [ ] Tests updated
- [ ] Documentation updated
```

---

## Failure Handling

### When Local Hooks Pass but CI Fails

**Likely causes**:
1. Different Rust version
2. Different platform (Linux vs macOS vs Windows)
3. New dependency with platform-specific code
4. Flaky test (intermittent failure)
5. CI runs additional checks (doc tests, examples)

**Recovery**:
```bash
# Match CI environment
rustup update
rustup install <CI_RUST_VERSION>

# Run CI checks locally
cargo test --doc
cargo test --examples
cargo test --all-features

# Update and retry
git add .
git commit --amend
git push --force-with-lease
```

### When CI Passes but Local Hooks Fail

**Unlikely but possible**:
- Local tool configuration different
- Hook installation corrupted

**Recovery**:
```bash
# Reinstall hooks
./.githooks/install.sh

# Update tools
rustup update
cargo install --force cargo-deny

# Verify
git commit --allow-empty -m "test"
```

---

## Metrics & Monitoring

### Track Hook Effectiveness

```bash
# Count format violations caught per month
git log --since="1 month ago" --grep="format" --oneline

# Count test failures prevented
git log --since="1 month ago" --grep="test" --oneline

# Average hook time
# (gather from team feedback)
```

### CI/CD Metrics

Use GitHub Actions insights:
- Job duration trends
- Failure rates
- Flaky test identification

Use GitLab CI metrics:
- Pipeline duration
- Success rate
- Performance trends

---

## Performance Optimization

### For Local Hooks

```bash
# Use incremental compilation (default in Rust 1.60+)
export CARGO_INCREMENTAL=1

# Cache compilation artifacts
export CARGO_INCREMENTAL=1
export RUSTFLAGS=-Cincremental=/tmp/cargo-incremental

# Limit parallel jobs if system is slow
export CARGO_BUILD_JOBS=2
```

### For CI/CD

```yaml
# Cache dependencies across builds
- uses: Swatinem/rust-cache@v2

# Parallel jobs for matrix testing
strategy:
  matrix:
    os: [ubuntu, macos, windows]
  max-parallel: 3

# Conditional steps
- if: github.event_name == 'pull_request'
  run: cargo test --lib  # Faster for PRs
```

---

## Troubleshooting CI/CD

### Job Timeout

**Problem**: Job takes >60 minutes

**Solution**:
```bash
# Skip slow features for PRs
- if: github.event_name == 'pull_request'
  run: cargo test --lib

# Full test on merge to main
- if: github.ref == 'refs/heads/main'
  run: cargo test --all-features
```

### Flaky Tests

**Problem**: Test passes locally but fails in CI

**Solution**:
```yaml
# Rerun failed tests
- run: cargo test --all-features --verbose

# Single-threaded for determinism
- run: cargo test --all-features -- --test-threads=1
```

### Tool Installation Failures

**Problem**: `cargo install cargo-deny` times out

**Solution**:
```yaml
# Use pre-compiled binaries
- name: Install cargo-deny
  uses: EmbarkStudios/cargo-deny-action@v1
```

---

## Documentation

Link CI configuration in README:

```markdown
## CI/CD

This project uses:
- **Local hooks**: Enforce quality before commit/push (`.githooks/`)
- **GitHub Actions**: Verify on every push to main and PRs

See:
- [Git Hooks Setup](GIT_HOOKS_QUICK_REFERENCE.md)
- [CI Configuration](.github/workflows/ci.yml)
```

---

## Security Considerations

### Secrets in CI/CD

Never hardcode secrets. Use:

```yaml
# GitHub
- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    token: ${{ secrets.CODECOV_TOKEN }}

# GitLab
- curl -H "PRIVATE-TOKEN: $CI_JOB_TOKEN" ...
```

### Dependency Caching

Verify cached dependencies are from trusted sources:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    cache-all-crates: true  # Cache all crates, not just workspace
```

---

## Summary

| Aspect | Details |
|--------|---------|
| **Philosophy** | Hooks = local fast feedback; CI = comprehensive verification |
| **Command alignment** | 95% identical between local and CI |
| **CI/CD coverage** | Lint, build, test, features, security, coverage |
| **Branch protection** | Require passing checks before merge |
| **Performance** | Hooks <2s; CI 2-5 minutes |
| **Documentation** | Linked in repo README |
| **Monitoring** | Track metrics to improve over time |

---

**Last updated**: 2026-06-14  
**For repo**: clap-noun-verb v26.6.1  
**Status**: ✅ Ready for CI/CD integration
