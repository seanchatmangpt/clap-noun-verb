//! Frontier - Advanced Agent-Grade Packages
//!
//! This module provides 10 frontier packages for building advanced agent-grade CLIs.
//! All features are optional and can be enabled individually or via meta-features.
//!
//! ## Feature Architecture (Three Tiers)
//!
//! ### Tier 1: Meta-features (Convenience Bundles)
//! - `frontier-all` - All 10 frontier features
//! - `frontier-semantic` - Semantic and network features
//! - `frontier-intelligence` - AI/ML and discovery features
//! - `frontier-quality` - Testing and verification features
//!
//! ### Tier 2: Individual Features (10 Frontier Packages)
//! 1. `meta-framework` - Self-modifying agent frameworks
//! 2. `rdf-composition` - Semantic ontology composition
//! 3. `executable-specs` - BDD specifications
//! 4. `fractal-patterns` - Self-similar command hierarchies
//! 5. `discovery-engine` - Dynamic capability discovery
//! 6. `federated-network` - Multi-host agent coordination
//! 7. `learning-trajectories` - ReasoningBank learning
//! 8. `reflexive-testing` - Self-testing systems
//! 9. `economic-sim` - Agent economy simulations
//! 10. `quantum-ready` - Post-quantum cryptography
//!
//! ### Tier 3: Shared Infrastructure
//! Existing features like `async`, `crypto`, `rdf`, `agent2028`, etc.
//!
//! ## Usage
//!
//! Enable features in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! clap-noun-verb = { version = "5.4", features = ["frontier-all"] }
//! # Or individual features:
//! clap-noun-verb = { version = "5.4", features = ["meta-framework", "discovery-engine"] }
//! ```

// =============================================================================
// FEATURE-GATED MODULE EXPORTS
// =============================================================================

// Error module - used by frontier features
pub mod error;

#[cfg(feature = "meta-framework")]
pub mod meta_framework;

#[cfg(feature = "rdf-composition")]
pub mod rdf_composition;

#[cfg(feature = "executable-specs")]
pub mod executable_specs;

#[cfg(feature = "fractal-patterns")]
pub mod fractal_patterns;

#[cfg(feature = "discovery-engine")]
pub mod discovery_engine;

#[cfg(feature = "federated-network")]
pub mod federated_network;

#[cfg(feature = "learning-trajectories")]
pub mod learning_trajectories;

#[cfg(feature = "reflexive-testing")]
pub mod reflexive_testing;

#[cfg(feature = "economic-sim")]
pub mod economic_sim;

#[cfg(feature = "quantum-ready")]
pub mod quantum_ready;
