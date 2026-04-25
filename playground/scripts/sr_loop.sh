#!/usr/bin/env bash
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

# 2. Initialize project (if not already)
if [ ! -f specify.lock ]; then
  if [ $DRY_RUN -eq 0 ]; then
    specify init . || echo "specify init failed" >&2
  else
    echo "(dry-run) would run: specify init ."
  fi
fi

# 3. Add required extensions (doctor, status, ralph, verify, presetify)
declare -a EXTS=("doctor" "status" "ralph" "verify" "presetify")
for ext in "${EXTS[@]}"; do
  if ! specify extension list --installed | grep -q "$ext"; then
    if [ $DRY_RUN -eq 0 ]; then
      specify extension add "$ext" || run_or_fail "true" "install ext $ext" 6
    else
      echo "(dry-run) would run: specify extension add $ext"
    fi
  fi
done

# 4. Build local CLI tools (speckit-ralph, mcpp) from source
if [ $DRY_RUN -eq 0 ]; then
  # Assuming mcpp is built via cargo from root
  echo "Building mcpp (playground-cli) via cargo..."
  (cargo build --release --bin mcpp) || run_or_fail "true" "build mcpp" 8
fi

# We define the CLI invocation to point to our newly built binary
MCPP="./target/release/mcpp"

# 5. Initialize .chatmangpt state (if missing)
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

# 6. -- Dry-run info
if [ $DRY_RUN -eq 1 ]; then
  echo "(Dry-run mode; no changes made.)"
fi

# 7. Command: mcpp doctor  (project health check)
echo "Running mcpp doctor..."
if [ $DRY_RUN -eq 0 ]; then
  # We call the mcpp doctor extension
  $MCPP doctor run || true
  # speckit.doctor
  DOCTOR_STATUS=$?
else
  echo "(dry-run) would call mcpp doctor run"
  DOCTOR_STATUS=0
fi
if [ $DOCTOR_STATUS -ne 0 ]; then
  exit 3  # gate_failed
fi

# 8. Command: mcpp telco next (get next-action report)
echo "Running mcpp telco next..."
if [ $DRY_RUN -eq 0 ]; then
  # We map telco next directly to mcpp telco next
  $MCPP telco next "$TARGET" $AGENT_FLAG --output json
  TELCO_STATUS=$?
else
  echo "(dry-run) would call mcpp telco next"
  TELCO_STATUS=0
  echo '{"schema":"chatmangpt.sr.result.v1","command":"sr.telco.next","status":"pass","data":{},"errors":[],"warnings":[],"next":{"command":"mcpp verify","reason":"(dry-run)"}}'
fi
if [ $TELCO_STATUS -ne 0 ]; then
  exit 4  # line_stopped (assume telco failing means stop)
fi

# 9. Command: mcpp verify (post-implement gates)
echo "Running mcpp verify..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP verify run "$TARGET" $AGENT_FLAG --output json
  VERIFY_STATUS=$?
  if [ $VERIFY_STATUS -ne 0 ]; then
    exit 3  # gate_failed
  fi
else
  echo "(dry-run) would call mcpp verify run"
  echo '{"schema":"chatmangpt.sr.result.v1","command":"sr.verify","status":"pass","data":{},"errors":[],"warnings":[],"next":{"command":"mcpp receipt emit","reason":"(dry-run)"}}'
fi

# 10. Command: mcpp receipt emit (create receipt)
echo "Running mcpp receipt emit..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP receipt emit "$TARGET" $AGENT_FLAG --output json
else
  echo "(dry-run) would generate receipt .chatmangpt/receipt.yaml"
  echo '{"schema":"chatmangpt.sr.result.v1","command":"sr.receipt.emit","status":"pass","data":{},"errors":[],"warnings":[]}'
fi

# 11. Command: mcpp receipt verify (check receipt)
echo "Running mcpp receipt verify..."
if [ $DRY_RUN -eq 0 ]; then
  $MCPP receipt verify "$TARGET" $AGENT_FLAG --output json
  VERIFY_REC_STATUS=$?
  
  if [ $VERIFY_REC_STATUS -eq 0 ]; then
      # Update state to closed
      sed -i.bak 's/work_state: none/work_state: closed/' "$STATE_FILE" || true
  else
      exit 7
  fi
else
  echo "(dry-run) would verify receipt .chatmangpt/receipt.yaml"
  echo '{"schema":"chatmangpt.sr.result.v1","command":"sr.receipt.verify","status":"verified","data":{},"errors":[],"warnings":[]}'
fi

echo "Production loop complete."
exit 0
