# Contributing to clap-noun-verb

Thank you for your interest in contributing! This document provides essential guidelines for contributors.

## Quick Start

1. **Clone and setup**:
   ```bash
   git clone https://github.com/seanchatmangpt/clap-noun-verb.git
   cd clap-noun-verb
   cargo build
   ```

2. **Run tests**:
   ```bash
   cargo make verify  # or cargo test
   ```

## Development Workflow

### Using cargo-make (Recommended)

```bash
cargo make verify         # Format, clippy, test
cargo make test-timeout   # Run tests with timeout
cargo make ci             # Full CI checks
```

### Using cargo directly

```bash
cargo fmt --check         # Check formatting
cargo clippy -- -D warnings
cargo test               # Run tests
```

## Code Standards

- **Formatting**: Use `cargo fmt`
- **Linting**: Use `cargo clippy -- -D warnings`
- **Error Handling**: Never use `unwrap()` or `expect()` in production code
- **Tests**: Write tests for all public functions with descriptive names
- **Documentation**: Document all public APIs with examples

## Architecture Guidelines

- **Traits**: Keep `dyn` compatible (no async trait methods)
- **Macros**: Simple, focused, with clear error messages
- **Error Types**: Use `thiserror` for structured errors

## Pull Request Process

1. Fork and create a feature branch
2. Write tests for new functionality
3. Run `cargo make verify` (format, clippy, tests)
4. Update documentation
5. Submit PR with clear description

## Publishing (Maintainers)

```bash
cargo make publish-all  # Complete publish workflow
```

## Questions?

- Open an issue for discussion
- Check existing issues and PRs
- Review documentation and examples

Thank you for contributing!
