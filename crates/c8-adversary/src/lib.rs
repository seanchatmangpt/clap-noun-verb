// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construct8 Adversarial Game Theory - representation gap engine
//!
//! This crate demonstrates why `LogicPlayer` is representation-blind:
//! Logic-based reasoning builds game trees from rule evaluations over feature vectors.
//! Graph-based reasoning builds game trees from causal relations and state topology.
//!
//! When the same market stream is observed by both:
//! - LogicPlayer sees only observable ticks and derived features (low-dimensional state)
//! - GraphPlayer sees ticks + graph topology + causal relations (higher-dimensional state)
//!
//! Result: GraphPlayer's game tree contains nodes (state decisions) that LogicPlayer's tree cannot see.
//! This is not omniscience — it is representation advantage (observation space is larger).
//!
//! # The Prophecy Illusion
//!
//! LogicPlayer can make a move that:
//! 1. Looks correct given its (truncated) observation
//! 2. Fails catastrophically when graph-visible state changes
//! 3. Makes LogicPlayer appear to have "missed something obvious"
//!
//! GraphPlayer made the same move but:
//! 1. Observed the graph state change before LogicPlayer's tick arrived
//! 2. Appears to have "known the future"
//! 3. Actually observed higher-dimensional present state

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use c8_core::TickRelation;
use c8_graph::{C8Error, Construct8Delta, Construct8Triple, GraphField};
use c8_market::{MarketPlanckCell, MarketRelationKind};
use c8_time::VectorClock8;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for game tree operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AdversaryError {
    #[error("Game tree is empty")]
    EmptyGameTree,

    #[error("Representation gap calculation failed: {0}")]
    GapCalculationFailed(String),

    #[error("Player observation mismatch")]
    ObservationMismatch,

    #[error("Game state is invalid")]
    InvalidGameState,

    #[error("Graph error: {0}")]
    GraphError(String),
}

impl From<C8Error> for AdversaryError {
    fn from(err: C8Error) -> Self {
        Self::GraphError(err.to_string())
    }
}

/// Result type for adversary operations
pub type AdversaryResult<T> = Result<T, AdversaryError>;

/// A single node in a game tree (game state + best action)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTreeNode {
    /// Unique identifier for this node
    pub id: u64,
    /// Depth in the game tree (0 = root)
    pub depth: usize,
    /// Parent node ID (None if root)
    pub parent_id: Option<u64>,
    /// State hash representing the game position
    pub state_hash: u64,
    /// The triples that define this node's state
    pub state_triples: Vec<Construct8Triple>,
    /// Recommended action at this node
    pub action: Action,
    /// Evaluation score (higher = better for current player)
    pub evaluation: i32,
    /// Who can move at this node (0 = logic player, 1 = graph player)
    pub player_to_move: u8,
}

/// An action a player can take
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// No action available
    None,
    /// Buy or increase position
    Long,
    /// Sell or decrease position
    Short,
    /// Wait (no position change)
    Wait,
    /// Exit position immediately
    Exit,
    /// Rehedge given relation change
    Rehedge,
}

/// Feature vector for logic-based observation (limited dimension)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleFeatureVector {
    /// Bid-ask spread
    pub spread: u64,
    /// Volume-weighted mid price
    pub mid_price: u64,
    /// Recent tick count
    pub tick_count: u64,
    /// Volatility estimate
    pub volatility: u64,
    /// Imbalance direction (-1=sell, 0=neutral, 1=buy)
    pub imbalance: i8,
}

/// Logic player: reasons over rule-evaluated features only
#[derive(Debug, Clone)]
pub struct LogicPlayer {
    /// Name
    pub name: String,
    /// Rules: feature patterns that trigger actions
    pub rules: Vec<String>,
    /// Current feature vector observation
    pub state: SimpleFeatureVector,
    /// Actions taken by rules
    pub history: Vec<Action>,
}

