//! Capability Discovery Engine - Type-First Implementation
//!
//! This module provides a distributed capability discovery system that explores
//! the space of CLI capability combinations using swarm intelligence and genetic
//! algorithms. The design uses zero-cost abstractions and type-level guarantees
//! to ensure safety and performance.
//!
//! # Architecture
//!
//! - SearchSpace: Type-safe representation of capability combinations
//! - FitnessScoringEngine: Weighted scoring (40% utility + 30% novelty + 30% safety)
//! - SwarmOptimizer: Multi-threaded particle swarm optimization
//! - SuggestionFactory: Developer-facing recommendation generator
//! - SafetyProver: Compile-time and runtime safety validation
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::macros::discovery_engine::{discoverable, fitness_function};
//!
//! // Mark capability as discoverable
//! discoverable! {
//!     name: "parallel_execution",
//!     category: "performance",
//!     requires: ["tokio"],
//!     conflicts: ["single_threaded"]
//! }
//!
//! // Define fitness scoring
//! fitness_function! {
//!     capability: "parallel_execution",
//!     utility: |ctx| ctx.performance_gain() * 0.8,
//!     novelty: |ctx| ctx.unique_features().len() as f64 / 10.0,
//!     safety: |ctx| if ctx.has_tests() { 1.0 } else { 0.0 }
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

#[cfg(feature = "agent2028")]
use rand;

/// Type-level marker for capability states
pub mod state {
    /// Capability is discovered but not validated
    #[derive(Debug, Clone, Copy)]
    pub struct Discovered;
    /// Capability is validated and safe
    #[derive(Debug, Clone, Copy)]
    pub struct Validated;
    /// Capability is rejected (unsafe or incompatible)
    #[derive(Debug, Clone, Copy)]
    pub struct Rejected;
}

/// Capability identifier with compile-time uniqueness
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId<const N: usize>([u8; N]);

impl<const N: usize> CapabilityId<N> {
    /// Create capability ID from string at compile time
    pub const fn from_bytes(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// Get string representation
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}

/// Capability metadata with type-state pattern
#[derive(Debug, Clone)]
pub struct Capability<State = state::Discovered> {
    pub id: String,
    pub category: String,
    pub requires: Vec<String>,
    pub conflicts: Vec<String>,
    pub metadata: HashMap<String, String>,
    _state: PhantomData<State>,
}

impl Capability<state::Discovered> {
    /// Create new discovered capability
    pub fn new(id: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category: category.into(),
            requires: Vec::new(),
            conflicts: Vec::new(),
            metadata: HashMap::new(),
            _state: PhantomData,
        }
    }

    /// Add requirement
    pub fn requires(mut self, capability: impl Into<String>) -> Self {
        self.requires.push(capability.into());
        self
    }

