#!/bin/sh
# Real fixture #6: `repeat.sh bang -c 3` prints "!" repeated `count` times.
if [ "$1" != "bang" ]; then
    echo "usage: repeat.sh bang -c <count>" >&2
    exit 1
fi
shift
count=1
while [ "$1" != "" ]; do
    if [ "$1" = "-c" ]; then
        shift
        count="$1"
    fi
    shift
done
i=0
out=""
while [ "$i" -lt "$count" ]; do
    out="${out}!"
    i=$((i + 1))
done
echo "$out"
exit 0
