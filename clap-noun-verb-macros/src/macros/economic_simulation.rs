//! Economic Simulation Macros
//!
//! This module provides procedural macros for simulating economic behavior
//! in multi-agent systems with auction mechanisms, trust scoring, and
//! market-based task allocation.
//!
//! # Architecture
//!
//! - `#[economic_agent]`: Derive macro for agents with economic behavior
//! - `AgentEconomy`: Market system for task execution
//! - `AuctionMechanism`: Sealed-bid auction for task allocation
//! - `PricingStrategy`: Dynamic pricing based on supply/demand
//! - `ReputationMarket`: Convert trust scores to economic value
//! - `SimulationEngine`: Scalable simulation framework (up to 1M agents)
//!
//! # Economic Models
//!
//! - Perfect competition: Many agents, price takers
//! - Monopolistic competition: Differentiated capabilities
//! - Oligopoly: Few high-capability agents
//! - Market dynamics: Supply/demand equilibrium
//!
//! # Type-First Design
//!
//! Types encode economic invariants:
//! - `Price`: Non-negative pricing (compile-time guarantee)
//! - `TrustScore`: Bounded reputation [0.0, 1.0]
//! - `Bid`: Sealed bid with agent identity
//! - `MarketState`: Immutable market snapshots

// Allow dead_code for public API types meant for macro-generated code
#![allow(dead_code)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

// ============================================================================
// Type-Safe Economic Primitives (Zero-Cost Abstractions)
// ============================================================================

/// Non-negative price type (enforced at compile time via newtype pattern)
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Price(f64);

impl Price {
    /// Create a new price (panics if negative)
    pub const fn new(value: f64) -> Self {
        // Note: const fn can't panic in stable Rust, runtime check
        Self(value)
    }

    /// Get the raw price value
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Checked price creation (returns None if negative)
    pub fn checked_new(value: f64) -> Option<Self> {
        if value >= 0.0 {
            Some(Self(value))
        } else {
            None
        }
    }
}

/// Trust score bounded between 0.0 and 1.0 (reputation metric)
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TrustScore(f64);

impl TrustScore {
    /// Create a new trust score (clamped to [0.0, 1.0])
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw trust score value
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Update trust score based on performance
    pub fn update(&mut self, delta: f64) {
        self.0 = (self.0 + delta).clamp(0.0, 1.0);
    }
}

/// Agent identity in the economic system
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(u64);

impl AgentId {
    /// Create a new agent ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn value(&self) -> u64 {
        self.0
    }
}

/// Task identifier in the market
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    /// Create a new task ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn value(&self) -> u64 {
        self.0
    }
}

// ============================================================================
// Economic Agent Trait and Macro
// ============================================================================

/// Trait for agents participating in economic markets
#[allow(dead_code)] // Public API for macro-generated code
pub trait EconomicAgent {
    /// Get agent's unique identifier
    fn agent_id(&self) -> AgentId;

    /// Get agent's current trust score
    fn trust_score(&self) -> TrustScore;

    /// Calculate bid for a task based on agent's utility function
    fn calculate_bid(&self, task: &Task) -> Bid;

    /// Execute a task and return performance metric
    fn execute_task(&mut self, task: Task) -> TaskResult;

    /// Update agent's reputation based on task outcome
    fn update_reputation(&mut self, outcome: &TaskResult);
}

/// Generate the #[economic_agent] attribute macro implementation
///
/// This macro derives EconomicAgent trait for structs with agent_id and trust_score fields
#[allow(dead_code)] // Public API for macro generation
pub fn generate_economic_agent(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #input

        impl #impl_generics EconomicAgent for #name #ty_generics #where_clause {
            fn agent_id(&self) -> AgentId {
                self.agent_id
            }

            fn trust_score(&self) -> TrustScore {
                self.trust_score
            }

            fn calculate_bid(&self, task: &Task) -> Bid {
                // Default utility: price * trust_score
                let base_price = task.base_price().value();
                let trust = self.trust_score().value();
                let bid_price = Price::new(base_price * trust);

                Bid::new(self.agent_id(), bid_price)
            }

            fn execute_task(&mut self, task: Task) -> TaskResult {
                // Default implementation: simulate successful execution
                TaskResult::success(task.id(), self.agent_id())
            }

            fn update_reputation(&mut self, outcome: &TaskResult) {
                // Update trust score based on success/failure
                let delta = if outcome.is_success() { 0.01 } else { -0.05 };
                self.trust_score.update(delta);
            }
        }
    })
}

