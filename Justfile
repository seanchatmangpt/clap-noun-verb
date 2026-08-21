test:
    cargo make test

test-full:
    cargo make test-all

polish:
    cargo make clippy && cargo make format

build:
    cargo build --workspace

clean:
    cargo clean

doc:
    cargo doc --workspace --no-deps

bench:
    cargo bench --workspace

ci: polish test-full

# Local-ecosystem dev convenience: runs ggen's real syn-based CHEAT-T01..T04
# scanner (vacuous-assert, tautological-result-check, no-assertion-test,
# mock-import) against this repo's own crates/*/{src,tests}, tests/,
# examples/, src/, tools/, benches/ trees, via a relative sibling path.
# Assumes ggen is checked out as a direct sibling of this repo at
# ../ggen (both clap-noun-verb and ggen are direct children of the same
# parent directory on this machine) -- if ggen isn't present there, this
# fails with cargo's own manifest-not-found error, which is acceptable; no
# fallback/detection logic is implemented here by design.
guard-cheat-scan:
    cargo run --quiet --manifest-path ../ggen/crates/ggen-cheat-scanner/Cargo.toml --bin ggen-cheat-scanner
