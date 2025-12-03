# Project Setup

## Quick Start

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/clap-noun-verb.git
cd clap-noun-verb

# Build
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic -- services status
```

## Development

```bash
# Format code
cargo make format

# Run linter
cargo make clippy

# Verify code quality
cargo make verify

# Run all CI checks
cargo make ci
```

## Project Structure

```
clap-noun-verb/
├── src/           # Source code
├── examples/      # Example programs
├── tests/         # Test suite
└── docs/          # Documentation
```

## Repository

- **GitHub**: https://github.com/seanchatmangpt/clap-noun-verb
- **crates.io**: https://crates.io/crates/clap-noun-verb
- **docs.rs**: https://docs.rs/clap-noun-verb
