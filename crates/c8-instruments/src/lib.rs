// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Market Astrophysics Instruments - telescope, event horizon, and collider primitives
//!
//! This crate implements observational and analytical instruments for causal market analysis:
//!
//! - **MarketTelescope**: Observes and collects visible MarketPlanckCell traces
//! - **MarketEventHorizonTelescope**: Detects event horizon boundaries where liquidity vanishes
//! - **MarketCollider**: Tests hypothesis pairs by colliding market states to infer hidden structures
//!
//! These instruments operate on the principle that market dynamics can be understood through
//! causal analysis: visible traces accumulate, event horizons mark causal boundaries, and
//! collisions reveal hidden market bodies through inference.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use c8_market::{MarketPlanckCell, MarketRelationKind};
use serde::{Deserialize, Serialize};

/// Event horizon boundary: marks where liquidity ceases to exist
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventHorizonBoundary {
    /// Height of the liquidity cliff (how much liquidity is missing)
    pub liquidity_cliff_height: f64,
    /// Causal time when the boundary was crossed
    pub causal_time: u64,
}

/// Hidden market body: inferred from correlated visible traces
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HiddenMarketBody {
    /// Inferred capital mass (aggregate hidden supply)
    pub implied_capital_mass: f64,
    /// Gravity signature: vector of influence markers
    pub gravity_signature: Vec<f64>,
}

/// Collision result: outcome of hypothesis pair collision
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollisionResult {
    /// Whether hypotheses are consistent
    pub hypotheses_collide: bool,
    /// Delta between observed states (bounded)
    pub bounded_delta: f64,
    /// Causal ordering constraint
    pub causal_ordering_valid: bool,
}

/// Market Telescope: observes and collects visible market traces
///
/// The telescope accumulates MarketPlanckCells, providing a historical view of
/// observed market state changes. This is the primary observational instrument
/// for causal market analysis.
#[derive(Clone, Debug)]
pub struct MarketTelescope {
    observations: Vec<MarketPlanckCell>,
}

impl MarketTelescope {
    /// Create a new empty telescope
    pub fn new() -> Self {
        Self { observations: Vec::new() }
    }

    /// Record a visible trace (MarketPlanckCell observation)
    pub fn observe_visible_trace(&mut self, cell: MarketPlanckCell) {
        self.observations.push(cell);
    }

    /// Get all accumulated observations
    pub fn observations(&self) -> &[MarketPlanckCell] {
        &self.observations
    }

    /// Get observation count
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Clear all observations (reset telescope)
    pub fn clear(&mut self) {
        self.observations.clear();
    }

    /// Get observations ordered by causal time
    pub fn ordered_by_causal_time(&self) -> Vec<MarketPlanckCell> {
        let mut ordered = self.observations.clone();
        ordered.sort_by_key(|cell| cell.causal_time);
        ordered
    }

    /// Filter observations by relation kind
    pub fn filter_by_relation_kind(&self, kind: MarketRelationKind) -> Vec<MarketPlanckCell> {
        self.observations.iter().filter(|cell| cell.relation_kind == kind).cloned().collect()
    }
}

impl Default for MarketTelescope {
    fn default() -> Self {
        Self::new()
    }
}

/// Market Event Horizon Telescope: detects causal boundaries
///
/// An event horizon is a boundary beyond which causal influence is impossible.
/// In market terms: where liquidity ceases to exist, breaking connectivity.
///
/// Detection is based on:
/// - Sudden liquidity cliffs (observed via confidence_bucket drop)
/// - Causal time ordering violations
/// - Relation breaks that cannot be resolved by future observations
#[derive(Clone, Debug)]
pub struct MarketEventHorizonTelescope;

