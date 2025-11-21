#!/bin/bash
# Pre-commit hook script for playground-cli
# Run this before committing to ensure code quality
#
# Installation:
#   cp scripts/pre-commit.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit

set -e

echo "Running pre-commit checks..."

# Check formatting
echo "Checking code formatting..."
if ! cargo fmt --check; then
    echo "ERROR: Code formatting check failed"
    echo "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi
echo "Formatting: OK"

# Run clippy
echo "Running clippy lints..."
if ! cargo clippy --all-targets -- -D warnings; then
    echo "ERROR: Clippy found warnings or errors"
    echo "Fix the issues reported above"
    exit 1
fi
echo "Clippy: OK"

# Run tests
echo "Running tests..."
if ! cargo test --all-targets; then
    echo "ERROR: Tests failed"
    echo "Fix the failing tests before committing"
    exit 1
fi
echo "Tests: OK"

echo ""
echo "All pre-commit checks passed!"
