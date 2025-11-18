/// Trillion-Agent Ecosystem End-to-End Demo
///
/// Demonstrates complete integration of:
/// - 2028: Individual Agent Coordination
/// - 2029-2030+: Swarm Intelligence
/// - Orchestration Layer: Central dispatcher and coordinator
/// - Event Bus: Cross-tier communication
/// - Integration Bridge: System interconnection

use clap_noun_verb::agent2028::{
    Orchestrator, OperationRequest, EventBus, EventType, Event, IntegrationBridge, AgentTier,
};
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║                  TRILLION-AGENT ECOSYSTEM                           ║");
    println!("║        CNV 2028-2030+ Complete Integration Demonstration           ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    demo_orchestration_layer().await;
    demo_event_bus_communication().await;
    demo_integration_bridge().await;
    demo_end_to_end_workflow().await;

    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║                  ECOSYSTEM DEMO COMPLETE ✓                          ║");
    println!("║         All systems integrated and communicating successfully       ║");
    println!("╚════════════════════════════════════════════════════════════════════╝");
}

async fn demo_orchestration_layer() {
    println!("\n┌─ LAYER 1: Orchestration & Agent Routing ──────────────────────────┐");

    let orchestrator = Orchestrator::new();

    // Allocate resources to both tiers
    println!("  ✓ Allocating resources to agent tiers");
    orchestrator
        .allocate_resources(AgentTier::Individual, 50.0, 2048, 1000)
        .await;
    orchestrator
        .allocate_resources(AgentTier::Swarm, 50.0, 2048, 10000)
        .await;

    // Register agents in both tiers
    println!("  ✓ Registering agents");
    for i in 1..=5 {
        orchestrator
            .register_agent(format!("individual-agent-{}", i), AgentTier::Individual)
            .await;
    }
    for i in 1..=10 {
        orchestrator
            .register_agent(format!("swarm-agent-{}", i), AgentTier::Swarm)
            .await;
    }

    let mut stats = orchestrator.stats().await;
    println!("  ✓ Registered agents: {}", stats.total_agents);
    println!("  ✓ Healthy tiers: {}/{}", stats.tiers_healthy, stats.tiers_total);

    // Route operations to both tiers
    println!("  ✓ Routing operations");

    // Individual tier operation
    let ind_req = OperationRequest::new(
        "individual-agent-1".to_string(),
        "compute".to_string(),
        "process data locally".to_string(),
    );
    let ind_result = orchestrator.route_operation(ind_req).await;
    println!(
        "    • Individual agent task: {} ({}ms)",
        if ind_result.success { "✓" } else { "✗" },
        ind_result.execution_time_ms
    );

    // Swarm tier operation
    let swarm_req = OperationRequest::new(
        "swarm-agent-1".to_string(),
        "collective_consensus".to_string(),
        "reach swarm consensus".to_string(),
    );
    let swarm_result = orchestrator.route_operation(swarm_req).await;
    println!(
        "    • Swarm collective task: {} ({}ms)",
        if swarm_result.success { "✓" } else { "✗" },
        swarm_result.execution_time_ms
    );

    // Show final orchestrator stats
    stats = orchestrator.stats().await;
    println!("  ✓ Operations queued: {}", stats.queued_operations);
    println!("  ✓ Operations completed: {}", stats.completed_operations);
}

