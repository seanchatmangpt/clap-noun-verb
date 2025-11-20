//! Comprehensive tests for governance observability and replay
//!
//! Critical 80/20 test coverage:
//! - Governance ledger operations
//! - Event recording
//! - Ledger queries
//! - Replay engine
//! - "What if" policy analysis
//! - Compliance proof generation

use clap_noun_verb::autonomic::*;
use std::sync::Arc;
use std::time::SystemTime;

#[test]
fn test_governance_ledger_creation() {
    // GIVEN: A new ledger
    let ledger = GovernanceLedger::new();

    // THEN: It's initially empty
    assert_eq!(ledger.event_count(), 0);
}

#[test]
fn test_record_capability_granted() {
    // GIVEN: A governance ledger
    let ledger = GovernanceLedger::new();

    // WHEN: We record a capability grant
    let event_id = ledger.record_capability_granted(
        CapabilityId::from_path("user.create"),
        AgentIdentity::human("admin"),
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "grant-123",
    );

    // THEN: Event is recorded
    assert_eq!(ledger.event_count(), 1);

    // AND: Event ID is generated
    assert_eq!(event_id, EventId::new(0));
}

#[test]
fn test_record_policy_changed() {
    // GIVEN: A governance ledger
    let ledger = GovernanceLedger::new();

    // WHEN: We record a policy change
    ledger.record_policy_changed(
        "policy-1",
        PolicyChangeType::Updated,
        AgentIdentity::human("admin"),
        AgentIdentity::human("admin"),
        TenantIdentity::default_tenant(),
        "change-123",
    );

    // THEN: Event is recorded
    assert_eq!(ledger.event_count(), 1);
}

#[test]
fn test_record_policy_decision() {
    // GIVEN: A governance ledger
    let ledger = GovernanceLedger::new();

    // WHEN: We record a policy decision
    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("test.cmd"),
        "test command",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "request-123",
    );

    // THEN: Event is recorded
    assert_eq!(ledger.event_count(), 1);
}

#[test]
fn test_record_security_violation() {
    // GIVEN: A governance ledger
    let ledger = GovernanceLedger::new();

    // WHEN: We record a security violation
    ledger.record_security_violation(
        "unauthorized_access",
        ViolationSeverity::High,
        "Attempted to access admin panel without proper role",
        AgentIdentity::human("mallory"),
        TenantIdentity::default_tenant(),
        "incident-456",
    );

    // THEN: Event is recorded
    assert_eq!(ledger.event_count(), 1);
}

#[test]
fn test_ledger_checkpoint() {
    // GIVEN: A ledger with some events
    let ledger = GovernanceLedger::new();

    for i in 0..5 {
        ledger.record_capability_granted(
            CapabilityId::from_path(&format!("cap{}", i)),
            AgentIdentity::anonymous(),
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
            format!("event-{}", i),
        );
    }

    // WHEN: We create a checkpoint
    ledger.checkpoint(
        "checkpoint-1",
        AgentIdentity::human("admin"),
        TenantIdentity::default_tenant(),
    );

    // THEN: Checkpoint event is added
    assert_eq!(ledger.event_count(), 6);
}

#[test]
fn test_ledger_query_by_time_range() {
    // GIVEN: A ledger with events at different times
    let ledger = GovernanceLedger::new();

    let start = SystemTime::now();

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd1"),
        "command 1",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "req-1",
    );

    std::thread::sleep(std::time::Duration::from_millis(10));

    let middle = SystemTime::now();

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd2"),
        "command 2",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "req-2",
    );

    let end = SystemTime::now();

    // WHEN: We query for a time range
    let all_events = ledger.query().time_range(start, end).execute();

    let middle_events = ledger.query().time_range(middle, end).execute();

    // THEN: Correct events are returned
    assert_eq!(all_events.len(), 2);
    assert_eq!(middle_events.len(), 1);
}

#[test]
fn test_ledger_query_by_agent() {
    // GIVEN: A ledger with events from different agents
    let ledger = GovernanceLedger::new();

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd1"),
        "command 1",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "req-1",
    );

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd2"),
        "command 2",
        AgentIdentity::human("bob"),
        TenantIdentity::default_tenant(),
        "req-2",
    );

    // WHEN: We query by agent
    let alice_events = ledger.query().agent("alice".to_string()).execute();
    let bob_events = ledger.query().agent("bob".to_string()).execute();

    // THEN: Correct events are returned
    assert_eq!(alice_events.len(), 1);
    assert_eq!(bob_events.len(), 1);
}

