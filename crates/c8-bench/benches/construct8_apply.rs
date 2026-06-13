// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use c8_core::hotpath::{apply_branchless_mask, batch_validate_construct8, Construct8Delta};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark applying 1 triple (3 deltas) through branchless mask.
fn construct8_apply_1_triple(c: &mut Criterion) {
    c.bench_function("construct8_apply_1_triple", |b| {
        b.iter(|| {
            let slots =
                black_box([Some(100u64), Some(200u64), Some(300u64), None, None, None, None, None]);
            let mask = black_box(0b00000111u8); // First 3 slots
            let values: Vec<u64> = apply_branchless_mask(mask, &slots).copied().collect();
            black_box(values)
        })
    });
}

/// Benchmark applying 2 triples (6 deltas) through branchless mask.
fn construct8_apply_2_triples(c: &mut Criterion) {
    c.bench_function("construct8_apply_2_triples", |b| {
        b.iter(|| {
            let slots = black_box([
                Some(100u64),
                Some(200u64),
                Some(300u64),
                Some(400u64),
                Some(500u64),
                Some(600u64),
                None,
                None,
            ]);
            let mask = black_box(0b00111111u8); // First 6 slots
            let values: Vec<u64> = apply_branchless_mask(mask, &slots).copied().collect();
            black_box(values)
        })
    });
}

/// Benchmark applying 4 triples (8 deltas) through branchless mask.
fn construct8_apply_4_triples(c: &mut Criterion) {
    c.bench_function("construct8_apply_4_triples", |b| {
        b.iter(|| {
            let slots = black_box([
                Some(100u64),
                Some(200u64),
                Some(300u64),
                Some(400u64),
                Some(500u64),
                Some(600u64),
                Some(700u64),
                Some(800u64),
            ]);
            let mask = black_box(0xFFu8); // All 8 slots
            let values: Vec<u64> = apply_branchless_mask(mask, &slots).copied().collect();
            black_box(values)
        })
    });
}

/// Benchmark batch validation of 8 deltas (maximum capacity).
fn construct8_apply_8_triples(c: &mut Criterion) {
    c.bench_function("construct8_apply_8_triples_validation", |b| {
        b.iter(|| {
            let deltas = black_box([
                Construct8Delta { slot: 0, value: 100, validated: false },
                Construct8Delta { slot: 1, value: 200, validated: false },
                Construct8Delta { slot: 2, value: 300, validated: false },
                Construct8Delta { slot: 3, value: 400, validated: false },
                Construct8Delta { slot: 4, value: 500, validated: false },
                Construct8Delta { slot: 5, value: 600, validated: false },
                Construct8Delta { slot: 6, value: 700, validated: false },
                Construct8Delta { slot: 7, value: 800, validated: false },
            ]);
            let results = batch_validate_construct8(&deltas);
            black_box(results)
        })
    });
}

criterion_group!(
    benches,
    construct8_apply_1_triple,
    construct8_apply_2_triples,
    construct8_apply_4_triples,
    construct8_apply_8_triples,
);
criterion_main!(benches);
