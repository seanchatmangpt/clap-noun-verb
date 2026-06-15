// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Benchmarks for the CLI dispatch path (registry build + route).
//!
//! Exercises the same `build_command` / `route` surface that is instrumented
//! with OpenTelemetry spans under the `otel` feature, so the benchmark and the
//! tracing instrumentation cover the same hot path.

use clap_noun_verb::registry::CommandRegistry;
use clap_noun_verb::{noun, verb, VerbArgs};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn build_registry() -> CommandRegistry {
    CommandRegistry::new().name("bench-cli").register_noun(noun!(
        "service",
        "Manage services",
        [
            verb!("status", "Show status", |_args: &VerbArgs| { Ok(()) }),
            verb!("start", "Start service", |_args: &VerbArgs| { Ok(()) }),
        ]
    ))
}

fn bench_build_command(c: &mut Criterion) {
    c.bench_function("dispatch/build_command", |b| {
        let registry = build_registry();
        b.iter(|| {
            let cmd = registry.build_command();
            black_box(cmd);
        });
    });
}

fn bench_route(c: &mut Criterion) {
    c.bench_function("dispatch/route", |b| {
        let registry = build_registry();
        let cmd = registry.build_command();
        let matches = cmd
            .clone()
            .try_get_matches_from(vec!["bench-cli", "service", "status"])
            .expect("matches");
        b.iter(|| {
            let res = registry.route(black_box(&matches));
            black_box(res.is_ok());
        });
    });
}

criterion_group!(benches, bench_build_command, bench_route);
criterion_main!(benches);