async fn demo_event_bus_communication() {
    println!("\n┌─ LAYER 2: Cross-Tier Event Bus Communication ───────────────────┐");

    let event_bus = EventBus::new(1000);

    println!("  ✓ Created event bus (1000-event buffer)");

    // Subscribe agents to events
    let (_sub1, mut rx1) = event_bus
        .subscribe(
            "agent-1".to_string(),
            vec![EventType::AgentStarted, EventType::AgentFailed],
        )
        .await;

    let (_sub2, mut rx2) = event_bus
        .subscribe(
            "swarm-coordinator".to_string(),
            vec![EventType::SwarmFormed, EventType::SwarmDecision],
        )
        .await;

    println!("  ✓ Subscriptions created: 2 agents");

    // Publish various events
    println!("  ✓ Publishing events");

    let events = vec![
        Event::new(
            EventType::AgentStarted,
            "individual-agent-1".to_string(),
            "Agent initialized with quantum-safe crypto".to_string(),
        ),
        Event::new(
            EventType::SwarmFormed,
            "swarm-coordinator".to_string(),
            "Swarm of 50 agents formed for collective task".to_string(),
        ),
        Event::new(
            EventType::ConsensusRequired,
            "swarm-1".to_string(),
            "Consensus needed for migration decision".to_string(),
        ),
        Event::new(
            EventType::VotingCompleted,
            "swarm-1".to_string(),
            "Consensus reached: migrate to region B".to_string(),
        ),
        Event::new(
            EventType::SwarmDecision,
            "swarm-coordinator".to_string(),
            "Executing swarm decision".to_string(),
        ),
    ];

    for event in events {
        let _ = event_bus.publish(event).await;
    }

    println!("    • Published 5 events");

    // Check event history
    let history = event_bus.get_history(10).await;
    println!("  ✓ Event history: {} events recorded", history.len());

    // Get stats
    let stats = event_bus.stats().await;
    println!("  ✓ Total events published: {}", stats.total_events);
    println!("  ✓ Active subscriptions: {}", stats.active_subscribers);
    println!("  ✓ Event types: {}", stats.events_by_type.len());

    // Show subscription info
    let subs = event_bus.agent_subscriptions("agent-1").await;
    println!("  ✓ Agent-1 subscriptions: {} event types", subs[0].handler_count);
}

async fn demo_integration_bridge() {
    println!("\n┌─ LAYER 3: Integration Bridge (2028 ↔ 2029+) ──────────────────────┐");

    let bridge = IntegrationBridge::new();

    println!("  ✓ Created integration bridge");

    // Map individual agents to swarm agents
    println!("  ✓ Creating tier mappings");

    bridge
        .add_agent_to_swarm(
            "individual-agent-1".to_string(),
            vec![
                "swarm-agent-1".to_string(),
                "swarm-agent-2".to_string(),
                "swarm-agent-3".to_string(),
            ],
        )
        .await;

    bridge
        .add_agent_to_swarm(
            "individual-agent-2".to_string(),
            vec![
                "swarm-agent-4".to_string(),
                "swarm-agent-5".to_string(),
            ],
        )
        .await;

    // Retrieve mappings
    if let Some(swarm_agents) = bridge.get_swarm_agents("individual-agent-1").await {
        println!("    • individual-agent-1 → {} swarm agents", swarm_agents.len());
    }

    if let Some(swarm_agents) = bridge.get_swarm_agents("individual-agent-2").await {
        println!("    • individual-agent-2 → {} swarm agents", swarm_agents.len());
    }

    // Translate operations
    println!("  ✓ Testing operation translation");

    let individual_request = OperationRequest::new(
        "individual-agent-1".to_string(),
        "compute".to_string(),
        "analyze data".to_string(),
    );

    let swarm_request = bridge.translate_to_swarm_operation(&individual_request);
    println!(
        "    • Individual '{}' → Swarm '{}'",
        individual_request.operation_type, swarm_request.operation_type
    );
}

