#!/bin/bash
# Pre-Release Quality Gate for clap-noun-verb
# Design for Lean Six Sigma (DfLSS) Automated Validation
# Exit on first failure (Andon signal - Stop the Line)

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DESIRED_VERSION="${1:-5.0.0}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo ""
echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "${BLUE}  PRE-RELEASE QUALITY GATE - DfLSS Validation${NC}"
echo "${BLUE}  Target Version: $DESIRED_VERSION${NC}"
echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Track gate failures
GATES_PASSED=0
GATES_FAILED=0

# Function to print gate header
print_gate() {
    local gate_num=$1
    local gate_name=$2
    echo "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo "${BLUE}│ GATE $gate_num: $gate_name${NC}"
    echo "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"
}

# Function to print success
print_success() {
    echo "${GREEN}✓ $1${NC}"
}

# Function to print failure
print_failure() {
    echo "${RED}✗ FAIL: $1${NC}"
}

# Function to print warning
print_warning() {
    echo "${YELLOW}⚠ WARNING: $1${NC}"
}

# Function to increment gate counters
gate_pass() {
    GATES_PASSED=$((GATES_PASSED + 1))
}

gate_fail() {
    GATES_FAILED=$((GATES_FAILED + 1))
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 1: Version Consistency (CRITICAL)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
print_gate "1" "Version Consistency"

main_version=$(grep -E '^version = "' Cargo.toml | head -1 | sed 's/version = "//;s/"//')
macros_version=$(grep -E '^version = "' clap-noun-verb-macros/Cargo.toml | head -1 | sed 's/version = "//;s/"//')

# Check main crate version
if [ "$main_version" != "$DESIRED_VERSION" ]; then
    print_failure "Main crate version = $main_version (expected $DESIRED_VERSION)"
    echo ""
    echo "   Fix with:"
    echo "   sed -i '' 's/version = \"$main_version\"/version = \"$DESIRED_VERSION\"/' Cargo.toml"
    gate_fail
    exit 1
else
    print_success "Main crate version: $main_version"
fi

# Check macros crate version
if [ "$macros_version" != "$DESIRED_VERSION" ]; then
    print_failure "Macros crate version = $macros_version (expected $DESIRED_VERSION)"
    echo ""
    echo "   Fix with:"
    echo "   sed -i '' 's/version = \"$macros_version\"/version = \"$DESIRED_VERSION\"/' clap-noun-verb-macros/Cargo.toml"
    gate_fail
    exit 1
else
    print_success "Macros crate version: $macros_version"
fi

# Check dependency reference in main Cargo.toml
dep_version=$(grep 'clap-noun-verb-macros = { version = "' Cargo.toml | sed 's/.*version = "//;s/".*//')
if [ -n "$dep_version" ] && [ "$dep_version" != "$DESIRED_VERSION" ]; then
    print_warning "Dependency reference version = $dep_version (expected $DESIRED_VERSION)"
    echo "   This is usually OK if using path dependency, but verify compatibility"
fi

gate_pass
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 2: Compilation Check (CRITICAL)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
print_gate "2" "Compilation Check"

if ! cargo make check > /tmp/check-output.txt 2>&1; then
    print_failure "Compilation errors detected"
    echo ""
    cat /tmp/check-output.txt
    gate_fail
    exit 1
fi

print_success "Compilation: PASS"
gate_pass
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 3: Test Pass Rate (CRITICAL)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
print_gate "3" "Test Pass Rate"

if ! cargo make test > /tmp/test-output.txt 2>&1; then
    print_failure "Tests failed"
    echo ""
    cat /tmp/test-output.txt
    gate_fail
    exit 1
fi

# Check for test failures
test_failures=$(grep -c "FAILED" /tmp/test-output.txt || echo "0")
if [ "$test_failures" != "0" ]; then
    print_failure "$test_failures test failures"
    echo ""
    grep "FAILED" /tmp/test-output.txt
    gate_fail
    exit 1
fi

# Count total tests
test_count=$(grep -E "test result:" /tmp/test-output.txt | tail -1 | sed 's/.*passed; \([0-9]*\) failed.*/\1/')
print_success "Tests: 100% PASS (0 failures)"
gate_pass
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 4: Compiler Warnings (HIGH PRIORITY)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
print_gate "4" "Compiler Warnings"

cargo make lint > /tmp/lint-output.txt 2>&1 || true
warning_count=$(grep -c "warning:" /tmp/lint-output.txt || echo "0")

if [ "$warning_count" != "0" ]; then
    print_failure "$warning_count compiler warnings detected"
    echo ""
    echo "   Warnings found:"
    grep "warning:" /tmp/lint-output.txt | head -10

    if [ "$warning_count" -gt 10 ]; then
        echo "   ... and $((warning_count - 10)) more warnings"
    fi

    echo ""
    echo "   Fix warnings before release. Run:"
    echo "   cargo make lint"
    gate_fail
    exit 1
fi

print_success "Warnings: 0"
gate_pass
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 5: Documentation (CRITICAL)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
print_gate "5" "Documentation"

# Check CHANGELOG
if ! grep -q "## \[$DESIRED_VERSION\]" CHANGELOG.md; then
    print_failure "CHANGELOG missing [$DESIRED_VERSION] entry"
    echo ""
    echo "   Add entry to CHANGELOG.md:"
    echo "   ## [$DESIRED_VERSION] - $(date +%Y-%m-%d)"
    echo "   ### Added"
    echo "   - Feature descriptions here"
    gate_fail
    exit 1
else
    print_success "CHANGELOG has [$DESIRED_VERSION] entry"
fi

# Check README (should have version references)
readme_version_count=$(grep -c "\"$DESIRED_VERSION\"" README.md || echo "0")
if [ "$readme_version_count" = "0" ]; then
    print_failure "README missing version $DESIRED_VERSION references"
    echo ""
    echo "   Update README.md with:"
    echo "   sed -i '' 's/\"[0-9]\+\.[0-9]\+\.[0-9]\+\"/\"$DESIRED_VERSION\"/g' README.md"
    gate_fail
    exit 1
else
    print_success "README has version $DESIRED_VERSION ($readme_version_count references)"
fi

gate_pass
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 6: Build System (MEDIUM PRIORITY)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
print_gate "6" "Build System"

if ! cargo make build-examples > /tmp/build-output.txt 2>&1; then
    print_failure "Example builds failed"
    echo ""
    cat /tmp/build-output.txt | tail -20
    gate_fail
    exit 1
fi

print_success "Examples: BUILD PASS"
gate_pass
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 7: Git Status (MEDIUM PRIORITY)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
print_gate "7" "Git Status"

# Check for uncommitted changes
if ! git diff --quiet; then
    print_warning "Uncommitted changes detected"
    echo "   Run 'git status' to review"
    echo "   Commit changes before release"
else
    print_success "Git working directory clean"
fi

# Check for unpushed commits
if [ "$(git log origin/main..HEAD --oneline | wc -l)" -gt 0 ]; then
    print_warning "Unpushed commits detected"
    echo "   Push commits before release: git push origin main"
else
    print_success "All commits pushed to origin"
fi

gate_pass
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# FINAL RESULTS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo ""
echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$GATES_FAILED" -eq 0 ]; then
    echo "${GREEN}  ✓ ALL GATES PASSED - Ready for Release!${NC}"
    echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "  ${GREEN}Gates Passed: $GATES_PASSED${NC}"
    echo ""
    echo "  ${BLUE}Next steps:${NC}"
    echo "    1. cargo make publish-dry-run-macros"
    echo "    2. cargo make publish-macros"
    echo "    3. cargo make publish-dry-run"
    echo "    4. cargo make publish"
    echo "    5. Verify on crates.io"
    echo "    6. Create git tag: git tag v$DESIRED_VERSION"
    echo "    7. Push tag: git push origin v$DESIRED_VERSION"
    echo ""
    exit 0
else
    echo "${RED}  ✗ GATES FAILED - Not Ready for Release${NC}"
    echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "  ${GREEN}Gates Passed: $GATES_PASSED${NC}"
    echo "  ${RED}Gates Failed: $GATES_FAILED${NC}"
    echo ""
    echo "  ${RED}Fix all failures before proceeding with release.${NC}"
    echo ""
    exit 1
fi
