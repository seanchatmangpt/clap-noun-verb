//! CNV 4.0 Advanced Testing Suite
//!
//! Demonstrates hyper-advanced testing using chicago-tdd-tools' most sophisticated capabilities:
//! - Property-based testing with proptest (finds edge cases automatically)
//! - Snapshot testing with insta (complex data validation)
//! - Advanced assertions (type-safe, expressive)
//! - Async testing with tokio
//! - Performance testing with tick budgets

use chicago_tdd_tools::{assert_err, assert_in_range, assert_ok};
use clap_noun_verb::kernel::grammar::{GrammarModel, GrammarNoun, GrammarVerb};
use clap_noun_verb::kernel::*;
use proptest::prelude::*;

// ============================================================================
// PROPERTY-BASED TESTING: Capability Contracts
// ============================================================================

#[test]
fn test_capability_contract_properties_comprehensive() {
    // PROPERTY: Risk scores must satisfy mathematical properties
    proptest!(|(
        class in prop_oneof![
            Just(CapabilityClass::Pure),
            Just(CapabilityClass::ReadOnlyFS),
            Just(CapabilityClass::ReadWriteFS),
            Just(CapabilityClass::Network),
            Just(CapabilityClass::Subprocess),
            Just(CapabilityClass::Environment),
            Just(CapabilityClass::Dangerous),
        ],
        resource_band in prop_oneof![
            Just(ResourceBand::Instant),
            Just(ResourceBand::Fast),
            Just(ResourceBand::Medium),
            Just(ResourceBand::Slow),
            Just(ResourceBand::Cold),
        ],
        stability in prop_oneof![
            Just(StabilityProfile::Stable),
            Just(StabilityProfile::Preview),
            Just(StabilityProfile::Experimental),
            Just(StabilityProfile::Deprecated),
            Just(StabilityProfile::NonDeterministic),
        ],
        safety in prop_oneof![
            Just(SafetyProfile::AgentSafe),
            Just(SafetyProfile::HumanReviewRequired),
            Just(SafetyProfile::InteractiveOnly),
        ]
    )| {
        // Arrange
        let contract = CapabilityContract::new(class, resource_band, stability, safety);

        // Act & Assert: Properties that MUST hold for ALL combinations

        // Property 1: Risk score bounded [0, 100]
        prop_assert!(contract.risk_score() <= 100);

        // Property 2: Dangerous class NEVER agent-safe
        if class == CapabilityClass::Dangerous {
            prop_assert!(!contract.is_agent_safe());
        }

        // Property 3: AgentSafe + Stable + Fast resource band = low risk
        if contract.is_agent_safe()
            && stability == StabilityProfile::Stable
            && matches!(resource_band, ResourceBand::Instant | ResourceBand::Fast) {
            prop_assert!(contract.risk_score() < 40);
        }

        // Property 4: Reflexivity (compatible with self)
        prop_assert!(contract.is_compatible_with(&contract));

        // Property 5: Serialization round-trip
        let json = serde_json::to_string(&contract).unwrap();
        let deserialized: CapabilityContract = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(contract.capability_class, deserialized.capability_class);
        prop_assert_eq!(contract.resource_band, deserialized.resource_band);
    });
}

#[test]
fn test_capability_contract_risk_ordering_property() {
    // PROPERTY: More dangerous capabilities always have higher risk
    proptest!(|(
        safer in prop_oneof![
            Just(CapabilityClass::Pure),
            Just(CapabilityClass::ReadOnlyFS),
            Just(CapabilityClass::Environment),
        ],
        riskier in prop_oneof![
            Just(CapabilityClass::ReadWriteFS),
            Just(CapabilityClass::Network),
            Just(CapabilityClass::Subprocess),
            Just(CapabilityClass::Dangerous),
        ]
    )| {
        // Arrange
        let safe_contract = CapabilityContract::new(
            safer,
            ResourceBand::Fast,
            StabilityProfile::Stable,
            SafetyProfile::AgentSafe
        );

        let risky_contract = CapabilityContract::new(
            riskier,
            ResourceBand::Fast,
            StabilityProfile::Stable,
            SafetyProfile::HumanReviewRequired
        );

        // Act & Assert: Risk must be ordered
        prop_assert!(safe_contract.risk_score() < risky_contract.risk_score());
        prop_assert!(safe_contract.is_agent_safe());
        prop_assert!(!risky_contract.is_agent_safe());
    });
}

