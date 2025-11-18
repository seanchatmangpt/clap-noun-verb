//! Example: Compile-Time Telemetry Validation
//!
//! This example demonstrates the compile-time telemetry validation system
//! that prevents "dead telemetry" (RPN 48 failure mode).
//!
//! Run with:
//!   cargo run --example telemetry_validation

use clap_noun_verb::autonomic::telemetry::{telemetry, TraceSpan};
use clap_noun_verb_macros::{declare_span, span};
use std::thread;
use std::time::Duration;

// Example 1: Basic span declaration and usage
// ✓ This compiles because span is both declared AND used
declare_span!(EXAMPLE_BASIC, "example.basic");

fn basic_example() {
    println!("=== Example 1: Basic Span Usage ===");

    let result = span!(EXAMPLE_BASIC, {
        println!("  Inside instrumented block");
        thread::sleep(Duration::from_millis(50));
        42
    });

    println!("  Result: {}", result);
    println!();
}

// Example 2: Nested spans
declare_span!(EXAMPLE_OUTER, "example.outer");
declare_span!(EXAMPLE_INNER, "example.inner");

fn nested_example() {
    println!("=== Example 2: Nested Spans ===");

    span!(EXAMPLE_OUTER, {
        println!("  Outer span started");

        span!(EXAMPLE_INNER, {
            println!("    Inner span started");
            thread::sleep(Duration::from_millis(30));
            println!("    Inner span finished");
        });

        println!("  Outer span finished");
    });

    println!();
}

// Example 3: Error handling
declare_span!(EXAMPLE_ERROR, "example.error");

fn error_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 3: Error Handling ===");

    let result: Result<i32, &str> = span!(EXAMPLE_ERROR, {
        println!("  Processing...");
        thread::sleep(Duration::from_millis(20));

        // Simulate conditional error
        if rand::random::<bool>() {
            Err("Random error occurred")
        } else {
            Ok(100)
        }
    });

    match result {
        Ok(val) => println!("  Success: {}", val),
        Err(e) => println!("  Error: {}", e),
    }

    println!();
    Ok(())
}

// Example 4: Multiple uses of same span
declare_span!(EXAMPLE_LOOP, "example.loop");

fn loop_example() {
    println!("=== Example 4: Loop with Span ===");

    for i in 1..=3 {
        let result = span!(EXAMPLE_LOOP, {
            println!("  Iteration {}", i);
            thread::sleep(Duration::from_millis(10));
            i * 10
        });
        println!("  Result: {}", result);
    }

    println!();
}

// Example 5: Complex data structures
#[derive(Debug)]
struct ProcessResult {
    items_processed: usize,
    duration_ms: u64,
}

declare_span!(EXAMPLE_COMPLEX, "example.complex");

fn complex_example() -> ProcessResult {
    println!("=== Example 5: Complex Data Processing ===");

    span!(EXAMPLE_COMPLEX, {
        let start = std::time::Instant::now();

        // Simulate processing
        let items = vec![1, 2, 3, 4, 5];
        let mut processed = 0;

        for item in items {
            println!("  Processing item: {}", item);
            thread::sleep(Duration::from_millis(5));
            processed += 1;
        }

        let duration = start.elapsed();

        ProcessResult {
            items_processed: processed,
            duration_ms: duration.as_millis() as u64,
        }
    })
}

// Example 6: Pattern matching
declare_span!(EXAMPLE_PATTERN, "example.pattern");

fn pattern_example(value: i32) -> &'static str {
    println!("=== Example 6: Pattern Matching ===");

    let result = span!(EXAMPLE_PATTERN, {
        match value {
            0 => "zero",
            1..=10 => "small",
            11..=100 => "medium",
            _ => "large",
        }
    });

    println!("  Value {} is {}", value, result);
    println!();
    result
}

// Example 7: Manual TraceSpan (lower-level API)
fn manual_span_example() {
    println!("=== Example 7: Manual TraceSpan ===");

    let mut span = TraceSpan::new_root("manual.operation");
    span.set_attribute("operation_type", "manual");
    span.set_attribute("version", "1.0");

    // Do work
    println!("  Working with manual span...");
    thread::sleep(Duration::from_millis(40));

    // Finish span
    let duration = span.finish();
    println!("  Manual span duration: {:?}", duration);
    println!();
}

// Example 8: Telemetry inspection
declare_span!(EXAMPLE_TELEMETRY, "example.telemetry");

fn telemetry_inspection_example() {
    println!("=== Example 8: Telemetry Inspection ===");

    // Set sample rate to capture all events
    telemetry().set_sample_rate(1);

    // Execute instrumented code
    for i in 1..=5 {
        span!(EXAMPLE_TELEMETRY, {
            thread::sleep(Duration::from_millis(10 * i as u64));
        });
    }

    // Inspect collected telemetry
    let snapshot = telemetry().snapshot();

    println!("  Telemetry Snapshot:");
    println!("  - Counters: {}", snapshot.counters.len());
    println!("  - Gauges: {}", snapshot.gauges.len());
    println!("  - Histograms: {}", snapshot.histograms.len());

    for (name, histogram) in &snapshot.histograms {
        if name.contains("example.telemetry") {
            println!("\n  Histogram: {}", name);
            println!("    Count: {}", histogram.count());
            println!("    Sum: {:?}", histogram.sum());
            if let Some(p50) = histogram.percentile(50.0) {
                println!("    P50: {:?}", p50);
            }
            if let Some(p95) = histogram.percentile(95.0) {
                println!("    P95: {:?}", p95);
            }
        }
    }

    println!();
}

// Example 9: Prometheus export
declare_span!(EXAMPLE_PROMETHEUS, "example.prometheus");

fn prometheus_example() {
    println!("=== Example 9: Prometheus Export ===");

    telemetry().set_sample_rate(1);

    // Generate some metrics
    for _ in 0..3 {
        span!(EXAMPLE_PROMETHEUS, {
            thread::sleep(Duration::from_millis(15));
        });
    }

    // Export in Prometheus format
    let prometheus_output = telemetry().export_prometheus();

    println!("  Prometheus Metrics (sample):");
    for line in prometheus_output.lines().take(15) {
        if !line.is_empty() {
            println!("    {}", line);
        }
    }

    println!("  ...");
    println!();
}

// NEGATIVE EXAMPLES (commented out - would fail to compile)
//
// These demonstrate the compile-time validation:

/*
// ❌ Example: Declared but never used
// This would FAIL to compile with:
// "Span 'UNUSED_SPAN' is declared but never used"
declare_span!(UNUSED_SPAN, "example.unused");

fn unused_example() {
    println!("This function never uses UNUSED_SPAN");
}
*/

/*
// ❌ Example: Used but never declared
// This would FAIL to compile with:
// "cannot find value `UNDECLARED_SPAN` in this scope"
fn undeclared_example() {
    span!(UNDECLARED_SPAN, {
        println!("This won't compile");
    });
}
*/

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Compile-Time Telemetry Validation Examples             ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Run all examples
    basic_example();
    nested_example();
    let _ = error_example();
    loop_example();

    let result = complex_example();
    println!("  Complex result: {:?}\n", result);

    pattern_example(42);
    manual_span_example();
    telemetry_inspection_example();
    prometheus_example();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  Key Takeaways                                           ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  ✓ All spans must be DECLARED with declare_span!        ║");
    println!("║  ✓ All spans must be USED with span! macro              ║");
    println!("║  ✓ Unused spans cause COMPILE-TIME ERRORS               ║");
    println!("║  ✓ No runtime overhead for validation                   ║");
    println!("║  ✓ Prevents RPN 48 (dead telemetry) failure mode        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
}
