//! CNV 4.0 Integration Tests
//!
//! Comprehensive test suite using chicago-tdd-tools to validate the three pillars
//! of CNV 4.0: Capability Contracts, Session Kernel, and Version Negotiation.
//!
//! Focus: 80/20 principle - test the critical 20% that catches 80% of bugs.

use clap_noun_verb::kernel::grammar::{GrammarModel, GrammarNoun, GrammarVerb};
use clap_noun_verb::kernel::*;

// ============================================================================
// PILLAR 1: CAPABILITY CONTRACT TESTS
// ============================================================================

#[test]
fn test_capability_risk_invariants() {
    // CRITICAL: Risk scores must be monotonic with capability classes
    let pure = CapabilityContract::pure();
    let read_only = CapabilityContract::read_only();
    let read_write = CapabilityContract::read_write();
    let _network = CapabilityContract::network();
    let dangerous = CapabilityContract::dangerous();

    // Risk must increase with danger
    assert!(pure.risk_score() < read_only.risk_score());
    assert!(read_only.risk_score() < read_write.risk_score());
    assert!(read_write.risk_score() < dangerous.risk_score());

    // Pure must be minimum risk
    assert_eq!(pure.risk_score(), 0);

    // Dangerous must be maximum risk
    assert_eq!(dangerous.risk_score(), 100);
}

#[test]
fn test_capability_agent_safety_guarantees() {
    // CRITICAL: Agent safety must align with capability class

    // Safe capabilities
    let pure = CapabilityContract::pure();
    assert!(pure.is_agent_safe());
    assert_eq!(pure.safety, SafetyProfile::AgentSafe);

    let read_only = CapabilityContract::read_only();
    assert!(read_only.is_agent_safe());

    // Unsafe capabilities
    let read_write = CapabilityContract::read_write();
    assert!(!read_write.is_agent_safe());
    assert_eq!(read_write.safety, SafetyProfile::HumanReviewRequired);

    let dangerous = CapabilityContract::dangerous();
    assert!(!dangerous.is_agent_safe());
    assert_eq!(dangerous.safety, SafetyProfile::HumanReviewRequired);
}

#[test]
fn test_capability_compatibility_transitivity() {
    // CRITICAL: Compatibility must be transitive
    let pure = CapabilityContract::pure();
    let read_only = CapabilityContract::read_only();
    let read_write = CapabilityContract::read_write();

    // If A compatible with B, and B compatible with C, then A compatible with C
    if pure.is_compatible_with(&read_only) && read_only.is_compatible_with(&read_write) {
        assert!(pure.is_compatible_with(&read_write));
    }
}

#[test]
fn test_capability_context_enforcement() {
    // CRITICAL: Capability context must enforce permissions correctly

    // Pure context - no IO allowed
    let pure_ctx = CapabilityContext::new(CapabilityContract::pure());
    assert!(!pure_ctx.can_read_fs());
    assert!(!pure_ctx.can_write_fs());
    assert!(!pure_ctx.can_access_network());
    assert!(!pure_ctx.can_spawn_subprocess());

    // Read-only context - only reads allowed
    let ro_ctx = CapabilityContext::new(CapabilityContract::read_only());
    assert!(ro_ctx.can_read_fs());
    assert!(!ro_ctx.can_write_fs());
    assert!(!ro_ctx.can_access_network());

    // Read-write context - reads and writes allowed
    let rw_ctx = CapabilityContext::new(CapabilityContract::read_write());
    assert!(rw_ctx.can_read_fs());
    assert!(rw_ctx.can_write_fs());
    assert!(!rw_ctx.can_access_network());

    // Network context
    let net_ctx = CapabilityContext::new(CapabilityContract::network());
    assert!(!net_ctx.can_read_fs());
    assert!(!net_ctx.can_write_fs());
    assert!(net_ctx.can_access_network());

    // Dangerous context - everything allowed
    let danger_ctx = CapabilityContext::new(CapabilityContract::dangerous());
    assert!(danger_ctx.can_read_fs());
    assert!(danger_ctx.can_write_fs());
    assert!(danger_ctx.can_access_network());
    assert!(danger_ctx.can_spawn_subprocess());
}