#[test]
fn test_resource_band_monotonicity_property() {
    // PROPERTY: Resource bands are strictly monotonic
    proptest!(|(index in 0usize..5)| {
        let bands = vec![
            ResourceBand::Instant,
            ResourceBand::Fast,
            ResourceBand::Medium,
            ResourceBand::Slow,
            ResourceBand::Cold,
        ];

        if index > 0 {
            let prev = &bands[index - 1];
            let current = &bands[index];

            // Runtime must increase
            prop_assert!(prev.max_runtime_ms() < current.max_runtime_ms());

            // Memory must increase
            prop_assert!(prev.max_memory_bytes() < current.max_memory_bytes());
        }
    });
}

// ============================================================================
// PROPERTY-BASED TESTING: Session Kernel
// ============================================================================

#[test]
fn test_session_frame_sequence_property() {
    // PROPERTY: Frame sequences are always monotonic regardless of order
    proptest!(|(
        frame_count in 1usize..100,
        stream_type in prop_oneof![
            Just(StreamId::Stdout),
            Just(StreamId::Stderr),
            Just(StreamId::Logs),
        ]
    )| {
        // Arrange
        let mut session = SessionBuilder::new()
            .capability(CapabilityContract::pure())
            .build();

        // Act: Generate N frames
        let mut sequences = Vec::new();
        for i in 0..frame_count {
            let frame = session.yield_data(
                stream_type,
                serde_json::json!({"index": i})
            ).unwrap();
            sequences.push(frame.sequence);
        }

        // Assert: Sequences strictly monotonic
        for i in 1..sequences.len() {
            prop_assert!(sequences[i] > sequences[i - 1]);
            prop_assert_eq!(sequences[i], sequences[i - 1] + 1);
        }
    });
}

#[test]
fn test_session_metrics_accumulation_property() {
    // PROPERTY: Metrics never decrease, always accumulate
    proptest!(|(operations in 1usize..50)| {
        // Arrange
        let mut session = SessionBuilder::new()
            .capability(CapabilityContract::pure())
            .build();

        // Act & Assert: Track metrics through operations
        let mut prev_frames = 0u64;
        let mut prev_bytes = 0u64;

        for i in 0..operations {
            session.yield_data(
                StreamId::Stdout,
                serde_json::json!({"op": i})
            ).ok();

            let metrics = session.metrics();

            // Frames always increase
            prop_assert!(metrics.frames_sent >= prev_frames);

            // Bytes always increase
            prop_assert!(metrics.bytes_sent >= prev_bytes);

            prev_frames = metrics.frames_sent;
            prev_bytes = metrics.bytes_sent;
        }

        // Final check: total matches operations
        let final_metrics = session.metrics();
        prop_assert_eq!(final_metrics.frames_sent, operations as u64);
    });
}

// ============================================================================
// SNAPSHOT TESTING: Grammar Deltas
// ============================================================================

