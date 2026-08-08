//! Behavioral integration proofs for the bounded frontier capability crown.

#[cfg(feature = "meta-framework")]
#[test]
fn meta_framework_requires_invariants_and_receipt_for_alive() {
    use clap_noun_verb::frontier::{AdmissionState, Invariant, MetaFramework};

    let mut framework = MetaFramework::new();
    framework.register_layer("admission").expect("unique layer");
    framework
        .admit_invariant(Invariant {
            id: "zero-unreceipted-actuation".to_string(),
            description: "Every effect has a receipt".to_string(),
            satisfied: true,
        })
        .expect("valid invariant");

    assert!(framework.validate_invariants());
    assert_eq!(framework.state(false), AdmissionState::Admitted);
    assert_eq!(framework.state(true), AdmissionState::Alive);
}

#[cfg(feature = "rdf-composition")]
#[test]
fn rdf_composition_is_deterministic_and_duplicate_free() {
    use clap_noun_verb::frontier::{RdfFragment, SemanticTriple};

    let triple = SemanticTriple {
        subject: "cnv:tool".to_string(),
        predicate: "cnv:defaultVerb".to_string(),
        object: "cnv:run".to_string(),
    };
    let mut left = RdfFragment::new();
    assert!(left.insert(triple.clone()).expect("valid triple"));
    assert!(!left.insert(triple.clone()).expect("duplicate is bounded"));

    let mut right = RdfFragment::new();
    right
        .insert(SemanticTriple {
            subject: "cnv:tool".to_string(),
            predicate: "rdf:type".to_string(),
            object: "cnv:Noun".to_string(),
        })
        .expect("valid triple");

    let composed = left.compose(&right);
    assert_eq!(composed.triples().len(), 2);
    assert_eq!(composed, right.compose(&left));
}

#[cfg(feature = "discovery-engine")]
#[test]
fn discovery_engine_resolves_exact_names_and_tags() {
    use clap_noun_verb::frontier::{DiscoveryEngine, DiscoveryRecord};
    use std::collections::BTreeSet;

    let mut engine = DiscoveryEngine::default();
    engine
        .register(DiscoveryRecord {
            name: "receipt-verify".to_string(),
            tags: BTreeSet::from(["receipt".to_string(), "replay".to_string()]),
            route: "receipt verify".to_string(),
        })
        .expect("unique capability");

    assert_eq!(engine.search("receipt-verify").len(), 1);
    assert_eq!(engine.search("replay").len(), 1);
    assert!(engine.search("unknown").is_empty());
}

#[cfg(feature = "learning-trajectories")]
#[test]
fn learning_trajectory_observes_monotonic_bounded_scores() {
    use clap_noun_verb::frontier::LearningTrajectory;

    let mut trajectory = LearningTrajectory::default();
    assert_eq!(trajectory.observe(0.25).expect("bounded score"), 0);
    assert_eq!(trajectory.observe(0.75).expect("bounded score"), 1);
    assert_eq!(trajectory.latest(), Some(0.75));
    assert!(trajectory.is_monotonic());
    assert!(trajectory.observe(1.1).is_err());
}

#[cfg(feature = "reflexive-testing")]
#[test]
fn reflexive_report_refuses_success_without_replay() {
    use clap_noun_verb::frontier::ReflexiveReport;

    assert!(!ReflexiveReport { passed: 45, failed: 0, replay_verified: false }.is_alive());
    assert!(ReflexiveReport { passed: 45, failed: 0, replay_verified: true }.is_alive());
}

#[cfg(feature = "quantum-ready")]
#[test]
fn quantum_policy_admits_only_declared_pqc_families() {
    use clap_noun_verb::frontier::{QuantumAlgorithm, QuantumReadyPolicy};

    let policy = QuantumReadyPolicy::post_quantum();
    assert!(policy.admits(QuantumAlgorithm::MlKem));
    assert!(policy.admits(QuantumAlgorithm::MlDsa));
    assert!(policy.admits(QuantumAlgorithm::SlhDsa));
}

#[cfg(feature = "economic-sim")]
#[test]
fn economic_simulation_allocates_to_the_highest_trust_capable_agent() {
    use clap_noun_verb::frontier::{Agent, AgentId, EconomicSimulation, Task, TaskId};

    let mut simulation = EconomicSimulation::new();
    for (id, trust) in [(1, 0.6), (2, 0.9)] {
        simulation
            .add_agent(Agent {
                id: AgentId(id),
                capabilities: vec!["compile".to_string()],
                trust_score: trust,
                valuation: 100.0,
            })
            .expect("valid agent");
    }
    simulation
        .add_task(Task { id: TaskId(7), required_capability: "compile".to_string(), value: 120.0 })
        .expect("valid task");

    let allocations = simulation.step().expect("bounded allocation");
    assert_eq!(allocations.len(), 1);
    assert_eq!(allocations[0].agent_id, AgentId(2));
    assert_eq!(simulation.allocations(), allocations.as_slice());
}

#[cfg(all(feature = "federated-network", feature = "async"))]
#[tokio::test]
async fn federated_capability_is_advertised_and_resolved() {
    use clap_noun_verb::frontier::{Capability, FederatedNetwork};

    let mut network = FederatedNetwork::new("crown-node").expect("valid node");
    network
        .advertise_capability(&Capability {
            name: "receipt-verify".to_string(),
            version: "26.7.62".to_string(),
            endpoint: "local://receipt-verify".to_string(),
        })
        .await
        .expect("valid capability");

    let resolved = network.resolve("receipt-verify").expect("advertised capability");
    assert_eq!(resolved.endpoint, "local://receipt-verify");
}
