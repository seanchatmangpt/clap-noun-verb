#!/usr/bin/env bash
# Copyright (c) 2024 Sean Chatman
# SPDX-License-Identifier: MIT OR Apache-2.0

set -euo pipefail

# Default target name
TARGET="mcp-plus"
DRY_RUN=0
AGENT=""

# Parse flags
while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run) DRY_RUN=1; shift ;;
    --target) TARGET="$2"; shift 2 ;;
    --agent) AGENT="$2"; shift 2 ;;
    *) echo "Usage: $0 [--dry-run] [--target TARGET] [--agent claude|gemini]" >&2; exit 2 ;;
  esac
done

AGENT_FLAG=""
if [ -n "$AGENT" ]; then
    AGENT_FLAG="--agent $AGENT"
fi

# Utility: safe run (exit on error with SR-style code)
run_or_fail() {
  CMD="$1"; DESC="$2"; CODE="$3"
  if ! eval "$CMD"; then
    echo "{\"command\":\"$DESC\",\"status\":\"fail\",\"message\":\"Command failed: $CMD\"}" >&2
    exit "$CODE"
  fi
}

# 1. Ensure Spec Kit CLI is installed
if ! command -v specify >/dev/null; then
  run_or_fail "pipx install git+https://github.com/github/spec-kit.git" \
              "install specify CLI" 6
fi

# 4. Build local CLI tools (mcpp, speckit-ralph)
if [ $DRY_RUN -eq 0 ]; then
  echo "Building mcpp..."
  (cd playground && cargo build --release --bin mcpp) || run_or_fail "true" "build mcpp" 8
  echo "Building speckit-ralph..."
  (cargo build --release -p speckit-ralph) || run_or_fail "true" "build speckit-ralph" 8
fi

# We define the CLI invocation to point to our newly built binaries
MCPP="./playground/target/release/mcpp"
RALPH="./target/release/speckit-ralph"

# 3. Initialize .chatmangpt state (if missing)
STATE_FILE=".chatmangpt/state.yaml"
if [ ! -f "$STATE_FILE" ]; then
  mkdir -p .chatmangpt
  cat <<EOF > "$STATE_FILE"
line_status: running
work_state: none
phase: none
active_delta: ""
completed_gates: []
EOF
  echo ".chatmangpt/state.yaml created."
fi

# 4. Loop
echo "Running mcpp doctor..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP doctor run || true
  DOCTOR_STATUS=$?
else
  echo "(dry-run) would call mcpp doctor run"
  DOCTOR_STATUS=0
fi
if [ $DOCTOR_STATUS -ne 0 ]; then exit 3; fi

echo "Running ralph run..."
if [ $DRY_RUN -eq 0 ]; then
  $RALPH run "Build the MCPP unified loop" || true
  RALPH_STATUS=$?
else
  echo "(dry-run) would call ralph run"
  RALPH_STATUS=0
fi
if [ $RALPH_STATUS -ne 0 ]; then exit 8; fi

echo "Running mcpp telco next..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP telco next --target "$TARGET" $AGENT_FLAG --output json
else
  echo '{"schema":"chatmangpt.sr.result.v1","command":"sr.telco.next","status":"pass","data":{},"errors":[],"warnings":[],"next":{"command":"mcpp verify","reason":"(dry-run)"}}'
fi

echo "Running mcpp verify..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP verify run --target "$TARGET" $AGENT_FLAG --output json
else
  echo '{"schema":"chatmangpt.sr.result.v1","command":"sr.verify","status":"pass","data":{},"errors":[],"warnings":[],"next":{"command":"mcpp receipt emit","reason":"(dry-run)"}}'
fi

echo "Running mcpp receipt emit..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP receipt emit --target "$TARGET" $AGENT_FLAG --output json
fi

echo "Running mcpp receipt sign..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP receipt sign --target "$TARGET" --output json
fi

echo "Running mcpp receipt verify..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP receipt verify --target "$TARGET" $AGENT_FLAG --output json
  sed -i.bak 's/work_state: none/work_state: closed/' "$STATE_FILE" || true
else
  echo '{"schema":"chatmangpt.sr.result.v1","command":"sr.receipt.verify","status":"verified","data":{},"errors":[],"warnings":[]}'
fi

echo "Production loop complete."
exit 0