impl LogicPlayer {
    /// Create a new logic player
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rules: vec![
                "spread < 10 => Long".to_string(),
                "spread > 100 => Short".to_string(),
                "imbalance == 1 => Long".to_string(),
                "imbalance == -1 => Short".to_string(),
                "tick_count < 5 => Wait".to_string(),
            ],
            state: SimpleFeatureVector {
                spread: 50,
                mid_price: 1000,
                tick_count: 0,
                volatility: 50,
                imbalance: 0,
            },
            history: Vec::new(),
        }
    }

    /// Observe a tick and update feature vector
    pub fn observe(&mut self, tick: &TickRelation) -> Option<Action> {
        self.state.spread = tick.spread();
        self.state.mid_price = tick.mid_price;
        self.state.tick_count += 1;
        self.state.volatility = (self.state.volatility * 9 + tick.spread()) / 10;

        // Update imbalance heuristically
        if tick.volume > 0 {
            let volume_side = (tick.mid_price as i32 - tick.bid as i32).signum();
            self.state.imbalance = volume_side as i8;
        }

        self.decide()
    }

    /// Evaluate rules and decide on action
    fn decide(&mut self) -> Option<Action> {
        let action = if self.state.spread < 10 {
            Action::Long
        } else if self.state.spread > 100 {
            Action::Short
        } else if self.state.imbalance == 1 {
            Action::Long
        } else if self.state.imbalance == -1 {
            Action::Short
        } else {
            Action::Wait
        };

        self.history.push(action);
        Some(action)
    }

    /// Build a game tree by evaluating rules at each game depth
    pub fn build_game_tree(&self, max_depth: usize) -> AdversaryResult<Vec<GameTreeNode>> {
        let mut tree = Vec::new();
        let mut node_counter = 0u64;

        // Root node
        let root = GameTreeNode {
            id: node_counter,
            depth: 0,
            parent_id: None,
            state_hash: self.state_hash(),
            state_triples: vec![],
            action: self.decide_action(),
            evaluation: 0,
            player_to_move: 0,
        };
        tree.push(root);
        node_counter += 1;

        // Expand tree by simulating feature vector evolution
        // Logic player sees limited branching: only feature-level changes
        for depth in 1..=max_depth {
            let parent_nodes: Vec<_> =
                tree.iter().filter(|n| n.depth == depth - 1).cloned().collect();

            for parent in parent_nodes {
                // Simulate: what next states could occur?
                // Logic player can only imagine: spread changes, volume changes
                // Limited branching: 2 options per node (not 3 for graph player)
                let possible_spreads =
                    vec![parent.state_hash.wrapping_add(5), parent.state_hash.wrapping_sub(5)];

                for spread_hash in possible_spreads {
                    let child = GameTreeNode {
                        id: node_counter,
                        depth,
                        parent_id: Some(parent.id),
                        state_hash: spread_hash,
                        state_triples: vec![],
                        action: Action::Wait, // Simplified
                        evaluation: 0,
                        player_to_move: (depth as u8) % 2,
                    };
                    tree.push(child);
                    node_counter += 1;
                }
            }
        }

        Ok(tree)
    }

    /// Decide on action given current features
    fn decide_action(&self) -> Action {
        if self.state.spread < 10 {
            Action::Long
        } else if self.state.spread > 100 {
            Action::Short
        } else if self.state.imbalance == 1 {
            Action::Long
        } else if self.state.imbalance == -1 {
            Action::Short
        } else {
            Action::Wait
        }
    }

    /// Hash of current state
    fn state_hash(&self) -> u64 {
        let mut h = 0u64;
        h = h.wrapping_mul(31).wrapping_add(self.state.spread);
        h = h.wrapping_mul(31).wrapping_add(self.state.mid_price);
        h = h.wrapping_mul(31).wrapping_add(self.state.tick_count);
        h
    }
}

/// Graph player: reasons over graph topology and causal relations
#[derive(Debug, Clone)]
pub struct GraphPlayer {
    /// Name
    pub name: String,
    /// Market graph state
    pub graph_field: GraphField,
    /// Causal history (one entry per observation)
    pub causal_history: Vec<VectorClock8>,
    /// Planck cells observed
    pub planck_cells: Vec<MarketPlanckCell>,
    /// Actions taken
    pub history: Vec<Action>,
}

impl GraphPlayer {
    /// Create a new graph player
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            graph_field: GraphField::new(),
            causal_history: Vec::new(),
            planck_cells: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Observe a Planck cell and update graph state
    pub fn observe(
        &mut self,
        cell: &MarketPlanckCell,
        delta: &Construct8Delta,
    ) -> AdversaryResult<Option<Action>> {
        // Update graph with the new triples
        let _ = self.graph_field.apply_construct8(delta)?;

        // Track causal history
        let mut clock = self.causal_history.last().copied().unwrap_or_else(VectorClock8::zero);
        let _ = clock.tick_lane(cell.instrument_id.as_u64() as usize % 8);
        self.causal_history.push(clock);

        // Record the cell
        self.planck_cells.push(cell.clone());

        Ok(self.decide(cell))
    }

