#!/usr/bin/env bash
set -euo pipefail

output=${1:-target/witnesses.txt}
mkdir -p "$(dirname "$output")"
: > "$output"
export LC_ALL=C

run_example() {
  local example=$1
  printf '=== %s ===\n' "$example" >> "$output"
  cargo run --quiet --example "$example" >> "$output"
}

run_example core_api
run_example verb_args
run_example error_handling
run_example proc_macro_verb
run_example output_formats
run_example command_tree
run_example app_context
run_example graph_api
run_example capability_registry
run_example diagnostics
run_example deprecation
run_example format_error_pipeline
run_example shell_completions

printf '=== repl_witness ===\n' >> "$output"
cargo run --quiet --example repl_witness --features repl >> "$output"

sha256sum "$output"
