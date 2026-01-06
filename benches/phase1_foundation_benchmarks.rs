//! Phase 1: Foundation Benchmarks
//!
//! Performance validation for basic framework operations:
//! - Compilation time (measured externally)
//! - Binary size growth (measured externally)
//! - Feature-gating overhead (zero-cost validation)
//! - Macro expansion performance
//! - Type-state transitions

use clap_noun_verb::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// =============================================================================
// Macro Expansion Benchmarks
// =============================================================================

fn bench_macro_expansion(c: &mut Criterion) {
    c.bench_function("verb_macro_basic", |b| {
        b.iter(|| {
            // Simulated macro expansion cost via registry lookup
            let registry = black_box(CommandRegistry::new());
            black_box(registry)
        });
    });
}

fn bench_auto_discovery(c: &mut Criterion) {
    c.bench_function("command_auto_discovery", |b| {
        b.iter(|| {
            // Linkme distributed slice iteration
            let count = black_box(crate::command_registry().len());
            black_box(count)
        });
    });
}

// =============================================================================
// Type-State Pattern Benchmarks
// =============================================================================

#[cfg(feature = "kernel")]
fn bench_type_state_transitions(c: &mut Criterion) {
    use clap_noun_verb::kernel::*;

    c.bench_function("receipt_state_transitions", |b| {
        b.iter(|| {
            let receipt = Receipt::new(black_box("cmd"));
            let with_input = receipt.with_input(b"test");
            let with_output = with_input.with_output(b"result");
            let signed = with_output.sign(b"signature");
            black_box(signed)
        });
    });
}

// =============================================================================
// Feature Flag Overhead Benchmarks
// =============================================================================

fn bench_default_build(c: &mut Criterion) {
    c.bench_function("default_feature_overhead", |b| {
        b.iter(|| {
            // Minimal operations with default features only
            let result: Result<String, anyhow::Error> = Ok(black_box("test".to_string()));
            black_box(result)
        });
    });
}

#[cfg(feature = "async")]
fn bench_async_overhead(c: &mut Criterion) {
    use tokio::runtime::Runtime;

    c.bench_function("async_feature_overhead", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let result = async { black_box("test".to_string()) }.await;
                black_box(result)
            })
        });
    });
}

#[cfg(feature = "crypto")]
fn bench_crypto_overhead(c: &mut Criterion) {
    use sha2::{Digest, Sha256};

    c.bench_function("crypto_feature_overhead", |b| {
        b.iter(|| {
            let mut hasher = Sha256::new();
            hasher.update(black_box(b"benchmark data"));
            let result = hasher.finalize();
            black_box(result)
        });
    });
}

// =============================================================================
// JSON Serialization Benchmarks
// =============================================================================

fn bench_json_output(c: &mut Criterion) {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestOutput {
        status: String,
        count: u64,
        data: Vec<String>,
    }

    let output = TestOutput {
        status: "success".to_string(),
        count: 100,
        data: vec!["item1".to_string(), "item2".to_string(), "item3".to_string()],
    };

    c.bench_function("json_serialization_small", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&output)).unwrap();
            black_box(json)
        });
    });
}

fn bench_json_output_large(c: &mut Criterion) {
    use serde::Serialize;

    #[derive(Serialize)]
    struct LargeOutput {
        items: Vec<Item>,
    }

    #[derive(Serialize)]
    struct Item {
        id: u64,
        name: String,
        value: f64,
    }

    let output = LargeOutput {
        items: (0..1000)
            .map(|i| Item { id: i, name: format!("item_{}", i), value: i as f64 * 1.5 })
            .collect(),
    };

    c.bench_function("json_serialization_large", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&output)).unwrap();
            black_box(json)
        });
    });
}

// =============================================================================
// Error Handling Benchmarks
// =============================================================================

fn bench_result_error_handling(c: &mut Criterion) {
    fn operation_that_succeeds() -> Result<u64, anyhow::Error> {
        Ok(42)
    }

    fn operation_that_fails() -> Result<u64, anyhow::Error> {
        Err(anyhow::anyhow!("simulated error"))
    }

    c.bench_function("result_success_path", |b| {
        b.iter(|| {
            let result = operation_that_succeeds();
            black_box(result)
        });
    });

    c.bench_function("result_error_path", |b| {
        b.iter(|| {
            let result = operation_that_fails();
            black_box(result)
        });
    });
}

// =============================================================================
// Benchmark Groups
// =============================================================================

criterion_group!(macro_benches, bench_macro_expansion, bench_auto_discovery,);

#[cfg(feature = "kernel")]
criterion_group!(type_state_benches, bench_type_state_transitions,);

criterion_group!(
    feature_overhead_benches,
    bench_default_build,
    #[cfg(feature = "async")]
    bench_async_overhead,
    #[cfg(feature = "crypto")]
    bench_crypto_overhead,
);

criterion_group!(json_benches, bench_json_output, bench_json_output_large,);

criterion_group!(error_benches, bench_result_error_handling,);

// Main benchmark runner
#[cfg(feature = "kernel")]
criterion_main!(
    macro_benches,
    type_state_benches,
    feature_overhead_benches,
    json_benches,
    error_benches,
);

#[cfg(not(feature = "kernel"))]
criterion_main!(macro_benches, feature_overhead_benches, json_benches, error_benches,);