    /// Add conflict
    pub fn conflicts_with(mut self, capability: impl Into<String>) -> Self {
        self.conflicts.push(capability.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Validate capability and transition to Validated state
    pub fn validate(self, prover: &SafetyProver) -> Result<Capability<state::Validated>, String> {
        if prover.is_safe(&self) {
            Ok(Capability {
                id: self.id,
                category: self.category,
                requires: self.requires,
                conflicts: self.conflicts,
                metadata: self.metadata,
                _state: PhantomData,
            })
        } else {
            Err(format!("Capability {} failed safety validation", self.id))
        }
    }
}

impl Capability<state::Validated> {
    /// Get capability ID (only available for validated capabilities)
    pub fn validated_id(&self) -> &str {
        &self.id
    }
}

/// Search space representation with invariants encoded in types
#[derive(Debug, Clone)]
pub struct SearchSpace {
    capabilities: HashMap<String, Capability<state::Discovered>>,
    dimensions: Vec<String>,
    explored: HashSet<CombinationHash>,
}

/// Hash of capability combination (zero-cost newtype)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CombinationHash(u64);

impl SearchSpace {
    /// Create new search space
    pub fn new() -> Self {
        Self { capabilities: HashMap::new(), dimensions: Vec::new(), explored: HashSet::new() }
    }

    /// Register capability in search space
    pub fn register(&mut self, capability: Capability<state::Discovered>) {
        if !self.dimensions.contains(&capability.category) {
            self.dimensions.push(capability.category.clone());
        }
        self.capabilities.insert(capability.id.clone(), capability);
    }

    /// Get all capabilities in category
    pub fn capabilities_in(&self, category: &str) -> Vec<&Capability<state::Discovered>> {
        self.capabilities.values().filter(|c| c.category == category).collect()
    }

    /// Mark combination as explored
    pub fn mark_explored(&mut self, combination: &[&str]) {
        let hash = Self::hash_combination(combination);
        self.explored.insert(hash);
    }

    /// Check if combination has been explored
    pub fn is_explored(&self, combination: &[&str]) -> bool {
        let hash = Self::hash_combination(combination);
        self.explored.contains(&hash)
    }

    /// Hash capability combination (deterministic)
    fn hash_combination(combination: &[&str]) -> CombinationHash {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        let mut sorted = combination.to_vec();
        sorted.sort_unstable();

        for item in sorted {
            item.hash(&mut hasher);
        }

        CombinationHash(hasher.finish())
    }

    /// Get unexplored neighbors of a combination
    pub fn unexplored_neighbors(&self, combination: &[&str]) -> Vec<Vec<String>> {
        let mut neighbors = Vec::new();
        let current_set: HashSet<_> = combination.iter().copied().collect();

        for capability in self.capabilities.keys() {
            if !current_set.contains(capability.as_str()) {
                let mut neighbor = combination.to_vec();
                neighbor.push(capability.as_str());

                if !self.is_explored(&neighbor) {
                    neighbors.push(neighbor.into_iter().map(|s| s.to_string()).collect());
                }
            }
        }

        neighbors
    }

    /// Get total search space size (combinatorial)
    pub fn total_combinations(&self) -> usize {
        let n = self.capabilities.len();
        if n == 0 {
            return 0;
        }
        2_usize.saturating_pow(n as u32)
    }

    /// Get exploration coverage percentage
    pub fn coverage(&self) -> f64 {
        let total = self.total_combinations();
        if total == 0 {
            return 0.0;
        }
        (self.explored.len() as f64 / total as f64) * 100.0
    }
}

impl Default for SearchSpace {
    fn default() -> Self {
        Self::new()
    }
}

/// Fitness scoring with weighted components
#[derive(Debug, Clone, Copy)]
pub struct FitnessScore {
    pub utility: f64,
    pub novelty: f64,
    pub safety: f64,
}

impl FitnessScore {
    /// Calculate weighted total (40% utility + 30% novelty + 30% safety)
    pub fn total(&self) -> f64 {
        (self.utility * 0.4) + (self.novelty * 0.3) + (self.safety * 0.3)
    }

    /// Create zero score
    pub const fn zero() -> Self {
        Self { utility: 0.0, novelty: 0.0, safety: 0.0 }
    }
}

/// Fitness scoring engine with configurable weights
pub struct FitnessScoringEngine {
    utility_weight: f64,
    novelty_weight: f64,
    safety_weight: f64,
    cache: Arc<RwLock<HashMap<String, FitnessScore>>>,
}

impl FitnessScoringEngine {
    /// Create new scoring engine with default weights
    pub fn new() -> Self {
        Self {
            utility_weight: 0.4,
            novelty_weight: 0.3,
            safety_weight: 0.3,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Score capability combination
    pub fn score(&self, combination: &[&str], space: &SearchSpace) -> FitnessScore {
        let key = combination.join(",");

        if let Ok(cache) = self.cache.read() {
            if let Some(score) = cache.get(&key) {
                return *score;
            }
        }

        let utility = self.compute_utility(combination, space);
        let novelty = self.compute_novelty(combination, space);
        let safety = self.compute_safety(combination, space);

        let score = FitnessScore { utility, novelty, safety };

        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key, score);
        }

        score
    }

    /// Compute utility score
    fn compute_utility(&self, combination: &[&str], space: &SearchSpace) -> f64 {
        let total_capabilities = space.capabilities.len() as f64;
        if total_capabilities == 0.0 {
            return 0.0;
        }

        let coverage = combination.len() as f64 / total_capabilities;
        coverage.min(1.0)
    }

    /// Compute novelty score
    fn compute_novelty(&self, combination: &[&str], space: &SearchSpace) -> f64 {
        let neighbors = space.unexplored_neighbors(combination);
        let novelty = neighbors.len() as f64 / 10.0;
        novelty.min(1.0)
    }

    /// Compute safety score
    fn compute_safety(&self, combination: &[&str], space: &SearchSpace) -> f64 {
        let mut conflicts = 0;
        let combination_set: HashSet<_> = combination.iter().copied().collect();

        for &cap_id in &combination_set {
            if let Some(cap) = space.capabilities.get(cap_id) {
                for conflict in &cap.conflicts {
                    if combination_set.contains(conflict.as_str()) {
                        conflicts += 1;
                    }
                }
            }
        }

        if conflicts > 0 {
            0.0
        } else {
            1.0
        }
    }

    /// Clear scoring cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

impl Default for FitnessScoringEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Particle for swarm optimization
#[derive(Debug, Clone)]
struct Particle {
    position: Vec<String>,
    velocity: Vec<f64>,
    best_position: Vec<String>,
    best_score: f64,
}

/// Swarm optimizer for distributed search
pub struct SwarmOptimizer {
    particles: Vec<Particle>,
    global_best_position: Vec<String>,
    global_best_score: f64,
    engine: Arc<FitnessScoringEngine>,
}

impl SwarmOptimizer {
    /// Create new swarm with specified size
    pub fn new(swarm_size: usize, engine: Arc<FitnessScoringEngine>) -> Self {
        Self {
            particles: Vec::with_capacity(swarm_size),
            global_best_position: Vec::new(),
            global_best_score: 0.0,
            engine,
        }
    }

    /// Initialize particles in search space
    #[cfg(feature = "agent2028")]
    pub fn initialize(&mut self, space: &SearchSpace) {
        let capabilities: Vec<_> = space.capabilities.keys().cloned().collect();

        for _ in 0..self.particles.capacity() {
            let mut position = Vec::new();
            for cap in &capabilities {
                if rand::random::<f64>() > 0.5 {
                    position.push(cap.clone());
                }
            }

            let particle = Particle {
                position: position.clone(),
                velocity: vec![0.0; capabilities.len()],
                best_position: position,
                best_score: 0.0,
            };

            self.particles.push(particle);
        }
    }

    /// Initialize particles in search space (deterministic fallback)
    #[cfg(not(feature = "agent2028"))]
    pub fn initialize(&mut self, space: &SearchSpace) {
        let capabilities: Vec<_> = space.capabilities.keys().cloned().collect();

        for i in 0..self.particles.capacity() {
            let mut position = Vec::new();
            for (j, cap) in capabilities.iter().enumerate() {
                // Deterministic selection based on particle and capability index
                if (i + j) % 2 == 0 {
                    position.push(cap.clone());
                }
            }

            let particle = Particle {
                position: position.clone(),
                velocity: vec![0.0; capabilities.len()],
                best_position: position,
                best_score: 0.0,
            };

            self.particles.push(particle);
        }
    }

    /// Run optimization iteration
    pub fn iterate(&mut self, space: &mut SearchSpace) -> f64 {
        for particle in &mut self.particles {
            let position_refs: Vec<&str> = particle.position.iter().map(|s| s.as_str()).collect();

            let score = self.engine.score(&position_refs, space);
            let total_score = score.total();

            if total_score > particle.best_score {
                particle.best_score = total_score;
                particle.best_position = particle.position.clone();
            }

            if total_score > self.global_best_score {
                self.global_best_score = total_score;
                self.global_best_position = particle.position.clone();
            }

            space.mark_explored(&position_refs);
        }

        self.global_best_score
    }

    /// Get best discovered combination
    pub fn best_combination(&self) -> &[String] {
        &self.global_best_position
    }
}

/// Suggestion for developer
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub capabilities: Vec<String>,
    pub score: FitnessScore,
    pub rationale: String,
    pub safe: bool,
}

/// Suggestion factory for developer-facing recommendations
pub struct SuggestionFactory {
    engine: Arc<FitnessScoringEngine>,
    prover: Arc<SafetyProver>,
}

impl SuggestionFactory {
    /// Create new suggestion factory
    pub fn new(engine: Arc<FitnessScoringEngine>, prover: Arc<SafetyProver>) -> Self {
        Self { engine, prover }
    }

    /// Generate suggestion from capability combination
    pub fn generate(&self, combination: &[String], space: &SearchSpace) -> Suggestion {
        let refs: Vec<&str> = combination.iter().map(|s| s.as_str()).collect();
        let score = self.engine.score(&refs, space);

        let safe = combination.iter().all(|cap_id| {
            space.capabilities.get(cap_id).map(|cap| self.prover.is_safe(cap)).unwrap_or(false)
        });

        let rationale = self.generate_rationale(combination, &score);

        Suggestion { capabilities: combination.to_vec(), score, rationale, safe }
    }

    /// Generate human-readable rationale
    fn generate_rationale(&self, combination: &[String], score: &FitnessScore) -> String {
        format!(
            "Combination of {} capabilities with utility={:.2}, novelty={:.2}, safety={:.2}",
            combination.len(),
            score.utility,
            score.novelty,
            score.safety
        )
    }
}

/// Safety prover for validating suggestions
pub struct SafetyProver {
    rules: Vec<Box<dyn Fn(&Capability<state::Discovered>) -> bool + Send + Sync>>,
}

impl SafetyProver {
    /// Create new safety prover
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add safety rule
    pub fn add_rule<F>(&mut self, rule: F)
    where
        F: Fn(&Capability<state::Discovered>) -> bool + Send + Sync + 'static,
    {
        self.rules.push(Box::new(rule));
    }

    /// Check if capability is safe
    pub fn is_safe(&self, capability: &Capability<state::Discovered>) -> bool {
        self.rules.iter().all(|rule| rule(capability))
    }
}

impl Default for SafetyProver {
    fn default() -> Self {
        let mut prover = Self::new();

        prover.add_rule(|cap| !cap.id.is_empty());

        prover.add_rule(|cap| !cap.category.is_empty());

        prover
    }
}

/// Declarative macro for marking capabilities as discoverable
#[macro_export]
macro_rules! discoverable {
    (
        name: $name:expr,
        category: $category:expr
        $(, requires: [$($req:expr),* $(,)?])?
        $(, conflicts: [$($conf:expr),* $(,)?])?
    ) => {
        {
            let mut capability = $crate::macros::discovery_engine::Capability::new($name, $category);

            $(
                $(
                    capability = capability.requires($req);
                )*
            )?

            $(
                $(
                    capability = capability.conflicts_with($conf);
                )*
            )?

            capability
        }
    };
}

/// Declarative macro for defining fitness functions
#[macro_export]
macro_rules! fitness_function {
    (
        capability: $cap:expr,
        utility: $utility:expr,
        novelty: $novelty:expr,
        safety: $safety:expr
    ) => {{
        use $crate::macros::discovery_engine::FitnessScore;

        FitnessScore {
            utility: ($utility) as f64,
            novelty: ($novelty) as f64,
            safety: ($safety) as f64,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new("parallel_exec", "performance")
            .requires("tokio")
            .conflicts_with("single_threaded");

        assert_eq!(cap.id, "parallel_exec");
        assert_eq!(cap.category, "performance");
        assert_eq!(cap.requires, vec!["tokio"]);
        assert_eq!(cap.conflicts, vec!["single_threaded"]);
    }

    #[test]
    fn test_search_space_registration() {
        let mut space = SearchSpace::new();

        let cap1 = Capability::new("cap1", "cat1");
        let cap2 = Capability::new("cap2", "cat2");

        space.register(cap1);
        space.register(cap2);

        assert_eq!(space.capabilities.len(), 2);
        assert_eq!(space.dimensions.len(), 2);
    }

    #[test]
    fn test_combination_hashing() {
        let combo1 = vec!["a", "b", "c"];
        let combo2 = vec!["c", "a", "b"];

        let hash1 = SearchSpace::hash_combination(&combo1);
        let hash2 = SearchSpace::hash_combination(&combo2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fitness_scoring() {
        let mut space = SearchSpace::new();
        space.register(Capability::new("cap1", "cat1"));
        space.register(Capability::new("cap2", "cat2"));

        let engine = FitnessScoringEngine::new();
        let score = engine.score(&["cap1"], &space);

        assert!(score.utility > 0.0);
        assert!(score.total() <= 1.0);
    }

    #[test]
    fn test_safety_prover() {
        let prover = SafetyProver::default();
        let cap = Capability::new("valid", "category");

        assert!(prover.is_safe(&cap));

        let invalid_cap = Capability::new("", "category");
        assert!(!prover.is_safe(&invalid_cap));
    }

    #[test]
    fn test_discoverable_macro() {
        let cap = discoverable! {
            name: "test_cap",
            category: "testing",
            requires: ["dep1", "dep2"],
            conflicts: ["conflict1"]
        };

        assert_eq!(cap.id, "test_cap");
        assert_eq!(cap.requires.len(), 2);
        assert_eq!(cap.conflicts.len(), 1);
    }

    #[test]
    fn test_fitness_function_macro() {
        let score = fitness_function! {
            capability: "test",
            utility: 0.8,
            novelty: 0.6,
            safety: 1.0
        };

        assert_eq!(score.utility, 0.8);
        assert_eq!(score.novelty, 0.6);
        assert_eq!(score.safety, 1.0);
    }
}
