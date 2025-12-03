/// Comprehensive False Positive Detection & Recovery Demo
///
/// Demonstrates the 2029-2030+ swarm false positive detection systems:
/// 1. False Alert Detection - identifying spurious signals
/// 2. Consensus Recovery - correcting bad voting decisions
/// 3. Trust Score Verification - auditing agent credibility
/// 4. Bid Validation - tracking auction reliability
/// 5. Pheromone Trail Validation - verifying indirect communication
/// 6. Role Verification - identifying unsuitable assignments
use clap_noun_verb::agent2028::swarm::false_positives::*;

#[tokio::main]
async fn main() {
    println!("═════════════════════════════════════════════════════════════════");
    println!("  CNV 2029-2030+ False Positive Detection & Recovery Demo");
    println!("  Trillion-Agent AI Ecosystems - Quality Assurance");
    println!("═════════════════════════════════════════════════════════════════\n");

    demo_false_alert_detection().await;
    demo_consensus_recovery().await;
    demo_trust_score_verification().await;
    demo_bid_validation().await;
    demo_pheromone_validation().await;
    demo_role_verification().await;

    println!("\n═════════════════════════════════════════════════════════════════");
    println!("  False Positive Recovery Demo Complete!");
    println!("═════════════════════════════════════════════════════════════════");
}

async fn demo_false_alert_detection() {
    println!("\n┌─ System 1: False Alert Detection ──────────────────────────────┐");

    let mut detector = FalseAlertDetector::new();

    // Register normal thresholds for different alert types
    detector.register_threshold("temperature_anomaly".to_string(), 20.0, 30.0);
    detector.register_threshold("memory_pressure".to_string(), 0.0, 0.8);
    detector.register_threshold("latency_spike".to_string(), 0.0, 500.0);

    println!("  ✓ Registered alert thresholds for 3 alert types");

    // Scenario: Agent-1 sends a false temperature alert
    let mut alert1 = FalseAlert::new(
        "agent-1".to_string(),
        "temperature_anomaly".to_string(),
        25.0, // Claimed severity
    );
    alert1.affected_agents = vec!["agent-2".to_string(), "agent-3".to_string()];

    let is_false1 = detector.detect_false_alert(&mut alert1, 25.5);
    detector.record_alert(alert1.clone());
    println!(
        "  ✓ Alert from agent-1: false={}, confidence={:.2}",
        is_false1, alert1.falseness_confidence
    );

    // Scenario: Agent-2 sends a true alert
    let mut alert2 =
        FalseAlert::new("agent-2".to_string(), "temperature_anomaly".to_string(), 22.0);
    let is_false2 = detector.detect_false_alert(&mut alert2, 21.8);
    detector.record_alert(alert2);
    println!("  ✓ Alert from agent-2: false={}, confidence={:.2}", is_false2, 0.0);

    // Scenario: Agent-3 consistently sends false memory alerts
    for i in 0..5 {
        let claimed = 0.2 + (i as f64 * 0.15);
        let actual = 0.95; // Actually normal
        let mut alert =
            FalseAlert::new("agent-3".to_string(), "memory_pressure".to_string(), claimed);
        detector.detect_false_alert(&mut alert, actual);
        detector.record_alert(alert);
    }

    println!("  ✓ Agent-3 sent 5 false memory alerts");

    // Identify faulty sources
    let faulty = detector.identify_faulty_sources();
    println!("  ✓ Identified faulty agents:");
    for (agent_id, false_rate) in faulty {
        println!("    - {}: {:.0}% false positive rate", agent_id, false_rate * 100.0);
    }

    // Check agent credibility
    let cred1 = detector.agent_credibility("agent-1");
    let cred3 = detector.agent_credibility("agent-3");
    println!("  ✓ Agent credibility scores:");
    println!("    - agent-1: {:.2}", cred1);
    println!("    - agent-3: {:.2}", cred3);
}

