# Test Unwrap Migration Guide

This guide describes how to migrate test assertions from standard `.unwrap()` and `.expect()` calls to the new lint-compliant, audit-friendly test prelude patterns.

## Motivation

Standard `.unwrap()` and `.expect()` calls in test files:
1. Trigger `clippy::unwrap_used` and `clippy::expect_used` lints.
2. Require verbose `#[allow(clippy::unwrap_used)]` annotations.
3. Obscure which parts of the codebase are using unsafe or unhandled unwraps versus clean test assertions.

## Test Prelude Patterns

Instead of using standard unwraps, import the test prelude:

```rust
use tests::common::test_prelude::*;
```

Then replace:
- `result.unwrap()` with `result.test_unwrap()`
- `result.expect("message")` with `result.test_expect("message")`
- `option.unwrap()` with `option.test_some("message")`

## Automated Migration

An automated migration script is provided at `scripts/migrate_test_unwraps.sh`.

Usage:
```bash
./scripts/migrate_test_unwraps.sh tests/your_test_file.rs
```
