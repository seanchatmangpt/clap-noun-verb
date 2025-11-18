/// CNV 2028 Innovations: Trillion-Agent Ecosystems
///
/// This module contains cutting-edge features for distributed multi-agent coordination,
/// learning systems, quantum-safe cryptography, trust networks, and self-healing autonomic systems.

pub mod coordination;
pub mod learning;
pub mod quantum_crypto;
pub mod trust_network;
pub mod marketplace;
pub mod self_healing;
pub mod audit_ledger;
pub mod prediction;

// 2029-2030+ Innovations: AI Agent Swarm Ecosystems
pub mod swarm;

// Orchestration & Integration Layer
pub mod orchestration;
pub mod event_bus;

// Thesis Framework (Hyper-Thesis Framework Integration)
pub mod thesis_framework;

pub use coordination::{AgentRegistry, CommandBroker, ConsensusEngine};
pub use learning::{ExecutionProfiler, ModelInference, AdaptationEngine};
pub use quantum_crypto::QuantumSafeAttestation;
pub use trust_network::{AgentIdentity, TrustScoreCalculator};
pub use marketplace::CapabilityMarket;
pub use self_healing::Autonomic;
pub use audit_ledger::DistributedAuditLedger;
pub use prediction::{WorkloadForecaster, CapacityPlanner};
pub use swarm::{
    PheromoneField, StigmergicProtocol, VotingProtocol, HiveMind, BoidAgent, FlockingBehavior,
    HerdingBehavior, SwarmingBehavior, FormationController, TaskMarket, RuleEngine,
    ParticleSwarmOptimizer, AntColonyOptimizer, FireflyAlgorithm, SwarmResilience, SwarmProtocol,
};
pub use orchestration::{Orchestrator, OperationRequest, OperationResult, IntegrationBridge, AgentTier};
pub use event_bus::{EventBus, Event, EventType, EventHandlerRegistry};
pub use thesis_framework::{
    Shard, ShardFamily, ShardStatus, LambdaSchedule, PiProfile, GammaChecker, CheckReport,
};