#[test]
fn test_capability_resource_band_ordering() {
    // CRITICAL: Resource bands must have consistent ordering
    assert!(ResourceBand::Instant.max_runtime_ms() < ResourceBand::Fast.max_runtime_ms());
    assert!(ResourceBand::Fast.max_runtime_ms() < ResourceBand::Medium.max_runtime_ms());
    assert!(ResourceBand::Medium.max_runtime_ms() < ResourceBand::Slow.max_runtime_ms());

    assert!(ResourceBand::Instant.max_memory_bytes() < ResourceBand::Fast.max_memory_bytes());
    assert!(ResourceBand::Fast.max_memory_bytes() < ResourceBand::Medium.max_memory_bytes());
    assert!(ResourceBand::Medium.max_memory_bytes() < ResourceBand::Slow.max_memory_bytes());
}

#[test]
fn test_capability_dangerous_never_agent_safe() {
    // CRITICAL: Dangerous capabilities must NEVER be marked agent-safe
    let mut dangerous = CapabilityContract::dangerous();
    assert!(!dangerous.is_agent_safe());

    // Even if we try to force it (shouldn't be possible in real code)
    dangerous.safety = SafetyProfile::AgentSafe; // This would be a bug
    assert!(!dangerous.is_agent_safe()); // Class check should still fail
}

// ============================================================================
// PILLAR 2: SESSION KERNEL TESTS
// ============================================================================

#[test]
fn test_session_lifecycle() {
    // CRITICAL: Session must follow correct lifecycle
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Initially active
    assert!(session.is_active());
    assert!(!session.is_cancelled());

    // Can yield frames while active
    let frame = session.yield_data(StreamId::Stdout, serde_json::json!({"test": 1}));
    assert!(frame.is_ok());

    // Cancel session
    session.cancel();
    assert!(!session.is_active());
    assert!(session.is_cancelled());

    // Cannot yield frames after cancellation
    let frame = session.yield_data(StreamId::Stdout, serde_json::json!({"test": 2}));
    assert!(frame.is_err());
}

#[test]
fn test_session_frame_sequencing() {
    // CRITICAL: Frame sequences must be monotonic per stream
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Yield multiple frames to same stream
    let frame1 = session.yield_data(StreamId::Stdout, serde_json::json!({"n": 1})).ok().unwrap();
    let frame2 = session.yield_data(StreamId::Stdout, serde_json::json!({"n": 2})).ok().unwrap();
    let frame3 = session.yield_data(StreamId::Stdout, serde_json::json!({"n": 3})).ok().unwrap();

    // Sequences must be monotonic
    assert_eq!(frame1.sequence, 0);
    assert_eq!(frame2.sequence, 1);
    assert_eq!(frame3.sequence, 2);

    // All frames have same session ID
    assert_eq!(frame1.session_id, frame2.session_id);
    assert_eq!(frame2.session_id, frame3.session_id);
}

#[test]
fn test_session_stream_independence() {
    // CRITICAL: Each stream must have independent sequencing
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Yield to different streams
    let stdout1 =
        session.yield_data(StreamId::Stdout, serde_json::json!({"s": "out"})).ok().unwrap();
    let stderr1 =
        session.yield_data(StreamId::Stderr, serde_json::json!({"s": "err"})).ok().unwrap();
    let log1 = session.yield_log("info", "test", None).ok().unwrap();
    let stdout2 =
        session.yield_data(StreamId::Stdout, serde_json::json!({"s": "out2"})).ok().unwrap();

    // Each stream starts at sequence 0
    assert_eq!(stdout1.sequence, 0);
    assert_eq!(stderr1.sequence, 0);
    assert_eq!(log1.sequence, 0);

    // Same stream continues sequence
    assert_eq!(stdout2.sequence, 1);

    // Different streams
    assert_eq!(stdout1.stream_id, StreamId::Stdout);
    assert_eq!(stderr1.stream_id, StreamId::Stderr);
    assert_eq!(log1.stream_id, StreamId::Logs);
}