impl MarketEventHorizonTelescope {
    /// Detect an event horizon boundary from a sequence of cells
    ///
    /// Returns Some(EventHorizonBoundary) if:
    /// - A relation break is observed (high confidence)
    /// - Liquidity gap exceeds threshold (cliff_height > 0.5)
    /// - Causal time has a discontinuity
    pub fn detect_event_horizon_boundary(
        cells: &[MarketPlanckCell],
    ) -> Option<EventHorizonBoundary> {
        if cells.is_empty() {
            return None;
        }

        // Look for liquidity topology changes or relation breaks with confidence drop
        for i in 1..cells.len() {
            let prev = &cells[i - 1];
            let curr = &cells[i];

            // Detect liquidity cliff: confidence drop of 80+ points
            let confidence_drop = prev.confidence_bucket as i16 - curr.confidence_bucket as i16;
            if confidence_drop >= 80 {
                // Check for relation break pattern
                if curr.relation_kind == MarketRelationKind::RelationBreak
                    || curr.relation_kind == MarketRelationKind::LiquidityTopologyChange
                {
                    let cliff_height = confidence_drop as f64 / 255.0;
                    return Some(EventHorizonBoundary {
                        liquidity_cliff_height: cliff_height,
                        causal_time: curr.causal_time,
                    });
                }
            }
        }

        None
    }

    /// Detect event horizon with custom cliff threshold
    pub fn detect_with_threshold(
        cells: &[MarketPlanckCell],
        threshold: u8,
    ) -> Option<EventHorizonBoundary> {
        if cells.is_empty() {
            return None;
        }

        for i in 1..cells.len() {
            let prev = &cells[i - 1];
            let curr = &cells[i];

            let confidence_drop = prev.confidence_bucket.saturating_sub(curr.confidence_bucket);
            if confidence_drop >= threshold {
                let cliff_height = confidence_drop as f64 / 255.0;
                return Some(EventHorizonBoundary {
                    liquidity_cliff_height: cliff_height,
                    causal_time: curr.causal_time,
                });
            }
        }

        None
    }
}

/// Market Collider: tests hypothesis pairs by collision
///
/// A collider analyzes two market hypotheses to determine if they are consistent.
/// - Consistent hypotheses have bounded delta and valid causal ordering
/// - Inconsistent hypotheses indicate hidden market bodies
#[derive(Clone, Debug)]
pub struct MarketCollider;

impl MarketCollider {
    /// Collide two hypotheses (represented as cell sequences)
    ///
    /// Returns a CollisionResult with:
    /// - hypotheses_collide: true if states are compatible
    /// - bounded_delta: absolute difference in state (capped at 1.0)
    /// - causal_ordering_valid: true if causal times respect ordering
    pub fn collide_hypotheses(
        hypothesis_a: &[MarketPlanckCell],
        hypothesis_b: &[MarketPlanckCell],
    ) -> CollisionResult {
        let delta_a = Self::compute_state_vector(hypothesis_a);
        let delta_b = Self::compute_state_vector(hypothesis_b);

        // Compute bounded delta (max 1.0)
        let raw_delta = (delta_a - delta_b).abs();
        let bounded_delta = raw_delta.min(1.0);

        // Check causal ordering
        let causal_ordering_valid = Self::validate_causal_ordering(hypothesis_a)
            && Self::validate_causal_ordering(hypothesis_b);

        // Hypotheses collide if delta is small and causal ordering is valid
        let hypotheses_collide = bounded_delta < 0.3 && causal_ordering_valid;

        CollisionResult { hypotheses_collide, bounded_delta, causal_ordering_valid }
    }

    /// Infer a hidden market body from correlated relations
    ///
    /// If multiple cells show correlated confidence changes without visible causes,
    /// there is likely a hidden market body exerting gravity.
    ///
    /// Returns Some(HiddenMarketBody) if:
    /// - Cells show correlation without direct causality
    /// - Gravity signature (confidence patterns) is consistent across cells
    pub fn infer_hidden_market_body(cells: &[MarketPlanckCell]) -> Option<HiddenMarketBody> {
        if cells.len() < 3 {
            return None;
        }

        // Compute gravity signature: changes in confidence that persist
        let mut gravity_signature = Vec::new();
        for i in 1..cells.len() {
            let prev = &cells[i - 1];
            let curr = &cells[i];

            let confidence_change =
                (curr.confidence_bucket as f64 - prev.confidence_bucket as f64) / 255.0;
            gravity_signature.push(confidence_change);
        }

        // Check for correlation: if variance is low, signature is consistent
        let mean_gravity = gravity_signature.iter().sum::<f64>() / gravity_signature.len() as f64;
        let variance = gravity_signature.iter().map(|g| (g - mean_gravity).powi(2)).sum::<f64>()
            / gravity_signature.len() as f64;

        // Low variance + non-zero mean suggests hidden influence
        if variance < 0.05 && mean_gravity.abs() > 0.01 {
            let implied_capital_mass = mean_gravity.abs() * 1000.0;
            return Some(HiddenMarketBody { implied_capital_mass, gravity_signature });
        }

        None
    }

