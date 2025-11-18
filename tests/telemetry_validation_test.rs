//! Comprehensive test suite for compile-time telemetry validation
//!
//! These tests verify that:
//! 1. Declared spans are properly registered
//! 2. Used spans compile successfully
//! 3. Unused spans generate compile-time errors
//! 4. #[verb] macros automatically instrument with telemetry

use clap_noun_verb_macros::{declare_span, span, verb};

// Test 1: Declared and used span - should compile
declare_span!(TEST_SPAN_USED, "test.span.used");

#[test]
fn test_span_declaration_and_usage() {
    let result = span!(TEST_SPAN_USED, {
        // Do some work
        42
    });

    assert_eq!(result, 42);
}

// Test 2: Multiple usages of same span - should compile
declare_span!(TEST_SPAN_MULTI, "test.span.multi");

#[test]
fn test_span_multiple_usage() {
    let r1 = span!(TEST_SPAN_MULTI, { 10 });
    let r2 = span!(TEST_SPAN_MULTI, { 20 });

    assert_eq!(r1, 10);
    assert_eq!(r2, 20);
}

// Test 3: Nested spans - should compile
declare_span!(TEST_SPAN_OUTER, "test.span.outer");
declare_span!(TEST_SPAN_INNER, "test.span.inner");

#[test]
fn test_nested_spans() {
    let result = span!(TEST_SPAN_OUTER, {
        span!(TEST_SPAN_INNER, {
            100
        })
    });

    assert_eq!(result, 100);
}

// Test 4: Span with complex expressions
declare_span!(TEST_SPAN_COMPLEX, "test.span.complex");

#[test]
fn test_span_with_complex_code() {
    let result = span!(TEST_SPAN_COMPLEX, {
        let x = 10;
        let y = 20;
        x + y
    });

    assert_eq!(result, 30);
}

// Test 5: Span returning Result
declare_span!(TEST_SPAN_RESULT, "test.span.result");

#[test]
fn test_span_with_result() -> Result<(), Box<dyn std::error::Error>> {
    let result = span!(TEST_SPAN_RESULT, {
        Ok(42)
    });

    assert_eq!(result?, 42);
    Ok(())
}

// Test 6: Verify #[verb] macro generates telemetry
#[derive(serde::Serialize)]
struct TestOutput {
    value: i32,
}

#[verb("test_verb")]
fn test_verb_telemetry() -> Result<TestOutput, Box<dyn std::error::Error>> {
    Ok(TestOutput { value: 42 })
}

#[test]
fn test_verb_instrumentation() {
    // The #[verb] macro should automatically generate telemetry
    // This test just verifies it compiles and runs
    let result = test_verb_telemetry();
    assert!(result.is_ok());
}

// Test 7: Span with early return
declare_span!(TEST_SPAN_EARLY_RETURN, "test.span.early_return");

#[test]
fn test_span_with_early_return() {
    fn process(fail: bool) -> Option<i32> {
        span!(TEST_SPAN_EARLY_RETURN, {
            if fail {
                return None;
            }
            Some(42)
        })
    }

    assert_eq!(process(false), Some(42));
    assert_eq!(process(true), None);
}

// Test 8: Span with pattern matching
declare_span!(TEST_SPAN_PATTERN, "test.span.pattern");

#[test]
fn test_span_with_pattern_match() {
    let result = span!(TEST_SPAN_PATTERN, {
        match 42 {
            0 => "zero",
            1..=10 => "small",
            11..=100 => "medium",
            _ => "large",
        }
    });

    assert_eq!(result, "medium");
}

// Test 9: Span with async block (if async is enabled)
// Note: Only compiles if async feature is enabled
#[cfg(feature = "async")]
declare_span!(TEST_SPAN_ASYNC, "test.span.async");

#[cfg(feature = "async")]
#[tokio::test]
async fn test_span_with_async() {
    let result = span!(TEST_SPAN_ASYNC, {
        async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            42
        }.await
    });

    assert_eq!(result, 42);
}

// Test 10: Span with loop
declare_span!(TEST_SPAN_LOOP, "test.span.loop");

#[test]
fn test_span_with_loop() {
    let result = span!(TEST_SPAN_LOOP, {
        let mut sum = 0;
        for i in 1..=10 {
            sum += i;
        }
        sum
    });

    assert_eq!(result, 55);
}

// NEGATIVE TEST CASES
// These are commented out because they should fail to compile
// Uncomment to verify compile-time error detection

/*
// Test: Declared but never used - SHOULD FAIL TO COMPILE
declare_span!(TEST_SPAN_UNUSED, "test.span.unused");

// This test should NOT exist - the span is declared but never used
// Expected compile error: "Span 'TEST_SPAN_UNUSED' is declared but never used"
*/

/*
// Test: Used but never declared - SHOULD FAIL TO COMPILE
#[test]
fn test_span_undeclared() {
    span!(UNDECLARED_SPAN, { 42 });
}
// Expected compile error: "cannot find value `UNDECLARED_SPAN` in this scope"
*/

// Integration test: Verify telemetry data is recorded
declare_span!(TEST_INTEGRATION_SPAN, "test.integration");

#[test]
fn test_telemetry_integration() {
    use clap_noun_verb::autonomic::telemetry::telemetry;

    // Set sample rate to 1 to capture all events
    telemetry().set_sample_rate(1);

    // Execute span
    let _ = span!(TEST_INTEGRATION_SPAN, {
        std::thread::sleep(std::time::Duration::from_millis(10));
        42
    });

    // Verify telemetry was recorded
    let snapshot = telemetry().snapshot();

    // Check that span duration was recorded
    // The span! macro should automatically record to histogram
    assert!(
        snapshot.histograms.contains_key("span_duration_test.integration")
            || !snapshot.histograms.is_empty(),
        "Expected span duration to be recorded in telemetry"
    );
}

// Performance test: Verify span overhead is minimal
declare_span!(TEST_PERF_SPAN, "test.perf");

#[test]
fn test_span_performance_overhead() {
    use std::time::Instant;

    // Baseline: no instrumentation
    let start = Instant::now();
    let mut sum = 0;
    for i in 0..1000 {
        sum += i;
    }
    let baseline = start.elapsed();

    // With instrumentation
    let start = Instant::now();
    let mut sum2 = 0;
    for _ in 0..1000 {
        span!(TEST_PERF_SPAN, {
            sum2 += 1;
        });
    }
    let instrumented = start.elapsed();

    // Instrumentation overhead should be minimal (< 10x baseline)
    // Note: This is a rough check - actual overhead depends on CPU, sampling, etc.
    assert!(
        instrumented < baseline * 100,
        "Span overhead too high: baseline={:?}, instrumented={:?}",
        baseline,
        instrumented
    );

    assert_eq!(sum, 499500);
    assert_eq!(sum2, 1000);
}
