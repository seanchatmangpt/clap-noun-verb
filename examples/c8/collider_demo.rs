// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Market Collider Demo: revealing hidden market bodies through hypothesis collision
//!
//! This example demonstrates:
//! 1. Two competing market hypotheses (liquidity vs. capital models)
//! 2. Collision of these hypotheses through shared observations
//! 3. Inference of hidden market body characteristics
//! 4. Gravity signature extraction (influence markers)
//! 5. Conformance analysis of visible vs. hidden dynamics

use c8_core::{InstrumentId, TickRelation, VenueId};
use c8_instruments::{HiddenMarketBody, MarketCollider};
use c8_market::{MarketPlanckCell, MarketRelationKind};
use serde_json::json;

fn main() {
    println!("=== Construct8: Market Collider Demo ===\n");

    // STEP 1: Define competing hypotheses
    println!("STEP 1: Define Market Hypotheses");
    println!("================================\n");

    let liquidity_hypothesis = create_liquidity_hypothesis();
    let capital_hypothesis = create_capital_hypothesis();

    println!("Hypothesis 1: Liquidity Model");
    println!("  - Focus: Orderbook topology, spread dynamics, depth");
    println!("  - Assumption: Market is efficient within depth");
    println!();
    println!("Hypothesis 2: Capital Model");
    println!("  - Focus: Capital pressure, imbalance, leverage cycles");
    println!("  - Assumption: Pricing follows hidden capital flows\n");

    // STEP 2: Create synthetic market observations
    println!("STEP 2: Generate Shared Market Observations");
    println!("==========================================\n");

    let observations = create_shared_observations();
    println!("Generated {} shared tick observations", observations.len());

    for (i, obs) in observations.iter().enumerate() {
        println!(
            "  Obs {}: mid={}, volume={}, spread={}",
            i,
            obs.mid_price,
            obs.volume,
            obs.spread()
        );
    }
    println!();

    // STEP 3: Run both hypotheses through observations
    println!("STEP 3: Run Both Hypotheses on Observations");
    println!("==========================================\n");

    let liquidity_planck =
        hypotheses_to_planck_cells(&observations, &liquidity_hypothesis, "Liquidity");
    let capital_planck = hypotheses_to_planck_cells(&observations, &capital_hypothesis, "Capital");

    println!("Liquidity model generated {} Planck cells", liquidity_planck.len());
    println!("Capital model generated {} Planck cells", capital_planck.len());
    println!();

    // STEP 4: Initialize collider
    println!("STEP 4: Initialize MarketCollider");
    println!("=================================\n");

    println!("Collider initialized");
    println!("Collision strategy: Detect misalignments in causal graphs\n");

    // STEP 5: Collide hypotheses
    println!("STEP 5: Collide Hypotheses");
    println!("==========================\n");

    let collision_result = MarketCollider::collide_hypotheses(&liquidity_planck, &capital_planck);

    println!("Hypothesis collision result:");
    println!("  - Hypotheses collide: {}", collision_result.hypotheses_collide);
    println!("  - Bounded delta: {:.4}", collision_result.bounded_delta);
    println!("  - Causal ordering valid: {}", collision_result.causal_ordering_valid);
    println!();

    // Count divergences between hypotheses
    let mut divergence_count = 0;
    for (liq_cell, cap_cell) in liquidity_planck.iter().zip(capital_planck.iter()) {
        if liq_cell.relation_kind != cap_cell.relation_kind {
            divergence_count += 1;
            println!(
                "  Divergence {}: Liquidity sees {:?}, Capital sees {:?}",
                divergence_count, liq_cell.relation_kind, cap_cell.relation_kind
            );
        }
    }
    println!();
    println!("Total divergences: {}\n", divergence_count);

    // STEP 6: Infer hidden market body
    println!("STEP 6: Infer Hidden Market Body");
    println!("================================\n");

    let hidden_body = MarketCollider::infer_hidden_market_body(&liquidity_planck)
        .or_else(|| MarketCollider::infer_hidden_market_body(&capital_planck))
        .unwrap_or_else(|| HiddenMarketBody {
            implied_capital_mass: 0.0,
            gravity_signature: vec![],
        });

    println!("Inferred Hidden Market Body:");
    println!("  - Implied capital mass: {:.2}", hidden_body.implied_capital_mass);
    println!("  - Gravity signature length: {}", hidden_body.gravity_signature.len());

    if !hidden_body.gravity_signature.is_empty() {
        println!("  - Gravity signature (influence markers):");
        for (i, sig) in hidden_body.gravity_signature.iter().enumerate() {
            println!("      Lane {}: {:.4}", i, sig);
        }
    }
    println!();

    // STEP 7: Generate output receipt
    println!("STEP 7: Collider Receipt (JSON)");
    println!("===============================\n");

    let receipt_json = json!({
        "implementation": "collider_demo",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "observations": observations.len(),
        "liquidity_cells": liquidity_planck.len(),
        "capital_cells": capital_planck.len(),
        "divergences_detected": divergence_count,
        "collision_analysis": {
            "hypotheses_collide": collision_result.hypotheses_collide,
            "bounded_delta": collision_result.bounded_delta,
            "causal_ordering_valid": collision_result.causal_ordering_valid,
            "divergence_count": divergence_count,
        },
        "hidden_market_body": {
            "implied_capital_mass": hidden_body.implied_capital_mass,
            "gravity_signature_markers": hidden_body.gravity_signature.len(),
            "gravity_signature": hidden_body.gravity_signature,
        },
        "market_astrophysics": {
            "principle": "Hidden market bodies reveal themselves through collisions. When two causal observation spaces diverge, their difference encodes the gravity of the unseen market structure.",
            "interpretation": "The collider doesn't see the hidden body directly; it infers its existence and shape from how visible light (liquidity, capital pressure) bends around it."
        }
    });

    println!("{}", serde_json::to_string_pretty(&receipt_json).unwrap_or_default());
    println!();

    println!("=== Demo Complete ===");
}

