# CLI Testing Checklist

Quick reference for verifying noun-verb commands work correctly.

## Pre-Development Checklist

- [ ] Define noun/verb structure (e.g., `services status`, `config set`)
- [ ] Document required arguments
- [ ] List optional flags/options
- [ ] Specify error scenarios
- [ ] Plan help text content

## Build & Compilation

- [ ] `cargo make format-check` passes
- [ ] `cargo make clippy` has no issues
- [ ] `cargo make lint` passes
- [ ] `cargo make check` succeeds
- [ ] `cargo make build` succeeds
- [ ] `cargo make build-release` succeeds
- [ ] Binary size ≤10 MB (check with `ls -lh target/release/myapp`)

## Manual Testing

- [ ] Run `./target/debug/myapp --help` - shows root help
- [ ] Run `./target/debug/myapp <noun> --help` - shows noun help
- [ ] Run `./target/debug/myapp <noun> <verb> --help` - shows verb help
- [ ] Execute valid command: `./target/debug/myapp <noun> <verb>`
- [ ] Try invalid noun: `./target/debug/myapp invalid` - shows error
- [ ] Try invalid verb: `./target/debug/myapp <noun> invalid` - shows error
- [ ] Missing required argument fails gracefully
- [ ] Global flags work: `./target/debug/myapp -v <noun> <verb>`

## Unit Test Coverage

### Command Structure
- [ ] Noun is registered in CLI
- [ ] All verbs exist under noun
- [ ] Command names are correct (no typos)
- [ ] Subcommand hierarchy is correct

### Argument Validation
- [ ] Required arguments enforced
- [ ] Optional arguments default correctly
- [ ] Flags (--flag) toggle on/off properly
- [ ] Value arguments accept input
- [ ] Type validation works (numeric, choice, etc.)
- [ ] Multiple values collected correctly

### Help Text
- [ ] Root help displays
- [ ] All nouns appear in root help
- [ ] Noun help displays all verbs
- [ ] Descriptions are present
- [ ] Required vs optional marked correctly

### Error Handling
- [ ] Invalid noun shows error
- [ ] Invalid verb shows error
- [ ] Missing required args shows error
- [ ] Invalid flag value shows error
- [ ] Error messages are helpful
- [ ] Suggestions (typo detection) work

### Exit Codes
- [ ] Success returns 0
- [ ] Argument error returns non-zero
- [ ] Execution error returns non-zero

## Integration Test Coverage

- [ ] Command parses from CLI arguments
- [ ] Multiple commands work in sequence
- [ ] State persists correctly
- [ ] Output is in expected format (JSON/plain)
- [ ] Workflows complete without errors

## Feature-Gated Testing

- [ ] Build with `--features wizard` succeeds
- [ ] Build with `--features federated-network` succeeds
- [ ] Build with `--features full` succeeds
- [ ] Run `cargo make check-all` succeeds
- [ ] Run `cargo make test-all` passes
- [ ] Run `cargo make test-frontier` for frontier features
- [ ] Feature-gated tests have `#[cfg(feature = "...")]`
- [ ] Feature-gated tests pass when feature enabled
- [ ] Feature-gated code absent when feature disabled

## Help Text Verification

### Content
- [ ] Description clearly explains what command does
- [ ] Arguments have clear help text
- [ ] Examples provided if helpful
- [ ] Defaults mentioned for optional args

### Formatting
- [ ] No text truncation in help
- [ ] Proper alignment
- [ ] Consistent terminology
- [ ] Grammar correct

### Completeness
- [ ] All nouns documented
- [ ] All verbs documented
- [ ] All arguments documented
- [ ] All flags documented

## Error Message Verification

### Accuracy
- [ ] Error message matches actual problem
- [ ] Noun/verb names quoted if possible
- [ ] Argument names clear

### Helpfulness
- [ ] Suggests likely fix
- [ ] Provides context
- [ ] No cryptic error codes without explanation

### Examples
```
❌ "Command failed"                    - Too vague
✓ "Command 'servces' not found. Did you mean: services?"  - Helpful with suggestion

❌ "Argument error"                    - Too vague
✓ "Required argument 'key' missing. Usage: config set KEY VALUE"  - Clear with context
```

## Output Format Testing

- [ ] JSON output is valid JSON
- [ ] JSON serializable with `serde_json`
- [ ] Plain text output is readable
- [ ] Consistent formatting
- [ ] No extraneous whitespace
- [ ] Proper escaping for special characters

## Performance Testing

- [ ] Command parses in < 100ms
- [ ] 1000 parses complete in < 1 second
- [ ] Memory usage reasonable (< 10MB per instance)
- [ ] No memory leaks on repeated runs

