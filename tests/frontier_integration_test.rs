//! Integration tests for Frontier Phase 3 features
//!
//! Tests follow Chicago TDD principles:
//! - State-based testing (verify outputs and state changes)
//! - Real collaborators (no mocks)
//! - AAA pattern (Arrange-Act-Assert)
//!
//! Phase 3 features tested:
//! - Feature 4: Capability Discovery Engine
//! - Feature 7: Learning Trajectories
//! - Feature 8: Reflexive Testing

#![cfg(any(
    feature = "discovery-engine",
    feature = "learning-trajectories",
    feature = "reflexive-testing"
))]

#[cfg(feature = "discovery-engine")]
mod discovery_engine_tests {
    use clap_noun_verb::frontier::discovery_engine::*;

    #[test]
    fn test_capability_optimizer_trait_abstraction() {
        // Arrange: Create search space
        let mut space = CapabilitySpace::new();
        space.add_capability("async", "runtime");
        space.add_capability("caching", "performance");
        space.add_capability("logging", "observability");

        // Act: Use trait abstraction with PSO optimizer
        let optimizer = PsoOptimizer::new();
        let results = optimizer.search(&space);

        // Assert: Verify results are optimal
        assert!(!results.is_empty(), "Should find at least one optimal combination");

        for result in &results {
            assert!(result.score.total() >= 0.5, "All results should have score >= 0.5");
            assert!(space.is_valid(&result.capabilities), "All results should be valid");
        }
    }

    #[test]
    fn test_multiple_optimization_algorithms() {
        // Arrange: Create search space
        let mut space = CapabilitySpace::new();
        space.add_capability("feature1", "cat1");
        space.add_capability("feature2", "cat2");
        space.add_capability("feature3", "cat3");

        // Act: Test multiple optimizers
        let pso = PsoOptimizer::new();
        let ga = GeneticOptimizer::new();
        let de = DifferentialEvolution::new();

        let pso_results = pso.search(&space);
        let ga_results = ga.search(&space);
        let de_results = de.search(&space);

        // Assert: All optimizers should find results
        assert!(!pso_results.is_empty(), "PSO should find results");
        assert!(!ga_results.is_empty(), "GA should find results");
        assert!(!de_results.is_empty(), "DE should find results");
    }

    #[test]
    fn test_fitness_score_weighted_calculation() {
        // Arrange: Create fitness scores
        let score1 = FitnessScore::new(0.8, 0.6, 1.0);
        let score2 = FitnessScore::new(1.0, 0.0, 0.5);

        // Act: Calculate totals
        let total1 = score1.total();
        let total2 = score2.total();

        // Assert: Verify weighted calculation (40% utility + 30% novelty + 30% safety)
        let expected1 = (0.8 * 0.4) + (0.6 * 0.3) + (1.0 * 0.3);
        let expected2 = (1.0 * 0.4) + (0.0 * 0.3) + (0.5 * 0.3);

        assert!((total1 - expected1).abs() < 1e-10, "Score 1 calculation should be correct");
        assert!((total2 - expected2).abs() < 1e-10, "Score 2 calculation should be correct");
    }

    #[test]
    fn test_capability_space_conflict_resolution() {
        // Arrange: Create space with conflicts
        let mut space = CapabilitySpace::new();
        space.add_capability("sync", "runtime");
        space.add_capability("async", "runtime");
        space.add_conflict("sync", "async");

        // Act: Test optimizer avoids conflicts
        let optimizer = PsoOptimizer::new();
        let results = optimizer.search(&space);

        // Assert: No results should contain conflicting capabilities
        for result in &results {
            let has_sync = result.capabilities.contains(&"sync".to_string());
            let has_async = result.capabilities.contains(&"async".to_string());
            assert!(
                !(has_sync && has_async),
                "Results should not contain conflicting capabilities"
            );
        }
    }

    #[test]
    fn test_capability_space_requirement_enforcement() {
        // Arrange: Create space with requirements
        let mut space = CapabilitySpace::new();
        space.add_capability("advanced", "feature");
        space.add_capability("basic", "foundation");
        space.add_requirement("advanced", "basic");

        // Act: Test optimizer respects requirements
        let optimizer = PsoOptimizer::new();
        let results = optimizer.search(&space);

        // Assert: All combinations with "advanced" should include "basic"
        for result in &results {
            if result.capabilities.contains(&"advanced".to_string()) {
                assert!(
                    result.capabilities.contains(&"basic".to_string()),
                    "Advanced capability should require basic capability"
                );
            }
        }
    }
}

