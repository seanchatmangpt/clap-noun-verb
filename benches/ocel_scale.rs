// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Criterion benchmarks for the OCEL 2.0 event-log surface at 10k/100k/1M
//! event scale (item #18 of the 25-prompt closure pass): `compute_signals`,
//! `drift_report`, and `to_rdf`, each run over real, synthetically-generated
//! `OcelDocument`s of the target size -- no mocked timing, real computation
//! over real in-memory documents.

use clap_noun_verb::ocel::{
    compute_signals, drift_report, to_rdf, EventAttributeValue, OcelDocument, OcelEvent,
    Relationship,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Build a real `OcelDocument` with `count` events, spread evenly across
/// `command_count` distinct `noun:verb` commands and alternating success.
fn synthetic_document(count: usize, command_count: usize) -> OcelDocument {
    let mut document = OcelDocument::empty();
    document.events = (0..count)
        .map(|i| {
            let command_index = i % command_count;
            OcelEvent {
                id: format!("evt-{i}"),
                event_type: "cli_invocation".to_string(),
                time: "2026-01-01T00:00:00Z".to_string(),
                attributes: vec![
                    EventAttributeValue {
                        name: "noun".to_string(),
                        value: serde_json::json!(format!("noun{command_index}")),
                    },
                    EventAttributeValue {
                        name: "verb".to_string(),
                        value: serde_json::json!(format!("verb{command_index}")),
                    },
                    EventAttributeValue {
                        name: "success".to_string(),
                        value: serde_json::json!(i % 3 != 0),
                    },
                ],
                relationships: vec![Relationship {
                    object_id: format!("command:noun{command_index}:verb{command_index}"),
                    qualifier: "regards".to_string(),
                }],
            }
        })
        .collect();
    document
}

fn admitted_commands(command_count: usize) -> Vec<(String, String)> {
    (0..command_count).map(|i| (format!("noun{i}"), format!("verb{i}"))).collect()
}

const SCALES: &[usize] = &[10_000, 100_000, 1_000_000];

fn bench_compute_signals(c: &mut Criterion) {
    let mut group = c.benchmark_group("ocel_scale/compute_signals");
    group.sample_size(10);
    for &scale in SCALES {
        let document = synthetic_document(scale, 50);
        let admitted = admitted_commands(50);
        let admitted_refs: Vec<(&str, &str)> =
            admitted.iter().map(|(n, v)| (n.as_str(), v.as_str())).collect();
        let now = chrono::Utc::now();
        group.bench_with_input(BenchmarkId::from_parameter(scale), &scale, |bencher, _| {
            bencher.iter(|| {
                black_box(compute_signals(
                    black_box(&admitted_refs),
                    black_box(&document),
                    std::time::Duration::from_secs(30 * 24 * 60 * 60),
                    now,
                    0.5,
                ))
            });
        });
    }
    group.finish();
}

fn bench_drift_report(c: &mut Criterion) {
    let mut group = c.benchmark_group("ocel_scale/drift_report");
    group.sample_size(10);
    for &scale in SCALES {
        let document = synthetic_document(scale, 50);
        let admitted = admitted_commands(50);
        let admitted_refs: Vec<(&str, &str)> =
            admitted.iter().map(|(n, v)| (n.as_str(), v.as_str())).collect();
        group.bench_with_input(BenchmarkId::from_parameter(scale), &scale, |bencher, _| {
            bencher
                .iter(|| black_box(drift_report(black_box(&admitted_refs), black_box(&document))));
        });
    }
    group.finish();
}

fn bench_to_rdf(c: &mut Criterion) {
    let mut group = c.benchmark_group("ocel_scale/to_rdf");
    group.sample_size(10);
    for &scale in SCALES {
        let document = synthetic_document(scale, 50);
        group.bench_with_input(BenchmarkId::from_parameter(scale), &scale, |bencher, _| {
            bencher.iter(|| black_box(to_rdf(black_box(&document))));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_compute_signals, bench_drift_report, bench_to_rdf);
criterion_main!(benches);
