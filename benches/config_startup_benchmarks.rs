use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use clap_noun_verb::autonomic::CapabilityGraph;
use clap_noun_verb::autonomic::{CapabilityId, InputSchema, OutputSchema, TypeSchema, PrimitiveType, EdgeType};
use clap_noun_verb::plugin::{Plugin, PluginRegistry, PluginCapability};
use clap_noun_verb::middleware::{Middleware, MiddlewarePipeline};
use clap_noun_verb::telemetry::TelemetryCollector;
use std::time::Duration;

/// Benchmark configuration graph construction
fn bench_config_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_graph_construction");

    for size in [10, 25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("graph_nodes", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut graph = CapabilityGraph::new();
                    for i in 0..size {
                        graph.add_node(
                            black_box(CapabilityId::from_path(&format!("config.item.{}", i))),
                            black_box(&format!("Config item {}", i)),
                            black_box(InputSchema::default()),
                            black_box(OutputSchema::new(TypeSchema::primitive(PrimitiveType::String))),
                            black_box(vec![]),
                        );
                    }
                    black_box(graph);
                });
            },
        );
    }

    // Graph with dependencies
    group.bench_function("graph_with_dependencies", |b| {
        b.iter(|| {
            let mut graph = CapabilityGraph::new();
            let mut nodes = Vec::new();

            // Create 20 nodes with dependency chain
            for i in 0..20 {
                let node = graph.add_node(
                    CapabilityId::from_path(&format!("config.dep.{}", i)),
                    &format!("Dependent config {}", i),
                    InputSchema::default(),
                    OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
                    vec![],
                );
                nodes.push(node);
            }

            // Add dependency edges
            for i in 0..19 {
                graph.add_edge(nodes[i], nodes[i + 1], EdgeType::Requires).ok();
            }

            black_box(graph);
        });
    });

    group.finish();
}