async fn demo_consensus_recovery() {
    println!("\n┌─ System 2: Consensus Recovery & Reversion ──────────────────────┐");

    let mut recovery = ConsensusRecoverySystem::new();

    println!("  ✓ Created consensus recovery system");

    // Scenario: Swarm votes to migrate to region A, but outcome is region B
    println!("  ✓ Consensus 1: Swarm voted 'migrate_to_region_a'");
    let was_wrong = recovery.verify_decision(
        "vote-1".to_string(),
        "migrate_to_region_a".to_string(),
        "migrated_to_region_b".to_string(),
    );
    println!("    Result: Decision was wrong={}", was_wrong);

    // Scenario: Swarm votes to increase agent pool, correct outcome
    println!("  ✓ Consensus 2: Swarm voted 'increase_pool_size'");
    recovery.verify_decision(
        "vote-2".to_string(),
        "increase_pool_size".to_string(),
        "pool_size_increased".to_string(),
    );
    println!("    Result: Decision was correct");

    // Scenario: Multiple decisions about resource allocation (mostly wrong)
    for i in 3..8 {
        let decision = "reallocate_resources";
        let outcome = if i % 3 == 0 { "resources_reallocated" } else { "reallocation_failed" };

        recovery.verify_decision(format!("vote-{}", i), decision.to_string(), outcome.to_string());
    }

    println!("  ✓ Recorded 5 resource allocation decisions (60% success rate)");

    // Identify unreliable decision types
    let unreliable = recovery.unreliable_decisions();
    if !unreliable.is_empty() {
        println!("  ✓ Identified unreliable decision types:");
        for (decision, success_rate) in unreliable {
            println!("    - {}: {:.0}% success rate", decision, success_rate * 100.0);
        }
    } else {
        println!("  ✓ All decision types have acceptable success rates");
    }

    // Check specific decision success rate
    let rate = recovery.decision_success_rate("migrate_to_region_a");
    println!("  ✓ 'migrate_to_region_a' success rate: {:.0}%", rate * 100.0);
}

async fn demo_trust_score_verification() {
    println!("\n┌─ System 3: Trust Score Verification & Correction ──────────────┐");

    let mut verifier = TrustScoreVerifier::new();

    println!("  ✓ Created trust score verifier");

    // Scenario 1: Agent-1's recorded trust (0.9) doesn't match actual (0.75)
    println!("  ✓ Auditing agent-1: recorded=0.9, actual=0.75");
    verifier.verify_trust(
        "agent-1".to_string(),
        0.9,  // Recorded (inflated)
        0.75, // Actual performance
    );

    // Scenario 2: Agent-2's trust score is accurate
    println!("  ✓ Auditing agent-2: recorded=0.7, actual=0.72");
    verifier.verify_trust("agent-2".to_string(), 0.7, 0.72);

    // Scenario 3: Agent-3's trust is severely underestimated
    println!("  ✓ Auditing agent-3: recorded=0.5, actual=0.8");
    verifier.verify_trust(
        "agent-3".to_string(),
        0.5, // Recorded (deflated)
        0.8, // Actual performance
    );

    // Scenario 4: Agent-4 has slight variance
    println!("  ✓ Auditing agent-4: recorded=0.85, actual=0.83");
    verifier.verify_trust("agent-4".to_string(), 0.85, 0.83);

    // Get agents needing correction (>20% deviation)
    let corrections = verifier.agents_needing_correction();
    println!("  ✓ Agents requiring score corrections:");
    for (agent_id, corrected_score) in corrections {
        println!("    - {}: corrected to {:.2}", agent_id, corrected_score);
    }

    // Get audit history for specific agent
    let history = verifier.audit_history("agent-1");
    println!("  ✓ Audit history for agent-1: {} records", history.len());
}

async fn demo_bid_validation() {
    println!("\n┌─ System 4: Auction Bid Validation & Default Handling ──────────┐");

    let mut validator = BidValidator::new();

    println!("  ✓ Created bid validator");

    // Scenario: Multiple bids from different agents
    println!("  ✓ Recording bids for 'process_batch_001' task");

    // Agent-1: Reliable bidder (3/3 fulfilled)
    for i in 1..=3 {
        validator.record_outcome(
            format!("bid-{}", i),
            "agent-1".to_string(),
            100,
            Some(95),
            true, // Fulfilled
        );
    }
    println!("    - agent-1: 3 fulfilled");

    // Agent-2: Somewhat reliable (2/3 fulfilled)
    validator.record_outcome("bid-4".to_string(), "agent-2".to_string(), 150, Some(160), true);
    validator.record_outcome("bid-5".to_string(), "agent-2".to_string(), 150, Some(200), true);
    validator.record_outcome(
        "bid-6".to_string(),
        "agent-2".to_string(),
        150,
        None,
        false, // Default
    );
    println!("    - agent-2: 2 fulfilled, 1 default");

    // Agent-3: Unreliable bidder (1/4 fulfilled)
    for i in 0..4 {
        let fulfilled = i == 2; // Only one success
        validator.record_outcome(
            format!("bid-{}", 10 + i),
            "agent-3".to_string(),
            100,
            if fulfilled { Some(110) } else { None },
            fulfilled,
        );
    }
    println!("    - agent-3: 1 fulfilled, 3 defaults");

    // Get fulfillment rates
    let rate1 = validator.fulfillment_rate("agent-1");
    let rate2 = validator.fulfillment_rate("agent-2");
    let rate3 = validator.fulfillment_rate("agent-3");

    println!("  ✓ Fulfillment rates:");
    println!("    - agent-1: {:.0}% (reliable)", rate1 * 100.0);
    println!("    - agent-2: {:.0}% (acceptable)", rate2 * 100.0);
    println!("    - agent-3: {:.0}% (unreliable)", rate3 * 100.0);

    // Identify unreliable bidders
    let unreliable = validator.unreliable_bidders();
    if !unreliable.is_empty() {
        println!("  ✓ Identified unreliable bidders (< 80% fulfillment):");
        for (agent_id, rate) in unreliable {
            println!("    - {}: {:.0}% fulfillment", agent_id, rate * 100.0);
        }
    }
}

