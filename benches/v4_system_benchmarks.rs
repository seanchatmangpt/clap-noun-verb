use clap_noun_verb::middleware::{
    Middleware, MiddlewarePipeline, MiddlewareRequest, MiddlewareResponse,
};
use clap_noun_verb::plugin::{Plugin, PluginCapability, PluginMetadata, PluginRegistry};
use clap_noun_verb::telemetry::{MetricsCollector, TelemetryCollector};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Benchmark plugin registration and discovery
fn bench_plugin_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_loading");

    // Test plugin implementation
    struct TestPlugin {
        name: String,
        version: String,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::Command, PluginCapability::Hook]
        }

        fn load(&mut self) -> clap_noun_verb::Result<()> {
            Ok(())
        }
    }

    // Cold start: plugin registration
    group.bench_function("cold_start_single_plugin", |b| {
        b.iter(|| {
            let mut registry = PluginRegistry::new();
            let plugin = Box::new(TestPlugin {
                name: "test-plugin".to_string(),
                version: "1.0.0".to_string(),
            });
            black_box(registry.register(plugin)).ok();
        });
    });

    // Benchmark plugin discovery with multiple plugins
    for count in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::new("registry_lookup", count), count, |b, &count| {
            let mut registry = PluginRegistry::new();
            for i in 0..count {
                let plugin = Box::new(TestPlugin {
                    name: format!("plugin-{}", i),
                    version: "1.0.0".to_string(),
                });
                registry.register(plugin).ok();
            }

            b.iter(|| {
                black_box(registry.list_all());
            });
        });
    }

    // Plugin metadata overhead
    group.bench_function("metadata_creation", |b| {
        b.iter(|| {
            black_box(
                PluginMetadata::new("test", "1.0.0")
                    .with_author("Benchmark")
                    .with_description("Test plugin for benchmarking")
                    .with_dependency("core-plugin"),
            );
        });
    });

    group.finish();
}

/// Benchmark middleware chain execution
fn bench_middleware_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("middleware_chain");

    // Test middleware implementation
    struct BenchMiddleware {
        name: String,
    }

    impl Middleware for BenchMiddleware {
        fn name(&self) -> &str {
            &self.name
        }

        fn before(&self, _request: &MiddlewareRequest) -> clap_noun_verb::Result<bool> {
            // Simulate lightweight processing
            black_box(std::hint::black_box(42));
            Ok(true)
        }

        fn after(&self, _response: &MiddlewareResponse) -> clap_noun_verb::Result<()> {
            black_box(std::hint::black_box(42));
            Ok(())
        }
    }

    // Benchmark different middleware chain lengths
    for layer_count in [0, 1, 3, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("before_execution", layer_count),
            layer_count,
            |b, &layer_count| {
                let mut pipeline = MiddlewarePipeline::new();
                for i in 0..layer_count {
                    pipeline = pipeline
                        .add(Box::new(BenchMiddleware { name: format!("middleware-{}", i) }));
                }

                let request = MiddlewareRequest::new("test-command")
                    .with_arg("input")
                    .with_arg("file.txt")
                    .with_requester("bench-user");

                b.iter(|| {
                    black_box(pipeline.execute_before(&request)).ok();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("after_execution", layer_count),
            layer_count,
            |b, &layer_count| {
                let mut pipeline = MiddlewarePipeline::new();
                for i in 0..layer_count {
                    pipeline = pipeline
                        .add(Box::new(BenchMiddleware { name: format!("middleware-{}", i) }));
                }

                let response = MiddlewareResponse::success("Command executed")
                    .with_metadata("duration", "150ms")
                    .with_metadata("status", "ok");

                b.iter(|| {
                    black_box(pipeline.execute_after(&response)).ok();
                });
            },
        );
    }

    // Full middleware pipeline (before + after)
    group.bench_function("full_pipeline_5_layers", |b| {
        let mut pipeline = MiddlewarePipeline::new();
        for i in 0..5 {
            pipeline =
                pipeline.add(Box::new(BenchMiddleware { name: format!("middleware-{}", i) }));
        }

        let request = MiddlewareRequest::new("test-command").with_arg("input").with_arg("file.txt");
        let response = MiddlewareResponse::success("OK");

        b.iter(|| {
            black_box(pipeline.execute_before(&request)).ok();
            black_box(pipeline.execute_after(&response)).ok();
        });
    });

    group.finish();
}

/// Benchmark telemetry collection overhead
fn bench_telemetry_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("telemetry_overhead");

    // Metric recording
    group.bench_function("record_command_execution", |b| {
        let telemetry = TelemetryCollector::new();
        b.iter(|| {
            black_box(telemetry.record_command("test-cmd", 150)).ok();
        });
    });

    group.bench_function("record_command_error", |b| {
        let telemetry = TelemetryCollector::new();
        b.iter(|| {
            black_box(telemetry.record_error("test-cmd", "test error")).ok();
        });
    });

    // Span creation
    group.bench_function("span_creation", |b| {
        let telemetry = TelemetryCollector::new();
        b.iter(|| {
            black_box(telemetry.span("test-operation"));
        });
    });

    // Telemetry enabled vs disabled
    group.bench_function("recording_enabled", |b| {
        let mut telemetry = TelemetryCollector::new();
        telemetry.enable();
        b.iter(|| {
            for i in 0..100 {
                black_box(telemetry.record_command(&format!("cmd-{}", i), 100 + i)).ok();
            }
        });
    });

    group.bench_function("recording_disabled", |b| {
        let mut telemetry = TelemetryCollector::new();
        telemetry.disable();
        b.iter(|| {
            for i in 0..100 {
                black_box(telemetry.record_command(&format!("cmd-{}", i), 100 + i)).ok();
            }
        });
    });

    group.finish();
}

