/// Comprehensive AI Agent Swarm Intelligence Demo
///
/// Demonstrates 2029-2030+ swarm innovations:
/// 1. Stigmergic Communication (pheromone-based coordination)
/// 2. Collective Intelligence (voting & hivemind)
/// 3. Swarm Behavior (flocking, formations)
/// 4. Task Allocation (distributed marketplace)
/// 5. Emergence (self-organizing rules)
/// 6. Swarm Optimization (PSO, ACO)
/// 7. Resilience (fault-tolerant swarming)
/// 8. Communication Protocols (gossip, broadcast)
use clap_noun_verb::agent2028::swarm::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("═════════════════════════════════════════════════════════════════");
    println!("  CNV 2029-2030+ Swarm Intelligence Innovations Demo");
    println!("  Trillion-Agent AI Ecosystems");
    println!("═════════════════════════════════════════════════════════════════\n");

    demo_stigmergic_coordination().await;
    demo_collective_intelligence().await;
    demo_swarm_behavior().await;
    demo_task_allocation().await;
    demo_emergence().await;
    demo_swarm_optimization().await;
    demo_resilience().await;
    demo_communication_protocols().await;

    println!("\n═════════════════════════════════════════════════════════════════");
    println!("  Swarm Intelligence Demo Complete!");
    println!("═════════════════════════════════════════════════════════════════");
}

async fn demo_stigmergic_coordination() {
    println!("\n┌─ Feature 1: Stigmergic Communication ───────────────────────────┐");

    let field = Arc::new(PheromoneField::new(0.3, 0.1));
    let protocol = StigmergicProtocol::new(field.clone());

    // Ant finds food and leaves trail
    println!("  ✓ Ant 1 finds food at (50, 50)");
    protocol.signal_resource(50, 50, "food".to_string(), "ant-1".to_string()).await;

    // Other ants follow the trail
    println!("  ✓ Ant 2 detects food trail");
    let path = protocol.find_nearest(30, 30, "resource:food", 50).await;
    if let Some((x, y)) = path {
        println!("  ✓ Ant 2 following trail to ({}, {})", x, y);
    }

    // Simulate diffusion and decay
    protocol.diffuse_pheromones().await;
    protocol.decay_pheromones().await;
    println!("  ✓ Pheromones diffused and decayed");

    let active = field.active_pheromones().await;
    println!("  ✓ Active pheromones: {} cells", active.len());
}

async fn demo_collective_intelligence() {
    println!("\n┌─ Feature 2: Collective Intelligence & HiveMind ────────────────┐");

    let voting = Arc::new(VotingProtocol::new());
    let hivemind = HiveMind::new(voting.clone());

    // Swarm votes on critical decision
    let voting_id =
        voting.create_pool("should_migrate".to_string(), "majority".to_string(), 60).await;

    println!("  ✓ Swarm voting on: should we migrate?");

    // Agents cast weighted votes
    voting.vote(&voting_id, "agent-1".to_string(), "yes".to_string(), 0.9, 1.0).await;
    voting.vote(&voting_id, "agent-2".to_string(), "yes".to_string(), 0.85, 1.0).await;
    voting.vote(&voting_id, "agent-3".to_string(), "no".to_string(), 0.6, 0.8).await;
    voting.vote(&voting_id, "agent-4".to_string(), "yes".to_string(), 0.95, 1.0).await;

    if let Some((decision, score)) = voting.get_consensus(&voting_id).await {
        println!("  ✓ Consensus reached: {} (score: {:.2})", decision, score);
    }

    // Update hivemind
    hivemind.update_from_voting(&voting_id, "migration_intent".to_string()).await;

    let state = hivemind.read().await;
    println!("  ✓ HiveMind updated with {} beliefs", state.collective_beliefs.len());

    if let Some((belief, confidence)) = hivemind.strongest_belief().await {
        println!("  ✓ Strongest belief: {} (confidence: {:.2})", belief, confidence);
    }
}