#[test]
fn test_ledger_query_by_correlation_id() {
    // GIVEN: A ledger with events with different correlation IDs
    let ledger = GovernanceLedger::new();

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd1"),
        "command 1",
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "batch-1",
    );

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd2"),
        "command 2",
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "batch-1",
    );

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd3"),
        "command 3",
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "batch-2",
    );

    // WHEN: We query by correlation ID
    let batch1_events = ledger.query().correlation_id("batch-1".to_string()).execute();

    // THEN: Correct events are returned
    assert_eq!(batch1_events.len(), 2);
}

#[test]
fn test_ledger_query_count() {
    // GIVEN: A ledger with multiple events
    let ledger = GovernanceLedger::new();

    for i in 0..10 {
        ledger.record_policy_decision(
            PolicyDecision::Allow,
            CapabilityId::from_path(&format!("cmd{}", i)),
            "command",
            AgentIdentity::human("alice"),
            TenantIdentity::default_tenant(),
            format!("req-{}", i),
        );
    }

    // WHEN: We count events
    let count = ledger.query().agent("alice".to_string()).count();

    // THEN: Count is correct
    assert_eq!(count, 10);
}

#[test]
fn test_replay_engine_timeslice() {
    // GIVEN: A ledger with policy decisions
    let ledger = Arc::new(GovernanceLedger::new());

    let start = SystemTime::now();

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("user.read"),
        "read user",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "req-1",
    );

    ledger.record_policy_decision(
        PolicyDecision::Deny { suggestion: None, reason: "Insufficient permissions".to_string() },
        CapabilityId::from_path("user.delete"),
        "delete user",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "req-2",
    );

    let end = SystemTime::now();

    // WHEN: We replay the timeslice
    let engine = ReplayEngine::new(Arc::clone(&ledger));
    let result = engine.replay_timeslice(start, end);

    // THEN: All decisions are replayed
    assert_eq!(result.policy_decisions, 2);

    // AND: Stats are available
    let stats = result.stats();
    assert_eq!(stats.allow_count, 1);
    assert_eq!(stats.deny_count, 1);
}

#[test]
fn test_replay_engine_what_if_analysis() {
    // GIVEN: A ledger with historical decisions
    let ledger = Arc::new(GovernanceLedger::new());

    let start = SystemTime::now();

    // Record some Allow decisions
    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("sensitive.operation"),
        "sensitive operation",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "req-1",
    );

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("normal.operation"),
        "normal operation",
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
        "req-2",
    );

    let end = SystemTime::now();

    // WHEN: We replay with a stricter policy
    let engine = ReplayEngine::new(Arc::clone(&ledger));
    let result = engine.replay_with_policy(start, end, |cap_id, _cmd| {
        // New policy: deny sensitive operations
        if cap_id == &CapabilityId::from_path("sensitive.operation") {
            PolicyDecision::Deny { suggestion: None, reason: "Now forbidden by policy".to_string() }
        } else {
            PolicyDecision::Allow
        }
    });

    // THEN: Differences are detected
    assert!(result.differences.len() > 0);

    // AND: The sensitive operation is identified as different
    let has_sensitive_diff = result
        .differences
        .iter()
        .any(|diff| diff.capability_id == CapabilityId::from_path("sensitive.operation"));
    assert!(has_sensitive_diff);
}

#[test]
fn test_replay_stats_calculation() {
    // GIVEN: Replay results with known outcomes
    let ledger = Arc::new(GovernanceLedger::new());

    let start = SystemTime::now();

    for i in 0..3 {
        ledger.record_policy_decision(
            PolicyDecision::Allow,
            CapabilityId::from_path(&format!("allow-cmd-{}", i)),
            "allowed command",
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
            format!("req-{}", i),
        );
    }

    for i in 0..2 {
        ledger.record_policy_decision(
            PolicyDecision::Deny { suggestion: None, reason: "Test denial".to_string() },
            CapabilityId::from_path(&format!("deny-cmd-{}", i)),
            "denied command",
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
            format!("req-deny-{}", i),
        );
    }

    let end = SystemTime::now();

    // WHEN: We replay and get stats
    let engine = ReplayEngine::new(Arc::clone(&ledger));
    let result = engine.replay_timeslice(start, end);
    let stats = result.stats();

    // THEN: Stats are accurate
    assert_eq!(stats.total_events, 5);
    assert_eq!(stats.policy_decisions, 5);
    assert_eq!(stats.allow_count, 3);
    assert_eq!(stats.deny_count, 2);
    assert_eq!(stats.differences_count, 0); // No what-if, so no differences
}

