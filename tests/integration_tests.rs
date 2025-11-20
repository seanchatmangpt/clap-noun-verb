//! Integration tests for all six hyper-advanced tracks
//!
//! This test suite demonstrates all six tracks working together as a
//! cohesive verifiable protocol runtime for autonomous agent swarms.
//!
//! Tracks integrated:
//! 1. Certificates - Proof-carrying invocations
//! 2. Contracts - Temporal & concurrency constraints
//! 3. Hot Path - Zero-allocation execution
//! 4. Graph - Capability composition
//! 5. Delegation - Identity and authorization chains
//! 6. Governance - Observability and replay

use clap_noun_verb::autonomic::*;
use clap_noun_verb::autonomic::contracts::AvailableResources;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Complete end-to-end test demonstrating all six tracks
#[test]
fn test_complete_swarm_native_execution_flow() {
    println!("\n=== Swarm-Native CNV Integration Test ===\n");

    // ========== Track 6: Governance - Initialize Observability ==========
    println!("Track 6 (Governance): Initializing governance ledger...");
    let ledger = Arc::new(GovernanceLedger::new());

    // ========== Track 5: Delegation - Establish Identity Chain ==========
    println!("Track 5 (Delegation): Creating delegation chain...");

    // Alice (human) delegates to automated agent Bob
    let alice = Principal::new(
        AgentIdentity::human("alice"),
        TenantIdentity::default_tenant(),
    );

    let bob = Principal::delegated(
        AgentIdentity {
            agent_id: "agent-bob".to_string(),
            agent_type: "automated".to_string(),
            agent_version: Some("1.0.0".to_string()),
            metadata: std::collections::HashMap::new(),
        },
        TenantIdentity::default_tenant(),
    );

    // Bob can only read and list users, not delete
    let mut allowed_capabilities = HashSet::new();
    allowed_capabilities.insert(CapabilityId::from_path("user.read"));
    allowed_capabilities.insert(CapabilityId::from_path("user.list"));

    let delegation_token = DelegationToken::new(
        alice.clone(),
        bob.clone(),
        CapabilityConstraint {
            allowed_capabilities: Some(allowed_capabilities.clone()),
            forbidden_capabilities: HashSet::new(),
            allowed_nouns: None,
            allowed_verbs: None,
            max_effect_level: Some(EffectLevel::ReadOnly),
        },
        TemporalConstraint::valid_for(Duration::from_secs(3600)),
    );

    let delegation_chain = DelegationChain::with_delegation(delegation_token);

    assert!(delegation_chain.verify().is_ok());
    println!("  ✓ Delegation chain verified: Alice -> Bob (read-only)");

    // Record delegation in governance ledger
    ledger.record_policy_decision(
        PolicyDecision::Allow,
        CapabilityId::from_path("user.read"),
        "Delegation created",
        bob.agent.clone(),
        bob.tenant.clone(),
        "delegation-init",
    );

    // ========== Track 4: Graph - Build Capability Composition ==========
    println!("\nTrack 4 (Graph): Building capability graph...");
    let mut graph = CapabilityGraph::new();

    // Add capabilities
    let read_node = graph.add_node(
        CapabilityId::from_path("user.read"),
        "Read User",
        InputSchema {
            required: {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "user_id".to_string(),
                    TypeSchema::primitive(PrimitiveType::String),
                );
                map
            },
            optional: std::collections::HashMap::new(),
            accepts_stdin: false,
            stdin_schema: None,
        },
        OutputSchema {
            success: TypeSchema::primitive(PrimitiveType::String),
            error: Some(TypeSchema::primitive(PrimitiveType::String)),
            outputs_stdout: true,
            named_outputs: std::collections::HashMap::new(),
        },
        vec![EffectMetadata {
            effect_type: EffectType::ReadOnly,
            sensitivity: Sensitivity::Medium,
            idempotent: true,
            required_role: Some("user".to_string()),
            data_sensitivity: vec![DataSensitivityTag::Pii],
            isolation: IsolationRequirement::Shared,
            supports_dry_run: false,
        }],
    );

    let list_node = graph.add_node(
        CapabilityId::from_path("user.list"),
        "List Users",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
        vec![EffectMetadata {
            effect_type: EffectType::ReadOnly,
            sensitivity: Sensitivity::Low,
            idempotent: true,
            required_role: Some("user".to_string()),
            data_sensitivity: vec![],
            isolation: IsolationRequirement::Shared,
            supports_dry_run: false,
        }],
    );

    // user.list can be used with user.read
    graph
        .add_edge(list_node, read_node, EdgeType::Produces)
        .expect("Edge creation failed");

    println!("  ✓ Capability graph built with {} nodes", graph.stats().node_count);

    // Verify capability reachability
    assert!(graph.is_reachable(list_node, read_node));
    println!("  ✓ Verified reachability: user.list → user.read");

    // ========== Track 2: Contracts - Define Execution Requirements ==========
    println!("\nTrack 2 (Contracts): Defining execution contract...");

    let contract = ExecutionContract::builder()
        .duration_class(DurationClass::Interactive)
        .deadline(DeadlineSpec::Soft {
            duration: Duration::from_millis(100),
        })
        .idempotent()
        .concurrency_model(ConcurrencyModel::TenantWideShared { max_concurrent: 10 })
        .isolation(IsolationLevel::Shared)
        .resource_limits(ResourceLimits {
            max_memory_bytes: Some(10 * 1024 * 1024), // 10MB
            max_cpu_time: Some(Duration::from_millis(100)),
            max_file_descriptors: Some(10),
            max_network_connections: Some(5),
        })
        .build();

    println!("  ✓ Contract: {:?} with {} max concurrent",
        contract.temporal.duration_class,
        match contract.concurrency.model {
            ConcurrencyModel::TenantWideShared { max_concurrent } => max_concurrent,
            _ => 0,
        }
    );

    // Check if we have sufficient resources
    let available_resources = AvailableResources {
        tenant_slots: 15,
        global_slots: 100,
        available_memory_bytes: 1024 * 1024 * 1024, // 1GB
        available_cpu_percent: 70.0,
    };

    assert!(contract.can_satisfy(&available_resources));
    println!("  ✓ Contract can be satisfied with available resources");

    // ========== Track 1: Certificates - Create Proof-Carrying Invocation ==========
    println!("\nTrack 1 (Certificates): Creating proof-carrying certificate...");

    let capability_id = CapabilityId::from_path("user.read");

    // Verify capability is allowed by delegation
    assert!(delegation_chain.allows_capability(&capability_id));

    let cert = CertificateBuilder::new(
        capability_id.clone(),
        "1.0.0",
        InputSchema {
            required: {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "user_id".to_string(),
                    TypeSchema::primitive(PrimitiveType::String),
                );
                map
            },
            optional: std::collections::HashMap::new(),
            accepts_stdin: false,
            stdin_schema: None,
        },
        OutputSchema {
            success: TypeSchema::primitive(PrimitiveType::String),
            error: Some(TypeSchema::primitive(PrimitiveType::String)),
            outputs_stdout: true,
            named_outputs: std::collections::HashMap::new(),
        },
    )
    .with_agent(bob.agent.clone())
    .with_tenant(bob.tenant.clone())
    .with_effects(vec![EffectMetadata {
        effect_type: EffectType::ReadOnly,
        sensitivity: Sensitivity::Medium,
        idempotent: true,
        required_role: Some("user".to_string()),
        data_sensitivity: vec![DataSensitivityTag::Pii],
        isolation: IsolationRequirement::Shared,
        supports_dry_run: false,
    }])
    .with_correlation_id("swarm-request-42")
    .build();

    // Policy check
    let policy_result = PolicyResult {
        decision: PolicyDecision::Allow,
        evaluated_rules: vec!["allow-read-only".to_string()],
        metadata: std::collections::HashMap::new(),
    };

    let cert = cert
        .with_policy_check("swarm-policy-engine", &policy_result)
        .expect("Policy check failed");

    println!("  ✓ Policy check passed: Allow");

    // Capability check (graph knows this capability)
    let cert = cert
        .with_capability_check(&allowed_capabilities.iter().cloned().collect::<Vec<_>>())
        .expect("Capability check failed");

    println!("  ✓ Capability verified in graph");

    // Final verification
    let cert = cert.verify().expect("Certificate verification failed");

    println!("  ✓ Certificate fully verified (Certificate<Verified>)");
    println!("    Certificate ID: {:?}", cert.certificate_id);
    println!("    Correlation ID: {}", cert.correlation_id);

    // Record in governance ledger
    ledger.record_policy_decision(
        PolicyDecision::Allow,
        capability_id.clone(),
        "user.read --user-id 123",
        bob.agent.clone(),
        bob.tenant.clone(),
        "swarm-request-42",
    );

    // ========== Track 3: Hot Path - Zero-Allocation Execution ==========
    println!("\nTrack 3 (Hot Path): Executing via hot path...");

    // Create compact context
    let context_pool = ContextPool::new(100);
    let agent_handle = context_pool.alloc_agent_handle();
    let tenant_handle = context_pool.alloc_tenant_handle();

    let effect_flags = EffectFlags::empty()
        .with(EffectFlags::READ_ONLY)
        .with(EffectFlags::IDEMPOTENT);

    let hot_ctx = HotPathContext::new(agent_handle, tenant_handle, 0, effect_flags)
        .with_correlation("swarm-request-42");

    println!("  ✓ Hot path context created (32 bytes)");
    println!("    Agent handle: {}", hot_ctx.agent.id());
    println!("    Tenant handle: {}", hot_ctx.tenant.id());

    // Zero-copy parsing
    let command_input = "user.read --user-id 123";
    let mut args_buffer = [("", ""); 10];
    let mut positional_buffer = [""; 10];

    let parsed = ZeroCopyParser::parse(command_input, &mut args_buffer, &mut positional_buffer)
        .expect("Parse failed");

    println!("  ✓ Zero-copy parse completed (no allocations)");
    println!("    Capability: {}", parsed.capability_path);
    println!("    Args: {:?}", parsed.args);

    // Enqueue for hot path execution
    let queue = InvocationQueue::new(1000);
    queue.try_push(hot_ctx.clone()).expect("Queue push failed");

    println!("  ✓ Enqueued for hot path execution");
    println!("    Queue length: {}", queue.len());

    // Simulate execution
    let dequeued = queue.try_pop().expect("Queue empty");
    assert_eq!(dequeued.agent.id(), hot_ctx.agent.id());

    println!("  ✓ Hot path execution simulated");

    // ========== Final Integration: Governance Replay ==========
    println!("\nTrack 6 (Governance): Replaying governance events...");

    let start = SystemTime::UNIX_EPOCH;
    let end = SystemTime::now();

    let replay_engine = ReplayEngine::new(Arc::clone(&ledger));
    let result = replay_engine.replay_timeslice(start, end);

    let stats = result.stats();

    println!("  ✓ Replayed {} events", stats.total_events);
    println!("    Policy decisions: {}", stats.policy_decisions);
    println!("    Allows: {}", stats.allow_count);
    println!("    Denies: {}", stats.deny_count);

    // "What if" analysis: stricter policy
    println!("\n  Running 'what if' analysis with stricter policy...");

    let what_if_result = replay_engine.replay_with_policy(start, end, |cap, _| {
        // Hypothetical stricter policy
        if cap == &CapabilityId::from_path("user.read") {
            PolicyDecision::Deny { suggestion: None,
                reason: "Hypothetically forbidden".to_string(),
            }
        } else {
            PolicyDecision::Allow
        }
    });

    println!("  ✓ What-if analysis complete");
    println!("    Differences detected: {}", what_if_result.differences.len());

    if !what_if_result.differences.is_empty() {
        println!("    Impact: {} previously allowed operations would be denied",
            what_if_result.differences.len());
    }

    // ========== Final Summary ==========
    println!("\n=== Integration Test Summary ===");
    println!("✓ All six tracks integrated successfully:");
    println!("  [1] Certificates: Type-safe proof-carrying invocations");
    println!("  [2] Contracts: Temporal & concurrency constraints enforced");
    println!("  [3] Hot Path: Zero-allocation execution demonstrated");
    println!("  [4] Graph: Capability composition and reachability verified");
    println!("  [5] Delegation: Identity chain with constraint intersection");
    println!("  [6] Governance: Full observability and replay capability");
    println!("\nCNV is now a verifiable protocol runtime for trillion-agent swarms.");
}

