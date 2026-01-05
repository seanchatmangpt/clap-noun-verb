#!/bin/bash
# 21-Point Test Matrix Execution Script
# Validates all feature configurations for clap-noun-verb project
# Follows Andon signal workflow: Stop the Line on errors

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test result tracking
TOTAL_TESTS=21
PASSED=0
FAILED=0
SKIPPED=0
TEST_RESULTS_FILE="/home/user/clap-noun-verb/docs/test_results_raw.txt"

# Initialize results file
echo "Test Matrix Execution Results - $(date)" > "$TEST_RESULTS_FILE"
echo "========================================" >> "$TEST_RESULTS_FILE"
echo "" >> "$TEST_RESULTS_FILE"

# Function to log test result
log_test() {
    local tier="$1"
    local name="$2"
    local status="$3"
    local details="${4:-}"

    echo -e "${BLUE}[${tier}]${NC} ${name}: ${status} ${details}" | tee -a "$TEST_RESULTS_FILE"
}

# Function to run test configuration
test_config() {
    local tier="$1"
    local name="$2"
    local features="$3"
    local extra_args="${4:-}"

    echo ""
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    log_test "$tier" "$name" "RUNNING" "features: $features"

    local test_start=$(date +%s)
    local temp_output=$(mktemp)

    # Build command based on features
    local check_cmd="cargo make check"
    local test_cmd="cargo make test"
    local lint_cmd="cargo make lint"

    if [ "$features" != "default" ] && [ "$features" != "no-default" ]; then
        check_cmd="cargo make check --features $features"
        test_cmd="cargo make test --features $features"
        lint_cmd="cargo make lint --features $features"
    elif [ "$features" = "no-default" ]; then
        check_cmd="cargo make check --no-default-features"
        test_cmd="cargo make test --no-default-features"
        lint_cmd="cargo make lint --no-default-features"
    fi

    # Step 1: Compilation check (CRITICAL ANDON SIGNAL)
    echo -e "${YELLOW}  → Step 1/3: Compilation check${NC}"
    if ! eval "$check_cmd" > "$temp_output" 2>&1; then
        echo -e "${RED}  ✗ ANDON SIGNAL (RED): Compilation failed!${NC}"
        echo "  Error output:"
        cat "$temp_output" | tail -20
        log_test "$tier" "$name" "${RED}FAILED${NC}" "(compilation)"
        echo "FAILED: $name (compilation)" >> "$TEST_RESULTS_FILE"
        FAILED=$((FAILED + 1))
        rm -f "$temp_output"
        return 1
    fi

    # Check for warnings (HIGH ANDON SIGNAL)
    if grep -q "warning:" "$temp_output"; then
        echo -e "${YELLOW}  ⚠ ANDON SIGNAL (YELLOW): Warnings detected${NC}"
        grep "warning:" "$temp_output" | head -5
    fi

    # Step 2: Test execution (CRITICAL ANDON SIGNAL)
    echo -e "${YELLOW}  → Step 2/3: Running tests${NC}"
    if ! eval "$test_cmd" > "$temp_output" 2>&1; then
        echo -e "${RED}  ✗ ANDON SIGNAL (RED): Tests failed!${NC}"
        echo "  Test failures:"
        grep "FAILED" "$temp_output" || cat "$temp_output" | tail -20
        log_test "$tier" "$name" "${RED}FAILED${NC}" "(tests)"
        echo "FAILED: $name (tests)" >> "$TEST_RESULTS_FILE"
        FAILED=$((FAILED + 1))
        rm -f "$temp_output"
        return 1
    fi

    # Step 3: Linting (HIGH ANDON SIGNAL)
    echo -e "${YELLOW}  → Step 3/3: Clippy linting${NC}"
    if ! eval "$lint_cmd" > "$temp_output" 2>&1; then
        echo -e "${YELLOW}  ⚠ ANDON SIGNAL (YELLOW): Clippy warnings/errors${NC}"
        cat "$temp_output" | tail -10
        # Don't fail on lint warnings for now, just log
    fi

    local test_end=$(date +%s)
    local duration=$((test_end - test_start))

    # Record binary size if built
    local binary_size="N/A"
    if [ -f "target/debug/clap-noun-verb" ]; then
        binary_size=$(ls -lh target/debug/clap-noun-verb | awk '{print $5}')
    fi

    echo -e "${GREEN}  ✓ All checks passed${NC} (${duration}s, binary: ${binary_size})"
    log_test "$tier" "$name" "${GREEN}PASSED${NC}" "(${duration}s, ${binary_size})"
    echo "PASSED: $name (${duration}s)" >> "$TEST_RESULTS_FILE"
    PASSED=$((PASSED + 1))

    rm -f "$temp_output"
    return 0
}

# ============================================================================
# TIER 0: BASELINE (1 test)
# ============================================================================
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   TIER 0: BASELINE TEST (1 config)    ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

test_config "Tier 0" "Baseline (default features)" "default"

# ============================================================================
# TIER 1: INDIVIDUAL FEATURES (10 tests)
# ============================================================================
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  TIER 1: INDIVIDUAL (10 configs)      ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

TIER1_FEATURES=(
    "meta-framework"
    "rdf-composition"
    "executable-specs"
    "fractal-patterns"
    "discovery-engine"
    "federated-network"
    "learning-trajectories"
    "reflexive-testing"
    "economic-sim"
    "quantum-ready"
)

for feature in "${TIER1_FEATURES[@]}"; do
    test_config "Tier 1" "$feature" "$feature" || true
done

# ============================================================================
# TIER 2: META-FEATURES (3 tests)
# ============================================================================
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   TIER 2: META-FEATURES (3 configs)   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

test_config "Tier 2" "frontier-semantic" "frontier-semantic" || true
test_config "Tier 2" "frontier-intelligence" "frontier-intelligence" || true
test_config "Tier 2" "frontier-quality" "frontier-quality" || true

# ============================================================================
# TIER 3: CRITICAL COMBINATIONS (6 tests)
# ============================================================================
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  TIER 3: CRITICAL COMBOS (6 configs)  ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

test_config "Tier 3" "meta-framework + rdf-composition" "meta-framework,rdf-composition" || true
test_config "Tier 3" "discovery-engine + learning-trajectories" "discovery-engine,learning-trajectories" || true
test_config "Tier 3" "federated-network + rdf-composition" "federated-network,rdf-composition" || true
test_config "Tier 3" "economic-sim + learning-trajectories" "economic-sim,learning-trajectories" || true
test_config "Tier 3" "fractal-patterns + meta-framework" "fractal-patterns,meta-framework" || true
test_config "Tier 3" "executable-specs + reflexive-testing" "executable-specs,reflexive-testing" || true

# ============================================================================
# TIER 4: EXTREMES (2 tests)
# ============================================================================
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║    TIER 4: EXTREMES (2 configs)       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

test_config "Tier 4" "frontier-all (all features)" "frontier-all" || true
test_config "Tier 4" "minimal (no-default-features)" "no-default" || true

# ============================================================================
# FINAL SUMMARY
# ============================================================================
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         TEST MATRIX SUMMARY            ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "Total tests:   ${BLUE}${TOTAL_TESTS}${NC}"
echo -e "Passed:        ${GREEN}${PASSED}${NC}"
echo -e "Failed:        ${RED}${FAILED}${NC}"
echo -e "Skipped:       ${YELLOW}${SKIPPED}${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed! Ready for production.${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed. Review Andon signals above.${NC}"
    exit 1
fi