/// Define a liquidity-focused hypothesis
fn create_liquidity_hypothesis() -> &'static str {
    "liquidity_depth_model"
}

/// Define a capital-focused hypothesis
fn create_capital_hypothesis() -> &'static str {
    "capital_pressure_model"
}

/// Create shared market observations both hypotheses will analyze
fn create_shared_observations() -> Vec<TickRelation> {
    vec![
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1000,
            50000,
            49900,
            50100,
            10000,
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            2000,
            50500,
            50250, // bid up, but spread wider
            50750,
            5000,
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            3000,
            49800,
            49400, // sharp drop
            50200,
            8000,
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            4000,
            50200,
            49800,
            50600,
            15000,
        ),
    ]
}

/// Convert observations to Planck cells using a specific hypothesis
fn hypotheses_to_planck_cells(
    observations: &[TickRelation],
    hypothesis: &str,
    label: &str,
) -> Vec<MarketPlanckCell> {
    let mut cells = Vec::new();

    for (i, obs) in observations.iter().enumerate() {
        // Each hypothesis interprets the same observation differently
        let relation_kind = if hypothesis.contains("liquidity") {
            // Liquidity model: focus on spread and volume
            if obs.spread() > 500 {
                MarketRelationKind::LiquidityTopologyChange
            } else if obs.volume < 5000 {
                MarketRelationKind::LatencyGeometry
            } else {
                MarketRelationKind::CapitalPressureShift
            }
        } else {
            // Capital model: focus on price movement and imbalance
            if i > 0 && observations[i - 1].mid_price != obs.mid_price {
                let move_size = (obs.mid_price as i64 - observations[i - 1].mid_price as i64).abs();
                if move_size > 500 {
                    MarketRelationKind::CapitalPressureShift
                } else {
                    MarketRelationKind::WavePhaseTransition
                }
            } else {
                MarketRelationKind::SettlementConstraint
            }
        };

        let cell = MarketPlanckCell::new(
            obs.instrument_id,
            obs.venue_id,
            relation_kind,
            i as u64,
            obs.timestamp,
        );
        cells.push(cell);

        println!("  {}-H[{}]: {:?}", label, i, relation_kind);
    }

    cells
}