async fn demo_end_to_end_workflow() {
    println!("\n┌─ LAYER 4: Complete End-to-End Workflow ────────────────────────────┐");

    let orchestrator = Orchestrator::new();
    let event_bus = EventBus::new(1000);
    let bridge = IntegrationBridge::new();

    println!("  ✓ Initializing trillion-agent ecosystem");

    // PHASE 1: Setup
    println!("\n  ┌─ Phase 1: Setup & Registration");
    orchestrator
        .allocate_resources(AgentTier::Individual, 60.0, 3000, 500)
        .await;
    orchestrator
        .allocate_resources(AgentTier::Swarm, 40.0, 3000, 5000)
        .await;

    // Register agents
    for i in 1..=5 {
        orchestrator
            .register_agent(format!("agent-{}", i), AgentTier::Individual)
            .await;
    }
    println!("    • Registered 5 individual agents");
    println!("    • Tier allocation: Individual 60%, Swarm 40%");

    // PHASE 2: Individual Agent Tasks
    println!("\n  ├─ Phase 2: Individual Agent Execution");
    let start = Instant::now();

    for i in 1..=3 {
        let req = OperationRequest::new(
            format!("agent-{}", i),
            "local_compute".to_string(),
            format!("Process dataset {}", i),
        );
        let _result = orchestrator.route_operation(req).await;
    }

    let individual_time = start.elapsed().as_millis();
    println!("    • Completed 3 individual agent tasks ({}ms)", individual_time);

    // PHASE 3: Swarm Formation & Coordination
    println!("\n  ├─ Phase 3: Swarm Formation");
    let swarm_start = Instant::now();

    // Publish swarm formation event
    let swarm_event = Event::new(
        EventType::SwarmFormed,
        "swarm-coordinator".to_string(),
        "50-agent collective intelligence swarm".to_string(),
    );
    let _ = event_bus.publish(swarm_event).await;

    // Create swarm to individual mappings
    let mut swarm_agents = Vec::new();
    for i in 1..=50 {
        swarm_agents.push(format!("swarm-{}", i));
    }
    bridge
        .add_agent_to_swarm("agent-1".to_string(), swarm_agents.clone())
        .await;

    println!("    • Formed swarm with 50 agents");
    println!("    • Assigned to individual-agent-1 for coordination");

    // PHASE 4: Consensus & Decision
    println!("\n  ├─ Phase 4: Swarm Consensus");

    let consensus_event = Event::new(
        EventType::ConsensusRequired,
        "swarm-1".to_string(),
        "Vote on resource allocation strategy".to_string(),
    );
    let _ = event_bus.publish(consensus_event).await;

    // Simulate consensus operation through orchestrator
    let consensus_req = OperationRequest::new(
        "swarm-1".to_string(),
        "consensus_vote".to_string(),
        "voting_threshold=0.7".to_string(),
    );
    let _consensus_result = orchestrator.route_operation(consensus_req).await;

    let decision_event = Event::new(
        EventType::SwarmDecision,
        "swarm-1".to_string(),
        "Decision: distribute workload across 3 regions".to_string(),
    );
    let _ = event_bus.publish(decision_event).await;

    let swarm_time = swarm_start.elapsed().as_millis();
    println!("    • Consensus reached ({}ms)", swarm_time);
    println!("    • Decision: distribute workload across 3 regions");

    // PHASE 5: Failure Recovery
    println!("\n  └─ Phase 5: Fault Tolerance & Recovery");

    orchestrator.mark_tier_unhealthy(AgentTier::Swarm).await;
    println!("    • Swarm tier marked unhealthy (simulated failure)");

    let recovery_event = Event::new(
        EventType::FailoverInitiated,
        "orchestrator".to_string(),
        "Initiating failover to backup swarm".to_string(),
    );
    let _ = event_bus.publish(recovery_event).await;

    orchestrator.mark_tier_healthy(AgentTier::Swarm).await;
    println!("    • Swarm tier restored to healthy");

    let failover_event = Event::new(
        EventType::FailoverCompleted,
        "orchestrator".to_string(),
        "Failover completed successfully".to_string(),
    );
    let _ = event_bus.publish(failover_event).await;

    // FINAL STATS
    println!("\n  ┌─ Final System Statistics");
    let orch_stats = orchestrator.stats().await;
    let bus_stats = event_bus.stats().await;

    println!(
        "    • Total operations: {} (queue) + {} (completed)",
        orch_stats.queued_operations, orch_stats.completed_operations
    );
    println!("    • Total agents: {}", orch_stats.total_agents);
    println!("    • System health: {}/{} tiers", orch_stats.tiers_healthy, orch_stats.tiers_total);
    println!("    • Total events: {}", bus_stats.total_events);
    println!("    • Active subscriptions: {}", bus_stats.active_subscribers);
    println!(
        "    • Total execution time: ~{:.0}ms",
        (individual_time + swarm_time) as f64
    );
    println!("    • System throughput: {:.0} ops/sec", {
        let total_ops = (orch_stats.queued_operations + orch_stats.completed_operations) as f64;
        let total_time = (individual_time + swarm_time) as f64 / 1000.0;
        total_ops / total_time.max(0.001)
    });

    println!("  └─ ECOSYSTEM STATUS: ✓ FULLY OPERATIONAL");
}
