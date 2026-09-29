#!/usr/bin/env bash
# argv is the manifest's full command path plus arguments (e.g. "cli greet
# World"), not just the trailing positional -- take the last token so this
# script doesn't care how many noun/verb segments ggen's ontology puts in
# front of it.
set -euo pipefail
echo "Hello, ${!#}!"
