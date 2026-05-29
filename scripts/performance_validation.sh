#!/bin/bash
# Performance Validation Script for ggen-clap-noun-verb Integration
# Measures compilation time, execution time, memory usage, and SLO compliance

set -e

echo "════════════════════════════════════════════════════════════════"
echo "  GGEN-CLAP-NOUN-VERB PERFORMANCE VALIDATION"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# SLO Targets
SLO_CLI_EXEC_MS=200
SLO_MEMORY_MB=15
SLO_INCREMENTAL_COMPILE_S=15

# Results tracking
RESULTS_FILE="/tmp/performance_results.txt"
> "$RESULTS_FILE"

# Function to check SLO
check_slo() {
    local name="$1"
    local actual="$2"
    local target="$3"
    local unit="$4"

    if (( $(echo "$actual <= $target" | bc -l) )); then
        echo -e "${GREEN}✅ PASS${NC}: $name = $actual$unit (target: ≤ $target$unit)"
        echo "PASS: $name = $actual$unit (target: ≤ $target$unit)" >> "$RESULTS_FILE"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}: $name = $actual$unit (target: ≤ $target$unit)"
        echo "FAIL: $name = $actual$unit (target: ≤ $target$unit)" >> "$RESULTS_FILE"
        return 1
    fi
}

# Function to get current time in milliseconds
current_time_ms() {
    perl -MTime::HiRes=time -e 'printf "%.0f\n", time * 1000'
}

# Function to measure time in milliseconds
measure_time_ms() {
    local cmd="$1"
    local start=$(current_time_ms)
    eval "$cmd" > /dev/null 2>&1
    local end=$(current_time_ms)
    echo $((end - start))
}

echo "📊 1. COMPILATION TIME MEASUREMENT"
echo "────────────────────────────────────────────────────────────────"

# Clean for fresh build
echo "  Cleaning build artifacts..."
cargo clean > /dev/null 2>&1

# Measure full compilation time
echo "  Measuring full compilation time..."
FULL_COMPILE_START=$(current_time_ms)
cargo build --release > /dev/null 2>&1
FULL_COMPILE_END=$(current_time_ms)
FULL_COMPILE_MS=$((FULL_COMPILE_END - FULL_COMPILE_START))
FULL_COMPILE_S=$(echo "scale=2; $FULL_COMPILE_MS / 1000" | bc)
echo "  Full compilation: ${FULL_COMPILE_S}s"
echo "MEASUREMENT: Full compilation = ${FULL_COMPILE_S}s" >> "$RESULTS_FILE"

# Touch a file to trigger incremental compile
echo "  Measuring incremental compilation time..."
touch src/lib.rs
INCREMENTAL_START=$(current_time_ms)
cargo build --release > /dev/null 2>&1
INCREMENTAL_END=$(current_time_ms)
INCREMENTAL_MS=$((INCREMENTAL_END - INCREMENTAL_START))
INCREMENTAL_S=$(echo "scale=2; $INCREMENTAL_MS / 1000" | bc)
echo "  Incremental compilation: ${INCREMENTAL_S}s"
echo "MEASUREMENT: Incremental compilation = ${INCREMENTAL_S}s" >> "$RESULTS_FILE"

# Check SLO
check_slo "Incremental compilation" "$INCREMENTAL_S" "$SLO_INCREMENTAL_COMPILE_S" "s"
INCREMENTAL_SLO=$?

echo ""
echo "📊 2. CLI EXECUTION TIME MEASUREMENT"
echo "────────────────────────────────────────────────────────────────"

# Build example CLI if it exists
if [ -f "examples/tutorial/basic.rs" ]; then
    echo "  Building tutorial/basic binary..."
    cargo build --release --bin tutorial_basic > /dev/null 2>&1

    # Measure execution time
    echo "  Measuring CLI execution time..."
    # Warm up to avoid OS first-run scan/cache overhead
    ./target/release/tutorial_basic --help > /dev/null 2>&1
    CLI_EXEC_MS=$(measure_time_ms "./target/release/tutorial_basic --help")
    echo "  CLI execution: ${CLI_EXEC_MS}ms"
    echo "MEASUREMENT: CLI execution = ${CLI_EXEC_MS}ms" >> "$RESULTS_FILE"

    # Check SLO
    check_slo "CLI execution" "$CLI_EXEC_MS" "$SLO_CLI_EXEC_MS" "ms"
    CLI_SLO=$?
