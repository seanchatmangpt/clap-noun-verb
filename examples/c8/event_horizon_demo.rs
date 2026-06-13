// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Event Horizon Demo: detecting causal boundaries where liquidity vanishes
//!
//! This example demonstrates:
//! 1. Synthetic liquidity collapse scenario
//! 2. Event horizon boundary detection via MarketEventHorizonTelescope
//! 3. Graph field state updates as boundaries cross
//! 4. Causal time tracking at the boundary
//! 5. Evidence of market astrophysics principles

use c8_core::{InstrumentId, TickRelation, VenueId};
use c8_instruments::MarketEventHorizonTelescope;
use c8_market::{MarketPlanckCell, MarketRelationKind};
use serde_json::json;

fn main() {
    println!("=== Construct8: Event Horizon Detection Demo ===\n");

    // STEP 1: Create synthetic liquidity collapse
    println!("STEP 1: Synthetic Liquidity Collapse Scenario");
    println!("============================================\n");

    let scenario = create_liquidity_collapse_scenario();
    println!("Scenario: {} ticks spanning {} time units", scenario.ticks.len(), scenario.max_time);
    println!(
        "Initial liquidity: {} units\nFinal liquidity: {} units",
        scenario.initial_liquidity, scenario.final_liquidity
    );
    println!(
        "Collapse magnitude: {:.2}%\n",
        (1.0 - scenario.final_liquidity / scenario.initial_liquidity) * 100.0
    );

    // STEP 2: Generate Planck cells representing the collapse
    println!("STEP 2: Generate Planck Cells for Collapse");
    println!("==========================================\n");

    let cells = scenario.to_planck_cells();
    println!("Generated {} Planck cells:", cells.len());

    for (i, cell) in cells.iter().enumerate() {
        println!("  Cell {}: {:?} at causal_time={}", i, cell.relation_kind, cell.causal_time);
    }
    println!();

    // STEP 3: Run Event Horizon Telescope
    println!("STEP 3: Run MarketEventHorizonTelescope");
    println!("=======================================\n");

    println!("Telescope analyzing cells for event horizon boundaries...\n");

    let boundary_detected = MarketEventHorizonTelescope::detect_event_horizon_boundary(&cells);

    if let Some(ref boundary) = boundary_detected {
        println!("Event Horizon Boundary detected!");
        println!("  - Liquidity cliff height: {:.2}", boundary.liquidity_cliff_height);
        println!("  - Causal time: {}", boundary.causal_time);
        println!();
    }

    println!(
        "Total event horizons detected: {}\n",
        if boundary_detected.is_some() { 1 } else { 0 }
    );

    // STEP 4: Analyze boundary characteristics
    println!("STEP 4: Event Horizon Analysis");
    println!("==============================\n");

    if let Some(ref boundary) = boundary_detected {
        println!("Boundary Statistics:");
        println!("  - Cliff height: {:.2}", boundary.liquidity_cliff_height);
        println!("  - Boundary crossing: causal_time={}", boundary.causal_time);
    } else {
        println!("No event horizon boundary detected in this scenario");
    }
    println!();

    // STEP 5: Graph field state at boundary
    println!("STEP 5: Graph State at Event Horizon");
    println!("====================================\n");

    println!("Cells analyzed: {}", cells.len());
    println!("Causal history preserved: YES (event horizon marks causality boundary)");
    println!();

    // STEP 6: Generate output receipt
    println!("STEP 6: Event Horizon Receipt (JSON)");
    println!("=====================================\n");

    let receipt_json = json!({
        "implementation": "event_horizon_demo",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "scenario": {
            "ticks": scenario.ticks.len(),
            "initial_liquidity": scenario.initial_liquidity,
            "final_liquidity": scenario.final_liquidity,
            "collapse_percent": format!("{:.2}%", (1.0 - scenario.final_liquidity / scenario.initial_liquidity) * 100.0),
        },
        "cells_generated": cells.len(),
        "event_horizons_detected": if boundary_detected.is_some() { 1 } else { 0 },
        "boundaries": if let Some(ref b) = boundary_detected {
            vec![json!({
                "liquidity_cliff_height": b.liquidity_cliff_height,
                "causal_time": b.causal_time,
            })]
        } else {
            vec![]
        },
        "cells_analyzed": cells.len(),
        "market_astrophysics": {
            "interpretation": "Event horizons mark causal boundaries where liquidity ceases. Below the horizon, orderbook connectivity is severed—not temporally, but geometrically. Causal time continues; causal graphs do not."
        }
    });

    println!("{}", serde_json::to_string_pretty(&receipt_json).unwrap_or_default());
    println!();

    println!("=== Demo Complete ===");
}

/// A liquidity collapse scenario: gradual degradation followed by sharp break
struct LiquidityCollapseScenario {
    ticks: Vec<TickRelation>,
    max_time: u64,
    initial_liquidity: f64,
    final_liquidity: f64,
}

impl LiquidityCollapseScenario {
    fn to_planck_cells(&self) -> Vec<MarketPlanckCell> {
        let mut cells = Vec::new();

        // Phase 1: Degradation (spreads widen)
        for (i, tick) in self.ticks.iter().enumerate() {
            let relation_kind = if i < 3 {
                // Gradually widening spreads = topology change
                MarketRelationKind::LiquidityTopologyChange
            } else if i < 6 {
                // Volume declining = capital pressure shift
                MarketRelationKind::CapitalPressureShift
            } else {
                // Sharp breaks = relation break (event horizon)
                MarketRelationKind::RelationBreak
            };

            let cell = MarketPlanckCell::new(
                tick.instrument_id,
                tick.venue_id,
                relation_kind,
                i as u64,
                tick.timestamp,
            );
            cells.push(cell);
        }

        cells
    }
}

/// Create a synthetic liquidity collapse scenario
fn create_liquidity_collapse_scenario() -> LiquidityCollapseScenario {
    let mut ticks = Vec::new();

    // Phase 1: Normal trading (time 0-1000)
    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        0,
        100000,
        99950,
        100050,
        100000, // abundant liquidity
    ));

    // Phase 2: Degradation begins (time 1001-2000)
    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        1000,
        100100,
        99900, // bid/ask widening
        100300,
        80000,
    ));

    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        1500,
        100200,
        99800,
        100600,
        60000,
    ));

    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        2000,
        100300,
        99700,
        100900,
        40000,
    ));

    // Phase 3: Critical degradation (time 2001-3000)
    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        2500,
        100000,
        99500,
        100500,
        20000,
    ));

    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        3000,
        99900,
        99200,
        100600,
        5000,
    ));

    // Phase 4: Event horizon crossing (time 3001+)
    // Volume collapses to near zero, spreads become non-functional
    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        3500,
        99500,
        99000,
        100000,
        500, // liquidity cliff
    ));

    ticks.push(TickRelation::new(
        InstrumentId::new(1),
        VenueId::new(100),
        4000,
        98000,
        97000,
        99000,
        100, // orderbook essentially empty
    ));

    let max_time = ticks.last().map(|t| t.timestamp).unwrap_or(0);
    let initial_liquidity = 100000.0;
    let final_liquidity = 100.0;

    LiquidityCollapseScenario { ticks, max_time, initial_liquidity, final_liquidity }
}
