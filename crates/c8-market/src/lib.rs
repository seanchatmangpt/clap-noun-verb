// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Market Planck Cell Modeler - smallest actionable relational market state
//!
//! This crate implements the Market Planck Cell (MPC) model: the smallest indivisible
//! unit of market state change. Each cell captures a single atomic relation (liquidity
//! change, capital pressure shift, settlement constraint, etc.) with deterministic
//! causal and monotonic timestamps, state hashes, and actionability classification.
//!
//! A MarketPlanckCell is NOT an event; it is a deterministic mapping of observable
//! market relations to graph state. Relation breaks, topology changes, and wave phase
//! transitions are detected via graph state queries, not branchy rule engines.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use c8_core::{InstrumentId, TickRelation, VenueId};
use c8_graph::{Construct8Delta, Construct8Triple, GraphField};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Classification of how a Planck cell mutation is actuated
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActuationClass {
    /// Immediate execution required (e.g., liquidity break)
    Immediate,
    /// Deferred but bounded within microseconds
    DeferredWithinMicros(u32),
    /// Bounded execution window (less strict deadline)
    Bounded,
    /// Market refused the mutation (e.g., circuit breaker)
    Refused,
}

/// Classification of market relation types
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MarketRelationKind {
    /// Connectivity or depth topology change
    LiquidityTopologyChange,
    /// Pressure from buy/sell imbalance
    CapitalPressureShift,
    /// Sudden loss of connectivity or liquidity
    RelationBreak,
    /// Wave/cyclical pattern phase transition
    WavePhaseTransition,
    /// Settlement or margin constraint activated
    SettlementConstraint,
    /// Latency geometry change (order propagation delay shifts)
    LatencyGeometry,
}

/// Market Planck Cell: smallest actionable relational market state
///
/// Represents a single atomic market relation change:
/// - Causal time: logical clock for causality ordering
/// - Monotonic time: wall-clock timestamp (no regressions)
/// - Pre/post state hashes: deterministic proof of state transition
/// - Delta mask: which of 8 RDF triple slots are occupied
/// - Confidence bucket: 0-255 credibility score
/// - Actuation class: how the market should respond
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MarketPlanckCell {
    /// Which instrument this relation affects
    pub instrument_id: InstrumentId,
    /// Which venue this relation is observed at
    pub venue_id: VenueId,
    /// Type of relation detected
    pub relation_kind: MarketRelationKind,
    /// Logical causal clock (for happened-before ordering)
    pub causal_time: u64,
    /// Monotonic wall-clock timestamp
    pub monotonic_time: u64,
    /// Hash of graph state before this cell's delta applied
    pub pre_state_hash: u64,
    /// Predictive hint for post-state (not authoritative)
    pub post_state_hint: u64,
    /// Bitmask of occupied Construct8 triple slots
    pub delta_mask: u8,
    /// Confidence score (0-255, where 255 is highest)
    pub confidence_bucket: u8,
    /// How this mutation should be actuated
    pub actuation_class: ActuationClass,
}

/// Error type for market operations
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarketError {
    /// Relation cannot be modeled within Construct8 bounds
    ExceedsConstruct8Max,
    /// Graph state is inconsistent with expectation
    StateHashMismatch,
    /// Insufficient confidence to classify as relation
    InsufficientConfidence,
    /// Graph operation failed
    GraphOperationFailed(String),
}

impl MarketPlanckCell {
    /// Create a new MarketPlanckCell with minimal fields
    pub fn new(
        instrument_id: InstrumentId,
        venue_id: VenueId,
        relation_kind: MarketRelationKind,
        causal_time: u64,
        monotonic_time: u64,
    ) -> Self {
        Self {
            instrument_id,
            venue_id,
            relation_kind,
            causal_time,
            monotonic_time,
            pre_state_hash: 0,
            post_state_hint: 0,
            delta_mask: 0,
            confidence_bucket: 128,
            actuation_class: ActuationClass::Bounded,
        }
    }

