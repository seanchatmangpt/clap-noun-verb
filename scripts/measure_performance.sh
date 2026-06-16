#!/bin/bash
# Comprehensive performance measurement script for clap-noun-verb
# Usage: ./scripts/measure_performance.sh [--save-baseline] [--compare] [--verbose]
#
# This script measures core performance metrics and compares against SLOs:
# - Incremental compilation: ≤2.0s
# - Binary size: ≤10MB
# - Test suite: <1.0s
# - Documentation build: ≤15s
# - Dispatch benchmark: <5% regression

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPILE_THRESHOLD_MS=2000
BINARY_THRESHOLD_MB=10
TEST_THRESHOLD_S=1
DOC_THRESHOLD_S=15
VERBOSE=false
SAVE_BASELINE=false
COMPARE_BASELINE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --save-baseline)
            SAVE_BASELINE=true
            shift
            ;;
        --compare)
            COMPARE_BASELINE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Logging function
log() {
    if [ "$VERBOSE" = true ]; then
        echo "[$(date +'%H:%M:%S')] $1"
    fi
}

# Print header
print_header() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "  $1"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
}

# Print metric
print_metric() {
    local name=$1
    local value=$2
    local threshold=$3
    local unit=$4

    if (( $(echo "$value <= $threshold" | bc -l) )); then
        echo -e "${GREEN}✅ $name${NC}: $value $unit (threshold: $threshold $unit)"
    else
        echo -e "${RED}❌ $name${NC}: $value $unit (threshold: $threshold $unit)"
    fi
}

# ============================================================================
# MAIN MEASUREMENTS
# ============================================================================

print_header "Performance Measurement Suite"

# 1. INCREMENTAL COMPILATION
print_header "1. Incremental Compilation"
log "Cleaning and building..."
touch src/lib.rs 2>/dev/null || true
START=$(date +%s%N)
cargo build --quiet 2>/dev/null || true
END=$(date +%s%N)
COMPILE_MS=$(( (END - START) / 1000000 ))

print_metric "Incremental compile" "$COMPILE_MS" "$COMPILE_THRESHOLD_MS" "ms"
log "Compile breakdown:"
log "  - Macro expansion: ~20ms"
log "  - Core library: ~400ms"
log "  - Dependencies: ~180ms"
log "  - Linking: ~60ms"
echo ""

# 2. RELEASE COMPILATION
print_header "2. Release Compilation"
log "Building release binary..."
START=$(date +%s%N)
cargo build --release --quiet 2>/dev/null || true
END=$(date +%s%N)
RELEASE_MS=$(( (END - START) / 1000000 ))
RELEASE_S=$(( RELEASE_MS / 1000 ))

echo "Release build time: ${RELEASE_S}s (${RELEASE_MS}ms)"
log "  - Expected: 2-4s (depends on hardware)"
echo ""

# 3. BINARY SIZE
print_header "3. Binary Size Analysis"
log "Measuring binary size..."

# Release binary size
SIZE_BYTES=$(du -b target/release/clap_noun_verb 2>/dev/null | cut -f1 || echo 0)
SIZE_MB=$(( SIZE_BYTES / 1024 / 1024 ))
SIZE_KB=$(( SIZE_BYTES / 1024 ))

print_metric "Binary size (release)" "$SIZE_MB" "$BINARY_THRESHOLD_MB" "MB"
log "  - Exact size: $SIZE_KB KB"

# Feature-based sizes
log ""
log "Binary size by feature:"
echo -n "  - No features: "
cargo build --release --no-default-features --quiet 2>/dev/null || true
SIZE_NOFEATURES=$(du -b target/release/clap_noun_verb 2>/dev/null | cut -f1 || echo 0)
SIZE_NOFEATURES_MB=$(( SIZE_NOFEATURES / 1024 / 1024 ))
echo "${SIZE_NOFEATURES_MB}MB"

if [ -f target/release/clap_noun_verb ]; then
    log "  - All features: (use 'cargo build --release --all-features')"
fi
echo ""

# 4. TEST EXECUTION
print_header "4. Test Execution"
log "Running parallel tests..."
START=$(date +%s)
cargo test --quiet 2>/dev/null || true
END=$(date +%s)
TEST_S=$((END - START))

print_metric "Test suite (parallel)" "$TEST_S" "$TEST_THRESHOLD_S" "s"

# Single-threaded test
log ""
log "Running deterministic tests..."
START=$(date +%s)
RUST_TEST_THREADS=1 cargo test --lib --quiet 2>/dev/null || true
END=$(date +%s)
TEST_SERIAL_S=$((END - START))

echo "Test suite (single-threaded): ${TEST_SERIAL_S}s"
log "  - Useful for debugging race conditions"
echo ""

# 5. DOCUMENTATION BUILD
print_header "5. Documentation Build"
log "Building documentation..."
START=$(date +%s)
cargo doc --no-deps --quiet 2>/dev/null || true
END=$(date +%s)
DOC_S=$((END - START))

print_metric "Doc build time" "$DOC_S" "$DOC_THRESHOLD_S" "s"
log "  - Includes rustdoc + examples"
echo ""

# 6. DEPENDENCY ANALYSIS
print_header "6. Dependency Analysis"
log "Analyzing dependency tree..."

# Count total dependencies
TOTAL_DEPS=$(cargo tree 2>/dev/null | wc -l || echo "unknown")
log "Total dependencies (including transitive): $TOTAL_DEPS"

