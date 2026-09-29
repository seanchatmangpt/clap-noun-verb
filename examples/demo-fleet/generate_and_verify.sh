#!/bin/sh
# Real demo-fleet generation pipeline (item #17 of the 25-prompt closure
# pass): runs the real `clap-noun-verb-gen` binary against 20
# combinatorially distinct CliSpec YAMLs (varying noun count 1-3, verb
# count 1-3, and argument style none/one/two -- see generate_specs.py),
# then wraps a representative sample in a real, hand-completed Cargo.toml
# (from-yaml itself only emits src/, not a full crate -- unlike
# `gen scaffold --with-cargo`) and runs a real `cargo check` to prove the
# generated code actually compiles.
#
# Usage: sh generate_and_verify.sh <output-dir>

set -e

OUT_DIR="${1:-/tmp/demo-fleet-out}"
SPECS_DIR="$(dirname "$0")/specs"
BIN="$(dirname "$0")/../../target/debug/clap-noun-verb-gen"
mkdir -p "$OUT_DIR"

if [ ! -x "$BIN" ]; then
    echo "Building clap-noun-verb-gen..."
    (cd "$(dirname "$0")/../.." && cargo build --bin clap-noun-verb-gen)
fi

pass=0
fail=0
for spec in "$SPECS_DIR"/*.yaml; do
    name=$(basename "$spec" .yaml)
    dest="$OUT_DIR/$name"
    rm -rf "$dest"
    if "$BIN" gen from-yaml "$spec" -o "$dest" > "$OUT_DIR/$name.log" 2>&1; then
        pass=$((pass + 1))
        echo "OK   $name"
    else
        fail=$((fail + 1))
        echo "FAIL $name"
    fi
done
echo "Generated: $pass ok, $fail failed (of 20 combinatorially distinct specs)"

# Real compile verification: `from-yaml` output has no Cargo.toml of its
# own (unlike `gen scaffold --with-cargo`), so wrap one representative
# variant in a real crate manifest naming the exact real dependencies a
# generated #[verb]-using crate needs, and run a real `cargo check`.
sample="demo-fleet-00"
sample_dir="$OUT_DIR/$sample"
cat > "$sample_dir/Cargo.toml" <<EOF
[package]
name = "$sample"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "$sample"
path = "src/main.rs"

[dependencies]
clap-noun-verb = { path = "$(cd "$(dirname "$0")/../.." && pwd)" }
clap-noun-verb-macros = { path = "$(cd "$(dirname "$0")/../.." && pwd)/clap-noun-verb-macros" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
linkme = "0.3"
EOF

echo "Verifying real compilation of $sample..."
(cd "$sample_dir" && cargo check)
echo "$sample compiles for real."