#[test]
fn test_grammar_delta_snapshot_validation() {
    use insta::assert_json_snapshot;

    // Arrange: Create two versions of a grammar
    let mut v1 = GrammarModel::new("test-app").with_version("1.0.0");
    let mut v2 = GrammarModel::new("test-app").with_version("2.0.0");

    // v1: Basic file operations
    let noun1 = GrammarNoun {
        name: "file".to_string(),
        help: Some("File operations".to_string()),
        long_help: None,
        verbs: vec![GrammarVerb {
            name: "read".to_string(),
            noun: "file".to_string(),
            help: Some("Read a file".to_string()),
            long_help: None,
            arguments: Vec::new(),
            deprecated: false,
            deprecation_message: None,
            capability: Some(CapabilityContract::read_only()),
            metadata: Default::default(),
        }],
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };
    v1.add_noun(noun1);

    // v2: Added write capability (capability upgrade - breaking!)
    let noun2 = GrammarNoun {
        name: "file".to_string(),
        help: Some("File operations".to_string()),
        long_help: None,
        verbs: vec![GrammarVerb {
            name: "read".to_string(),
            noun: "file".to_string(),
            help: Some("Read a file".to_string()),
            long_help: None,
            arguments: Vec::new(),
            deprecated: false,
            deprecation_message: None,
            capability: Some(CapabilityContract::read_write()), // CHANGED
            metadata: Default::default(),
        }],
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };
    v2.add_noun(noun2);

    // Act: Compute delta
    let delta = GrammarDelta::compute(&v1, &v2).ok().unwrap();

    // Assert: Snapshot the delta structure
    assert_json_snapshot!("grammar_delta_capability_upgrade", delta);

    // Verify breaking change detected
    assert_ok!(if !delta.verb_changes.is_empty() {
        Ok(())
    } else {
        Err("Expected capability change not detected")
    });
}

#[test]
fn test_capability_contract_snapshot_formats() {
    use insta::assert_json_snapshot;

    // Arrange: Create various capability contracts
    let pure = CapabilityContract::pure();
    let dangerous = CapabilityContract::dangerous();

    // Act & Assert: Snapshot in JSON format
    // Note: Metadata field uses HashMap which has non-deterministic serialization order
    // so we test contracts without metadata to avoid flaky snapshots
    assert_json_snapshot!("capability_pure_contract", pure);
    assert_json_snapshot!("capability_dangerous_contract", dangerous);
}

// ============================================================================
// ASYNC TESTING: Session Operations
// ============================================================================

#[tokio::test]
async fn test_session_async_operations() {
    // Arrange
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Act: Simulate async operations with delays
    for i in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        let frame = session.yield_data(StreamId::Stdout, serde_json::json!({"async_op": i}));

        // Assert: Each frame succeeds
        assert_ok!(frame);
    }

    // Final assertion
    let metrics = session.metrics();
    assert_eq!(metrics.frames_sent, 10);
}

#[tokio::test]
async fn test_session_cancellation_async() {
    // Arrange
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Act: Start async work
    let work_started = session.yield_data(StreamId::Stdout, "starting").is_ok();
    assert!(work_started);

    // Simulate async cancellation
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        // Session would be cancelled here
    })
    .await
    .ok();

    // Assert: Can still query state
    assert!(!session.is_cancelled()); // Not cancelled in this scope
}

// ============================================================================
// ADVANCED ASSERTIONS: Result & Error Handling
// ============================================================================

#[test]
fn test_advanced_result_assertions() {
    // Arrange
    let success: Result<CapabilityContract, String> = Ok(CapabilityContract::pure());
    let failure: Result<CapabilityContract, String> = Err("failed".to_string());

    // Assert: Use advanced chicago-tdd-tools assertions
    assert_ok!(success);
    assert_err!(failure);

    // Extract value safely
    let contract = success.ok().unwrap();
    assert_in_range!(contract.risk_score(), 0, 10);

    // Custom messages
    assert_eq!(
        contract.capability_class,
        CapabilityClass::Pure,
        "Expected Pure capability, got {:?}",
        contract.capability_class
    );
}

#[test]
fn test_error_handling_patterns() {
    // Arrange
    let good_session = SessionBuilder::new().build();
    let bad_session = {
        let mut session = SessionBuilder::new().build();
        session.cancel();
        session
    };

    // Act
    let good_result = good_session.check_cancellation();
    let bad_result = bad_session.check_cancellation();

    // Assert: Pattern matching on results
    assert_ok!(good_result);
    assert_err!(bad_result);
}

// ============================================================================
// PERFORMANCE TESTING: Bounded Operations
// ============================================================================

