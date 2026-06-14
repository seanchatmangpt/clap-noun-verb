// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Behavioral integration tests for `clap_noun_verb::async_verb`.
//!
//! Covers the public helpers `run_async` and `create_runtime`. Both live in a
//! core (non-feature-gated) module and tokio is an unconditional dependency,
//! so no `#[cfg]` gating is required.
//!
//! NOTE on error paths: `run_async` and `create_runtime` only map a tokio
//! `Builder::new_current_thread().enable_all().build()` failure to
//! `NounVerbError::execution_error`. That builder does not fail under normal
//! process conditions, and there is no public seam to inject a failure, so the
//! error-mapping branches are not exercisable from outside the crate. We assert
//! the success contract instead (see summary).

use clap_noun_verb::async_verb::{create_runtime, run_async};
use clap_noun_verb::Result;

#[test]
fn test_run_async_with_ok_future_returns_inner_value() {
    // Arrange
    async fn produce() -> Result<u32> {
        Ok(7 + 35)
    }

    // Act
    let value = run_async(produce()).expect("future resolved Ok");

    // Assert
    assert_eq!(value, 42);
}

#[test]
fn test_run_async_propagates_err_from_future() {
    // Arrange
    async fn failing() -> Result<u32> {
        Err(clap_noun_verb::error::NounVerbError::execution_error("boom"))
    }

    // Act
    let result = run_async(failing());

    // Assert: the Err produced inside the future is returned verbatim.
    let err = result.expect_err("future resolved Err");
    assert!(err.to_string().contains("boom"), "error message preserved: {err}");
}

#[test]
fn test_run_async_runs_awaited_work_and_returns_string() {
    // Arrange: a future that performs an await point before resolving.
    async fn inner() -> Result<String> {
        let part = async { "noun" }.await;
        Ok(format!("{part}-verb"))
    }

    // Act
    let value = run_async(inner()).expect("awaited future resolved Ok");

    // Assert
    assert_eq!(value, "noun-verb");
}

#[test]
fn test_create_runtime_produces_runtime_that_executes_futures() {
    // Arrange
    let rt = create_runtime().expect("runtime built");

    // Act: drive a concrete async computation on the returned runtime.
    let sum = rt.block_on(async { 2u64 + 40 });

    // Assert
    assert_eq!(sum, 42);
}

#[test]
fn test_create_runtime_is_reusable_across_multiple_block_on_calls() {
    // Arrange
    let rt = create_runtime().expect("runtime built");

    // Act: reuse the same runtime for several independent async operations.
    let a = rt.block_on(async { 10u32 });
    let b = rt.block_on(async { a * 2 });
    let c = rt.block_on(async { b + 1 });

    // Assert
    assert_eq!((a, b, c), (10, 20, 21));
}
