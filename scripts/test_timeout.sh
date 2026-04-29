#!/bin/bash
# MCPP Test Timeout Enforcer (Poka-Yoke)
# Usage: ./scripts/test_timeout.sh [package_dir] [timeout_duration]

PKG_DIR=${1:-.}
TIMEOUT=${2:-0.01s} # 10ms default

echo "Enforcing $TIMEOUT cap on all tests in $PKG_DIR..."

# Pre-build to ensure compilation isn't part of the timeout
(cd "$PKG_DIR" && cargo test --no-run --quiet)

# Extract package name from Cargo.toml
PKG_NAME=$(grep -m1 "^name =" "$PKG_DIR/Cargo.toml" | cut -d'"' -f2)

# List all tests
TESTS=$(cargo test -p "$PKG_NAME" -- --list | grep ": test" | cut -d: -f1)

if [ -z "$TESTS" ]; then
  echo "No tests found."
  exit 0
fi

FAILED_TESTS=()

for test in $TESTS; do
  echo -n "Testing $test... "
  # Run individual test with timeout
  if timeout "$TIMEOUT" cargo test -p "$PKG_NAME" "$test" -- --quiet > /dev/null 2>&1; then
    echo "✅ PASS"
  else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
      echo "❌ TIMEOUT (> $TIMEOUT)"
      FAILED_TESTS+=("$test (TIMEOUT)")
    else
      echo "❌ FAIL (Exit $EXIT_CODE)"
      FAILED_TESTS+=("$test (FAILED)")
    fi
  fi
done

if [ ${#FAILED_TESTS[@]} -ne 0 ]; then
  echo ""
  echo "Total Failures: ${#FAILED_TESTS[@]}"
  for fail in "${FAILED_TESTS[@]}"; do
    echo "  - $fail"
  done
  exit 1
fi

echo "All tests passed within $TIMEOUT cap."
exit 0