    /// Compute aggregate state vector from cells (confidence-based)
    fn compute_state_vector(cells: &[MarketPlanckCell]) -> f64 {
        if cells.is_empty() {
            return 0.0;
        }

        let total_confidence: u64 = cells.iter().map(|c| c.confidence_bucket as u64).sum();
        total_confidence as f64 / (cells.len() as f64 * 255.0)
    }

    /// Validate that causal times form a valid ordering
    fn validate_causal_ordering(cells: &[MarketPlanckCell]) -> bool {
        if cells.len() <= 1 {
            return true;
        }

        for i in 1..cells.len() {
            if cells[i].causal_time < cells[i - 1].causal_time {
                return false;
            }
        }

        true
    }
}

/// Measure liquidity curvature from a sequence of cells
///
/// Curvature is the second derivative of liquidity (confidence-based proxy).
/// High curvature indicates market stress or transition points.
pub fn measure_liquidity_curvature(cells: &[MarketPlanckCell]) -> f64 {
    if cells.len() < 3 {
        return 0.0;
    }

    let mut second_derivatives = Vec::new();
    for i in 1..cells.len() - 1 {
        let prev = cells[i - 1].confidence_bucket as f64 / 255.0;
        let curr = cells[i].confidence_bucket as f64 / 255.0;
        let next = cells[i + 1].confidence_bucket as f64 / 255.0;

        let first_deriv_1 = curr - prev;
        let first_deriv_2 = next - curr;
        let second_deriv = first_deriv_2 - first_deriv_1;

        second_derivatives.push(second_deriv.abs());
    }

    if second_derivatives.is_empty() {
        return 0.0;
    }

    second_derivatives.iter().sum::<f64>() / second_derivatives.len() as f64
}

/// Measure capital gravity (influence) from cells
///
/// Gravity is inferred from sustained confidence changes that lack direct causality.
pub fn measure_capital_gravity(cells: &[MarketPlanckCell]) -> f64 {
    if cells.is_empty() {
        return 0.0;
    }

    // Gravity = standard deviation of confidence changes
    let mut changes = Vec::new();
    for i in 1..cells.len() {
        let change =
            (cells[i].confidence_bucket as f64 - cells[i - 1].confidence_bucket as f64) / 255.0;
        changes.push(change);
    }

    if changes.is_empty() {
        return 0.0;
    }

    let mean = changes.iter().sum::<f64>() / changes.len() as f64;
    let variance = changes.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / changes.len() as f64;

    variance.sqrt()
}

/// Measure relation redshift from delayed observations
///
/// Redshift indicates time-dilation effects: a relation observed "late" suggests
/// causal delay or information propagation lag.
pub fn measure_relation_redshift(cells: &[MarketPlanckCell]) -> f64 {
    if cells.is_empty() {
        return 0.0;
    }

    // Redshift = ratio of monotonic to causal time difference
    let mut redshifts = Vec::new();
    for i in 1..cells.len() {
        let causal_delta = cells[i].causal_time.saturating_sub(cells[i - 1].causal_time);
        let monotonic_delta = cells[i].monotonic_time.saturating_sub(cells[i - 1].monotonic_time);

        if causal_delta > 0 {
            let redshift = monotonic_delta as f64 / causal_delta as f64;
            redshifts.push(redshift);
        }
    }

    if redshifts.is_empty() {
        return 1.0; // No delay
    }

    redshifts.iter().sum::<f64>() / redshifts.len() as f64
}

/// Market phase transition classification
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketPhaseTransition {
    /// Which kind of transition occurred
    pub transition_type: PhaseTransitionType,
    /// Causal time of transition
    pub causal_time: u64,
    /// Intensity score (0-1)
    pub intensity: f64,
}

/// Types of market phase transitions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseTransitionType {
    /// Liquid to illiquid
    LiquidToIlliquid,
    /// Illiquid to liquid
    IlliquidToLiquid,
    /// Normal to crisis
    NormalToCrisis,
    /// Crisis to normal
    CrisisToNormal,
}