/// Benchmark configuration parsing simulation
fn bench_config_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");

    // JSON parsing simulation
    group.bench_function("parse_small_json", |b| {
        let config = r#"{"name": "app", "version": "1.0.0", "enabled": true}"#;
        b.iter(|| {
            black_box(serde_json::from_str::<serde_json::Value>(config).ok());
        });
    });

    group.bench_function("parse_medium_json", |b| {
        let config = r#"{
            "name": "app",
            "version": "1.0.0",
            "plugins": ["plugin1", "plugin2", "plugin3"],
            "middleware": ["auth", "logging", "metrics"],
            "settings": {
                "timeout": 30,
                "retries": 3,
                "buffer_size": 8192
            }
        }"#;
        b.iter(|| {
            black_box(serde_json::from_str::<serde_json::Value>(config).ok());
        });
    });

    // Large config parsing
    group.bench_function("parse_large_json", |b| {
        let mut config_parts = vec![r#"{"entries": ["#.to_string()];
        for i in 0..1000 {
            config_parts.push(format!(r#"{{"id": {}, "name": "entry{}", "value": {}}}"#, i, i, i));
            if i < 999 {
                config_parts.push(",".to_string());
            }
        }
        config_parts.push(r#"]}"#.to_string());
        let config = config_parts.join("");

        b.iter(|| {
            black_box(serde_json::from_str::<serde_json::Value>(&config).ok());
        });
    });

    group.finish();
}

/// Benchmark hot reload performance
fn bench_hot_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_reload");

    group.bench_function("detect_config_change", |b| {
        let old_hash = 12345u64;
        let new_hash = 12346u64;
        b.iter(|| {
            black_box(old_hash != new_hash);
        });
    });

    group.bench_function("rebuild_config_graph", |b| {
        b.iter(|| {
            let mut graph = CapabilityGraph::new();
            for i in 0..20 {
                graph.add_node(
                    CapabilityId::from_path(&format!("config.{}", i)),
                    &format!("Config {}", i),
                    InputSchema::default(),
                    OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
                    vec![],
                );
            }
            black_box(graph);
        });
    });

    group.finish();
}

/// Benchmark startup sequence
fn bench_startup_sequence(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_sequence");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(15));

    struct MinimalPlugin {
        name: String,
    }

    impl Plugin for MinimalPlugin {
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
            Ok(())
        }
    }

    struct MinimalMiddleware {
        name: String,
    }

    impl Middleware for MinimalMiddleware {
        fn name(&self) -> &str {
            &self.name
        }
    }

    // Phase 1: Plugin loading
    group.bench_function("phase1_plugin_loading", |b| {
        b.iter(|| {
            let mut registry = PluginRegistry::new();
            for i in 0..5 {
                registry.register(Box::new(MinimalPlugin {
                    name: format!("plugin-{}", i),
                })).ok();
            }
            black_box(registry);
        });
    });

    // Phase 2: Middleware setup
    group.bench_function("phase2_middleware_setup", |b| {
        b.iter(|| {
            let mut pipeline = MiddlewarePipeline::new();
            for i in 0..5 {
                pipeline = pipeline.add(Box::new(MinimalMiddleware {
                    name: format!("middleware-{}", i),
                }));
            }
            black_box(pipeline);
        });
    });

    // Phase 3: Config graph construction
    group.bench_function("phase3_config_graph", |b| {
        b.iter(|| {
            let mut graph = CapabilityGraph::new();
            for i in 0..10 {
                graph.add_node(
                    CapabilityId::from_path(&format!("cmd.{}", i)),
                    &format!("Command {}", i),
                    InputSchema::default(),
                    OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
                    vec![],
                );
            }
            black_box(graph);
        });
    });

    // Phase 4: Telemetry initialization
    group.bench_function("phase4_telemetry_init", |b| {
        b.iter(|| {
            let telemetry = TelemetryCollector::new();
            black_box(telemetry);
        });
    });

    // Full cold start simulation
    group.bench_function("full_cold_start", |b| {
        b.iter(|| {
            // 1. Plugin loading
            let mut registry = PluginRegistry::new();
            for i in 0..5 {
                registry.register(Box::new(MinimalPlugin {
                    name: format!("plugin-{}", i),
                })).ok();
            }

            // 2. Middleware setup
            let mut pipeline = MiddlewarePipeline::new();
            for i in 0..5 {
                pipeline = pipeline.add(Box::new(MinimalMiddleware {
                    name: format!("middleware-{}", i),
                }));
            }

            // 3. Config graph
            let mut graph = CapabilityGraph::new();
            for i in 0..10 {
                graph.add_node(
                    CapabilityId::from_path(&format!("cmd.{}", i)),
                    &format!("Command {}", i),
                    InputSchema::default(),
                    OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
                    vec![],
                );
            }

            // 4. Telemetry
            let telemetry = TelemetryCollector::new();

            black_box((registry, pipeline, graph, telemetry));
        });
    });

    group.finish();
}

/// Benchmark memory footprint of config structures
fn bench_config_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_memory_footprint");

    group.bench_function("capability_id_creation", |b| {
        b.iter(|| {
            let ids: Vec<_> = (0..100)
                .map(|i| CapabilityId::from_path(&format!("app.module.{}", i)))
                .collect();
            black_box(ids);
        });
    });

    group.bench_function("graph_node_storage", |b| {
        b.iter(|| {
            let mut graph = CapabilityGraph::new();
            let nodes: Vec<_> = (0..100)
                .map(|i| {
                    graph.add_node(
                        CapabilityId::from_path(&format!("node.{}", i)),
                        &format!("Node {}", i),
                        InputSchema::default(),
                        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
                        vec![],
                    )
                })
                .collect();
            black_box((graph, nodes));
        });
    });

    group.finish();
}

/// Benchmark configuration validation
fn bench_config_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_validation");

    group.bench_function("validate_capability_path", |b| {
        b.iter(|| {
            let path = "app.module.submodule.command";
            black_box(!path.is_empty() && path.contains('.'));
        });
    });

    group.bench_function("validate_version_string", |b| {
        let version = "1.2.3";
        b.iter(|| {
            let parts: Vec<&str> = version.split('.').collect();
            black_box(parts.len() == 3);
        });
    });

    group.bench_function("validate_dependency_chain", |b| {
        let deps = vec!["dep1", "dep2", "dep3"];
        b.iter(|| {
            for dep in &deps {
                black_box(!dep.is_empty());
            }
        });
    });

    group.finish();
}

criterion_group! {
    name = config_startup_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_config_graph_construction,
        bench_config_parsing,
        bench_hot_reload,
        bench_startup_sequence,
        bench_config_memory_footprint,
        bench_config_validation
}

criterion_main!(config_startup_benches);