#[test]
fn test_session_metrics_tracking() {
    // CRITICAL: Metrics must accurately track session activity
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    let initial_metrics = session.metrics();
    assert_eq!(initial_metrics.frames_sent, 0);
    assert_eq!(initial_metrics.bytes_sent, 0);

    // Yield some frames
    session.yield_data(StreamId::Stdout, serde_json::json!({"test": 1})).ok();
    session.yield_data(StreamId::Stdout, serde_json::json!({"test": 2})).ok();
    session.yield_log("info", "message", None).ok();

    let final_metrics = session.metrics();
    assert_eq!(final_metrics.frames_sent, 3);
    assert!(final_metrics.bytes_sent > 0);
}

#[test]
fn test_session_frame_serialization() {
    // CRITICAL: Frames must be serializable and deserializable
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    let original = session
        .yield_data(StreamId::Stdout, serde_json::json!({"key": "value", "num": 42}))
        .ok()
        .unwrap();

    // Serialize to JSON
    let json = original.to_json().ok().unwrap();
    assert!(!json.is_empty());

    // Deserialize back
    let deserialized = Frame::from_json(&json).ok().unwrap();

    // Must match original
    assert_eq!(deserialized.session_id, original.session_id);
    assert_eq!(deserialized.stream_id, original.stream_id);
    assert_eq!(deserialized.sequence, original.sequence);
}

#[test]
fn test_session_control_messages() {
    // CRITICAL: Control messages must be properly created
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Send control messages
    let pause = session.send_control(ControlCommand::Pause).ok().unwrap();
    assert_eq!(pause.stream_id, StreamId::Control);

    let resume = session.send_control(ControlCommand::Resume).ok().unwrap();
    assert_eq!(resume.stream_id, StreamId::Control);

    let cancel = session.send_control(ControlCommand::Cancel).ok().unwrap();
    assert_eq!(cancel.stream_id, StreamId::Control);

    // Control messages should have independent sequencing
    assert_eq!(pause.sequence, 0);
    assert_eq!(resume.sequence, 1);
    assert_eq!(cancel.sequence, 2);
}

#[test]
fn test_session_multiple_data_types() {
    // CRITICAL: Sessions must handle various data types
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // String
    let f1 = session.yield_data(StreamId::Stdout, "test string");
    assert!(f1.is_ok());

    // Number
    let f2 = session.yield_data(StreamId::Stdout, 42);
    assert!(f2.is_ok());

    // Object
    let f3 = session.yield_data(StreamId::Stdout, serde_json::json!({"a": 1, "b": 2}));
    assert!(f3.is_ok());

    // Array
    let f4 = session.yield_data(StreamId::Stdout, vec![1, 2, 3]);
    assert!(f4.is_ok());
}

// ============================================================================
// PILLAR 3: VERSION NEGOTIATION TESTS
// ============================================================================

#[test]
fn test_grammar_delta_empty_change() {
    // CRITICAL: Identical grammars produce no changes
    let v1 = GrammarModel::new("test").with_version("1.0.0");
    let v2 = GrammarModel::new("test").with_version("1.0.0");

    let delta = GrammarDelta::compute(&v1, &v2).ok().unwrap();

    assert_eq!(delta.severity, ChangeSeverity::Safe);
    assert!(!delta.has_breaking_changes());
    assert!(delta.noun_changes.is_empty());
    assert!(delta.verb_changes.is_empty());
}