/// Benchmark metrics collector performance
fn bench_metrics_collector(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_collector");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("counter_increment", |b| {
        let metrics = MetricsCollector::new();
        b.iter(|| {
            for i in 0..1000 {
                black_box(metrics.record_command_execution(&format!("cmd-{}", i % 10), 100)).ok();
            }
        });
    });

    group.bench_function("command_count_tracking", |b| {
        let metrics = MetricsCollector::new();
        // Pre-populate
        for i in 0..100 {
            metrics.record_command_execution(&format!("cmd-{}", i), 100).ok();
        }

        b.iter(|| {
            black_box(metrics.command_count());
        });
    });

    group.finish();
}

/// Benchmark startup performance simulation
fn bench_startup_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_simulation");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(15));

    struct TestPlugin {
        name: String,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::Command]
        }

        fn load(&mut self) -> clap_noun_verb::Result<()> {
            // Simulate minimal initialization
            std::hint::black_box(42);
            Ok(())
        }
    }

    struct BenchMiddleware {
        name: String,
    }

    impl Middleware for BenchMiddleware {
        fn name(&self) -> &str {
            &self.name
        }
    }

    // Simulate cold start with plugin loading + middleware setup
    group.bench_function("cold_start_full_stack", |b| {
        b.iter(|| {
            // Plugin system initialization
            let mut registry = PluginRegistry::new();
            for i in 0..3 {
                let plugin = Box::new(TestPlugin { name: format!("plugin-{}", i) });
                registry.register(plugin).ok();
            }

            // Middleware pipeline setup
            let mut pipeline = MiddlewarePipeline::new();
            for i in 0..3 {
                pipeline =
                    pipeline.add(Box::new(BenchMiddleware { name: format!("middleware-{}", i) }));
            }

            // Telemetry initialization
            let telemetry = TelemetryCollector::new();

            black_box((registry, pipeline, telemetry));
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Request/Response object creation
    group.bench_function("request_creation", |b| {
        b.iter(|| {
            black_box(
                MiddlewareRequest::new("test-command")
                    .with_arg("input")
                    .with_arg("file.txt")
                    .with_arg("output")
                    .with_arg("result.txt")
                    .with_requester("user-123"),
            );
        });
    });

    group.bench_function("response_creation", |b| {
        b.iter(|| {
            black_box(
                MiddlewareResponse::success("Command completed successfully")
                    .with_metadata("duration", "250ms")
                    .with_metadata("bytes_processed", "1024")
                    .with_metadata("status_code", "200"),
            );
        });
    });

    group.finish();
}

criterion_group! {
    name = v4_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_plugin_loading,
        bench_middleware_chain,
        bench_telemetry_overhead,
        bench_metrics_collector,
        bench_startup_simulation,
        bench_memory_patterns
}

criterion_main!(v4_benches);