    /// Convert this Planck cell to a Construct8Delta (max 8 triples)
    ///
    /// Maps the relation kind and cell metadata to RDF-like triples:
    /// - (instrument, relation_kind, venue)
    /// - (cell, confidence, confidence_bucket)
    /// - (cell, actuation, actuation_class_id)
    /// - etc. up to 8 triples total
    pub fn to_construct8_delta(&self) -> Result<Construct8Delta, MarketError> {
        let mut delta = Construct8Delta::new();

        // Triple 1: (instrument_id, relation_kind_id, venue_id)
        let relation_kind_id = self.relation_kind_to_id();
        let triple1 = Construct8Triple::new(
            self.instrument_id.as_u64(),
            relation_kind_id,
            self.venue_id.as_u64(),
        );
        delta.push_checked(triple1).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        // Triple 2: (cell_id, confidence, confidence_bucket)
        let cell_id = self.cell_id();
        let triple2 = Construct8Triple::new(cell_id, 1001, self.confidence_bucket as u64);
        delta.push_checked(triple2).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        // Triple 3: (cell_id, actuation_class, actuation_id)
        let actuation_id = self.actuation_class_to_id();
        let triple3 = Construct8Triple::new(cell_id, 1002, actuation_id);
        delta.push_checked(triple3).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        // Triple 4: (cell_id, causal_time, causal_time_value)
        let triple4 = Construct8Triple::new(cell_id, 1003, self.causal_time);
        delta.push_checked(triple4).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        // Triple 5: (cell_id, monotonic_time, monotonic_time_value)
        let triple5 = Construct8Triple::new(cell_id, 1004, self.monotonic_time);
        delta.push_checked(triple5).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        // Triple 6: (cell_id, pre_state_hash, pre_state_hash_value)
        let triple6 = Construct8Triple::new(cell_id, 1005, self.pre_state_hash);
        delta.push_checked(triple6).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        // Triple 7: (cell_id, post_state_hint, post_state_hint_value)
        let triple7 = Construct8Triple::new(cell_id, 1006, self.post_state_hint);
        delta.push_checked(triple7).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        // Triple 8: (cell_id, delta_mask, delta_mask_value)
        let triple8 = Construct8Triple::new(cell_id, 1007, self.delta_mask as u64);
        delta.push_checked(triple8).map_err(|_| MarketError::ExceedsConstruct8Max)?;

        Ok(delta)
    }

    /// Create a MarketPlanckCell from a tick relation
    ///
    /// This is the entry point: when a new tick arrives, we check if it triggers
    /// a relation change in the graph. If it does, we emit a Planck cell.
    pub fn from_tick_relation(
        tick: TickRelation,
        prev_graph: &GraphField,
        current_graph: &GraphField,
        causal_time: u64,
    ) -> Option<Self> {
        let pre_hash = prev_graph.state_hash();
        let post_hash = current_graph.state_hash();

        // No relation change = no Planck cell
        if pre_hash == post_hash {
            return None;
        }

        // Determine relation kind based on spread and topology
        let relation_kind = if !tick.is_normal_spread() {
            MarketRelationKind::LiquidityTopologyChange
        } else {
            MarketRelationKind::CapitalPressureShift
        };

        let confidence = if current_graph.triple_count() > 10 { 250 } else { 128 };

        Some(Self {
            instrument_id: tick.instrument_id,
            venue_id: tick.venue_id,
            relation_kind,
            causal_time,
            monotonic_time: tick.timestamp,
            pre_state_hash: pre_hash,
            post_state_hint: post_hash,
            delta_mask: 0xFF, // All slots occupied
            confidence_bucket: confidence,
            actuation_class: ActuationClass::Immediate,
        })
    }

    /// Get a deterministic cell ID based on component fields
    fn cell_id(&self) -> u64 {
        let mut result = 0u64;
        result = result.wrapping_mul(31).wrapping_add(self.instrument_id.as_u64());
        result = result.wrapping_mul(31).wrapping_add(self.venue_id.as_u64());
        result = result.wrapping_mul(31).wrapping_add(self.causal_time);
        result
    }

    /// Map relation kind to a numeric ID for RDF encoding
    fn relation_kind_to_id(&self) -> u64 {
        match self.relation_kind {
            MarketRelationKind::LiquidityTopologyChange => 2001,
            MarketRelationKind::CapitalPressureShift => 2002,
            MarketRelationKind::RelationBreak => 2003,
            MarketRelationKind::WavePhaseTransition => 2004,
            MarketRelationKind::SettlementConstraint => 2005,
            MarketRelationKind::LatencyGeometry => 2006,
        }
    }

    /// Map actuation class to a numeric ID for RDF encoding
    fn actuation_class_to_id(&self) -> u64 {
        match self.actuation_class {
            ActuationClass::Immediate => 3001,
            ActuationClass::DeferredWithinMicros(micros) => 3000 + micros as u64,
            ActuationClass::Bounded => 3002,
            ActuationClass::Refused => 3003,
        }
    }
}