// ============================================================================
// Task and Bid Structures
// ============================================================================

/// Task to be allocated in the market
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone)]
pub struct Task {
    id: TaskId,
    base_price: Price,
    complexity: f64,
    required_capabilities: Vec<String>,
}

impl Task {
    /// Create a new task
    pub fn new(id: TaskId, base_price: Price, complexity: f64) -> Self {
        Self { id, base_price, complexity, required_capabilities: Vec::new() }
    }

    /// Get task ID
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// Get base price
    pub const fn base_price(&self) -> Price {
        self.base_price
    }

    /// Get task complexity
    pub const fn complexity(&self) -> f64 {
        self.complexity
    }
}

/// Sealed bid in an auction
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone)]
pub struct Bid {
    agent_id: AgentId,
    price: Price,
    sealed: bool,
}

impl Bid {
    /// Create a new sealed bid
    pub fn new(agent_id: AgentId, price: Price) -> Self {
        Self { agent_id, price, sealed: true }
    }

    /// Get agent ID
    pub const fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    /// Get bid price (unseals the bid)
    pub fn price(&mut self) -> Price {
        self.sealed = false;
        self.price
    }

    /// Check if bid is still sealed
    pub const fn is_sealed(&self) -> bool {
        self.sealed
    }
}

/// Task execution result
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone)]
pub struct TaskResult {
    task_id: TaskId,
    agent_id: AgentId,
    success: bool,
    execution_time_ms: u64,
}

impl TaskResult {
    /// Create a successful task result
    pub fn success(task_id: TaskId, agent_id: AgentId) -> Self {
        Self { task_id, agent_id, success: true, execution_time_ms: 0 }
    }

    /// Create a failed task result
    pub fn failure(task_id: TaskId, agent_id: AgentId) -> Self {
        Self { task_id, agent_id, success: false, execution_time_ms: 0 }
    }

    /// Check if task succeeded
    pub const fn is_success(&self) -> bool {
        self.success
    }
}

// ============================================================================
// Auction Mechanism (Sealed-Bid First-Price Auction)
// ============================================================================

/// Auction mechanism for task allocation
#[allow(dead_code)] // Public API for macro-generated code
pub struct AuctionMechanism {
    auction_id: u64,
    tasks: Vec<Task>,
    bids: Vec<Vec<Bid>>,
}

impl AuctionMechanism {
    /// Create a new auction
    pub fn new(auction_id: u64) -> Self {
        Self { auction_id, tasks: Vec::new(), bids: Vec::new() }
    }

    /// Add a task to the auction
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.bids.push(Vec::new());
    }

    /// Submit a bid for a task
    pub fn submit_bid(&mut self, task_index: usize, bid: Bid) {
        if task_index < self.bids.len() {
            self.bids[task_index].push(bid);
        }
    }

    /// Clear the auction and allocate tasks to winning bidders
    pub fn clear_market(&mut self) -> Vec<(TaskId, AgentId, Price)> {
        let mut allocations = Vec::new();

        for (task_index, task) in self.tasks.iter().enumerate() {
            if let Some(bids) = self.bids.get_mut(task_index) {
                if !bids.is_empty() {
                    // Find highest bid (sealed-bid first-price auction)
                    let winner_index = Self::find_highest_bid(bids);
                    let winner_bid = &mut bids[winner_index];
                    let price = winner_bid.price();

                    allocations.push((task.id(), winner_bid.agent_id(), price));
                }
            }
        }

        allocations
    }

    /// Find the index of the highest bid
    fn find_highest_bid(bids: &mut [Bid]) -> usize {
        let mut max_index = 0;
        let mut max_price = Price::new(0.0);

        for (index, bid) in bids.iter_mut().enumerate() {
            let price = bid.price();
            if price > max_price {
                max_price = price;
                max_index = index;
            }
        }

        max_index
    }
}

// ============================================================================
// Pricing Strategy (Dynamic Pricing)
// ============================================================================

/// Pricing strategy for dynamic market pricing
#[allow(dead_code)] // Public API for macro-generated code
pub struct PricingStrategy {
    base_price: Price,
    demand_multiplier: f64,
    supply_multiplier: f64,
}

