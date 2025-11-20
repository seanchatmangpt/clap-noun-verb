//! Comprehensive tests for proof-carrying command certificates
//!
//! Critical 80/20 test coverage:
//! - Type-state machine transitions
//! - Policy check enforcement
//! - Capability verification
//! - Expiration handling
//! - Serialization/deserialization
//! - Certificate caching

#![allow(clippy::unwrap_used)] // Test code: unwrap is acceptable for test assertions

use clap_noun_verb::autonomic::certificates::Verified;
use clap_noun_verb::autonomic::*;
use std::time::Duration;

#[test]
fn test_certificate_type_state_transitions() {
    // GIVEN: An unchecked certificate
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("user.create"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .with_agent(AgentIdentity::human("alice"))
    .with_tenant(TenantIdentity::default_tenant())
    .build();

    // WHEN: Policy check passes
    let policy_result = PolicyResult {
        decision: PolicyDecision::Allow,
        evaluated_rules: vec!["allow-all".to_string()],
        metadata: std::collections::HashMap::new(),
    };

    let cert =
        cert.with_policy_check("test-engine", &policy_result).expect("Policy check should succeed");

    // THEN: Certificate is now PolicyChecked
    // AND: Can transition to CapabilityChecked
    let available = vec![CapabilityId::from_path("user.create")];
    let cert = cert.with_capability_check(&available).expect("Capability check should succeed");

    // AND: Can transition to Verified
    let cert = cert.verify().expect("Verification should succeed");

    // AND: Can access verified methods
    assert_eq!(cert.capability_id(), &CapabilityId::from_path("user.create"));
    assert!(cert.is_valid());
}

#[test]
fn test_certificate_policy_denial_blocks_transition() {
    // GIVEN: An unchecked certificate
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("admin.delete"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .build();

    // WHEN: Policy check denies
    let policy_result = PolicyResult {
        decision: PolicyDecision::Deny {
            suggestion: None,
            reason: "Insufficient permissions".to_string(),
        },
        evaluated_rules: vec!["deny-admin".to_string()],
        metadata: std::collections::HashMap::new(),
    };

    let result = cert.with_policy_check("test-engine", &policy_result);

    // THEN: Transition fails
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CertificateError::PolicyDenied(_)));
}

#[test]
fn test_certificate_missing_capability_blocks_transition() {
    // GIVEN: A policy-checked certificate
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("user.create"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .build();

    let policy_result = PolicyResult {
        decision: PolicyDecision::Allow,
        evaluated_rules: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let cert = cert.with_policy_check("test", &policy_result).unwrap();

    // WHEN: Capability is not available
    let available = vec![CapabilityId::from_path("user.read")]; // Different capability

    let result = cert.with_capability_check(&available);

    // THEN: Transition fails
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CertificateError::CapabilityNotAvailable(_)));
}

#[test]
fn test_certificate_expiration() {
    // GIVEN: A certificate that expires soon
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("temp.action"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .with_expiration(Duration::from_millis(1))
    .build();

    // WHEN: We pass through the full pipeline quickly
    let policy_result = PolicyResult {
        decision: PolicyDecision::Allow,
        evaluated_rules: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let cert = cert
        .with_policy_check("test", &policy_result)
        .unwrap()
        .with_capability_check(&[CapabilityId::from_path("temp.action")])
        .unwrap();

    // AND: Wait for expiration
    std::thread::sleep(Duration::from_millis(10));

    // THEN: Verification fails due to expiration
    let result = cert.verify();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CertificateError::Expired));
}

#[test]
fn test_certificate_serialization_roundtrip() {
    // GIVEN: A fully verified certificate
    let cert = create_verified_certificate();

    // WHEN: We export it
    let exported = cert.export().expect("Export should succeed");

    // THEN: We can import it back
    let imported = Certificate::<Verified>::import(&exported).expect("Import should succeed");

    // AND: Properties are preserved
    assert_eq!(cert.certificate_id, imported.certificate_id);
    assert_eq!(cert.capability_id, imported.capability_id);
    assert_eq!(cert.version, imported.version);
}

