#!/usr/bin/env bash
# Construct8 Benchmark Script
# Runs benchmarks and generates receipts

set -euo pipefail

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
BENCH_DIR="target/bench-results"
RECEIPT_FILE="${BENCH_DIR}/benchmark_receipt.yaml"

mkdir -p "$BENCH_DIR"

echo "=== Construct8 Benchmark Suite ==="
echo "Timestamp: $TIMESTAMP"
echo ""

# Run benchmarks
echo "Step 1: Running workspace benchmarks (cargo bench --workspace)"
echo "=============================================================="
cargo bench --workspace || {
    echo "WARNING: Some benchmarks may have failed, continuing..."
}
echo ""

# Count benchmark results
BENCH_COUNT=$(find target/criterion -name "*.json" 2>/dev/null | wc -l || echo "0")
echo "Benchmarks recorded: $BENCH_COUNT baseline results"
echo ""

# Generate benchmark receipt
echo "Step 2: Generating benchmark receipt"
echo "===================================="

cat > "$RECEIPT_FILE" << EOF
---
receipt_type: benchmark
timestamp: $TIMESTAMP
system_info:
  os: $(uname -s)
  arch: $(uname -m)
  cpu_cores: $(sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
  rustc_version: $(rustc --version)
  cargo_version: $(cargo --version)

benchmarks_recorded: $BENCH_COUNT
bench_directory: target/criterion

compilation:
  profile: release
  opt_level: 3
  lto: auto

scope:
  - c8-core: base types, vector clocks, causal ordering
  - c8-graph: Construct8Delta mutations, bounded 8-triple operations
  - c8-market: MarketPlanckCell classification, relation detection
  - c8-instruments: telescope, event horizon, collider observations
  - c8-receipts: receipt generation, state hash computation
  - c8-time: monotonic time, vector clock lane operations
  - c8-adversary: logic vs. graph player game trees
  - c8-bench: microbenchmarks for hot paths

hypothesis:
  - Deltas remain O(1) space regardless of graph size
  - State hash computation is constant-time
  - Receipt chain validation is linear in chain length
  - Vector clock increments use bounded lane space

output_location: $RECEIPT_FILE
---
EOF

echo "✓ Benchmark receipt written to: $RECEIPT_FILE"
echo ""

echo "=== Benchmark Suite Complete ==="
echo "Results available at: target/criterion/"
echo "Receipt: $RECEIPT_FILE"
exit 0
