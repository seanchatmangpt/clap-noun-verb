//! Integration tests for Capability Discovery Engine
//!
//! Tests follow Chicago TDD principles:
//! - State-based testing (verify outputs and state changes)
//! - Real collaborators (no mocks)
//! - AAA pattern (Arrange-Act-Assert)

#![cfg(feature = "agent2028")]

use clap_noun_verb::macros_discovery_engine::*;
use std::sync::Arc;

#[test]
fn test_search_space_exploration() {
    // Arrange: Create search space with capabilities
    let mut space = SearchSpace::new();

    let cap1 = Capability::new("parallel_execution", "performance")
        .requires("tokio")
        .with_metadata("impact", "high");

    let cap2 = Capability::new("caching", "performance").with_metadata("impact", "medium");

    let cap3 = Capability::new("logging", "observability").conflicts_with("silent_mode");

    space.register(cap1);
    space.register(cap2);
    space.register(cap3);

    // Act: Explore combinations
    let combination = vec!["parallel_execution", "caching"];
    space.mark_explored(&combination);

    // Assert: Verify exploration state
    assert!(space.is_explored(&combination));
    // Note: capabilities field is private, verify through exploration status instead
    assert!(space.coverage() > 0.0);

    // Assert: Verify unexplored neighbors exist
    let neighbors = space.unexplored_neighbors(&combination);
    assert!(!neighbors.is_empty());
}

#[test]
fn test_fitness_scoring_weights() {
    // Arrange: Create search space and scoring engine
    let mut space = SearchSpace::new();
    space.register(Capability::new("feature1", "cat1"));
    space.register(Capability::new("feature2", "cat2"));

    let engine = FitnessScoringEngine::new();

    // Act: Score a combination
    let combination = vec!["feature1"];
    let score = engine.score(&combination, &space);

    // Assert: Verify score components and weights
    assert!(score.utility >= 0.0 && score.utility <= 1.0);
    assert!(score.novelty >= 0.0 && score.novelty <= 1.0);
    assert!(score.safety >= 0.0 && score.safety <= 1.0);

    // Verify weighted total (40% utility + 30% novelty + 30% safety)
    let expected_total = (score.utility * 0.4) + (score.novelty * 0.3) + (score.safety * 0.3);
    assert!((score.total() - expected_total).abs() < 1e-10);
}

#[test]
fn test_safety_prover_validation() {
    // Arrange: Create safety prover with rules
    let mut prover = SafetyProver::new();

    prover.add_rule(|cap| !cap.id.is_empty());
    prover.add_rule(|cap| !cap.category.is_empty());
    prover.add_rule(|cap| cap.requires.len() < 10);

    // Act & Assert: Validate safe capability
    let safe_cap = Capability::new("valid_cap", "category");
    assert!(prover.is_safe(&safe_cap));

    // Act & Assert: Reject unsafe capability
    let unsafe_cap = Capability::new("", "category");
    assert!(!prover.is_safe(&unsafe_cap));
}

#[test]
fn test_capability_type_state_transitions() {
    // Arrange: Create discovered capability and prover
    let cap = Capability::new("test_cap", "testing").requires("dependency");

    let prover = SafetyProver::default();

    // Act: Validate capability (state transition)
    let validated = cap.validate(&prover);

    // Assert: Verify successful validation
    assert!(validated.is_ok());

    let validated_cap = validated.unwrap();
    assert_eq!(validated_cap.validated_id(), "test_cap");
}

#[test]
fn test_suggestion_factory_generation() {
    // Arrange: Create factory with engine and prover
    let mut space = SearchSpace::new();
    space.register(Capability::new("cap1", "cat1"));
    space.register(Capability::new("cap2", "cat2"));

    let engine = Arc::new(FitnessScoringEngine::new());
    let prover = Arc::new(SafetyProver::default());
    let factory = SuggestionFactory::new(engine, prover);

    // Act: Generate suggestion
    let combination = vec!["cap1".to_string(), "cap2".to_string()];
    let suggestion = factory.generate(&combination, &space);

    // Assert: Verify suggestion properties
    assert_eq!(suggestion.capabilities.len(), 2);
    assert!(suggestion.safe);
    assert!(!suggestion.rationale.is_empty());
    assert!(suggestion.score.total() >= 0.0);
}

