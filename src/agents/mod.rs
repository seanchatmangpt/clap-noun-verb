//! Semantic Agent Coordinator
//!
//! This module provides advanced agent coordination capabilities including:
//! - Type-state machines for compile-time safety
//! - Semantic discovery via RDF/SPARQL
//! - Byzantine fault-tolerant swarm coordination
//! - MAPE-K autonomic loops for self-healing
//!
//! # Feature Flags
//!
//! - `agent2028` - Core agent capabilities
//! - `rdf` - Semantic discovery with RDF/SPARQL
//! - `autonomic` - MAPE-K autonomic loops
//! - `async` - Async coordination primitives

pub mod state;

#[cfg(feature = "rdf")]
pub mod semantic;

pub mod swarm;

#[cfg(feature = "autonomic")]
pub mod autonomic;

// Re-exports for convenience
pub use state::{AgentState, Escalated, Registered, Trusted, Unregistered, Verified};

#[cfg(feature = "rdf")]
pub use semantic::{Capability, SemanticDiscovery, SparqlQueryBuilder};

pub use swarm::{
    AgentInfo, AgentRegistry, ByzantineDetector, GossipProtocol, SwarmCoordinator, TaskAuction,
    TrustScore,
};

#[cfg(feature = "autonomic")]
pub use autonomic::{
    AdaptiveParameter, AnomalyDetector, AutonomicLoop, MapekPhase, SelfHealingAction,
};
