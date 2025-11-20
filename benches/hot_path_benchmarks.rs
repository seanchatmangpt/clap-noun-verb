use clap_noun_verb::autonomic::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Benchmark InvocationQueue throughput
fn bench_queue_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue_throughput");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let queue = InvocationQueue::new(size);
                for i in 0..size {
                    let _ = queue.try_push(black_box(i));
                }
                for _ in 0..size {
                    let _ = queue.try_pop();
                }
            });
        });
    }

    group.finish();
}

/// Benchmark ContextPool handle allocation
fn bench_context_pool_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_pool_allocation");

    for count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let pool = ContextPool::new(count * 2);
            b.iter(|| {
                for _ in 0..count {
                    black_box(pool.alloc_agent_handle());
                    black_box(pool.alloc_tenant_handle());
                }
            });
        });
    }

    group.finish();
}

/// Benchmark HotPathContext creation
fn bench_hot_path_context_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path_context");

    group.bench_function("context_creation", |b| {
        b.iter(|| {
            HotPathContext::new(
                black_box(AgentHandle::new(1)),
                black_box(TenantHandle::new(1)),
                black_box(42),
                black_box(EffectFlags::empty().with(EffectFlags::READ_ONLY)),
            )
        });
    });

    group.bench_function("context_with_correlation", |b| {
        b.iter(|| {
            HotPathContext::new(
                black_box(AgentHandle::new(1)),
                black_box(TenantHandle::new(1)),
                black_box(42),
                black_box(EffectFlags::empty()),
            )
            .with_correlation(black_box("correlation-id-12345"))
        });
    });

    group.finish();
}

/// Benchmark InvocationArena allocation
fn bench_arena_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_allocation");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let arena: InvocationArena = InvocationArena::new(1024 * 1024);
                for i in 0..size {
                    black_box(arena.alloc(i as u64));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark ZeroCopyParser argument extraction
fn bench_zero_copy_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy_parser");

    let inputs = vec![
        ("simple", "arg1 arg2 arg3"),
        ("medium", "user create --name alice --role admin --email alice@example.com"),
        ("complex", "deployment update prod-cluster --replicas 10 --timeout 300s --strategy rolling --health-check /health --port 8080"),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("parse", name), &input, |b, &input| {
            b.iter(|| {
                let parser = ZeroCopyParser::new(black_box(input));
                for i in 0..parser.arg_count() {
                    black_box(parser.arg(i));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("named", name), &input, |b, &input| {
            b.iter(|| {
                let parser = ZeroCopyParser::new(black_box(input));
                black_box(parser.named("name"));
                black_box(parser.named("role"));
                black_box(parser.named("replicas"));
            });
        });
    }

    group.finish();
}

/// Benchmark EffectFlags bitfield operations
fn bench_effect_flags(c: &mut Criterion) {
    let mut group = c.benchmark_group("effect_flags");

    group.bench_function("create_and_check", |b| {
        b.iter(|| {
            let flags = EffectFlags::empty()
                .with(black_box(EffectFlags::READ_ONLY))
                .with(black_box(EffectFlags::NETWORK));

            black_box(flags.is_read_only());
            black_box(flags.has(EffectFlags::NETWORK));
            black_box(flags.has(EffectFlags::PRIVILEGED));
        });
    });

    group.bench_function("merge_operations", |b| {
        let base = EffectFlags::empty().with(EffectFlags::READ_ONLY);
        b.iter(|| {
            let flags =
                base.with(black_box(EffectFlags::NETWORK)).with(black_box(EffectFlags::STORAGE));
            black_box(flags);
        });
    });

    group.finish();
}

/// Benchmark CapabilityId creation and hashing
fn bench_capability_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("capability_id");

    let paths = vec![
        "user.create",
        "deployment.update",
        "database.migration.apply",
        "api.v2.resource.action.subaction",
    ];

    for path in paths {
        group.bench_with_input(BenchmarkId::new("from_path", path), &path, |b, &path| {
            b.iter(|| {
                black_box(CapabilityId::from_path(black_box(path)));
            });
        });

        group.bench_with_input(BenchmarkId::new("from_path_versioned", path), &path, |b, &path| {
            b.iter(|| {
                black_box(CapabilityId::from_path_versioned(black_box(path), black_box("1.0.0")));
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = hot_path_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_queue_throughput,
        bench_context_pool_allocation,
        bench_hot_path_context_creation,
        bench_arena_allocation,
        bench_zero_copy_parser,
        bench_effect_flags,
        bench_capability_id
}

criterion_main!(hot_path_benches);
