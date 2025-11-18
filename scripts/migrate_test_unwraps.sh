#!/bin/bash
# Migration script for converting .unwrap()/.expect() to test-safe patterns
#
# This script converts test files to use the new test_prelude utilities:
# - .unwrap() → .test_unwrap()
# - .expect("msg") → .test_expect("msg")
# - Adds test_prelude import at the top
#
# Usage: ./scripts/migrate_test_unwraps.sh [test_file.rs]

set -e

TEST_FILE="$1"

if [ -z "$TEST_FILE" ]; then
    echo "Usage: $0 <test_file.rs>"
    exit 1
fi

if [ ! -f "$TEST_FILE" ]; then
    echo "Error: File not found: $TEST_FILE"
    exit 1
fi

echo "Migrating: $TEST_FILE"

# Backup original
cp "$TEST_FILE" "${TEST_FILE}.bak"

# Add test_prelude import if not present
if ! grep -q "use.*test_prelude" "$TEST_FILE"; then
    # Find the first use statement and insert before it
    sed -i.tmp '0,/^use /s/^use /use tests::common::test_prelude::*;\nuse /' "$TEST_FILE"
    rm -f "${TEST_FILE}.tmp"
fi

# Replace .unwrap() with .test_unwrap()
sed -i.tmp 's/\.unwrap()/\.test_unwrap()/g' "$TEST_FILE"
rm -f "${TEST_FILE}.tmp"

# Replace .expect("...") with .test_expect("...")
# This regex handles multi-line expects
perl -i -pe 's/\.expect\(/\.test_expect\(/g' "$TEST_FILE"

# Count changes
CHANGES=$(diff -u "${TEST_FILE}.bak" "$TEST_FILE" | grep -c '^[+-]' || true)

echo "✓ Migration complete: $CHANGES lines changed"
echo "  Original backed up to: ${TEST_FILE}.bak"
echo ""
echo "To verify:"
echo "  cargo clippy --tests -- -D clippy::unwrap_used -D clippy::expect_used"
echo ""
echo "To rollback:"
echo "  mv ${TEST_FILE}.bak $TEST_FILE"
