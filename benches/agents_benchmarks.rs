//! Performance benchmarks for Semantic Agent Coordinator
//!
//! Validates <100ms operation latency targets

#![cfg(feature = "agent2028")]

use clap_noun_verb::agents::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// =============================================================================
// State Machine Benchmarks
// =============================================================================

fn bench_state_transitions(c: &mut Criterion) {
    c.bench_function("state_machine_full_lifecycle", |b| {
        b.iter(|| {
            let agent = AgentState::<Unregistered>::new(black_box("bench-agent".to_string()))
                .register(vec!["nlp".to_string()])
                .verify(b"proof")
                .trust(0.8)
                .unwrap();

            black_box(agent)
        });
    });
}

fn bench_trust_updates(c: &mut Criterion) {
    let agent = AgentState::<Unregistered>::new("bench-agent".to_string())
        .register(vec!["nlp".to_string()])
        .verify(b"proof")
        .trust(0.7)
        .unwrap();

    c.bench_function("trust_score_update", |b| {
        b.iter(|| {
            let updated = agent.clone().update_trust(black_box(0.05));
            black_box(updated)
        });
    });
}

// =============================================================================
// Semantic Discovery Benchmarks
// =============================================================================

#[cfg(feature = "rdf")]
fn bench_semantic_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_registration");

    for cap_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(cap_count),
            cap_count,
            |b, &cap_count| {
                let capabilities: Vec<Capability> = (0..cap_count)
                    .map(|i| {
                        Capability::new(format!("cap-{}", i), format!("Capability {}", i))
                            .with_tag("tag1")
                            .with_tag("tag2")
                    })
                    .collect();

                b.iter(|| {
                    let mut discovery = SemanticDiscovery::new();
                    discovery.register_agent(black_box("bench-agent"), capabilities.clone());
                    black_box(discovery)
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "rdf")]
fn bench_semantic_query(c: &mut Criterion) {
    let mut discovery = SemanticDiscovery::new();

    // Populate with agents
    for i in 0..100 {
        discovery.register_agent(
            &format!("agent-{}", i),
            vec![Capability::new(format!("cap-{}", i % 10), "Capability")],
        );
    }

    let query = SparqlQueryBuilder::new().select_agents_with_capability("cap-0").build();

    c.bench_function("semantic_query_100_agents", |b| {
        b.iter(|| {
            let results = discovery.query(black_box(&query));
            black_box(results)
        });
    });
}

#[cfg(feature = "rdf")]
fn bench_semantic_matching(c: &mut Criterion) {
    let caps1 = vec![Capability::new("nlp", "NLP").with_tag("text").with_tag("language")];

    let caps2 = vec![Capability::new("vision", "Vision").with_tag("text").with_tag("image")];

    c.bench_function("semantic_match_score", |b| {
        b.iter(|| {
            let score =
                SemanticDiscovery::semantic_match_score(black_box(&caps1), black_box(&caps2));
            black_box(score)
        });
    });
}

// =============================================================================
// Swarm Coordination Benchmarks
// =============================================================================

fn bench_agent_fitness_calculation(c: &mut Criterion) {
    let mut agent = AgentInfo::new("bench-agent", vec!["nlp", "vision"]);
    agent.success_count = 50;
    agent.failure_count = 10;
    agent.trust_score = 0.8;

    let required_caps = vec!["nlp".to_string()];

    c.bench_function("agent_fitness_score", |b| {
        b.iter(|| {
            let fitness = agent.fitness_score(black_box(&required_caps));
            black_box(fitness)
        });
    });
}

fn bench_task_auction(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_auction");

    for agent_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(agent_count),
            agent_count,
            |b, &agent_count| {
                let mut coordinator = SwarmCoordinator::new();

                for i in 0..agent_count {
                    let mut agent = AgentInfo::new(format!("agent-{}", i), vec!["nlp", "vision"]);
                    agent.trust_score = 0.5 + (i as f64 / agent_count as f64) * 0.5;
                    agent.success_count = i as u64 * 10;
                    coordinator.register_agent(agent);
                }

                let auction = TaskAuction::new("bench-task", vec!["nlp"], 1.0);

                b.iter(|| {
                    let winner = coordinator.run_auction(black_box(auction.clone()));
                    black_box(winner)
                });
            },
        );
    }

    group.finish();
}

fn bench_byzantine_detection(c: &mut Criterion) {
    let mut detector = ByzantineDetector::new();

    // Populate voting history
    for _ in 0..100 {
        detector.record_vote("agent-1", true);
        detector.record_vote("agent-2", false);
    }

    c.bench_function("byzantine_detection", |b| {
        b.iter(|| {
            let is_byz = detector.is_byzantine(black_box("agent-2"), black_box(true));
            black_box(is_byz)
        });
    });
}

fn bench_trust_score_calculation(c: &mut Criterion) {
    let calculator = TrustScore::default();

    c.bench_function("bayesian_trust_score", |b| {
        b.iter(|| {
            let trust = calculator.calculate(black_box(80), black_box(20));
            black_box(trust)
        });
    });
}

