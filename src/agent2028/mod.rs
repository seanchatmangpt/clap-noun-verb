pub mod audit_ledger;
/// CNV 2028 Innovations: Trillion-Agent Ecosystems
///
/// This module contains cutting-edge features for distributed multi-agent coordination,
/// learning systems, quantum-safe cryptography, trust networks, and self-healing autonomic systems.
pub mod coordination;
pub mod learning;
pub mod marketplace;
pub mod prediction;
pub mod quantum_crypto;
pub mod self_healing;
pub mod trust_network;

// 2029-2030+ Innovations: AI Agent Swarm Ecosystems
pub mod swarm;

// Orchestration & Integration Layer
pub mod event_bus;
pub mod orchestration;

// Thesis Framework (Hyper-Thesis Framework Integration)
pub mod thesis_framework;

pub use audit_ledger::DistributedAuditLedger;
pub use coordination::{AgentRegistry, CommandBroker, ConsensusEngine};
pub use event_bus::{Event, EventBus, EventHandlerRegistry, EventType};
pub use learning::{AdaptationEngine, ExecutionProfiler, ModelInference};
pub use marketplace::CapabilityMarket;
pub use orchestration::{
    AgentTier, IntegrationBridge, OperationRequest, OperationResult, Orchestrator,
};
pub use prediction::{CapacityPlanner, WorkloadForecaster};
pub use quantum_crypto::QuantumSafeAttestation;
pub use self_healing::Autonomic;
pub use swarm::{
    AntColonyOptimizer, BoidAgent, FireflyAlgorithm, FlockingBehavior, FormationController,
    HerdingBehavior, HiveMind, ParticleSwarmOptimizer, PheromoneField, RuleEngine,
    StigmergicProtocol, SwarmProtocol, SwarmResilience, SwarmingBehavior, TaskMarket,
    VotingProtocol,
};
pub use thesis_framework::{
    CheckReport, GammaChecker, LambdaSchedule, PiProfile, Shard, ShardFamily, ShardStatus,
};
pub use trust_network::{AgentIdentity, TrustScoreCalculator};