#[cfg(feature = "learning-trajectories")]
mod learning_trajectories_tests {
    use clap_noun_verb::frontier::learning_trajectories::*;

    #[test]
    fn test_learning_trajectory_ml_path_recommendation() {
        // Arrange: Create ML engine with skills
        let mut ml = LearningTrajectoryML::new();

        let beginner = CompetencyLevel::new("beginner", 0.2);
        let intermediate = CompetencyLevel::new("intermediate", 0.5);
        let advanced = CompetencyLevel::new("advanced", 0.8);

        ml.add_skill(beginner.clone());
        ml.add_skill(intermediate.clone());
        ml.add_skill(advanced.clone());

        ml.add_prerequisite("intermediate", "beginner", 1.0).ok();
        ml.add_prerequisite("advanced", "intermediate", 2.0).ok();

        // Act: Recommend path from beginner to advanced
        let path = ml.recommend_path(&beginner, &advanced);

        // Assert: Path should be valid and include all steps
        assert!(path.is_ok(), "Should find a valid path");

        let path = path.ok().unwrap_or_else(|| unreachable!());
        assert!(path.step_count() >= 2, "Path should have multiple steps");
        assert!(path.confidence > 0.0, "Path should have positive confidence");
    }

    #[test]
    fn test_ml_model_type_selection() {
        // Arrange: Create engines with different models
        let lr = LearningTrajectoryML::with_model(MLModel::LinearRegression);
        let rf = LearningTrajectoryML::with_model(MLModel::RandomForest);
        let svm = LearningTrajectoryML::with_model(MLModel::SVM);

        // Act: Verify model types
        assert_eq!(lr.model_type(), MLModel::LinearRegression);
        assert_eq!(rf.model_type(), MLModel::RandomForest);
        assert_eq!(svm.model_type(), MLModel::SVM);

        // Assert: Different models should give different confidence scores
        let step1 = CompetencyLevel::new("step1", 0.5);
        let step2 = CompetencyLevel::new("step2", 0.7);
        let path = TrajectoryPath::new(vec![step1, step2]);

        let lr_perf = lr.predict_performance(&path);
        let rf_perf = rf.predict_performance(&path);
        let svm_perf = svm.predict_performance(&path);

        assert_ne!(lr_perf, rf_perf, "Different models should predict differently");
        assert_ne!(rf_perf, svm_perf, "Different models should predict differently");
    }

    #[test]
    fn test_byzantine_fault_tolerance() {
        // Arrange: Create detector
        let detector = ByzantineDetector::new();

        // Act: Test outlier detection with Byzantine fault
        let clean_values = vec![0.8, 0.9, 0.85, 0.88, 0.82];
        let byzantine_values = vec![0.8, 0.9, 0.85, 100.0, 0.88];

        let clean_outliers = detector.detect_outliers(&clean_values);
        let byzantine_outliers = detector.detect_outliers(&byzantine_values);

        // Assert: Byzantine fault should be detected
        assert!(clean_outliers.is_empty(), "Clean data should have no outliers");
        assert!(!byzantine_outliers.is_empty(), "Byzantine fault should be detected");
    }

    #[test]
    fn test_consensus_filtering() {
        // Arrange: Create ML engine
        let ml = LearningTrajectoryML::new();

        // Act: Filter consensus with outlier
        let values = vec![0.8, 0.9, 0.85, 10.0, 0.88];
        let filtered = ml.filter_consensus(&values);

        // Assert: Outlier should be filtered out
        assert!(filtered.len() < values.len(), "Outlier should be filtered");
        assert!(!filtered.contains(&10.0), "Extreme value should be removed");
    }

