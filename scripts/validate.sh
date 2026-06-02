#!/usr/bin/env bash
# Construct8 Integration Validation Script
# Runs formatting, linting, and testing across all crates

set -euo pipefail

echo "=== Construct8 Validation Gate ==="
echo ""

# Formatting check
echo "Step 1: Checking code formatting (cargo fmt --all --check)"
echo "============================================================"
cargo fmt --all --check || {
    echo "ERROR: Code formatting check failed"
    exit 1
}
echo "✓ Code formatting check passed"
echo ""

# Clippy linting
echo "Step 2: Running Clippy linter (cargo clippy --workspace --lib)"
echo "=============================================================="
cargo clippy --workspace --lib -- -D warnings || {
    echo "ERROR: Clippy linting failed"
    exit 1
}
echo "✓ Clippy linting passed"
echo ""

# Unit and integration tests
echo "Step 3: Running test suite (cargo test --workspace)"
echo "===================================================="
cargo test --workspace || {
    echo "ERROR: Test suite failed"
    exit 1
}
echo "✓ Test suite passed"
echo ""

# Example builds (to ensure all dependencies are available)
echo "Step 4: Building examples"
echo "========================="
cargo build --examples || {
    echo "ERROR: Example build failed"
    exit 1
}
echo "✓ Example builds successful"
echo ""

echo "=== Validation Complete: ALL CHECKS PASSED ==="
exit 0