async fn demo_swarm_behavior() {
    println!("\n┌─ Feature 3: Swarm Behavior Patterns ────────────────────────────┐");

    let mut boids = vec![
        BoidAgent::new("boid-1".to_string(), 10.0, 10.0),
        BoidAgent::new("boid-2".to_string(), 15.0, 15.0),
        BoidAgent::new("boid-3".to_string(), 20.0, 20.0),
        BoidAgent::new("boid-4".to_string(), 25.0, 25.0),
        BoidAgent::new("boid-5".to_string(), 30.0, 30.0),
    ];

    let behavior = FlockingBehavior::new();

    println!("  ✓ Created flock of {} boids", boids.len());

    // Simulate flocking behavior
    for iteration in 0..10 {
        // Create a copy of boids to use as neighbors (to avoid borrow conflicts)
        let boids_copy = boids.clone();

        for boid in &mut boids {
            behavior.apply(boid, &boids_copy);
            boid.update();
            boid.wrap_edges(100.0, 100.0);
        }

        if iteration == 0 || iteration == 9 {
            let avg_x = boids.iter().map(|b| b.position.x).sum::<f64>() / boids.len() as f64;
            let avg_y = boids.iter().map(|b| b.position.y).sum::<f64>() / boids.len() as f64;
            println!("  ✓ Iteration {}: Swarm center at ({:.1}, {:.1})", iteration, avg_x, avg_y);
        }
    }

    println!("  ✓ Flocking complete - swarm self-organized into formation");
}

async fn demo_task_allocation() {
    println!("\n┌─ Feature 4: Distributed Task Allocation ────────────────────────┐");

    let market = TaskMarket::new();

    println!("  ✓ Opening task market");

    // List tasks
    let tasks = vec![
        SwarmTask::new("Process ML batch".to_string(), vec!["ml.inference".to_string()], 9, 500.0),
        SwarmTask::new("Query database".to_string(), vec!["database.query".to_string()], 7, 300.0),
        SwarmTask::new("Compute hash".to_string(), vec!["compute".to_string()], 5, 100.0),
    ];

    for task in tasks {
        let task_id = task.task_id.clone();
        market.list_task(task).await;
        println!("  ✓ Listed task: {} (reward: ${})", task_id, 500.0);
    }

    // Agents bid on tasks
    let open = market.get_open_tasks().await;
    println!("  ✓ {} tasks available in market", open.len());

    for task in open {
        let bid =
            TaskBid::new(task.task_id.clone(), "agent-compute-1".to_string(), 250.0, 120, 0.92, 2);
        market.place_bid(bid).await;
    }

    // Run auctions
    for task_id in ["task-1", "task-2"] {
        if let Some(winner) = market.run_auction(task_id, 10).await {
            println!("  ✓ Task {} assigned to {}", task_id, winner);
        }
    }

    let (open, assigned, completed) = market.auction_metrics().await;
    println!(
        "  ✓ Auction metrics - Open: {}, Assigned: {}, Completed: {}",
        open, assigned, completed
    );
}

async fn demo_emergence() {
    println!("\n┌─ Feature 5: Emergent Behavior & Self-Organization ────────────────┐");

    let mut engine = RuleEngine::new();

    // Add simple behavioral rules
    engine.add_rule(Rule::new(
        "avoid_crowd".to_string(),
        "crowded".to_string(),
        "disperse".to_string(),
    ));
    engine.add_rule(Rule::new("seek_food".to_string(), "hungry".to_string(), "forage".to_string()));
    engine.add_rule(Rule::new(
        "cluster".to_string(),
        "isolated".to_string(),
        "move_to_center".to_string(),
    ));

    println!("  ✓ Added 3 behavioral rules");

    // Simulate agent states triggering rules
    let actions = engine.evaluate("crowded and hungry");
    println!("  ✓ Agent state 'crowded and hungry' triggered {} actions", actions.len());

    // Record successes
    engine.record_success("avoid_crowd");
    engine.record_success("seek_food");

    let best = engine.get_best_rules(2);
    println!("  ✓ Best performing rules: {}", best.len());

    // Self-organization
    let mut organizer = SelfOrganizer::new();
    organizer.assign_role("agent-1".to_string(), "scout".to_string());
    organizer.assign_role("agent-2".to_string(), "forager".to_string());

    let roles = organizer.get_roles();
    println!("  ✓ Self-organized into {} distinct roles", roles.len());
}

