#!/bin/sh
# Trivial real fixture: `greet.sh greet <name>` echoes a greeting to stdout.
if [ "$1" = "greet" ] && [ -n "$2" ]; then
    echo "Hello, $2!"
    exit 0
fi
echo "usage: greet.sh greet <name>" >&2
exit 1