/// Detect a relation break state via graph state inspection
///
/// A relation break is evidenced by:
/// - Drop in subject count (liquidity providers disappear)
/// - Sudden reduction in triple density
/// - Broken connectivity (isolated subject nodes)
///
/// This uses graph state queries, NOT branchy if/else rules.
pub fn detect_relation_break_state(prev_graph: &GraphField, current_graph: &GraphField) -> bool {
    let prev_count = prev_graph.triple_count();
    let curr_count = current_graph.triple_count();

    // More than 50% drop in triples indicates break (inclusive on boundary)
    if prev_count > 0 && curr_count <= prev_count / 2 {
        return true;
    }

    // Subject disappearance check
    let prev_subjects = prev_graph.subjects();
    let curr_subjects = current_graph.subjects();

    if prev_subjects.len() > 2 && curr_subjects.len() <= prev_subjects.len() / 2 {
        return true;
    }

    false
}

/// Detect a liquidity topology change via graph state
///
/// Topology change is signaled by:
/// - Addition of new subjects (new market makers)
/// - New predicate types (new relation kinds)
/// - Expansion of object set for existing predicates
///
/// Returns Some(cell) if topology change is detected, None otherwise.
pub fn detect_liquidity_topology_state(
    instrument_id: InstrumentId,
    venue_id: VenueId,
    prev_graph: &GraphField,
    current_graph: &GraphField,
    causal_time: u64,
    monotonic_time: u64,
) -> Option<MarketPlanckCell> {
    let prev_subjects = prev_graph.subjects();
    let curr_subjects = current_graph.subjects();

    // New subject added (new market participant)
    let has_new_subject = curr_subjects.iter().any(|s| !prev_subjects.contains(s));
    if !has_new_subject {
        return None;
    }

    Some(MarketPlanckCell {
        instrument_id,
        venue_id,
        relation_kind: MarketRelationKind::LiquidityTopologyChange,
        causal_time,
        monotonic_time,
        pre_state_hash: prev_graph.state_hash(),
        post_state_hint: current_graph.state_hash(),
        delta_mask: 0xFF,
        confidence_bucket: 200,
        actuation_class: ActuationClass::Immediate,
    })
}