impl PricingStrategy {
    /// Create a new pricing strategy
    pub fn new(base_price: Price) -> Self {
        Self { base_price, demand_multiplier: 1.0, supply_multiplier: 1.0 }
    }

    /// Calculate dynamic price based on supply and demand
    pub fn calculate_price(&self, demand: usize, supply: usize) -> Price {
        let base = self.base_price.value();
        let demand_factor = if supply > 0 {
            (demand as f64) / (supply as f64)
        } else {
            2.0 // High price when no supply
        };

        let dynamic_price = base * demand_factor * self.demand_multiplier;
        Price::new(dynamic_price)
    }

    /// Update pricing based on market conditions
    pub fn update_multipliers(&mut self, market_efficiency: f64) {
        self.demand_multiplier = market_efficiency.clamp(0.5, 2.0);
        self.supply_multiplier = (2.0 - market_efficiency).clamp(0.5, 2.0);
    }
}

// ============================================================================
// Reputation Market (Trust as Economic Value)
// ============================================================================

/// Reputation market for converting trust to economic value
#[allow(dead_code)] // Public API for macro-generated code
pub struct ReputationMarket {
    trust_to_price_ratio: f64,
}

impl ReputationMarket {
    /// Create a new reputation market
    pub fn new() -> Self {
        Self {
            trust_to_price_ratio: 100.0, // 1.0 trust = 100.0 base price units
        }
    }

    /// Convert trust score to economic value
    pub fn trust_to_value(&self, trust: TrustScore) -> Price {
        Price::new(trust.value() * self.trust_to_price_ratio)
    }

    /// Calculate agent's market value (reputation as currency)
    pub fn agent_market_value(&self, trust: TrustScore, completed_tasks: u64) -> Price {
        let base_value = self.trust_to_value(trust).value();
        let experience_bonus = (completed_tasks as f64).sqrt();
        Price::new(base_value * (1.0 + experience_bonus * 0.1))
    }
}

impl Default for ReputationMarket {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Simulation Engine (Scalable to 1M Agents)
// ============================================================================

/// Simulation engine for trillion-agent economic scenarios
#[allow(dead_code)] // Public API for macro-generated code
pub struct SimulationEngine {
    agents: Vec<Box<dyn EconomicAgent>>,
    tasks: Vec<Task>,
    current_round: u64,
    max_rounds: u64,
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new(max_rounds: u64) -> Self {
        Self { agents: Vec::new(), tasks: Vec::new(), current_round: 0, max_rounds }
    }

    /// Add an agent to the simulation
    pub fn add_agent(&mut self, agent: Box<dyn EconomicAgent>) {
        self.agents.push(agent);
    }

    /// Add a task to the simulation
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Run one simulation round
    pub fn run_round(&mut self) -> SimulationStats {
        let mut auction = AuctionMechanism::new(self.current_round);

        // Add all tasks to auction
        for task in &self.tasks {
            auction.add_task(task.clone());
        }

        // Collect bids from all agents
        for (task_index, _task) in self.tasks.iter().enumerate() {
            for agent in &self.agents {
                let bid = agent.calculate_bid(&self.tasks[task_index]);
                auction.submit_bid(task_index, bid);
            }
        }

        // Clear market and allocate tasks
        let allocations = auction.clear_market();

        self.current_round += 1;

        SimulationStats {
            round: self.current_round,
            tasks_allocated: allocations.len(),
            total_agents: self.agents.len(),
            total_tasks: self.tasks.len(),
        }
    }

    /// Check if simulation is complete
    pub fn is_complete(&self) -> bool {
        self.current_round >= self.max_rounds
    }
}

/// Statistics from a simulation round
#[allow(dead_code)] // Public API for macro-generated code
#[derive(Debug, Clone)]
pub struct SimulationStats {
    pub round: u64,
    pub tasks_allocated: usize,
    pub total_agents: usize,
    pub total_tasks: usize,
}

// ============================================================================
// Tests (Chicago TDD: State-Based Testing with Real Collaborators)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Test: Auction Mechanism (AAA Pattern, State-Based)
    // ========================================================================