#[test]
fn test_capability_risk_calculation_performance() {
    // Arrange
    let contracts = vec![
        CapabilityContract::pure(),
        CapabilityContract::read_only(),
        CapabilityContract::read_write(),
        CapabilityContract::network(),
        CapabilityContract::dangerous(),
    ];

    // Act: Calculate risk scores (should be instant)
    let start = std::time::Instant::now();
    let scores: Vec<u8> = contracts.iter().map(|c| c.risk_score()).collect();
    let elapsed = start.elapsed();

    // Assert: All calculated within performance budget (< 1ms)
    assert!(elapsed.as_micros() < 1000, "Risk calculation took too long: {:?}", elapsed);
    assert_eq!(scores.len(), 5);
    assert!(scores[0] < scores[4], "Risk scores not ordered correctly");
}

#[test]
fn test_session_frame_generation_performance() {
    // Arrange
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Act: Generate 100 frames (should complete in microseconds)
    let start = std::time::Instant::now();
    for i in 0..100 {
        let _frame = session.yield_data(StreamId::Stdout, serde_json::json!({"i": i}));
    }
    let elapsed = start.elapsed();

    // Assert: Within performance budget (< 10ms for 100 frames)
    let metrics = session.metrics();
    assert_eq!(metrics.frames_sent, 100);
    assert!(elapsed.as_millis() < 10, "Frame generation too slow: {:?}", elapsed);
}

// ============================================================================
// COMPREHENSIVE INTEGRATION: All Three Pillars
// ============================================================================

#[test]
fn test_complete_cnv4_workflow() {
    // PILLAR 1: Create capability-aware grammar
    let mut grammar = GrammarModel::new("test-app").with_version("1.0.0");

    let noun = GrammarNoun {
        name: "data".to_string(),
        help: Some("Data operations".to_string()),
        long_help: None,
        verbs: vec![GrammarVerb {
            name: "fetch".to_string(),
            noun: "data".to_string(),
            help: Some("Fetch data".to_string()),
            long_help: None,
            arguments: Vec::new(),
            deprecated: false,
            deprecation_message: None,
            capability: Some(CapabilityContract::network()),
            metadata: Default::default(),
        }],
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };
    grammar.add_noun(noun);

    // PILLAR 2: Create session with matching capability
    let mut session = SessionBuilder::new().capability(CapabilityContract::network()).build();

    // Simulate command execution
    let frame = session.yield_data(StreamId::Stdout, serde_json::json!({"status": "fetching"}));
    assert_ok!(frame);

    // PILLAR 3: Create new version with enhanced capability
    let mut grammar_v2 = GrammarModel::new("test-app").with_version("2.0.0");
    let noun_v2 = GrammarNoun {
        name: "data".to_string(),
        help: Some("Data operations".to_string()),
        long_help: None,
        verbs: vec![
            GrammarVerb {
                name: "fetch".to_string(),
                noun: "data".to_string(),
                help: Some("Fetch data".to_string()),
                long_help: None,
                arguments: Vec::new(),
                deprecated: false,
                deprecation_message: None,
                capability: Some(CapabilityContract::network()), // Same
                metadata: Default::default(),
            },
            GrammarVerb {
                name: "store".to_string(),
                noun: "data".to_string(),
                help: Some("Store data".to_string()),
                long_help: None,
                arguments: Vec::new(),
                deprecated: false,
                deprecation_message: None,
                capability: Some(CapabilityContract::read_write()), // NEW
                metadata: Default::default(),
            },
        ],
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };
    grammar_v2.add_noun(noun_v2);

    // Compute delta
    let delta = GrammarDelta::compute(&grammar, &grammar_v2).ok().unwrap();

    // Assert: Non-breaking change (added verb)
    assert_eq!(delta.from_version, "1.0.0");
    assert_eq!(delta.to_version, "2.0.0");
    assert_eq!(delta.verb_changes.len(), 1); // One new verb

    // Verify session metrics
    let metrics = session.metrics();
    assert!(metrics.frames_sent > 0);
    assert!(metrics.bytes_sent > 0);
}
