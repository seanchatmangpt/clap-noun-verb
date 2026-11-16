#!/usr/bin/env bash
# The "executable" clap-noun-verb-any wraps for the autofde-lab planner
# fabric. `python -m autofde_lab.fabric` is two tokens (interpreter + module
# flag), and ProcessExecutor spawns exactly one executable with argv appended
# -- this shim absorbs that mismatch so the manifest's argv reconstruction
# (noun/verb + flags only) never has to know about `python -m`, `uv run`, or
# which virtualenv is active. This is the same pattern any language's runtime
# invocation (`node script.js`, `java -jar x.jar`, `go run .`) would use to
# become a single cnv-any executable.
set -euo pipefail
cd "$(dirname "$0")/../../playground/autofde-lab"
# argv is the ggen-emitted command path plus arguments (e.g. "fabric solve
# PDDLDomain ..."). ggen's schema-pack always groups commands under a real
# cnv:Noun (it reserves the literal noun name "root"); the real
# autofde_lab.fabric CLI has no such prefix, so strip it here -- the same
# adapter role this shim already plays for "python -m autofde_lab.fabric".
shift
exec uv run --no-sync python -m autofde_lab.fabric "$@"
