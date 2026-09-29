#!/bin/sh
# Real fixture #4: `list-fruits.sh list --item a --item b` joins repeated --item values.
if [ "$1" != "list" ]; then
    echo "usage: list-fruits.sh list --item <name> [--item <name> ...]" >&2
    exit 1
fi
shift
out=""
while [ "$1" != "" ]; do
    if [ "$1" = "--item" ]; then
        shift
        if [ -z "$out" ]; then out="$1"; else out="$out,$1"; fi
    fi
    shift
done
echo "$out"
exit 0
