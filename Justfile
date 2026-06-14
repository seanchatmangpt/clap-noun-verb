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
