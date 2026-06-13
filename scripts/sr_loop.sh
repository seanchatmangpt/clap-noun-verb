#!/bin/bash
# Copyright (c) 2024 Sean Chatman
# SPDX-License-Identifier: MIT OR Apache-2.0

set -e

echo "--- Starting Ralph Loop Closure ---"

# 1. Emit RalphPlan
echo "Emitting RalphPlan..."
cargo run -p speckit-ralph -- main run --goal "Implement Ralph Loop" > /dev/null

if [ ! -f "ralph_plan.json" ]; then
    echo "Error: ralph_plan.json not found."
    exit 1
fi
echo "✓ RalphPlan emitted."

# 2. Doctor Validation
echo "Running Doctor validation..."
cargo run -p playground -- doctor

# 3. Receipt Lifecycle
echo "Creating and signing receipt..."
cargo run -p playground -- receipt create > /dev/null
cargo run -p playground -- receipt sign > /dev/null
cargo run -p playground -- receipt verify
cargo run -p playground -- chain verify

# 4. State Advance
echo "Advancing portfolio state..."
# Update state file placeholder (minimal manual update for demonstration)
sed -i '' 's/phase: none/phase: plan/g' .chatmangpt/state.yaml || true

# Record Receipt
DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo "### Loop Closure Receipt: $DATE" >> PORTFOLIO_RALPH_LOOP_RECEIPT.md
echo "- Status: Verified" >> PORTFOLIO_RALPH_LOOP_RECEIPT.md

echo "--- Ralph Loop Closure Complete ---"