fn bench_gossip_propagation(c: &mut Criterion) {
    let mut protocol = GossipProtocol::new();

    let message = GossipMessage {
        id: "msg-001".to_string(),
        source: "agent-001".to_string(),
        payload: "benchmark message".to_string(),
        timestamp: std::time::SystemTime::now(),
        ttl: 10,
    };

    c.bench_function("gossip_should_propagate", |b| {
        b.iter(|| {
            let should_prop = protocol.should_propagate(black_box(&message));
            black_box(should_prop)
        });
    });
}

// =============================================================================
// MAPE-K Autonomic Loop Benchmarks
// =============================================================================

#[cfg(feature = "autonomic")]
fn bench_metric_recording(c: &mut Criterion) {
    let mut autonomic = AutonomicLoop::new();

    c.bench_function("metric_recording", |b| {
        b.iter(|| {
            autonomic.record_metric(black_box("response_time_ms"), black_box(120.0));
        });
    });
}

#[cfg(feature = "autonomic")]
fn bench_anomaly_detection(c: &mut Criterion) {
    let mut detector = AnomalyDetector::new();

    // Establish baseline
    for i in 0..100 {
        detector.record_metric(&Metric::new("cpu_usage", 50.0 + (i % 10) as f64));
    }

    let test_metric = Metric::new("cpu_usage", 150.0);

    c.bench_function("anomaly_detection", |b| {
        b.iter(|| {
            let anomaly = detector.detect_anomaly(black_box(&test_metric));
            black_box(anomaly)
        });
    });
}

#[cfg(feature = "autonomic")]
fn bench_adaptive_parameter_adjustment(c: &mut Criterion) {
    let mut param = AdaptiveParameter::new("max_connections", 50.0, 10.0, 100.0);

    c.bench_function("adaptive_parameter_adjust", |b| {
        b.iter(|| {
            param.adjust(black_box(5.0));
        });
    });
}

#[cfg(feature = "autonomic")]
fn bench_full_mapek_cycle(c: &mut Criterion) {
    c.bench_function("full_mapek_cycle", |b| {
        b.iter(|| {
            let mut autonomic = AutonomicLoop::new();

            // Establish baseline
            for i in 0..30 {
                autonomic.record_metric("response_time_ms", 100.0 + (i % 5) as f64);
            }

            // Inject anomaly
            autonomic.record_metric("response_time_ms", 500.0);

            // Run cycle
            let result = autonomic.run_cycle();
            black_box(result)
        });
    });
}

// =============================================================================
// Integration Benchmarks
// =============================================================================

#[cfg(all(feature = "rdf", feature = "autonomic"))]
fn bench_end_to_end_workflow(c: &mut Criterion) {
    c.bench_function("end_to_end_agent_workflow", |b| {
        b.iter(|| {
            // State machine
            let agent = AgentState::<Unregistered>::new(black_box("e2e-agent".to_string()))
                .register(vec!["nlp".to_string()])
                .verify(b"proof")
                .trust(0.9)
                .unwrap();

            // Swarm coordination
            let mut coordinator = SwarmCoordinator::new();
            let mut agent_info = AgentInfo::new("e2e-agent", vec!["nlp"]);
            agent_info.trust_score = agent.get_trust_score();
            coordinator.register_agent(agent_info);

            // Semantic discovery
            let mut discovery = SemanticDiscovery::new();
            discovery.register_agent("e2e-agent", vec![Capability::new("nlp", "NLP")]);

            // Auction
            let auction = TaskAuction::new("e2e-task", vec!["nlp"], 1.0);
            let _winner = coordinator.run_auction(auction);

            black_box(coordinator)
        });
    });
}

// =============================================================================
// Benchmark Groups
// =============================================================================

criterion_group!(state_machine_benches, bench_state_transitions, bench_trust_updates,);

#[cfg(feature = "rdf")]
criterion_group!(
    semantic_benches,
    bench_semantic_registration,
    bench_semantic_query,
    bench_semantic_matching,
);

criterion_group!(
    swarm_benches,
    bench_agent_fitness_calculation,
    bench_task_auction,
    bench_byzantine_detection,
    bench_trust_score_calculation,
    bench_gossip_propagation,
);

#[cfg(feature = "autonomic")]
criterion_group!(
    autonomic_benches,
    bench_metric_recording,
    bench_anomaly_detection,
    bench_adaptive_parameter_adjustment,
    bench_full_mapek_cycle,
);

#[cfg(all(feature = "rdf", feature = "autonomic"))]
criterion_group!(integration_benches, bench_end_to_end_workflow,);

// Main benchmark runner
#[cfg(all(feature = "rdf", feature = "autonomic"))]
criterion_main!(
    state_machine_benches,
    semantic_benches,
    swarm_benches,
    autonomic_benches,
    integration_benches,
);

#[cfg(all(feature = "rdf", not(feature = "autonomic")))]
criterion_main!(state_machine_benches, semantic_benches, swarm_benches,);

#[cfg(all(not(feature = "rdf"), feature = "autonomic"))]
criterion_main!(state_machine_benches, swarm_benches, autonomic_benches,);

#[cfg(all(not(feature = "rdf"), not(feature = "autonomic")))]
criterion_main!(state_machine_benches, swarm_benches,);