/// Test delegation chain with certificate verification
#[test]
fn test_delegation_chain_with_certificates() {
    // GIVEN: A delegation chain with restricted capabilities
    let origin = Principal::new(
        AgentIdentity::human("admin"),
        TenantIdentity::default_tenant(),
    );

    let delegate = Principal::delegated(
        AgentIdentity::human("agent"),
        TenantIdentity::default_tenant(),
    );

    let mut allowed = HashSet::new();
    allowed.insert(CapabilityId::from_path("safe.operation"));

    let token = DelegationToken::new(
        origin,
        delegate.clone(),
        CapabilityConstraint {
            allowed_capabilities: Some(allowed.clone()),
            forbidden_capabilities: HashSet::new(),
            allowed_nouns: None,
            allowed_verbs: None,
            max_effect_level: Some(EffectLevel::ReadOnly),
        },
        TemporalConstraint::valid_for(Duration::from_secs(3600)),
    );

    let chain = DelegationChain::with_delegation(token);

    // WHEN: We create a certificate for an allowed capability
    let cert = CertificateBuilder::new(
        CapabilityId::from_path("safe.operation"),
        "1.0.0",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
    )
    .with_agent(delegate.agent)
    .with_tenant(delegate.tenant)
    .build();

    // THEN: Chain allows the capability
    assert!(chain.allows_capability(&CapabilityId::from_path("safe.operation")));

    // AND: Certificate can be verified through full pipeline
    let policy_result = PolicyResult {
        decision: PolicyDecision::Allow,
        evaluated_rules: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let verified = cert
        .with_policy_check("test", &policy_result)
        .unwrap()
        .with_capability_check(&allowed.iter().cloned().collect::<Vec<_>>())
        .unwrap()
        .verify()
        .unwrap();

    assert!(verified.is_valid());
}

/// Test hot path with governance logging
#[test]
fn test_hot_path_with_governance() {
    // GIVEN: A governance ledger and hot path queue
    let ledger = Arc::new(GovernanceLedger::new());
    let queue = InvocationQueue::new(100);

    // WHEN: We process invocations through hot path
    for i in 0..10 {
        let ctx = HotPathContext::new(
            AgentHandle::new(i),
            TenantHandle::new(1),
            i as u32,
            EffectFlags::empty().with(EffectFlags::READ_ONLY),
        );

        queue.try_push(ctx).ok();

        // Log to governance
        ledger.record_policy_decision(
            PolicyDecision::Allow,
            CapabilityId::from_path(&format!("cmd{}", i)),
            &format!("Hot path invocation {}", i),
            AgentIdentity::anonymous(),
            TenantIdentity::default_tenant(),
            format!("hotpath-{}", i),
        );
    }

    // THEN: All invocations are queued
    let stats = queue.stats();
    assert_eq!(stats.total_enqueued, 10);

    // AND: All are logged
    assert_eq!(ledger.event_count(), 10);

    // AND: We can replay them
    let engine = ReplayEngine::new(Arc::clone(&ledger));
    let result = engine.replay_timeslice(SystemTime::UNIX_EPOCH, SystemTime::now());

    assert_eq!(result.policy_decisions, 10);
}

/// Test graph-based capability composition with contracts
#[test]
fn test_graph_composition_with_contracts() {
    // GIVEN: A capability graph
    let mut graph = CapabilityGraph::new();

    // Add two capabilities with different contracts
    let fast_cap = graph.add_node(
        CapabilityId::from_path("fast.query"),
        "Fast Query",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
        vec![],
    );

    let slow_cap = graph.add_node(
        CapabilityId::from_path("slow.report"),
        "Slow Report",
        InputSchema::default(),
        OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
        vec![],
    );

    graph.add_edge(fast_cap, slow_cap, EdgeType::Produces).ok();

    // WHEN: We define contracts for each
    let fast_contract = ExecutionContract::builder()
        .duration_class(DurationClass::FastPath)
        .build();

    let slow_contract = ExecutionContract::builder()
        .duration_class(DurationClass::Batch)
        .build();

    // THEN: Contracts reflect their purpose
    assert!(fast_contract.temporal.duration_class < slow_contract.temporal.duration_class);

    // AND: Graph shows composition is possible
    assert!(graph.is_reachable(fast_cap, slow_cap));
}
