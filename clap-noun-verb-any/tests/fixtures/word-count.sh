#!/bin/sh
# Real fixture #2: `word-count.sh count <text> [--verbose]` counts words in text.
if [ "$1" != "count" ] || [ -z "$2" ]; then
    echo "usage: word-count.sh count <text> [--verbose]" >&2
    exit 1
fi
text="$2"
verbose=0
shift 2
while [ "$1" != "" ]; do
    if [ "$1" = "--verbose" ]; then verbose=1; fi
    shift
done
count=$(printf '%s' "$text" | wc -w | tr -d ' ')
if [ "$verbose" = "1" ]; then
    echo "word count for '$text': $count"
else
    echo "$count"
fi
exit 0