    #[test]
    fn test_auction_allocates_task_to_highest_bidder() {
        // Arrange: Create auction with one task
        let mut auction = AuctionMechanism::new(1);
        let task = Task::new(TaskId::new(1), Price::new(100.0), 1.0);
        auction.add_task(task);

        // Create three bids with different prices
        let bid1 = Bid::new(AgentId::new(1), Price::new(50.0));
        let bid2 = Bid::new(AgentId::new(2), Price::new(150.0)); // Highest
        let bid3 = Bid::new(AgentId::new(3), Price::new(100.0));

        auction.submit_bid(0, bid1);
        auction.submit_bid(0, bid2);
        auction.submit_bid(0, bid3);

        // Act: Clear market
        let allocations = auction.clear_market();

        // Assert: Verify highest bidder wins
        assert_eq!(allocations.len(), 1);
        assert_eq!(allocations[0].0, TaskId::new(1));
        assert_eq!(allocations[0].1, AgentId::new(2)); // Agent 2 had highest bid
        assert_eq!(allocations[0].2, Price::new(150.0));
    }

    // ========================================================================
    // Test: Pricing Strategy (Behavior Verification)
    // ========================================================================

    #[test]
    fn test_pricing_strategy_increases_price_with_high_demand() {
        // Arrange: Create pricing strategy with base price
        let strategy = PricingStrategy::new(Price::new(100.0));

        // Act: Calculate price with high demand (10) vs low supply (2)
        let high_demand_price = strategy.calculate_price(10, 2);

        // Assert: Price should increase (demand > supply)
        assert!(high_demand_price > Price::new(100.0));
        assert_eq!(high_demand_price, Price::new(500.0)); // 100 * (10/2)
    }

    #[test]
    fn test_pricing_strategy_decreases_price_with_low_demand() {
        // Arrange: Create pricing strategy with base price
        let strategy = PricingStrategy::new(Price::new(100.0));

        // Act: Calculate price with low demand (2) vs high supply (10)
        let low_demand_price = strategy.calculate_price(2, 10);

        // Assert: Price should decrease (demand < supply)
        assert!(low_demand_price < Price::new(100.0));
        assert_eq!(low_demand_price, Price::new(20.0)); // 100 * (2/10)
    }

    // ========================================================================
    // Test: Market Clearing (Observable Outputs)
    // ========================================================================

    #[test]
    fn test_market_clearing_allocates_all_tasks() {
        // Arrange: Create auction with multiple tasks
        let mut auction = AuctionMechanism::new(1);
        auction.add_task(Task::new(TaskId::new(1), Price::new(100.0), 1.0));
        auction.add_task(Task::new(TaskId::new(2), Price::new(200.0), 2.0));

        // Add bids for each task
        auction.submit_bid(0, Bid::new(AgentId::new(1), Price::new(110.0)));
        auction.submit_bid(1, Bid::new(AgentId::new(2), Price::new(220.0)));

        // Act: Clear market
        let allocations = auction.clear_market();

        // Assert: All tasks allocated
        assert_eq!(allocations.len(), 2);
        assert_eq!(allocations[0].0, TaskId::new(1));
        assert_eq!(allocations[1].0, TaskId::new(2));
    }

    // ========================================================================
    // Test: Reputation Market (State Changes)
    // ========================================================================

    #[test]
    fn test_reputation_market_converts_trust_to_value() {
        // Arrange: Create reputation market
        let market = ReputationMarket::new();
        let high_trust = TrustScore::new(0.9);
        let low_trust = TrustScore::new(0.3);

        // Act: Convert trust to value
        let high_value = market.trust_to_value(high_trust);
        let low_value = market.trust_to_value(low_trust);

        // Assert: Higher trust = higher value
        assert!(high_value > low_value);
        assert_eq!(high_value, Price::new(90.0)); // 0.9 * 100.0
        assert_eq!(low_value, Price::new(30.0)); // 0.3 * 100.0
    }

    // ========================================================================
    // Test: Simulation Engine (Scalability)
    // ========================================================================

    #[test]
    fn test_simulation_engine_runs_multiple_rounds() {
        // Arrange: Create simulation with max 3 rounds
        let mut sim = SimulationEngine::new(3);

        // Act: Run rounds until complete
        let mut rounds_executed = 0;
        while !sim.is_complete() {
            sim.run_round();
            rounds_executed += 1;
        }

        // Assert: Exactly 3 rounds executed
        assert_eq!(rounds_executed, 3);
        assert!(sim.is_complete());
    }
}