## Example Application Testing

- [ ] `cargo make build-examples` succeeds
- [ ] Examples run without panicking
- [ ] Example help is accurate
- [ ] Example commands execute correctly
- [ ] Example output is JSON-serializable (if applicable)

## Regression Testing

- [ ] Help text matches previous version
- [ ] Command structure unchanged (unless intentional)
- [ ] Exit codes same for same inputs
- [ ] Performance not degraded
- [ ] No new unwanted warnings/errors

## Documentation

- [ ] README has command examples
- [ ] Examples show common use cases
- [ ] Help text aligns with README
- [ ] Doc comments on public types
- [ ] CLI architecture documented

## Continuous Integration

- [ ] All checks pass: `cargo make ci`
- [ ] Tests deterministic: `cargo make test-lib-deterministic`
- [ ] No flaky tests (run multiple times)
- [ ] Compilation warnings fixed
- [ ] Clippy warnings addressed

## Before Committing

- [ ] All tests pass: `cargo make test`
- [ ] No format issues: `cargo make format` run
- [ ] Linting clean: `cargo make lint`
- [ ] Documentation builds: `cargo make doc`
- [ ] Example builds: `cargo make build-examples`

## Before Merging/Publishing

- [ ] Feature checklist complete
- [ ] Performance within SLOs
  - [ ] Incremental compilation ≤ 2s
  - [ ] Binary size ≤ 10 MB
- [ ] All documentation updated
- [ ] CHANGELOG.md updated (if applicable)
- [ ] Version bump (if releasing)
- [ ] Feature flags updated in Cargo.toml (if needed)

## Testing by Feature Type

### Basic Commands
- [ ] Structure test (noun/verb exist)
- [ ] Help test
- [ ] Error handling test
- [ ] Basic execution test

### Commands with Arguments
- [ ] Required argument enforcement
- [ ] Optional argument defaults
- [ ] Type validation
- [ ] Help includes argument description

### Commands with Flags
- [ ] Flag toggling (on/off)
- [ ] Global flags propagate
- [ ] Short form (-v) and long form (--verbose) work
- [ ] Flag with value (--config /path)

### Wizard/AI-Integrated Commands
- [ ] Feature compilation with `--features wizard`
- [ ] Session creation
- [ ] Prompt handling
- [ ] Timeout handling
- [ ] Error recovery

### Federated/Network Commands
- [ ] Feature compilation with `--features federated-network`
- [ ] Network calls (mocked in tests)
- [ ] Distributed commands
- [ ] Failure scenarios

## Debugging Tips

### Test Fails to Compile
```bash
# Check feature is enabled
cargo build --features wizard

# Check Cargo.toml syntax
cargo check
```

### Test Hangs
```bash
# Run with timeout
timeout 30 cargo test test_name -- --nocapture

# Run single-threaded
cargo test -- --test-threads=1
```

### Help Text Assertions Fail
```rust
// Debug by printing actual help
let mut help_output = Vec::new();
cmd.write_help(&mut help_output).unwrap();
println!("ACTUAL HELP:\n{}", String::from_utf8_lossy(&help_output));
```

### Flaky Tests
```bash
# Run multiple times to detect flakiness
for i in {1..10}; do cargo test --release || break; done
```

### Performance Issues
```bash
# Time a specific test
time cargo test test_name -- --nocapture --test-threads=1

# Check incremental build speed
touch src/lib.rs
time cargo build
```

## Quick Commands Reference

```bash
# Format code and check
cargo make format && cargo make format-check

# Run all checks
cargo make lint

# Test everything
cargo make test-all

# Test frontier features
cargo make test-frontier

# Build and check with all features
cargo make check-all

# Run CI suite
cargo make ci

# Build release binary
cargo make build-release

# Build documentation
cargo make doc

# Build all examples
cargo make build-examples
```

## Exit Codes Convention

- **0** - Success
- **1** - General error
- **2** - Misuse of command (argument error)
- **126** - Command cannot execute
- **127** - Command not found

## Testing Priority Order

1. **Critical** - Command structure, required arguments, error paths
2. **High** - Help text, argument parsing, valid workflows
3. **Medium** - Optional arguments, flags, formatting
4. **Low** - Performance, edge cases, stress tests

## Sign-Off Checklist

Before considering a command "done":

- [ ] Structure correct (noun/verb discoverable)
- [ ] Arguments validated properly
- [ ] Help text complete and accurate
- [ ] Error messages helpful
- [ ] Tests passing (unit + integration)
- [ ] No compiler warnings
- [ ] Clippy clean
- [ ] Manual testing done
- [ ] Performance acceptable
- [ ] Documentation updated
