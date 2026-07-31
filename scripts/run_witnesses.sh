#!/usr/bin/env bash
set -euo pipefail

output=${1:-target/witnesses.txt}
mkdir -p "$(dirname "$output")"
: > "$output"
export LC_ALL=C
export NO_COLOR=1

run_example() {
  local example=$1
  printf '=== %s ===\n' "$example" >> "$output"
  cargo run --quiet --example "$example" >> "$output"
}

run_feature_example() {
  local example=$1
  local features=$2
  printf '=== %s [%s] ===\n' "$example" "$features" >> "$output"
  cargo run --quiet --example "$example" --features "$features" >> "$output"
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
run_example agent_cli_builder
run_example ontology_to_cli
run_example revops_revenue_dashboard
run_example revops_sales_pipeline
run_example revops_financial_forecast
run_example revops_email_sequences
run_example revops_cs_checkins

run_feature_example repl_witness repl
run_feature_example frontier_discovery_engine_demo discovery-engine
run_feature_example frontier_reflexive_testing_demo reflexive-testing
run_feature_example semantic_coordinator frontier-all

sha256sum "$output"
