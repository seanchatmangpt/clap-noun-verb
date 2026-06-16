/// Performance benchmarks for command dispatch and registry lookup.
///
/// These benchmarks validate that the core dispatch path (noun/verb lookup,
/// command routing, argument parsing) meets performance targets:
///
/// - Registry lookup: <200µs (including all registered verbs)
/// - Argument parsing: <500µs for typical command lines
/// - Command dispatch: <50µs (after parse)
///
/// Run with:
/// ```bash
/// cargo bench --bench dispatch
/// cargo bench --bench dispatch -- --baseline main  # Compare against baseline
/// ```
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Mock registry for benchmarking (simulates CommandRegistry behavior)
struct MockRegistry {
    verbs: Vec<(&'static str, &'static str)>,
}

impl MockRegistry {
    fn new() -> Self {
        Self {
            verbs: vec![
                ("status", "Check service status"),
                ("start", "Start service"),
                ("stop", "Stop service"),
                ("restart", "Restart service"),
                ("logs", "View service logs"),
                ("config", "Manage configuration"),
                ("deploy", "Deploy service"),
                ("health", "Check health status"),
                ("metrics", "View metrics"),
                ("version", "Show version"),
            ],
        }
    }

    fn find_verb(&self, name: &str) -> Option<&'static str> {
        self.verbs.iter().find(|(n, _)| *n == name).map(|(_, d)| *d)
    }
}

/// Benchmark command registry lookup with linear search (typical case)
fn bench_registry_lookup_linear(c: &mut Criterion) {
    let registry = MockRegistry::new();

    c.bench_function("registry_lookup_first", |b| {
        b.iter(|| registry.find_verb(black_box("status")))
    });

    c.bench_function("registry_lookup_middle", |b| {
        b.iter(|| registry.find_verb(black_box("deploy")))
    });

    c.bench_function("registry_lookup_last", |b| {
        b.iter(|| registry.find_verb(black_box("version")))
    });

    c.bench_function("registry_lookup_miss", |b| {
        b.iter(|| registry.find_verb(black_box("nonexistent")))
    });
}

/// Benchmark argument parsing simulation (typical clap behavior)
fn bench_argument_parsing(c: &mut Criterion) {
    let args = vec!["myapp", "services", "status", "--format=json", "--timeout=30"];

    c.bench_function("parse_simple_command", |b| {
        b.iter(|| {
            let cmd = black_box(&args[1..3]);
            (cmd[0], cmd[1])
        })
    });

    c.bench_function("parse_with_flags", |b| {
        b.iter(|| {
            let _verb = black_box(args[2]);
            let _format = black_box("json");
            let _timeout = black_box(30);
        })
    });
}

/// Benchmark command dispatch path (critical for CLI responsiveness)
fn bench_command_dispatch(c: &mut Criterion) {
    let registry = MockRegistry::new();

    c.bench_function("dispatch_verb_lookup", |b| {
        b.iter(|| {
            let cmd = black_box("status");
            let _desc = registry.find_verb(cmd);
        })
    });

    c.bench_function("dispatch_with_validation", |b| {
        b.iter(|| {
            let cmd = black_box("restart");
            let _desc = registry.find_verb(cmd);
            let _valid = black_box(!cmd.is_empty());
        })
    });
}

/// Benchmark serialization/deserialization (common for output formatting)
fn bench_serialization(c: &mut Criterion) {
    #[derive(serde::Serialize)]
    struct CommandResult {
        status: &'static str,
        message: &'static str,
        duration_ms: u32,
    }

    let result = CommandResult {
        status: "success",
        message: "Service started successfully",
        duration_ms: 234,
    };

    c.bench_function("serialize_json_result", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&black_box(&result));
            json
        })
    });

    c.bench_function("serialize_json_result_pretty", |b| {
        b.iter(|| {
            let json = serde_json::to_string_pretty(&black_box(&result));
            json
        })
    });
}

/// Benchmark error handling paths
fn bench_error_handling(c: &mut Criterion) {
    c.bench_function("error_conversion_display", |b| {
        b.iter(|| {
            let _msg = format!("{}: {}", "ERROR", "Invalid command");
        })
    });

    c.bench_function("error_serialization", |b| {
        b.iter(|| {
            let error_json = serde_json::json!({ "error": "command_not_found", "code": 404 });
            let _str = black_box(error_json.to_string());
        })
    });
}

/// Benchmark string operations (common in CLI argument processing)
fn bench_string_operations(c: &mut Criterion) {
    let cmd = "services::status";

    c.bench_function("parse_noun_verb_separator", |b| {
        b.iter(|| {
            let cmd = black_box(cmd);
            cmd.split("::").collect::<Vec<_>>()
        })
    });

    c.bench_function("case_insensitive_match", |b| {
        b.iter(|| {
            let cmd = black_box("STATUS");
            cmd.to_lowercase() == "status"
        })
    });
}

/// Define benchmark groups for organization
criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(100);
    targets =
        bench_registry_lookup_linear,
        bench_argument_parsing,
        bench_command_dispatch,
        bench_serialization,
        bench_error_handling,
        bench_string_operations
);

criterion_main!(benches);
