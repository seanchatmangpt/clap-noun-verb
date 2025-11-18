/// CNV 2029-2030+ Swarm Intelligence Systems
///
/// AI Agent Swarm Interaction - Emergent collective intelligence through
/// bio-inspired algorithms, stigmergic communication, and self-organization.

pub mod stigmergy;
pub mod collective_intelligence;
pub mod swarm_behavior;
pub mod task_allocation;
pub mod emergence;
pub mod optimization;
pub mod resilience;
pub mod communication;

pub use stigmergy::{PheromoneField, StigmergicProtocol};
pub use collective_intelligence::{VotingProtocol, HiveMind, ConsensusType as SwarmConsensusType};
pub use swarm_behavior::{BoidAgent, FlockingBehavior, HerdingBehavior, SwarmingBehavior, FormationController};
pub use task_allocation::{TaskMarket, SwarmTask, TaskBid};
pub use emergence::{RuleEngine, SelfOrganizer, Rule};
pub use optimization::{ParticleSwarmOptimizer, AntColonyOptimizer, FireflyAlgorithm};
pub use resilience::{SwarmResilience, HealthStatus};
pub use communication::{SwarmProtocol, SwarmMessage, MessageType};
