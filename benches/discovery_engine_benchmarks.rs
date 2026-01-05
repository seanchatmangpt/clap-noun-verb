//! Benchmark tests for Capability Discovery Engine
//!
//! Validates performance SLOs for:
//! - Search space operations
//! - Fitness scoring
//! - Swarm optimization
//! - Suggestion generation

#![cfg(feature = "agent2028")]

use clap_noun_verb::macros_discovery_engine::*;
use std::sync::Arc;
use std::time::Instant;

/// Benchmark search space exploration coverage
#[test]
fn bench_search_space_coverage() {
    let start = Instant::now();

    let mut space = SearchSpace::new();

    // Register 10 capabilities
    for i in 0..10 {
        space.register(Capability::new(format!("cap{}", i), format!("category{}", i % 3)));
    }

    // Mark 100 combinations as explored
    for _ in 0..100 {
        let combo = vec!["cap0"];
        space.mark_explored(&combo);
    }

    let _coverage = space.coverage();

    let duration = start.elapsed();

    // SLO: Search space operations should complete in < 10ms
    assert!(
        duration.as_millis() < 10,
        "Search space operations took {}ms (SLO: <10ms)",
        duration.as_millis()
    );

    println!("Search space coverage benchmark: {}ms", duration.as_millis());
}

/// Benchmark fitness scoring performance
#[test]
fn bench_fitness_scoring() {
    let start = Instant::now();

    let mut space = SearchSpace::new();

    for i in 0..20 {
        space.register(Capability::new(format!("cap{}", i), format!("cat{}", i % 4)));
    }

    let engine = FitnessScoringEngine::new();

    // Score 1000 combinations
    for i in 0..1000 {
        let combo = vec![format!("cap{}", i % 20).as_str()];
        let _score = engine.score(&combo, &space);
    }

    let duration = start.elapsed();

    // SLO: 1000 scoring operations should complete in < 100ms
    assert!(
        duration.as_millis() < 100,
        "Fitness scoring took {}ms (SLO: <100ms)",
        duration.as_millis()
    );

    println!("Fitness scoring benchmark: {}ms for 1000 operations", duration.as_millis());
}

/// Benchmark swarm optimization iterations
#[test]
fn bench_swarm_optimization() {
    let start = Instant::now();

    let mut space = SearchSpace::new();

    for i in 0..15 {
        space.register(Capability::new(format!("cap{}", i), format!("cat{}", i % 3)));
    }

    let engine = Arc::new(FitnessScoringEngine::new());
    let mut swarm = SwarmOptimizer::new(10, engine);

    swarm.initialize(&space);

    // Run 50 optimization iterations
    for _ in 0..50 {
        let _score = swarm.iterate(&mut space);
    }

    let duration = start.elapsed();

    // SLO: 50 swarm iterations should complete in < 500ms
    assert!(
        duration.as_millis() < 500,
        "Swarm optimization took {}ms (SLO: <500ms)",
        duration.as_millis()
    );

    println!("Swarm optimization benchmark: {}ms for 50 iterations", duration.as_millis());
}

/// Benchmark suggestion generation
#[test]
fn bench_suggestion_generation() {
    let start = Instant::now();

    let mut space = SearchSpace::new();

    for i in 0..10 {
        space.register(Capability::new(format!("cap{}", i), format!("cat{}", i % 2)));
    }

    let engine = Arc::new(FitnessScoringEngine::new());
    let prover = Arc::new(SafetyProver::default());
    let factory = SuggestionFactory::new(engine, prover);

    // Generate 100 suggestions
    for i in 0..100 {
        let combo = vec![format!("cap{}", i % 10)];
        let _suggestion = factory.generate(&combo, &space);
    }

    let duration = start.elapsed();

    // SLO: 100 suggestion generations should complete in < 50ms
    assert!(
        duration.as_millis() < 50,
        "Suggestion generation took {}ms (SLO: <50ms)",
        duration.as_millis()
    );

    println!("Suggestion generation benchmark: {}ms for 100 suggestions", duration.as_millis());
}

/// Benchmark safety validation
#[test]
fn bench_safety_validation() {
    let start = Instant::now();

    let mut prover = SafetyProver::new();

    // Add 5 safety rules
    prover.add_rule(|cap| !cap.id.is_empty());
    prover.add_rule(|cap| !cap.category.is_empty());
    prover.add_rule(|cap| cap.requires.len() < 10);
    prover.add_rule(|cap| cap.conflicts.len() < 5);
    prover.add_rule(|cap| cap.id.len() < 100);

    // Validate 1000 capabilities
    for i in 0..1000 {
        let cap = Capability::new(format!("cap{}", i), format!("cat{}", i % 5))
            .requires("dep1")
            .conflicts_with("conflict1");

        let _is_safe = prover.is_safe(&cap);
    }

    let duration = start.elapsed();

    // SLO: 1000 safety validations should complete in < 20ms
    assert!(
        duration.as_millis() < 20,
        "Safety validation took {}ms (SLO: <20ms)",
        duration.as_millis()
    );

    println!("Safety validation benchmark: {}ms for 1000 validations", duration.as_millis());
}

/// Benchmark cache effectiveness
#[test]
fn bench_cache_effectiveness() {
    let mut space = SearchSpace::new();

    for i in 0..10 {
        space.register(Capability::new(format!("cap{}", i), format!("cat{}", i % 2)));
    }

    let engine = FitnessScoringEngine::new();
    let combo = vec!["cap0", "cap1"];

    // First call (cache miss)
    let start_uncached = Instant::now();
    let _score1 = engine.score(&combo, &space);
    let uncached_duration = start_uncached.elapsed();

    // Second call (cache hit)
    let start_cached = Instant::now();
    let _score2 = engine.score(&combo, &space);
    let cached_duration = start_cached.elapsed();

    // Cached call should be faster (or at least not slower)
    assert!(
        cached_duration <= uncached_duration,
        "Cached call ({:?}) was slower than uncached ({:?})",
        cached_duration,
        uncached_duration
    );

    println!(
        "Cache effectiveness: uncached={}μs, cached={}μs",
        uncached_duration.as_micros(),
        cached_duration.as_micros()
    );
}
