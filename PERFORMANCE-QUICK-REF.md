# Performance Quick Reference

**clap-noun-verb v26.9.1** — Quick commands for common performance tasks.

## SLO Status

```bash
cargo make slo-check
```

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Incremental Build | 0.66s | ≤2s | ✅ PASS |
| Binary Size | 2.2MB | ≤10MB | ✅ PASS |
| Test Suite | <1s | <1s (parallel) | ✅ PASS |

---

## Quick Commands

### Development (Fastest Iteration)

```bash
# Fast incremental build
cargo build

# Check without linking
cargo check

# Run tests quickly
cargo make test

# Single test
cargo test test_name --quiet
```

**Expected**: 0.4-0.7s per command

### Diagnostics

```bash
# Check what changed
cargo make format-check
cargo make clippy

# Measure compile time
time cargo build --quiet

# Measure after clean
cargo clean && time cargo build --release --quiet

# Check binary size
ls -lh target/release/clap-noun-verb
```

### Feature Testing

```bash
# No features (fastest)
cargo check --no-default-features

# Default features
cargo check

# All features
cargo make check-all

# Specific feature
cargo check --features repl
```

### Benchmarking

```bash
# Run benchmarks
cargo make bench

# Compare to baseline
cargo make bench-compare

# Save new baseline
cargo make bench-baseline
```

### Caching

```bash
# Enable sccache for team caching
export RUSTC_WRAPPER=sccache

# Check sccache stats
sccache -s

# Clear cache
cargo clean
rm -rf ~/.cargo/registry/cache/
```

---

## Common Scenarios

### "Build is slow (>2s)"

```bash
# 1. Check what changed
git status

# 2. Check dependencies
git diff Cargo.toml

# 3. Rebuild from scratch
cargo clean && time cargo build --release --quiet

# 4. Profile build
cargo install cargo-build-times
cargo build-times
```

### "Binary is large (>10MB)"

```bash
# Check size
cargo build --release && ls -lh target/release/clap-noun-verb

# Compare features
cargo build --release --no-default-features
ls -lh target/release/clap-noun-verb

# Analyze bloat
cargo install cargo-bloat
cargo bloat --release -n 10
```

### "Tests are slow (>1s)"

```bash
# Run in parallel (default)
time cargo test --quiet

# Run with single thread
time cargo test --quiet -- --test-threads=1

# Profile slow test
cargo test test_name -- --nocapture --test-threads=1
```

### "Macro compilation is slow"

```bash
# Expand macros
cargo install cargo-expand
cargo expand --lib

# Measure macro overhead
time cargo build -p clap-noun-verb-macros --quiet
```

---

## Performance Targets (SLOs)

### Incremental Compilation (Target: ≤2s)

Measured after touching one source file:

```bash
touch src/lib.rs
time cargo build --quiet
```

**Status**: 0.66s ✅ (33% margin)

### Binary Size (Target: ≤10MB)

Measured in release mode (stripped):

```bash
cargo build --release --quiet
strip target/release/clap-noun-verb
ls -lh target/release/clap-noun-verb
```

**Status**: 2.2MB ✅ (78% margin)

### Test Suite (Target: <1s parallel)

All tests must complete in <1 second:

```bash
time cargo make test
```

**Status**: 0.3-0.6s ✅ (40-70% margin)

---

## Environment Setup (Optional)

### Enable sccache (Distributed Cache)

```bash
# Install
cargo install sccache

# Add to ~/.bashrc or ~/.zshrc
export RUSTC_WRAPPER=sccache
export SCCACHE_DIR=$HOME/.cache/sccache
export SCCACHE_MAX_FRAME_LENGTH=104857600

# Verify
sccache -s
```

**Expected**: 50-70% speedup on cache hits.

### Enable mold Linker (Linux/macOS with Homebrew)

```bash
# Install
brew install mold  # macOS
# OR
sudo apt-get install mold  # Ubuntu 22.04+

# Add to .cargo/config.toml
# See PERFORMANCE.md for details

# Verify
which mold
```

**Expected**: 30-50% faster linking.

---

## Files & Documentation

- **Full guide**: [`PERFORMANCE.md`](./PERFORMANCE.md)
  - Detailed strategies for all metrics
  - Tool installation & configuration
  - CI/CD integration examples
  - Troubleshooting guide

- **Benchmarks**: [`benches/dispatch.rs`](./benches/dispatch.rs)
  - Command registry lookup
  - Argument parsing
  - Serialization
  - Error handling

- **Build tasks**: [`Makefile.toml`](./Makefile.toml)
  - `cargo make test` — Quick tests
  - `cargo make bench` — Run benchmarks
  - `cargo make slo-check` — Validate SLOs
  - `cargo make ci` — Full CI pipeline

---

## CI Integration

Add to GitHub Actions workflow:

```yaml
- name: Validate SLOs
  run: cargo make slo-check

- name: Measure performance
  run: |
    echo "⏱️ Incremental build:"
    touch src/lib.rs && time cargo build --quiet
    echo "📦 Binary size:"
    ls -lh target/release/clap-noun-verb
    echo "🧪 Test suite:"
    time cargo make test
```

---

## Troubleshooting

| Issue | Command | Expected |
|-------|---------|----------|
| Build >2s | `time cargo build --quiet` | ≤2s |
| Binary >10MB | `ls -lh target/release/clap-noun-verb` | ≤10MB |
| Tests >1s | `time cargo make test` | <1s |
| Slow macro | `cargo expand --lib \| wc -l` | <1000 lines |
| Missing cache | `sccache -s` | >0 cache hits |

---

## Links

- [Full Performance Guide](./PERFORMANCE.md)
- [Contributing Guide](./CONTRIBUTING.md)
- [Architecture Guide](./CLAUDE.md)

**Last Updated**: 2026-06-14
