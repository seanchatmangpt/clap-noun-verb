use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use clap_noun_verb::autonomic::*;
use std::time::Duration;

/// Benchmark CapabilityGraph node and edge operations
fn bench_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_construction");

    for size in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut graph = CapabilityGraph::new();
                let mut nodes = Vec::new();

                // Add nodes
                for i in 0..size {
                    let node = graph.add_node(
                        black_box(CapabilityId::from_path(&format!("cap_{}", i))),
                        black_box(&format!("Node {}", i)),
                        black_box(InputSchema::default()),
                        black_box(OutputSchema::new(TypeSchema::primitive(PrimitiveType::String))),
                        black_box(vec![]),
                    );
                    nodes.push(node);
                }

                // Add edges (chain pattern)
                for i in 0..size - 1 {
                    let _ = graph.add_edge(
                        black_box(nodes[i]),
                        black_box(nodes[i + 1]),
                        black_box(EdgeType::Produces),
                    );
                }

                black_box(graph);
            });
        });
    }

    group.finish();
}

/// Benchmark reachability queries
fn bench_reachability_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_reachability");

    // Create test graphs of different sizes
    for size in [10, 50, 100].iter() {
        let mut graph = CapabilityGraph::new();
        let mut nodes = Vec::new();

        for i in 0..*size {
            let node = graph.add_node(
                CapabilityId::from_path(&format!("cap_{}", i)),
                &format!("Node {}", i),
                InputSchema::default(),
                OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
                vec![],
            );
            nodes.push(node);
        }

        // Create chain
        for i in 0..*size - 1 {
            graph.add_edge(nodes[i], nodes[i + 1], EdgeType::Produces).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::new("reachable", size),
            &(&graph, &nodes),
            |b, (graph, nodes)| {
                b.iter(|| {
                    black_box(graph.is_reachable(
                        black_box(nodes[0]),
                        black_box(nodes[nodes.len() - 1]),
                    ))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("shortest_path", size),
            &(&graph, &nodes),
            |b, (graph, nodes)| {
                b.iter(|| {
                    black_box(graph.shortest_path(
                        black_box(nodes[0]),
                        black_box(nodes[nodes.len() - 1]),
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark graph statistics calculation
fn bench_graph_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_stats");

    for size in [10, 50, 100, 200].iter() {
        let mut graph = CapabilityGraph::new();
        let mut nodes = Vec::new();

        for i in 0..*size {
            let node = graph.add_node(
                CapabilityId::from_path(&format!("cap_{}", i)),
                &format!("Node {}", i),
                InputSchema::default(),
                OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
                vec![],
            );
            nodes.push(node);
        }

        // Create some edges
        for i in 0..*size - 1 {
            graph.add_edge(nodes[i], nodes[i + 1], EdgeType::Produces).unwrap();
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, graph| {
            b.iter(|| {
                black_box(graph.stats());
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = graph_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_graph_construction,
        bench_reachability_queries,
        bench_graph_stats
}

criterion_main!(graph_benches);