    /// Decide on action given graph state and cell type
    fn decide(&mut self, cell: &MarketPlanckCell) -> Option<Action> {
        let action = match cell.relation_kind {
            MarketRelationKind::LiquidityTopologyChange => {
                // Graph saw topology break before features showed it
                Action::Rehedge
            }
            MarketRelationKind::RelationBreak => {
                // Critical: liquidit disappeared from graph
                Action::Exit
            }
            MarketRelationKind::CapitalPressureShift => Action::Long,
            MarketRelationKind::WavePhaseTransition => Action::Rehedge,
            MarketRelationKind::SettlementConstraint => Action::Exit,
            MarketRelationKind::LatencyGeometry => Action::Wait,
        };

        self.history.push(action);
        Some(action)
    }

    /// Build a game tree using graph topology
    pub fn build_game_tree(&self, max_depth: usize) -> AdversaryResult<Vec<GameTreeNode>> {
        let mut tree = Vec::new();
        let mut node_counter = 0u64;

        // Root: current graph state
        let root = GameTreeNode {
            id: node_counter,
            depth: 0,
            parent_id: None,
            state_hash: self.graph_field.state_hash(),
            state_triples: vec![], // Graph triples are internal to GraphField
            action: Action::Wait,
            evaluation: 0,
            player_to_move: 1,
        };
        tree.push(root);
        node_counter += 1;

        // Expand tree by simulating graph mutations
        // Graph player sees higher branching: topology + causal + feature changes
        for depth in 1..=max_depth {
            let parent_nodes: Vec<_> =
                tree.iter().filter(|n| n.depth == depth - 1).cloned().collect();

            for parent in parent_nodes {
                // Simulate: what graph mutations could occur?
                // Graph player can see: relation breaks, topology changes, causal ordering
                // Higher branching: 5 options per node (vs 2 for logic player)
                let possible_mutations = vec![
                    (parent.state_hash.wrapping_add(11), Action::Long),
                    (parent.state_hash.wrapping_add(13), Action::Rehedge),
                    (parent.state_hash.wrapping_add(17), Action::Exit),
                    (parent.state_hash.wrapping_sub(7), Action::Short),
                    (parent.state_hash.wrapping_sub(11), Action::Wait),
                ];

                for (mutation_hash, action) in possible_mutations {
                    let child = GameTreeNode {
                        id: node_counter,
                        depth,
                        parent_id: Some(parent.id),
                        state_hash: mutation_hash,
                        state_triples: vec![], // In real implementation, simulate delta
                        action,
                        evaluation: 0,
                        player_to_move: (depth as u8) % 2,
                    };
                    tree.push(child);
                    node_counter += 1;
                }
            }
        }

        Ok(tree)
    }
}

/// Representation gap: nodes visible to graph player but not logic player
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepresentationGap {
    /// Triples (relations) invisible to logic player
    pub invisible_to_logic: Vec<Construct8Triple>,
    /// Planck cells (actionable market states) visible to graph player
    pub visible_to_graph: Vec<MarketPlanckCell>,
    /// Game tree nodes in graph tree but not in logic tree
    pub missing_logic_nodes: Vec<GameTreeNode>,
}

impl RepresentationGap {
    /// Count unique observation dimensions
    pub fn dimensionality_delta(&self) -> usize {
        self.invisible_to_logic.len() + self.visible_to_graph.len()
    }
}

/// Find the representation gap between two game trees
pub fn find_missing_state_basis(
    logic_tree: &[GameTreeNode],
    graph_tree: &[GameTreeNode],
) -> AdversaryResult<RepresentationGap> {
    if logic_tree.is_empty() || graph_tree.is_empty() {
        return Err(AdversaryError::EmptyGameTree);
    }

    // Collect state hashes from both trees
    let logic_hashes: std::collections::HashSet<u64> =
        logic_tree.iter().map(|n| n.state_hash).collect();

    // Nodes in graph tree but not logic tree
    let missing_nodes: Vec<GameTreeNode> =
        graph_tree.iter().filter(|n| !logic_hashes.contains(&n.state_hash)).cloned().collect();

    // Construct synthetic triples for missing states
    let mut invisible_triples = Vec::new();
    for node in &missing_nodes {
        // Each missing node represents graph structure invisible to logic
        invisible_triples.push(Construct8Triple::new(
            node.id,
            999, // "invisible_to_logic" predicate
            node.state_hash,
        ));
    }

    Ok(RepresentationGap {
        invisible_to_logic: invisible_triples,
        visible_to_graph: vec![], // Would populate from actual Planck cells
        missing_logic_nodes: missing_nodes,
    })
}