else
    echo "  ⚠️  No example CLI found to measure"
    CLI_SLO=1
fi

echo ""
echo "📊 3. MEMORY USAGE MEASUREMENT"
echo "────────────────────────────────────────────────────────────────"

# Check if /usr/bin/time exists
if command -v /usr/bin/time > /dev/null 2>&1; then
    echo "  Measuring memory usage with /usr/bin/time..."
    if [ -f "./target/release/tutorial_basic" ]; then
        if /usr/bin/time -v true >/dev/null 2>&1; then
            MEM_OUTPUT=$(/usr/bin/time -v ./target/release/tutorial_basic --help 2>&1 | grep "Maximum resident set size")
            MEM_KB=$(echo "$MEM_OUTPUT" | awk '{print $6}')
        elif /usr/bin/time -l true >/dev/null 2>&1; then
            MEM_OUTPUT=$(/usr/bin/time -l ./target/release/tutorial_basic --help 2>&1 | grep -i "maximum resident set size")
            MEM_BYTES=$(echo "$MEM_OUTPUT" | awk '{print $1}')
            MEM_KB=$((MEM_BYTES / 1024))
        else
            MEM_KB=0
        fi

        MEM_MB=$(echo "scale=2; $MEM_KB / 1024" | bc)
        echo "  Peak memory: ${MEM_MB}MB"
        echo "MEASUREMENT: Peak memory = ${MEM_MB}MB" >> "$RESULTS_FILE"

        # Check SLO
        check_slo "Memory usage" "$MEM_MB" "$SLO_MEMORY_MB" "MB"
        MEM_SLO=$?
    else
        echo "  ⚠️  Binary not found"
        MEM_SLO=1
    fi
else
    echo "  ⚠️  /usr/bin/time not available, using cargo build memory estimation"
    MEM_SLO=1
fi

echo ""
echo "📊 4. BENCHMARK SUITE EXECUTION"
echo "────────────────────────────────────────────────────────────────"

# Run hot_path_benchmarks (no features required)
echo "  Running hot_path_benchmarks..."
if cargo bench --bench hot_path_benchmarks --no-fail-fast 2>&1 | tee -a "$RESULTS_FILE"; then
    echo -e "  ${GREEN}✅${NC} hot_path_benchmarks completed"
else
    echo -e "  ${RED}❌${NC} hot_path_benchmarks failed"
fi

echo ""
# Run phase1_foundation_benchmarks (no features required)
echo "  Running phase1_foundation_benchmarks..."
if cargo bench --bench phase1_foundation_benchmarks --no-fail-fast 2>&1 | tee -a "$RESULTS_FILE"; then
    echo -e "  ${GREEN}✅${NC} phase1_foundation_benchmarks completed"
else
    echo -e "  ${RED}❌${NC} phase1_foundation_benchmarks failed"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "  PERFORMANCE VALIDATION SUMMARY"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Count pass/fail
PASS_COUNT=0
FAIL_COUNT=0

if [ $INCREMENTAL_SLO -eq 0 ]; then ((PASS_COUNT++)); else ((FAIL_COUNT++)); fi
if [ $CLI_SLO -eq 0 ]; then ((PASS_COUNT++)); else ((FAIL_COUNT++)); fi
if [ $MEM_SLO -eq 0 ]; then ((PASS_COUNT++)); else ((FAIL_COUNT++)); fi

echo "SLO Compliance: $PASS_COUNT passed, $FAIL_COUNT failed"
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✅ ALL SLOs MET - PERFORMANCE VALIDATED${NC}"
    echo "RESULT: ALL SLOs MET" >> "$RESULTS_FILE"
    exit 0
else
    echo -e "${RED}❌ PERFORMANCE SLOs FAILED - OPTIMIZATION NEEDED${NC}"
    echo "RESULT: PERFORMANCE SLOs FAILED" >> "$RESULTS_FILE"
    echo ""
    echo "RECOMMENDATIONS:"

    if [ $INCREMENTAL_SLO -ne 0 ]; then
        echo "  - Incremental compilation: Review dependency graph, enable parallel compilation"
    fi

    if [ $CLI_SLO -ne 0 ]; then
        echo "  - CLI execution: Profile startup time, reduce dependencies, optimize argument parsing"
    fi

    if [ $MEM_SLO -ne 0 ]; then
        echo "  - Memory usage: Profile allocations, use arena allocators, reduce data copies"
    fi

    exit 1
fi