    #[test]
    fn test_petgraph_dag_shortest_path() {
        // Arrange: Create complex skill graph
        let mut ml = LearningTrajectoryML::new();

        let skill_a = CompetencyLevel::new("a", 0.2);
        let skill_b = CompetencyLevel::new("b", 0.4);
        let skill_c = CompetencyLevel::new("c", 0.6);
        let skill_d = CompetencyLevel::new("d", 0.8);

        ml.add_skill(skill_a.clone());
        ml.add_skill(skill_b.clone());
        ml.add_skill(skill_c.clone());
        ml.add_skill(skill_d.clone());

        ml.add_prerequisite("b", "a", 1.0).ok();
        ml.add_prerequisite("c", "a", 2.0).ok();
        ml.add_prerequisite("d", "b", 1.5).ok();
        ml.add_prerequisite("d", "c", 1.0).ok();

        // Act: Find path from A to D (should choose shorter path)
        let path = ml.recommend_path(&skill_a, &skill_d);

        // Assert: Path should exist and be efficient
        assert!(path.is_ok(), "Should find path in DAG");

        let path = path.ok().unwrap_or_else(|| unreachable!());
        assert!(!path.is_empty(), "Path should have steps");
    }
}

#[cfg(feature = "reflexive-testing")]
mod reflexive_testing_tests {
    use clap_noun_verb::frontier::reflexive_testing::*;

    #[test]
    fn test_property_generator_from_rdf() {
        // Arrange: Generate from RDF ontology
        let generator = PropertyGenerator::from_rdf("http://example.com/ontology");

        // Act: Get generated strategies
        let strategies = generator.strategies();

        // Assert: Should auto-generate property tests from RDF
        assert!(!strategies.is_empty(), "Should generate strategies from RDF");
        assert!(generator.strategy_count() >= 3, "Should generate multiple strategies");
    }

    #[test]
    fn test_coverage_tracker_threshold_enforcement() {
        // Arrange: Create tracker with 80% threshold
        let mut tracker = CoverageTracker::new();

        // Act: Add module coverage
        tracker.add_module_coverage("discovery_engine", CoverageStats::new(85, 100));
        tracker.add_module_coverage("learning_trajectories", CoverageStats::new(90, 100));
        tracker.add_module_coverage("reflexive_testing", CoverageStats::new(82, 100));

        // Assert: All modules should meet 80% threshold
        assert!(tracker.all_modules_meet_threshold(), "All modules should meet 80% threshold");

        let overall = tracker.overall_coverage();
        assert!(overall.meets_phase3_target(), "Overall coverage should meet Phase 3 target (80%)");
    }

    #[test]
    fn test_coverage_stats_phase3_target() {
        // Arrange: Create coverage stats
        let good = CoverageStats::new(85, 100);
        let bad = CoverageStats::new(75, 100);

        // Act & Assert: Test Phase 3 target (80%)
        assert!(good.meets_phase3_target(), "85% should meet Phase 3 target");
        assert!(!bad.meets_phase3_target(), "75% should not meet Phase 3 target");
    }

    #[test]
    fn test_reflexive_tester_integration() {
        // Arrange: Create reflexive tester from RDF
        let tester = ReflexiveTester::from_rdf("http://example.com/ontology");

        // Act: Generate test suite
        let test_suite = tester.generate_test_suite();

        // Assert: Should generate proptest code
        assert!(!test_suite.is_empty(), "Should generate test suite");

        for test in &test_suite {
            assert!(test.contains("proptest!"), "Generated tests should use proptest");
            assert!(test.contains("#[test]"), "Generated tests should have test attribute");
        }
    }

    #[test]
    fn test_coverage_report_generation() {
        // Arrange: Create tracker with multiple modules
        let mut tracker = CoverageTracker::new();
        tracker.add_module_coverage("module_a", CoverageStats::new(80, 100));
        tracker.add_module_coverage("module_b", CoverageStats::new(90, 100));

        // Act: Generate report
        let report = tracker.generate_report();

        // Assert: Report should contain coverage information
        assert!(report.contains("Coverage Report"), "Report should have title");
        assert!(report.contains("Overall:"), "Report should have overall stats");
        assert!(report.contains("module_a"), "Report should list module A");
        assert!(report.contains("module_b"), "Report should list module B");
    }

    #[test]
    fn test_proptest_strategy_generation() {
        // Arrange: Create property generator
        let mut generator = PropertyGenerator::new();

        generator.add_strategy(
            PropertyStrategy::new("numeric_test")
                .with_input_type("i32")
                .with_constraint("range(0..100)")
                .with_property("non_negative"),
        );

        // Act: Generate proptest code
        let code = generator.generate_proptest_code();

        // Assert: Generated code should be valid
        assert_eq!(code.len(), 1, "Should generate one test");
        assert!(code[0].contains("numeric_test"), "Should include strategy name");
        assert!(code[0].contains("i32"), "Should include input type");
    }
}