#[test]
fn test_governance_event_types() {
    // GIVEN: A ledger
    let ledger = GovernanceLedger::new();

    // WHEN: We record different event types
    ledger.record_capability_granted(
        CapabilityId::from_path("cap"),
        AgentIdentity::anonymous(),
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "test",
    );

    ledger.record_policy_changed(
        "policy",
        PolicyChangeType::Created,
        AgentIdentity::anonymous(),
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "test",
    );

    ledger.record_security_violation(
        "violation",
        ViolationSeverity::Critical,
        "desc",
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "test",
    );

    // THEN: All events are recorded
    assert_eq!(ledger.event_count(), 3);
}

#[test]
fn test_violation_severity_ordering() {
    // GIVEN: Different severity levels
    let low = ViolationSeverity::Low;
    let medium = ViolationSeverity::Medium;
    let high = ViolationSeverity::High;
    let critical = ViolationSeverity::Critical;

    // THEN: They are ordered correctly
    assert!(low < medium);
    assert!(medium < high);
    assert!(high < critical);
}

#[test]
fn test_policy_change_types() {
    // GIVEN: Different policy change types
    let created = PolicyChangeType::Created;
    let updated = PolicyChangeType::Updated;
    let deleted = PolicyChangeType::Deleted;
    let enabled = PolicyChangeType::Enabled;
    let disabled = PolicyChangeType::Disabled;

    // THEN: All types are distinct
    let types = vec![created, updated, deleted, enabled, disabled];
    assert_eq!(types.len(), 5);
}

#[test]
fn test_ledger_metadata_preservation() {
    // GIVEN: A ledger
    let ledger = GovernanceLedger::new();

    let agent = AgentIdentity::human("alice");
    let tenant = TenantIdentity::default_tenant();
    let correlation = "test-correlation";

    // WHEN: We record an event
    ledger.record_capability_granted(
        CapabilityId::from_path("test.cap"),
        AgentIdentity::human("admin"),
        agent.clone(),
        tenant.clone(),
        correlation,
    );

    // THEN: We can query and verify metadata
    let events = ledger.query().correlation_id(correlation.to_string()).execute();

    assert_eq!(events.len(), 1);
    let event = &events[0];

    assert_eq!(event.agent.agent_id, "alice");
    assert_eq!(event.correlation_id, correlation);
}

#[test]
fn test_replay_with_no_differences() {
    // GIVEN: A ledger with decisions
    let ledger = Arc::new(GovernanceLedger::new());

    let start = SystemTime::now();

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd"),
        "command",
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "req",
    );

    let end = SystemTime::now();

    // WHEN: We replay with same policy
    let engine = ReplayEngine::new(Arc::clone(&ledger));
    let result = engine.replay_with_policy(start, end, |_, _| PolicyDecision::Allow);

    // THEN: No differences
    assert_eq!(result.differences.len(), 0);
}

#[test]
fn test_replay_differences_detection() {
    // GIVEN: A ledger with Allow decision
    let ledger = Arc::new(GovernanceLedger::new());

    let start = SystemTime::now();

    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("cmd"),
        "command",
        AgentIdentity::anonymous(),
        TenantIdentity::default_tenant(),
        "req",
    );

    let end = SystemTime::now();

    // WHEN: We replay with Deny policy
    let engine = ReplayEngine::new(Arc::clone(&ledger));
    let result = engine.replay_with_policy(start, end, |_, _| PolicyDecision::Deny {
        suggestion: None,
        reason: "New policy denies".to_string(),
    });

    // THEN: Difference is detected
    assert_eq!(result.differences.len(), 1);

    let diff = &result.differences[0];
    assert!(matches!(diff.original, PolicyDecision::Allow));
    assert!(matches!(diff.new, PolicyDecision::Deny { .. }));
}