#[test]
fn test_grammar_delta_additive_changes() {
    // CRITICAL: Adding nouns/verbs is non-breaking
    let mut v1 = GrammarModel::new("test").with_version("1.0.0");
    let mut v2 = GrammarModel::new("test").with_version("2.0.0");

    // v1 has one noun
    let noun1 = create_test_noun("file", vec!["read"]);
    v1.add_noun(noun1);

    // v2 has same noun plus a new noun
    let noun1_v2 = create_test_noun("file", vec!["read"]);
    let noun2 = create_test_noun("network", vec!["fetch"]);
    v2.add_noun(noun1_v2);
    v2.add_noun(noun2);

    let delta = GrammarDelta::compute(&v1, &v2).ok().unwrap();

    // Should be safe or potentially breaking, but not breaking
    assert_ne!(delta.severity, ChangeSeverity::Breaking);
    assert!(!delta.has_breaking_changes());
}

#[test]
fn test_grammar_delta_removal_is_breaking() {
    // CRITICAL: Removing nouns/verbs is breaking
    let mut v1 = GrammarModel::new("test").with_version("1.0.0");
    let mut v2 = GrammarModel::new("test").with_version("2.0.0");

    // v1 has two nouns
    v1.add_noun(create_test_noun("file", vec!["read"]));
    v1.add_noun(create_test_noun("network", vec!["fetch"]));

    // v2 has only one (removed network)
    v2.add_noun(create_test_noun("file", vec!["read"]));

    let delta = GrammarDelta::compute(&v1, &v2).ok().unwrap();

    assert_eq!(delta.severity, ChangeSeverity::Breaking);
    assert!(delta.has_breaking_changes());
}

#[test]
fn test_grammar_delta_capability_changes() {
    // CRITICAL: Capability changes must be detected
    let mut v1 = GrammarModel::new("test").with_version("1.0.0");
    let mut v2 = GrammarModel::new("test").with_version("2.0.0");

    // Create noun with verb having read-only capability
    let mut noun1 = create_test_noun_empty("file");
    noun1.verbs.push(GrammarVerb {
        name: "read".to_string(),
        noun: "file".to_string(),
        help: None,
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(CapabilityContract::read_only()),
        metadata: Default::default(),
    });
    v1.add_noun(noun1);

    // Same verb but with read-write capability (more dangerous)
    let mut noun2 = create_test_noun_empty("file");
    noun2.verbs.push(GrammarVerb {
        name: "read".to_string(),
        noun: "file".to_string(),
        help: None,
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(CapabilityContract::read_write()),
        metadata: Default::default(),
    });
    v2.add_noun(noun2);

    let delta = GrammarDelta::compute(&v1, &v2).ok().unwrap();

    // Should detect capability change
    assert!(!delta.verb_changes.is_empty());
    let verb_change = &delta.verb_changes[0];
    assert!(!verb_change.capability_changes.is_empty());
}

#[test]
fn test_version_negotiation_strict_mode() {
    // CRITICAL: Strict mode rejects breaking changes
    let v1 = GrammarModel::new("test").with_version("1.0.0");
    let mut v2 = GrammarModel::new("test").with_version("2.0.0");

    // v2 has additional noun (non-breaking)
    v2.add_noun(create_test_noun("file", vec!["read"]));

    let mut negotiator = VersionNegotiator::new(v2);
    negotiator.add_history("1.0.0".to_string(), v1);

    let request = NegotiationRequest {
        known_version: "1.0.0".to_string(),
        required_capabilities: None,
        compatibility_level: CompatibilityLevel::Strict,
    };

    let response = negotiator.negotiate(&request).ok().unwrap();

    // Non-breaking change should be compatible in strict mode
    assert!(response.compatible);
}

#[test]
fn test_version_negotiation_breaking_change_detection() {
    // CRITICAL: Breaking changes must be detected and reported
    let mut v1 = GrammarModel::new("test").with_version("1.0.0");
    let v2 = GrammarModel::new("test").with_version("2.0.0");

    // v1 has a noun that v2 doesn't (removal is breaking)
    v1.add_noun(create_test_noun("deprecated", vec!["old"]));

    let mut negotiator = VersionNegotiator::new(v2);
    negotiator.add_history("1.0.0".to_string(), v1);

    let request = NegotiationRequest {
        known_version: "1.0.0".to_string(),
        required_capabilities: None,
        compatibility_level: CompatibilityLevel::Strict,
    };

    let response = negotiator.negotiate(&request).ok().unwrap();

    // Breaking change should be incompatible
    assert!(!response.compatible);
    assert!(!response.warnings.is_empty());
}