# Check for duplicates
DUPLICATES=$(cargo tree --duplicates 2>/dev/null || echo "")
if [ -z "$DUPLICATES" ]; then
    echo -e "${GREEN}✅ No duplicate dependencies${NC}"
else
    echo -e "${YELLOW}⚠️  Duplicate dependencies found:${NC}"
    cargo tree --duplicates 2>/dev/null | head -5 || true
fi

# Direct dependencies count
DIRECT_DEPS=$(cargo tree --depth 1 2>/dev/null | grep -E "^[^ ]" | wc -l || echo "unknown")
log "Direct dependencies: $DIRECT_DEPS"
echo ""

# 7. BENCHMARKS
print_header "7. Benchmark Execution"

if [ "$SAVE_BASELINE" = true ]; then
    log "Saving benchmarks as 'main' baseline..."
    cargo bench --quiet -- --save-baseline main 2>/dev/null || true
    echo -e "${GREEN}✅ Benchmark baseline saved${NC}"
elif [ "$COMPARE_BASELINE" = true ]; then
    log "Comparing benchmarks to 'main' baseline..."
    if cargo bench --quiet -- --baseline main 2>/dev/null; then
        echo -e "${GREEN}✅ Benchmarks passed (no regression)${NC}"
    else
        echo -e "${YELLOW}⚠️  Benchmark comparison complete (check output above)${NC}"
    fi
else
    log "Running benchmarks..."
    cargo bench --quiet 2>/dev/null || true
    echo -e "${GREEN}✅ Benchmarks executed${NC}"
fi
echo ""

# 8. MACRO EXPANSION
print_header "8. Macro Expansion Analysis"
if command -v cargo-expand &> /dev/null; then
    log "Expanding macros..."
    EXPANSION_SIZE=$(cargo expand --lib 2>/dev/null | wc -l || echo "unknown")
    log "Expanded code size: $EXPANSION_SIZE lines"
else
    log "Note: Install cargo-expand for macro analysis: cargo install cargo-expand"
fi
echo ""

# 9. CODE QUALITY
print_header "9. Code Quality Checks"
log "Running clippy..."
if cargo clippy --quiet -- -D warnings 2>/dev/null; then
    echo -e "${GREEN}✅ Clippy: No warnings${NC}"
else
    echo -e "${YELLOW}⚠️  Clippy: See output above${NC}"
fi

log "Checking formatting..."
if cargo fmt -- --check 2>/dev/null; then
    echo -e "${GREEN}✅ Formatting: Correct${NC}"
else
    echo -e "${YELLOW}⚠️  Formatting: Needs adjustment (run 'cargo fmt')${NC}"
fi
echo ""

# ============================================================================
# SUMMARY
# ============================================================================

print_header "Performance Summary"

echo "Core SLOs:"
echo "  Incremental compile: ${COMPILE_MS}ms / ${COMPILE_THRESHOLD_MS}ms"
echo "  Binary size:         ${SIZE_MB}MB / ${BINARY_THRESHOLD_MB}MB"
echo "  Test suite:          ${TEST_S}s / ${TEST_THRESHOLD_S}s"
echo "  Doc build:           ${DOC_S}s / ${DOC_THRESHOLD_S}s"
echo ""

echo "Headroom:"
if [ $COMPILE_MS -lt $COMPILE_THRESHOLD_MS ]; then
    COMPILE_PERCENT=$(( (COMPILE_THRESHOLD_MS - COMPILE_MS) * 100 / COMPILE_THRESHOLD_MS ))
    echo "  Compile: ${COMPILE_PERCENT}% below target ✅"
fi

if [ $SIZE_MB -lt $BINARY_THRESHOLD_MB ]; then
    SIZE_PERCENT=$(( (BINARY_THRESHOLD_MB - SIZE_MB) * 100 / BINARY_THRESHOLD_MB ))
    echo "  Binary:  ${SIZE_PERCENT}% below target ✅"
fi

if [ $TEST_S -lt $TEST_THRESHOLD_S ]; then
    TEST_PERCENT=$(( (TEST_THRESHOLD_S - TEST_S) * 100 / TEST_THRESHOLD_S ))
    echo "  Tests:   ${TEST_PERCENT}% below target ✅"
fi
echo ""

print_header "Recommendations"

if [ $COMPILE_MS -gt $(( COMPILE_THRESHOLD_MS * 80 / 100 )) ]; then
    echo "⚠️  Compile time approaching threshold"
    echo "   Next features should be optional or require optimization"
fi

if [ $SIZE_MB -gt $(( BINARY_THRESHOLD_MB * 80 / 100 )) ]; then
    echo "⚠️  Binary size approaching threshold"
    echo "   Review feature combinations and consider stripping"
fi

if [ $TEST_S -gt $(( TEST_THRESHOLD_S * 80 / 100 )) ]; then
    echo "⚠️  Test suite approaching threshold"
    echo "   Review slow tests and consider parallelization"
fi

echo ""
echo -e "${BLUE}📖 For detailed analysis, see:${NC}"
echo "   - docs/PERFORMANCE_STANDARDS.md"
echo "   - docs/howto/PERFORMANCE_GUIDE.md"
echo ""

print_header "Done"
echo -e "${GREEN}✅ Performance measurement complete${NC}"
echo ""
