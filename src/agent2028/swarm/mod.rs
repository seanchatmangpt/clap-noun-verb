pub mod collective_intelligence;
pub mod communication;
pub mod emergence;
pub mod false_positives;
pub mod optimization;
pub mod resilience;
/// CNV 2029-2030+ Swarm Intelligence Systems
///
/// AI Agent Swarm Interaction - Emergent collective intelligence through
/// bio-inspired algorithms, stigmergic communication, and self-organization.
pub mod stigmergy;
pub mod swarm_behavior;
pub mod task_allocation;

pub use collective_intelligence::{ConsensusType as SwarmConsensusType, HiveMind, VotingProtocol};
pub use communication::{MessageType, SwarmMessage, SwarmProtocol};
pub use emergence::{Rule, RuleEngine, SelfOrganizer};
pub use false_positives::{
    AlertSeverity, BidValidation, BidValidator, ConsensusRecoverySystem, ConsensusVerification,
    FalseAlert, FalseAlertDetector, PheromoneValidator, RoleVerification, RoleVerifier,
    TrustScoreAudit, TrustScoreVerifier,
};
pub use optimization::{AntColonyOptimizer, FireflyAlgorithm, ParticleSwarmOptimizer};
pub use resilience::{HealthStatus, SwarmResilience};
pub use stigmergy::{PheromoneField, StigmergicProtocol};
pub use swarm_behavior::{
    BoidAgent, FlockingBehavior, FormationController, HerdingBehavior, SwarmingBehavior,
};
pub use task_allocation::{SwarmTask, TaskBid, TaskMarket};