#[test]
fn test_certificate_with_effects() {
    // GIVEN: A certificate with declared effects
    let effects = vec![
        EffectMetadata {
            effect_type: EffectType::MutateState,
            sensitivity: Sensitivity::High,
            idempotent: true,
            required_role: Some("admin".to_string()),
            data_sensitivity: vec![DataSensitivityTag::Pii],
            isolation: IsolationRequirement::Isolated,
            supports_dry_run: false,
        },
        EffectMetadata {
            effect_type: EffectType::NetworkAccess,
            sensitivity: Sensitivity::Medium,
            idempotent: false,
            required_role: None,
            data_sensitivity: vec![],
            isolation: IsolationRequirement::Shared,
            supports_dry_run: false,
        },
    ];

    let cert = CertificateBuilder::new(
        CapabilityId::from_path("data.sync"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .with_effects(effects.clone())
    .build();

    // THEN: Effects are preserved
    assert_eq!(cert.effects.len(), 2);
    assert_eq!(cert.effects[0].effect_type, EffectType::MutateState);
    assert_eq!(cert.effects[1].effect_type, EffectType::NetworkAccess);
}

#[test]
fn test_certified_invocation_wrapping() {
    // GIVEN: A verified certificate and parsed arguments
    let cert = create_verified_certificate();
    let cert_id = cert.certificate_id.clone();
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    let args_for_invocation = args.clone();

    // WHEN: We create a certified invocation
    let invocation = CertifiedInvocation::new(cert, args_for_invocation);

    // THEN: Both certificate and args are accessible
    assert_eq!(invocation.certificate().certificate_id, cert_id);
    assert_eq!(invocation.args(), &args);

    // AND: Can decompose back
    let (retrieved_cert, retrieved_args) = invocation.into_parts();
    assert_eq!(retrieved_cert.certificate_id, cert_id);
    assert_eq!(retrieved_args, args);
}

#[test]
fn test_certificate_correlation_id() {
    // GIVEN: Certificates with the same correlation ID
    let correlation_id = "batch-123";

    let cert1 = CertificateBuilder::new(
        CapabilityId::from_path("op1"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .with_correlation_id(correlation_id)
    .build();

    let cert2 = CertificateBuilder::new(
        CapabilityId::from_path("op2"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .with_correlation_id(correlation_id)
    .build();

    // THEN: They share the same correlation ID
    assert_eq!(cert1.correlation_id, correlation_id);
    assert_eq!(cert2.correlation_id, correlation_id);
}

#[test]
fn test_certificate_policy_trace() {
    // GIVEN: A certificate that passed policy evaluation
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("test.cmd"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .build();

    let policy_result = PolicyResult {
        decision: PolicyDecision::Allow,
        evaluated_rules: vec!["rule1".to_string(), "rule2".to_string()],
        metadata: std::collections::HashMap::new(),
    };

    let cert = cert.with_policy_check("engine-1", &policy_result).unwrap();

    // THEN: Policy trace is recorded
    assert_eq!(cert.policy_trace.policy_engine_id, "engine-1");
    assert_eq!(cert.policy_trace.evaluated_rules.len(), 2);
    assert_eq!(cert.policy_trace.matched_rule, None); // PolicyResult doesn't provide matched_rule
    assert!(matches!(cert.policy_trace.decision, PolicyDecision::Allow));
}

#[test]
fn test_certificate_schema_hashes() {
    // GIVEN: Specific input and output schemas
    let mut required = std::collections::HashMap::new();
    required.insert("name".to_string(), TypeSchema::primitive(PrimitiveType::String));
    let input_schema = InputSchema {
        required,
        optional: std::collections::HashMap::new(),
        accepts_stdin: false,
        stdin_schema: None,
    };

    let output_schema = OutputSchema {
        success: TypeSchema::primitive(PrimitiveType::String),
        error: Some(TypeSchema::primitive(PrimitiveType::String)),
        outputs_stdout: true,
        named_outputs: std::collections::HashMap::new(),
    };

    // WHEN: We create a certificate
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("test"),
        "1.0.0",
        input_schema.clone(),
        output_schema.clone(),
    )
    .build();

    // THEN: Schema hashes are computed deterministically
    let expected_input_hash = SchemaHash::from_input_schema(&input_schema);
    let expected_output_hash = SchemaHash::from_output_schema(&output_schema);

    assert_eq!(cert.input_schema_hash, expected_input_hash);
    assert_eq!(cert.output_schema_hash, expected_output_hash);

    // AND: Same schemas produce same hashes
    let cert2 = CertificateBuilder::new(
        CapabilityId::from_path("test2"),
        "1.0.0",
        input_schema.clone(),
        output_schema.clone(),
    )
    .build();

    assert_eq!(cert.input_schema_hash, cert2.input_schema_hash);
    assert_eq!(cert.output_schema_hash, cert2.output_schema_hash);
}

// Helper function to create a verified certificate for tests
fn create_verified_certificate() -> Certificate<Verified> {
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("test.operation"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .with_agent(AgentIdentity::human("test-user"))
    .with_tenant(TenantIdentity::default_tenant())
    .build();

    let policy_result = PolicyResult {
        decision: PolicyDecision::Allow,
        evaluated_rules: vec![],
        metadata: std::collections::HashMap::new(),
    };

    cert.with_policy_check("test-engine", &policy_result)
        .unwrap()
        .with_capability_check(&[CapabilityId::from_path("test.operation")])
        .unwrap()
        .verify()
        .unwrap()
}
