// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "common/deterministic.rs"]
mod deterministic;

use deterministic::{DeterministicRuntime, TestContext};
use std::time::Duration;

#[test]
fn test_kernel_deterministic_runtime_spawn() {
    let mut runtime = DeterministicRuntime::new();
    let mut task = runtime.spawn(Box::pin(async { 42 }));

    // Poll the future and get the inner value
    assert_eq!(task.poll(), std::task::Poll::Ready(42));
}

#[test]
fn test_kernel_deterministic_runtime_advance() {
    let mut runtime = DeterministicRuntime::new();
    runtime.advance(Duration::from_secs(5));

    // Verify advance doesn't panic and works with zero duration
    runtime.advance(Duration::from_secs(0));
}

#[test]
fn test_kernel_test_context_isolation() {
    let ctx1 = TestContext::new();
    let ctx2 = TestContext::new();

    // Verify paths are isolated and exist
    let path1 = ctx1.temp_dir();
    let path2 = ctx2.temp_dir();

    assert!(path1.exists());
    assert!(path2.exists());
    assert_ne!(path1, path2);
}