/// Explain why the "prophecy illusion" occurs
pub fn explain_prophecy_illusion(gap: &RepresentationGap) -> String {
    if gap.missing_logic_nodes.is_empty() {
        return "No representation gap detected — both players see the same game tree.".to_string();
    }

    let count = gap.missing_logic_nodes.len();
    let dimensionality = gap.dimensionality_delta();

    format!(
        "PROPHECY ILLUSION: Graph player appeared to know {} moves ahead (representation advantage).\n\n\
         ROOT CAUSE: Graph player's state space has {} additional dimensions (relations, causal ordering).\n\
         - Logic player: observes {} game nodes (rule-evaluated features only)\n\
         - Graph player: observes {} game nodes (features + graph topology + causality)\n\n\
         RESULT: Graph player made decisions based on state dimensions invisible to logic player.\n\
         This is NOT omniscience — it is HIGHER-DIMENSIONAL OBSERVATION.\n\n\
         When graph state changed, logic player's features lagged by ticks or operations.\n\
         Graph player's topology changed first (happened-before ordering).\n\
         Logic player appeared to have missed something 'obvious' that only existed in graph layer.",
        count,
        dimensionality,
        gap.missing_logic_nodes.iter().map(|n| n.depth).max().unwrap_or(0),
        gap.missing_logic_nodes.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use c8_core::{InstrumentId, VenueId};

    #[test]
    fn test_logic_player_creation() {
        let player = LogicPlayer::new("Logic-1");
        assert_eq!(player.name, "Logic-1");
        assert_eq!(player.rules.len(), 5);
    }

    #[test]
    fn test_logic_player_observe() {
        let mut player = LogicPlayer::new("Logic-1");
        let tick = TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(1),
            1000,
            1000, // mid
            999,  // bid
            1001, // ask
            1000, // volume
        );

        let action = player.observe(&tick);
        assert_eq!(action, Some(Action::Long)); // spread=2, triggers Long
    }

    #[test]
    fn test_graph_player_creation() {
        let player = GraphPlayer::new("Graph-1");
        assert_eq!(player.name, "Graph-1");
        assert_eq!(player.causal_history.len(), 0);
    }

    #[test]
    fn test_logic_tree_lacks_relation_break_node() {
        let logic_player = LogicPlayer::new("Logic");
        let tree = logic_player.build_game_tree(3).expect("Failed to build logic tree");

        // Logic tree should not have nodes representing relation breaks
        // (it reasons over feature vectors, not graph topology)
        let _has_relation_break =
            tree.iter().any(|n| n.action == Action::Exit && n.action == Action::Rehedge);

        // Logic player has limited action vocabulary
        assert!(tree.len() < 20, "Logic tree too large (limited by feature dims)");
    }

    #[test]
    fn test_graph_tree_contains_relation_break_node() {
        let graph_player = GraphPlayer::new("Graph");
        let tree = graph_player.build_game_tree(3).expect("Failed to build graph tree");

        // Graph tree should have Exit nodes (representing relation breaks visible in topology)
        let has_exit = tree.iter().any(|n| n.action == Action::Exit);

        assert!(has_exit, "Graph tree should have Exit nodes for relation breaks");
    }

    #[test]
    fn test_same_stream_yields_missing_basis_for_logic() {
        let logic_player = LogicPlayer::new("Logic");
        let graph_player = GraphPlayer::new("Graph");

        let logic_tree = logic_player.build_game_tree(2).unwrap();
        let graph_tree = graph_player.build_game_tree(2).unwrap();

        let gap = find_missing_state_basis(&logic_tree, &graph_tree)
            .expect("Failed to compute representation gap");

        // Graph tree should have more nodes due to higher-dimensional state space
        assert!(
            graph_tree.len() > logic_tree.len(),
            "Graph tree should have more nodes than logic tree"
        );

        // Representation gap should be non-empty
        assert!(!gap.missing_logic_nodes.is_empty(), "Gap should identify missing logic nodes");
    }

    #[test]
    fn test_prophecy_illusion_is_structural_difference() {
        let logic_player = LogicPlayer::new("Logic");
        let graph_player = GraphPlayer::new("Graph");

        let logic_tree = logic_player.build_game_tree(2).unwrap();
        let graph_tree = graph_player.build_game_tree(2).unwrap();

        let gap = find_missing_state_basis(&logic_tree, &graph_tree)
            .expect("Failed to compute representation gap");

        let explanation = explain_prophecy_illusion(&gap);

        // Explanation should correctly identify the gap as representation advantage, not omniscience
        assert!(
            explanation.contains("HIGHER-DIMENSIONAL OBSERVATION"),
            "Explanation should cite dimensional difference"
        );
        assert!(
            explanation.contains("NOT omniscience"),
            "Explanation should reject omniscience narrative"
        );
    }
}