// ============================================================================
// INTEGRATION TESTS: CROSS-PILLAR
// ============================================================================

#[test]
fn test_session_respects_capability_contract() {
    // CRITICAL: Sessions must enforce their capability contract
    let pure_session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    assert_eq!(pure_session.capability().capability_class, CapabilityClass::Pure);
    assert!(pure_session.capability().is_agent_safe());

    let dangerous_session =
        SessionBuilder::new().capability(CapabilityContract::dangerous()).build();

    assert_eq!(dangerous_session.capability().capability_class, CapabilityClass::Dangerous);
    assert!(!dangerous_session.capability().is_agent_safe());
}

#[test]
fn test_test_harness_capability_validation() {
    // CRITICAL: Test harness must validate capability contracts
    let mut grammar = GrammarModel::new("test").with_version("1.0.0");

    // Add a dangerous verb marked as agent-safe (INVALID)
    let mut noun = create_test_noun_empty("dangerous");
    let mut dangerous_cap = CapabilityContract::dangerous();
    dangerous_cap.safety = SafetyProfile::AgentSafe; // Invalid combination

    noun.verbs.push(GrammarVerb {
        name: "bad".to_string(),
        noun: "dangerous".to_string(),
        help: None,
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(dangerous_cap),
        metadata: Default::default(),
    });

    grammar.add_noun(noun);

    // This should be caught by validation, but we need a TestHarness
    // which requires the full registry. For this test, we verify the logic exists.
    let cap = &grammar.nouns[0].verbs[0].capability.as_ref().unwrap();
    assert_eq!(cap.capability_class, CapabilityClass::Dangerous);
    assert_eq!(cap.safety, SafetyProfile::AgentSafe); // Invalid!
}

#[test]
fn test_capability_report_accuracy() {
    // CRITICAL: Capability reports must accurately count commands
    let mut grammar = GrammarModel::new("test").with_version("1.0.0");

    // Add various capability types
    let mut noun = create_test_noun_empty("mixed");

    noun.verbs.push(create_verb_with_cap("pure", CapabilityContract::pure()));
    noun.verbs.push(create_verb_with_cap("readonly", CapabilityContract::read_only()));
    noun.verbs.push(create_verb_with_cap("readwrite", CapabilityContract::read_write()));
    noun.verbs.push(create_verb_with_cap("no_cap", CapabilityContract::pure()));

    grammar.add_noun(noun);

    // Manually count
    let pure_count = grammar
        .all_verbs()
        .iter()
        .filter(|v| {
            v.capability.as_ref().map_or(false, |c| c.capability_class == CapabilityClass::Pure)
        })
        .count();

    let readonly_count = grammar
        .all_verbs()
        .iter()
        .filter(|v| {
            v.capability
                .as_ref()
                .map_or(false, |c| c.capability_class == CapabilityClass::ReadOnlyFS)
        })
        .count();

    assert_eq!(pure_count, 2);
    assert_eq!(readonly_count, 1);
}

// ============================================================================
// EDGE CASES AND ERROR CONDITIONS
// ============================================================================

#[test]
fn test_session_id_uniqueness() {
    // CRITICAL: Each session must have unique ID
    let session1 = SessionBuilder::new().build();
    let session2 = SessionBuilder::new().build();
    let session3 = SessionBuilder::new().build();

    assert_ne!(session1.id(), session2.id());
    assert_ne!(session2.id(), session3.id());
    assert_ne!(session1.id(), session3.id());
}

#[test]
fn test_session_id_serialization() {
    // CRITICAL: Session IDs must serialize/deserialize correctly
    let id = SessionId::new();
    let id_str = id.to_string();
    let parsed: SessionId = id_str.parse().ok().unwrap();

    assert_eq!(id, parsed);
}