impl fmt::Display for MarketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketError::ExceedsConstruct8Max => {
                write!(f, "Relation exceeds Construct8 maximum of 8 triples")
            }
            MarketError::StateHashMismatch => write!(f, "Graph state hash mismatch"),
            MarketError::InsufficientConfidence => {
                write!(f, "Insufficient confidence to classify as relation")
            }
            MarketError::GraphOperationFailed(msg) => write!(f, "Graph operation failed: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_alone_not_equal_planck_cell() {
        // A tick relation by itself is not a Planck cell.
        // A Planck cell is only emitted when the tick causes a graph state change.
        let tick =
            TickRelation::new(InstrumentId::new(1), VenueId::new(1), 1000, 100, 99, 101, 1000);

        // Empty graph: same state before and after
        let prev_graph = GraphField::new();
        let current_graph = GraphField::new();

        let cell = MarketPlanckCell::from_tick_relation(tick, &prev_graph, &current_graph, 100);
        assert!(cell.is_none(), "Tick with no state change should not emit Planck cell");
    }

    #[test]
    fn test_relation_change_emits_planck_cell() {
        // When a tick causes a graph state change, a Planck cell is emitted.
        let tick =
            TickRelation::new(InstrumentId::new(1), VenueId::new(1), 1000, 100, 99, 101, 1000);

        let prev_graph = GraphField::new();

        let mut current_graph = GraphField::new();
        // Simulate a state change by adding a triple
        let triple = c8_graph::Construct8Triple::new(1, 2, 3);
        current_graph.add_triple(triple);

        let cell = MarketPlanckCell::from_tick_relation(tick, &prev_graph, &current_graph, 100);
        assert!(cell.is_some(), "Tick with state change should emit Planck cell");

        let cell = cell.unwrap();
        assert_eq!(cell.instrument_id, InstrumentId::new(1));
        assert_eq!(cell.venue_id, VenueId::new(1));
        assert_eq!(cell.causal_time, 100);
        assert_eq!(cell.monotonic_time, 1000);
    }

    #[test]
    fn test_planck_cell_emits_construct8_delta_max_8() {
        let cell = MarketPlanckCell::new(
            InstrumentId::new(42),
            VenueId::new(99),
            MarketRelationKind::LiquidityTopologyChange,
            500,
            2000,
        );

        let delta = cell.to_construct8_delta().expect("should convert to delta");

        // Should have exactly 8 triples (all fields encoded)
        assert_eq!(delta.triple_count(), 8);
        assert_eq!(delta.mask(), 0xFF);

        // Verify that delta can be applied to a graph
        let mut graph = GraphField::new();
        let stats = graph.apply_construct8(&delta).expect("should apply delta");
        assert_eq!(stats.applied, 8);
    }

    #[test]
    fn test_relation_break_without_branchy_rules() {
        // Relation break detection uses graph state inspection, not if/else branches.
        let mut prev_graph = GraphField::new();
        let triple1 = c8_graph::Construct8Triple::new(1, 2, 3);
        let triple2 = c8_graph::Construct8Triple::new(1, 5, 6);
        let triple3 = c8_graph::Construct8Triple::new(10, 2, 7);
        let triple4 = c8_graph::Construct8Triple::new(10, 5, 8);
        let triple5 = c8_graph::Construct8Triple::new(20, 2, 9);

        prev_graph.add_triple(triple1);
        prev_graph.add_triple(triple2);
        prev_graph.add_triple(triple3);
        prev_graph.add_triple(triple4);
        prev_graph.add_triple(triple5);

        // Current graph: only 2 triples (60% drop, clearly a break)
        let mut current_graph = GraphField::new();
        current_graph.add_triple(triple1);
        current_graph.add_triple(triple2);

        let is_break = detect_relation_break_state(&prev_graph, &current_graph);
        assert!(is_break, "Graph state shows >50% triple loss = relation break");
    }

    #[test]
    fn test_wave_phase_as_graph_state_not_mysticism() {
        // Topology change is detected by graph state (new subjects), not mystical patterns.
        let mut prev_graph = GraphField::new();
        prev_graph.add_triple(c8_graph::Construct8Triple::new(1, 2, 3));

        let mut current_graph = GraphField::new();
        current_graph.add_triple(c8_graph::Construct8Triple::new(1, 2, 3));
        // New subject added (subject 100)
        current_graph.add_triple(c8_graph::Construct8Triple::new(100, 2, 3));

        let cell = detect_liquidity_topology_state(
            InstrumentId::new(1),
            VenueId::new(1),
            &prev_graph,
            &current_graph,
            500,
            2000,
        );

        assert!(cell.is_some(), "Graph state shows new subject = topology change");
        let cell = cell.unwrap();
        assert_eq!(cell.relation_kind, MarketRelationKind::LiquidityTopologyChange);
    }

    #[test]
    fn test_market_planck_cell_creation() {
        let cell = MarketPlanckCell::new(
            InstrumentId::new(1),
            VenueId::new(2),
            MarketRelationKind::CapitalPressureShift,
            100,
            1000,
        );

        assert_eq!(cell.instrument_id, InstrumentId::new(1));
        assert_eq!(cell.venue_id, VenueId::new(2));
        assert_eq!(cell.relation_kind, MarketRelationKind::CapitalPressureShift);
        assert_eq!(cell.causal_time, 100);
        assert_eq!(cell.monotonic_time, 1000);
        assert_eq!(cell.confidence_bucket, 128);
        assert_eq!(cell.actuation_class, ActuationClass::Bounded);
    }

    #[test]
    fn test_actuation_class_variants() {
        let immediate = ActuationClass::Immediate;
        let deferred = ActuationClass::DeferredWithinMicros(100);
        let bounded = ActuationClass::Bounded;
        let refused = ActuationClass::Refused;

        assert_eq!(immediate, ActuationClass::Immediate);
        assert_eq!(deferred, ActuationClass::DeferredWithinMicros(100));
        assert_eq!(bounded, ActuationClass::Bounded);
        assert_eq!(refused, ActuationClass::Refused);
    }

    #[test]
    fn test_market_relation_kind_variants() {
        let kinds = [
            MarketRelationKind::LiquidityTopologyChange,
            MarketRelationKind::CapitalPressureShift,
            MarketRelationKind::RelationBreak,
            MarketRelationKind::WavePhaseTransition,
            MarketRelationKind::SettlementConstraint,
            MarketRelationKind::LatencyGeometry,
        ];

        for kind in kinds {
            let cell = MarketPlanckCell::new(InstrumentId::new(1), VenueId::new(1), kind, 0, 0);
            assert_eq!(cell.relation_kind, kind);
        }
    }

    #[test]
    fn test_construct8_delta_from_cell_max_8() {
        let cell = MarketPlanckCell {
            instrument_id: InstrumentId::new(777),
            venue_id: VenueId::new(888),
            relation_kind: MarketRelationKind::RelationBreak,
            causal_time: 123,
            monotonic_time: 456,
            pre_state_hash: 111,
            post_state_hint: 222,
            delta_mask: 0xAA,
            confidence_bucket: 200,
            actuation_class: ActuationClass::DeferredWithinMicros(50),
        };

        let delta = cell.to_construct8_delta().expect("should convert to delta");
        assert_eq!(delta.triple_count(), 8);
        assert_eq!(delta.mask(), 0xFF); // All 8 slots occupied
    }
}