async fn demo_swarm_optimization() {
    println!("\n┌─ Feature 6: Swarm Optimization Algorithms ───────────────────────┐");

    // Particle Swarm Optimization
    let mut pso = ParticleSwarmOptimizer::new(20, 3);

    // Simple sphere function (minimize sum of squares)
    let fitness = |x: &[f64]| -x.iter().map(|v| v * v).sum::<f64>();

    println!("  ✓ Running PSO with 20 particles");
    pso.optimize(50, &fitness);

    let pso_solution = pso.best_solution();
    println!("  ✓ PSO found solution with fitness: {:.2}", pso_solution.fitness);

    // Ant Colony Optimization
    let mut aco = AntColonyOptimizer::new(15, 10);

    println!("  ✓ Running ACO with 15 ants on 10-city problem");
    aco.optimize(30);

    let aco_solution = aco.best_solution();
    println!("  ✓ ACO found solution with fitness: {:.2}", aco_solution.fitness);

    // Firefly Algorithm
    let mut fireflies = FireflyAlgorithm::new(25, 2);

    println!("  ✓ Running Firefly Algorithm with 25 fireflies");
    for _ in 0..50 {
        fireflies.update(&fitness);
    }

    let firefly_solution = fireflies.best_solution();
    println!("  ✓ Firefly found solution with fitness: {:.2}", firefly_solution.fitness);

    println!("  ✓ All swarm optimization algorithms converged");
}

async fn demo_resilience() {
    println!("\n┌─ Feature 7: Swarm Resilience & Adaptation ──────────────────────┐");

    let mut resilience = SwarmResilience::new();

    println!("  ✓ Registering 10 agents for role redundancy");
    for i in 1..=10 {
        resilience.register_for_role(format!("agent-{}", i), "scout".to_string());
    }

    let initial_metrics = resilience.metrics();
    println!(
        "  ✓ Initial state: {}/{} agents healthy",
        initial_metrics.healthy_agents, initial_metrics.total_agents
    );

    // Simulate agent failures
    println!("  ✓ Agent 1 fails...");
    resilience.degrade_gracefully("agent-1");

    println!("  ✓ Agent 3 degraded...");
    resilience.set_agent_health("agent-3", HealthStatus::Degraded);

    let updated_metrics = resilience.metrics();
    println!(
        "  ✓ After failures: {}/{} agents healthy",
        updated_metrics.healthy_agents, updated_metrics.total_agents
    );

    if resilience.is_functional() {
        println!(
            "  ✓ Swarm still functional at {:.0}% capacity",
            updated_metrics.functional_capacity * 100.0
        );
    }

    let tolerance = resilience.failure_tolerance();
    println!("  ✓ Swarm can tolerate {} more agent failures", tolerance);

    resilience.adapt_role_assignment("continue_scouting");
    println!("  ✓ Roles dynamically adapted to failures");
}

async fn demo_communication_protocols() {
    println!("\n┌─ Feature 8: Swarm Communication Protocols ───────────────────────┐");

    let mut protocol = SwarmProtocol::new(true);

    // Register agents and neighbors
    println!("  ✓ Registering 6 agents with local topology");
    protocol
        .register_agent("agent-1".to_string(), vec!["agent-2".to_string(), "agent-3".to_string()]);
    protocol
        .register_agent("agent-2".to_string(), vec!["agent-1".to_string(), "agent-4".to_string()]);
    protocol
        .register_agent("agent-3".to_string(), vec!["agent-1".to_string(), "agent-5".to_string()]);
    protocol
        .register_agent("agent-4".to_string(), vec!["agent-2".to_string(), "agent-6".to_string()]);
    protocol.register_agent("agent-5".to_string(), vec!["agent-3".to_string()]);
    protocol.register_agent("agent-6".to_string(), vec!["agent-4".to_string()]);

    // Broadcast local message
    let msg = SwarmMessage::new(
        "agent-1".to_string(),
        "danger detected!".to_string(),
        MessageType::LocalBroadcast,
    );

    let neighbors = protocol.local_broadcast(msg, "agent-1".to_string());
    println!("  ✓ Local broadcast reached {} neighbors", neighbors.len());

    // Gossip for epidemic spread
    let msg2 = SwarmMessage::new(
        "agent-2".to_string(),
        "resource discovered".to_string(),
        MessageType::RegionalGossip,
    );

    protocol.gossip(msg2, "agent-2".to_string());
    println!("  ✓ Regional gossip initiated from agent-2");

    // Test dynamic topology
    protocol.adapt_topology("agent-1".to_string(), "agent-4".to_string(), 0.85);
    println!("  ✓ Dynamic topology adapted (added high-quality link)");

    // Communication overhead
    let overhead = protocol.communication_overhead();
    println!("  ✓ Communication overhead: {:.0} bytes/agent/sec", overhead);

    // Protocol negotiation
    let agreed = protocol.negotiate_protocol("agent-1", "agent-2");
    println!("  ✓ Agents agreed on protocol: {}", agreed);
}