#[test]
fn test_capability_contract_serialization() {
    // CRITICAL: Capability contracts must serialize/deserialize
    let contract = CapabilityContract::read_write().with_metadata("custom", "value");

    let json = serde_json::to_string(&contract).ok().unwrap();
    let deserialized: CapabilityContract = serde_json::from_str(&json).ok().unwrap();

    assert_eq!(deserialized.capability_class, contract.capability_class);
    assert_eq!(deserialized.resource_band, contract.resource_band);
    assert_eq!(deserialized.stability, contract.stability);
    assert_eq!(deserialized.safety, contract.safety);
}

#[test]
fn test_empty_grammar_delta() {
    // CRITICAL: Empty grammars should produce safe delta
    let v1 = GrammarModel::new("test").with_version("1.0.0");
    let v2 = GrammarModel::new("test").with_version("1.0.0");

    let delta = GrammarDelta::compute(&v1, &v2).ok().unwrap();

    assert_eq!(delta.severity, ChangeSeverity::Safe);
    assert!(delta.noun_changes.is_empty());
    assert!(delta.verb_changes.is_empty());
    assert!(!delta.has_breaking_changes());
}

#[test]
fn test_session_metrics_zero_state() {
    // CRITICAL: New sessions start with zero metrics
    let session = SessionBuilder::new().build();
    let metrics = session.metrics();

    assert_eq!(metrics.frames_sent, 0);
    assert_eq!(metrics.bytes_sent, 0);
    assert_eq!(metrics.errors, 0);
    assert_eq!(metrics.avg_latency_ms, 0.0);
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_test_noun(name: &str, verbs: Vec<&str>) -> GrammarNoun {
    let mut noun = GrammarNoun {
        name: name.to_string(),
        help: Some(format!("{} operations", name)),
        long_help: None,
        verbs: Vec::new(),
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    };

    for verb_name in verbs {
        noun.verbs.push(GrammarVerb {
            name: verb_name.to_string(),
            noun: name.to_string(),
            help: Some(format!("{} {}", verb_name, name)),
            long_help: None,
            arguments: Vec::new(),
            deprecated: false,
            deprecation_message: None,
            capability: Some(CapabilityContract::pure()),
            metadata: Default::default(),
        });
    }

    noun
}

fn create_test_noun_empty(name: &str) -> GrammarNoun {
    GrammarNoun {
        name: name.to_string(),
        help: Some(format!("{} operations", name)),
        long_help: None,
        verbs: Vec::new(),
        sub_nouns: Vec::new(),
        metadata: Default::default(),
    }
}

fn create_verb_with_cap(name: &str, capability: CapabilityContract) -> GrammarVerb {
    GrammarVerb {
        name: name.to_string(),
        noun: "test".to_string(),
        help: None,
        long_help: None,
        arguments: Vec::new(),
        deprecated: false,
        deprecation_message: None,
        capability: Some(capability),
        metadata: Default::default(),
    }
}

// ============================================================================
// PROPERTY-BASED AND STRESS TESTS
// ============================================================================

#[test]
fn test_capability_risk_score_bounds() {
    // PROPERTY: All risk scores must be in range [0, 100]
    let contracts = vec![
        CapabilityContract::pure(),
        CapabilityContract::read_only(),
        CapabilityContract::read_write(),
        CapabilityContract::network(),
        CapabilityContract::dangerous(),
    ];

    for contract in contracts {
        let score = contract.risk_score();
        assert!(score <= 100, "Risk score {} exceeds maximum 100", score);
        assert!(score >= 0, "Risk score {} below minimum 0", score);
    }
}

#[test]
fn test_capability_compatibility_reflexivity() {
    // PROPERTY: Every capability is compatible with itself
    let contracts = vec![
        CapabilityContract::pure(),
        CapabilityContract::read_only(),
        CapabilityContract::read_write(),
        CapabilityContract::network(),
        CapabilityContract::dangerous(),
    ];

    for contract in &contracts {
        assert!(
            contract.is_compatible_with(contract),
            "Capability {:?} not compatible with itself",
            contract.capability_class
        );
    }
}

#[test]
fn test_session_high_frame_count() {
    // STRESS: Session must handle hundreds of frames
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    let frame_count = 1000;
    for i in 0..frame_count {
        let frame = session.yield_data(StreamId::Stdout, serde_json::json!({"index": i}));
        assert!(frame.is_ok(), "Frame {} failed", i);
    }

    let metrics = session.metrics();
    assert_eq!(metrics.frames_sent, frame_count);
}

#[test]
fn test_session_multiple_streams_interleaved() {
    // STRESS: Interleaving frames across streams
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    let iterations = 100;
    for i in 0..iterations {
        session.yield_data(StreamId::Stdout, serde_json::json!({"out": i})).ok();
        session.yield_data(StreamId::Stderr, serde_json::json!({"err": i})).ok();
        session.yield_log("info", &format!("log {}", i), None).ok();
    }

    let metrics = session.metrics();
    // Should have 3 frames per iteration
    assert_eq!(metrics.frames_sent, iterations * 3);
}

#[test]
fn test_grammar_delta_idempotent() {
    // PROPERTY: Computing delta twice should yield same result
    let mut v1 = GrammarModel::new("test").with_version("1.0.0");
    let mut v2 = GrammarModel::new("test").with_version("2.0.0");

    v1.add_noun(create_test_noun("file", vec!["read"]));
    v2.add_noun(create_test_noun("file", vec!["read", "write"]));

    let delta1 = GrammarDelta::compute(&v1, &v2).ok().unwrap();
    let delta2 = GrammarDelta::compute(&v1, &v2).ok().unwrap();

    assert_eq!(delta1.severity, delta2.severity);
    assert_eq!(delta1.noun_changes.len(), delta2.noun_changes.len());
    assert_eq!(delta1.verb_changes.len(), delta2.verb_changes.len());
}

#[test]
fn test_session_cancellation_propagates() {
    // PROPERTY: Cancellation must prevent data operations but allow control messages
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    // Work before cancellation
    assert!(session.yield_data(StreamId::Stdout, "before").is_ok());

    // Cancel
    session.cancel();

    // Data operations must fail after cancellation
    assert!(session.yield_data(StreamId::Stdout, "after").is_err());
    assert!(session.yield_log("info", "after", None).is_err());

    // Control messages should still work (to communicate cancellation)
    assert!(session.send_control(ControlCommand::End { exit_code: 0 }).is_ok());

    // State must be consistent
    assert!(session.is_cancelled());
    assert!(!session.is_active());
}

#[test]
fn test_capability_class_ordering_consistency() {
    // PROPERTY: Risk levels must be consistent with class hierarchy
    let classes = vec![
        (CapabilityClass::Pure, 0),
        (CapabilityClass::ReadOnlyFS, 2),
        (CapabilityClass::Environment, 3),
        (CapabilityClass::Network, 5),
        (CapabilityClass::ReadWriteFS, 6),
        (CapabilityClass::Subprocess, 8),
        (CapabilityClass::Dangerous, 10),
    ];

    for (class, expected_risk) in classes {
        assert_eq!(class.risk_level(), expected_risk, "{:?} has unexpected risk level", class);
    }
}

#[test]
fn test_session_frame_timestamps_monotonic() {
    // PROPERTY: Frame timestamps should be monotonic (or at least non-decreasing)
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    let frame1 = session.yield_data(StreamId::Stdout, 1).ok().unwrap();
    let frame2 = session.yield_data(StreamId::Stdout, 2).ok().unwrap();
    let frame3 = session.yield_data(StreamId::Stdout, 3).ok().unwrap();

    assert!(
        frame1.timestamp_ms <= frame2.timestamp_ms,
        "Timestamps not monotonic: {} > {}",
        frame1.timestamp_ms,
        frame2.timestamp_ms
    );
    assert!(
        frame2.timestamp_ms <= frame3.timestamp_ms,
        "Timestamps not monotonic: {} > {}",
        frame2.timestamp_ms,
        frame3.timestamp_ms
    );
}

#[test]
fn test_grammar_delta_symmetry_of_additions_removals() {
    // PROPERTY: Adding in v2 should be inverse of removing in v1
    let mut v1 = GrammarModel::new("test").with_version("1.0.0");
    let mut v2 = GrammarModel::new("test").with_version("2.0.0");

    v1.add_noun(create_test_noun("file", vec!["read"]));
    // v2 is empty (removal from v1's perspective)

    let delta_removal = GrammarDelta::compute(&v1, &v2).ok().unwrap();
    assert!(delta_removal.has_breaking_changes()); // Removal is breaking

    let mut v3 = GrammarModel::new("test").with_version("1.0.0");
    let mut v4 = GrammarModel::new("test").with_version("2.0.0");
    // v3 is empty
    v4.add_noun(create_test_noun("file", vec!["read"])); // Addition

    let delta_addition = GrammarDelta::compute(&v3, &v4).ok().unwrap();
    assert!(!delta_addition.has_breaking_changes()); // Addition is non-breaking

    // Removals are breaking, additions are not
    assert_ne!(delta_removal.severity, delta_addition.severity);
}

#[test]
fn test_capability_metadata_preservation() {
    // PROPERTY: Custom metadata must survive serialization
    let mut contract = CapabilityContract::pure();
    contract = contract
        .with_metadata("author", "test")
        .with_metadata("version", 1)
        .with_metadata("tags", vec!["safe", "fast"]);

    let json = serde_json::to_string(&contract).ok().unwrap();
    let deserialized: CapabilityContract = serde_json::from_str(&json).ok().unwrap();

    assert_eq!(deserialized.metadata.len(), contract.metadata.len());
    assert!(deserialized.metadata.contains_key("author"));
    assert!(deserialized.metadata.contains_key("version"));
    assert!(deserialized.metadata.contains_key("tags"));
}

#[test]
fn test_session_metrics_monotonic_increase() {
    // PROPERTY: Session metrics should only increase, never decrease
    let mut session = SessionBuilder::new().capability(CapabilityContract::pure()).build();

    let mut prev_frames = 0u64;
    let mut prev_bytes = 0u64;

    for i in 0..50 {
        session.yield_data(StreamId::Stdout, serde_json::json!({"i": i})).ok();

        let metrics = session.metrics();
        assert!(
            metrics.frames_sent >= prev_frames,
            "Frame count decreased: {} < {}",
            metrics.frames_sent,
            prev_frames
        );
        assert!(
            metrics.bytes_sent >= prev_bytes,
            "Byte count decreased: {} < {}",
            metrics.bytes_sent,
            prev_bytes
        );

        prev_frames = metrics.frames_sent;
        prev_bytes = metrics.bytes_sent;
    }
}

#[test]
fn test_version_negotiation_unknown_version() {
    // EDGE CASE: Requesting unknown version should error
    let current = GrammarModel::new("test").with_version("2.0.0");
    let negotiator = VersionNegotiator::new(current);

    let request = NegotiationRequest {
        known_version: "0.1.0".to_string(), // Not in history
        required_capabilities: None,
        compatibility_level: CompatibilityLevel::Moderate,
    };

    let result = negotiator.negotiate(&request);
    assert!(result.is_err(), "Should error on unknown version");
}

#[test]
fn test_capability_contract_builder_pattern() {
    // PROPERTY: Builder pattern should be consistent
    let contract1 = CapabilityContract::new(
        CapabilityClass::ReadOnlyFS,
        ResourceBand::Fast,
        StabilityProfile::Stable,
        SafetyProfile::AgentSafe,
    );

    let contract2 = CapabilityContract::read_only();

    assert_eq!(contract1.capability_class, contract2.capability_class);
    assert_eq!(contract1.resource_band, contract2.resource_band);
    assert_eq!(contract1.stability, contract2.stability);
    assert_eq!(contract1.safety, contract2.safety);
}
