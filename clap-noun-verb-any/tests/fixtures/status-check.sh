#!/bin/sh
# Real fixture #5: `status-check.sh check --fail` exits non-zero on demand,
# proving a wrapped foreign binary's real failure path (non-zero exit,
# stderr captured) surfaces through Gateway/OCEL unchanged.
if [ "$1" != "check" ]; then
    echo "usage: status-check.sh check [--fail]" >&2
    exit 2
fi
if [ "$2" = "--fail" ]; then
    echo "simulated failure" >&2
    exit 1
fi
echo "ok"
exit 0