#[test]
fn test_discoverable_macro() {
    // Arrange & Act: Use discoverable macro
    let cap = discoverable! {
        name: "macro_test",
        category: "testing",
        requires: ["dep1", "dep2"],
        conflicts: ["conflict1"]
    };

    // Assert: Verify capability properties
    assert_eq!(cap.id, "macro_test");
    assert_eq!(cap.category, "testing");
    assert_eq!(cap.requires.len(), 2);
    assert_eq!(cap.conflicts.len(), 1);
    assert!(cap.requires.contains(&"dep1".to_string()));
    assert!(cap.conflicts.contains(&"conflict1".to_string()));
}

#[test]
fn test_fitness_function_macro() {
    // Arrange & Act: Use fitness_function macro
    let utility_score = 0.8;
    let novelty_score = 0.6;
    let safety_score = 1.0;

    let score = fitness_function! {
        capability: "test",
        utility: utility_score,
        novelty: novelty_score,
        safety: safety_score
    };

    // Assert: Verify score values
    assert_eq!(score.utility, 0.8);
    assert_eq!(score.novelty, 0.6);
    assert_eq!(score.safety, 1.0);

    // Assert: Verify weighted total
    let expected = (0.8 * 0.4) + (0.6 * 0.3) + (1.0 * 0.3);
    assert!((score.total() - expected).abs() < 1e-10);
}

#[test]
fn test_conflict_detection_in_scoring() {
    // Arrange: Create capabilities with conflicts
    let mut space = SearchSpace::new();

    let cap1 = Capability::new("feature_a", "cat1").conflicts_with("feature_b");
    let cap2 = Capability::new("feature_b", "cat2").conflicts_with("feature_a");

    space.register(cap1);
    space.register(cap2);

    let engine = FitnessScoringEngine::new();

    // Act: Score conflicting combination
    let conflicting = vec!["feature_a", "feature_b"];
    let score = engine.score(&conflicting, &space);

    // Assert: Verify safety score is 0 due to conflict
    assert_eq!(score.safety, 0.0);

    // Act: Score non-conflicting combination
    let safe = vec!["feature_a"];
    let safe_score = engine.score(&safe, &space);

    // Assert: Verify safety score is 1 without conflicts
    assert_eq!(safe_score.safety, 1.0);
}

#[test]
fn test_search_space_coverage_calculation() {
    // Arrange: Create search space
    let mut space = SearchSpace::new();
    space.register(Capability::new("cap1", "cat1"));
    space.register(Capability::new("cap2", "cat2"));

    let initial_coverage = space.coverage();

    // Act: Mark some combinations as explored
    space.mark_explored(&["cap1"]);
    space.mark_explored(&["cap2"]);
    space.mark_explored(&["cap1", "cap2"]);

    let after_coverage = space.coverage();

    // Assert: Verify coverage increased
    assert!(after_coverage > initial_coverage);
    assert!(after_coverage <= 100.0);

    // Assert: Verify total combinations is correct (2^n for n capabilities)
    assert_eq!(space.total_combinations(), 4);
}

#[test]
fn test_scoring_cache_behavior() {
    // Arrange: Create engine and space
    let mut space = SearchSpace::new();
    space.register(Capability::new("cap1", "cat1"));

    let engine = FitnessScoringEngine::new();

    // Act: Score same combination twice
    let combination = vec!["cap1"];
    let score1 = engine.score(&combination, &space);
    let score2 = engine.score(&combination, &space);

    // Assert: Verify scores are identical (cached)
    assert_eq!(score1.utility, score2.utility);
    assert_eq!(score1.novelty, score2.novelty);
    assert_eq!(score1.safety, score2.safety);

    // Act: Clear cache and score again
    engine.clear_cache();
    let score3 = engine.score(&combination, &space);

    // Assert: Verify score is recalculated (should still be same value)
    assert_eq!(score1.total(), score3.total());
}
