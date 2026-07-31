// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Criterion benchmarks for the real registry-build and route path.

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
            verb!("stop", "Stop service", |_args: &VerbArgs| { Ok(()) }),
        ]
    ))
}

fn bench_build_command(c: &mut Criterion) {
    c.bench_function("dispatch/build_command", |bencher| {
        let registry = build_registry();
        bencher.iter(|| black_box(registry.build_command()));
    });
}

fn bench_route(c: &mut Criterion) {
    c.bench_function("dispatch/route", |bencher| {
        let registry = build_registry();
        let matches = registry
            .build_command()
            .try_get_matches_from(["bench-cli", "service", "status"])
            .expect("benchmark command must parse");
        bencher.iter(|| black_box(registry.route(black_box(&matches))));
    });
}

criterion_group!(benches, bench_build_command, bench_route);
criterion_main!(benches);