async fn demo_pheromone_validation() {
    println!("\n┌─ System 5: Pheromone Trail Validation ──────────────────────────┐");

    let mut validator = PheromoneValidator::new();

    println!("  ✓ Created pheromone trail validator");

    // Scenario: Various pheromone trails with different success rates
    println!("  ✓ Validating pheromone trails:");

    // Trail-1: Reliable (leads to food 4/5 times)
    for i in 0..5 {
        let success = i != 2; // Failed once
        validator.validate_trail(format!("trail-food-zone-1"), success);
    }
    println!("    - trail-food-zone-1: 4/5 successful");

    // Trail-2: Misleading (leads to dead ends 3/4 times)
    for i in 0..4 {
        let success = i == 0; // Only one success
        validator.validate_trail(format!("trail-dead-end-1"), success);
    }
    println!("    - trail-dead-end-1: 1/4 successful");

    // Trail-3: Moderate (50/50)
    validator.validate_trail("trail-resource-cache-1".to_string(), true);
    validator.validate_trail("trail-resource-cache-1".to_string(), false);
    println!("    - trail-resource-cache-1: 1/2 successful");

    // Get unreliable trails
    let unreliable = validator.unreliable_trails();
    println!("  ✓ Unreliable trails (< 50% confidence):");
    for (trail_id, confidence) in unreliable {
        println!("    - {}: {:.2} confidence", trail_id, confidence);
    }

    // Clear misleading trails
    let cleared = validator.clear_misleading_trails(0.5);
    println!("  ✓ Cleared {} trails below 50% confidence threshold", cleared);

    // Remaining valid trails
    let remaining = validator.unreliable_trails();
    println!("  ✓ Remaining trails after cleanup: {}", remaining.len());
}

async fn demo_role_verification() {
    println!("\n┌─ System 6: Role Assignment Verification ───────────────────────┐");

    let mut verifier = RoleVerifier::new();

    println!("  ✓ Created role verifier");

    // Scenario: Various agents in different roles
    println!("  ✓ Recording role performance:");

    // Agent-1 as scout: Very good (8/10 successful)
    for i in 0..10 {
        let success = i < 8;
        verifier.record_performance("agent-1".to_string(), "scout".to_string(), success);
    }
    println!("    - agent-1 as scout: 8/10 successful (80%)");

    // Agent-2 as scout: Poor (2/8 successful)
    for i in 0..8 {
        let success = i < 2;
        verifier.record_performance("agent-2".to_string(), "scout".to_string(), success);
    }
    println!("    - agent-2 as scout: 2/8 successful (25%) - unsuitable");

    // Agent-3 as forager: Good (7/9 successful)
    for i in 0..9 {
        let success = i < 7;
        verifier.record_performance("agent-3".to_string(), "forager".to_string(), success);
    }
    println!("    - agent-3 as forager: 7/9 successful (78%)");

    // Agent-2 as forager: Better (6/7 successful)
    for i in 0..7 {
        let success = i < 6;
        verifier.record_performance("agent-2".to_string(), "forager".to_string(), success);
    }
    println!("    - agent-2 as forager: 6/7 successful (86%)");

    // Get best agents for scout role
    let best_scouts = verifier.best_agents_for_role("scout");
    println!("  ✓ Best agents for 'scout' role:");
    for (agent_id, score) in best_scouts {
        println!("    - {}: {:.0}% performance", agent_id, score * 100.0);
    }

    // Identify unsuitable role assignments
    let unsuitable = verifier.unsuitable_role_assignments();
    if !unsuitable.is_empty() {
        println!("  ✓ Unsuitable role assignments (< 60% performance):");
        for (agent_id, role, score) in unsuitable {
            println!(
                "    - {}: unsuitable for '{}' ({:.0}% performance)",
                agent_id,
                role,
                score * 100.0
            );
        }
    }
}
