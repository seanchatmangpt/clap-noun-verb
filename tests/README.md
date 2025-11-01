# Test Suite Organization

## Test Files

### Default Tests (run with `cargo test`)
- `attribute_macro_acceptance.rs` - Acceptance tests for v3 attribute macro API
- `validation_acceptance.rs` - Acceptance tests for automatic validation
- `cli_builder.rs`, `cli_router.rs`, `cli_validator.rs` - CLI component tests
- `logic_handler.rs`, `runtime_executor.rs`, `runtime_interceptor.rs` - Runtime tests
- `unit.rs`, `edge_cases.rs` - Unit and edge case tests

### Ignored Tests (run with `cargo test --ignored`)

#### `integration_examples.rs`
Integration tests that verify all examples compile and can execute commands.

**To run:**
```bash
cargo test --ignored                                    # All ignored tests
cargo test --test integration_examples -- --ignored     # Integration tests only
```

## Test Performance

- **Default test suite**: Completes in <1 second (required)
- **Integration tests**: May take longer (compile + execute examples)

## Adding New Tests

- **Default tests**: Add to existing files or create new test files in `tests/`
- **Integration tests**: Add to `integration_examples.rs` with `#[ignore]`