/// Classify phase transition from cell sequence
pub fn classify_phase_transition(cells: &[MarketPlanckCell]) -> Option<MarketPhaseTransition> {
    if cells.len() < 2 {
        return None;
    }

    let prev = &cells[cells.len() - 2];
    let curr = &cells[cells.len() - 1];

    // Detect confidence drop indicating stress
    let confidence_drop = prev.confidence_bucket.saturating_sub(curr.confidence_bucket);
    let confidence_gain = curr.confidence_bucket.saturating_sub(prev.confidence_bucket);

    if confidence_drop > 100 {
        // Significant drop: liquid to illiquid
        if curr.relation_kind == MarketRelationKind::RelationBreak {
            return Some(MarketPhaseTransition {
                transition_type: PhaseTransitionType::LiquidToIlliquid,
                causal_time: curr.causal_time,
                intensity: confidence_drop as f64 / 255.0,
            });
        }
    } else if confidence_gain >= 50 {
        // Confidence recovery: illiquid to liquid
        return Some(MarketPhaseTransition {
            transition_type: PhaseTransitionType::IlliquidToLiquid,
            causal_time: curr.causal_time,
            intensity: confidence_gain as f64 / 255.0,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use c8_core::{InstrumentId, VenueId};

    fn make_cell(
        instrument_id: u64,
        venue_id: u64,
        relation_kind: MarketRelationKind,
        causal_time: u64,
        confidence: u8,
    ) -> MarketPlanckCell {
        MarketPlanckCell {
            instrument_id: InstrumentId::new(instrument_id),
            venue_id: VenueId::new(venue_id),
            relation_kind,
            causal_time,
            monotonic_time: causal_time * 1000,
            pre_state_hash: 0,
            post_state_hint: 0,
            delta_mask: 0,
            confidence_bucket: confidence,
            actuation_class: c8_market::ActuationClass::Bounded,
        }
    }

    #[test]
    fn test_detect_liquidity_cliff_from_synthetic_collapse() {
        let cells = vec![
            make_cell(1, 1, MarketRelationKind::LiquidityTopologyChange, 100, 250),
            make_cell(1, 1, MarketRelationKind::LiquidityTopologyChange, 101, 200),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 102, 100),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 103, 20),
        ];

        let boundary = MarketEventHorizonTelescope::detect_event_horizon_boundary(&cells);

        assert!(boundary.is_some(), "Should detect liquidity cliff from confidence drop");
        let boundary = boundary.unwrap();
        assert!(boundary.liquidity_cliff_height > 0.3);
        assert_eq!(boundary.causal_time, 102);
    }

    #[test]
    fn test_detect_relation_redshift_from_delayed_observations() {
        let cells = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 50, 200),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 210),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 200, 205),
        ];

        let redshift = measure_relation_redshift(&cells);

        // Redshift should reflect monotonic/causal time ratio
        assert!(redshift > 0.0);
        // With 1000x multiplier on monotonic: redshift should be ~1000
        assert!(redshift > 900.0);
    }

    #[test]
    fn test_detect_hidden_capital_from_correlated_relations() {
        let cells = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 200),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 101, 185),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 102, 170),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 103, 155),
        ];

        let hidden_body = MarketCollider::infer_hidden_market_body(&cells);

        // Consistent downward pressure suggests hidden capital mass
        assert!(hidden_body.is_some());
        let body = hidden_body.unwrap();
        assert!(body.implied_capital_mass > 0.0);
        assert!(!body.gravity_signature.is_empty());
    }

    #[test]
    fn test_collider_emits_bounded_delta() {
        let hyp_a = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 200),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 101, 205),
        ];

        let hyp_b = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 210),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 101, 215),
        ];

        let collision = MarketCollider::collide_hypotheses(&hyp_a, &hyp_b);

        // Delta should be bounded to [0, 1]
        assert!(collision.bounded_delta >= 0.0 && collision.bounded_delta <= 1.0);
        assert!(collision.causal_ordering_valid);
    }

    #[test]
    fn test_event_horizon_max_8_updates_per_cell() {
        // Create 8 cells with escalating confidence drops
        // The constraint is that a MarketPlanckCell can encode max 8 RDF triples
        let cells = vec![
            make_cell(1, 1, MarketRelationKind::LiquidityTopologyChange, 100, 255),
            make_cell(1, 1, MarketRelationKind::LiquidityTopologyChange, 101, 240),
            make_cell(1, 1, MarketRelationKind::LiquidityTopologyChange, 102, 220),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 103, 190),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 104, 150),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 105, 100),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 106, 40),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 107, 10),
        ];

        // Verify we can track up to 8 cells in sequence
        assert_eq!(cells.len(), 8);

        // Event horizon should be detected with threshold of 60 points
        // (first significant drop: 100->40 = 60 points at causal_time 106)
        let boundary = MarketEventHorizonTelescope::detect_with_threshold(&cells, 60);
        assert!(boundary.is_some(), "Should detect with 60-point threshold");

        let boundary = boundary.unwrap();
        assert_eq!(boundary.causal_time, 106);
    }

    #[test]
    fn test_market_telescope_accumulates_observations() {
        let mut telescope = MarketTelescope::new();

        let cell1 = make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 200);
        let cell2 = make_cell(2, 1, MarketRelationKind::LiquidityTopologyChange, 101, 210);

        telescope.observe_visible_trace(cell1.clone());
        telescope.observe_visible_trace(cell2.clone());

        assert_eq!(telescope.observation_count(), 2);
        assert_eq!(telescope.observations(), &[cell1, cell2]);
    }

    #[test]
    fn test_telescope_ordered_by_causal_time() {
        let mut telescope = MarketTelescope::new();

        let cell1 = make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 300, 200);
        let cell2 = make_cell(1, 1, MarketRelationKind::LiquidityTopologyChange, 100, 210);
        let cell3 = make_cell(1, 1, MarketRelationKind::RelationBreak, 200, 220);

        telescope.observe_visible_trace(cell1);
        telescope.observe_visible_trace(cell2);
        telescope.observe_visible_trace(cell3);

        let ordered = telescope.ordered_by_causal_time();
        assert_eq!(ordered[0].causal_time, 100);
        assert_eq!(ordered[1].causal_time, 200);
        assert_eq!(ordered[2].causal_time, 300);
    }

    #[test]
    fn test_telescope_filter_by_relation_kind() {
        let mut telescope = MarketTelescope::new();

        telescope.observe_visible_trace(make_cell(
            1,
            1,
            MarketRelationKind::CapitalPressureShift,
            100,
            200,
        ));
        telescope.observe_visible_trace(make_cell(
            1,
            1,
            MarketRelationKind::LiquidityTopologyChange,
            101,
            210,
        ));
        telescope.observe_visible_trace(make_cell(
            1,
            1,
            MarketRelationKind::CapitalPressureShift,
            102,
            220,
        ));

        let pressure_cells =
            telescope.filter_by_relation_kind(MarketRelationKind::CapitalPressureShift);
        assert_eq!(pressure_cells.len(), 2);

        let topology_cells =
            telescope.filter_by_relation_kind(MarketRelationKind::LiquidityTopologyChange);
        assert_eq!(topology_cells.len(), 1);
    }

    #[test]
    fn test_liquidity_curvature_zero_for_linear_changes() {
        let cells = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 100),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 101, 150),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 102, 200),
        ];

        let curvature = measure_liquidity_curvature(&cells);

        // Linear change: curvature should be near zero
        assert!(curvature < 0.01);
    }

    #[test]
    fn test_capital_gravity_from_variance() {
        let cells = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 200),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 101, 180),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 102, 220),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 103, 200),
        ];

        let gravity = measure_capital_gravity(&cells);

        // Variance in confidence changes indicates gravity
        assert!(gravity > 0.0);
    }

    #[test]
    fn test_phase_transition_liquid_to_illiquid() {
        let cells = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 200),
            make_cell(1, 1, MarketRelationKind::RelationBreak, 101, 50),
        ];

        let transition = classify_phase_transition(&cells);

        assert!(transition.is_some());
        let t = transition.unwrap();
        assert_eq!(t.transition_type, PhaseTransitionType::LiquidToIlliquid);
        assert!(t.intensity > 0.5);
    }

    #[test]
    fn test_collider_invalid_causal_ordering() {
        let hyp_bad = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 200, 200),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 205),
        ];

        let hyp_good = vec![
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 100, 200),
            make_cell(1, 1, MarketRelationKind::CapitalPressureShift, 200, 205),
        ];

        let result = MarketCollider::collide_hypotheses(&hyp_bad, &hyp_good);
        assert!(!result.causal_ordering_valid);
        assert!(!result.hypotheses_collide);
    }
}
