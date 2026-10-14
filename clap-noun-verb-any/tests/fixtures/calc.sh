#!/bin/sh
# Real fixture #3: `calc.sh add <a> <b>` adds two integers.
if [ "$1" != "add" ] || [ -z "$2" ] || [ -z "$3" ]; then
    echo "usage: calc.sh add <a> <b>" >&2
    exit 1
fi
echo $(( $2 + $3 ))
exit 0
