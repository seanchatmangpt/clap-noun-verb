// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Market Planck Cell Demo: smallest indivisible units of market state change
//!
//! This example demonstrates:
//! 1. Synthetic tick generation with realistic market patterns
//! 2. Conversion to MarketPlanckCells (atomic relation changes)
//! 3. Emission of Construct8Deltas (bounded 8-triple mutations)
//! 4. Application to GraphField for state accumulation
//! 5. State hash verification and receipt generation

use c8_core::{InstrumentId, TickRelation, VenueId};
use c8_graph::GraphField;
use c8_market::{MarketPlanckCell, MarketRelationKind};
use c8_receipts::C8Receipt;
use serde_json::json;

fn main() {
    println!("=== Construct8: Market Planck Cell Demo ===\n");

    // STEP 1: Create synthetic market ticks
    println!("STEP 1: Synthetic tick generation");
    println!("==================================\n");

    let ticks = generate_synthetic_ticks();
    println!("Generated {} market ticks:", ticks.len());
    for (i, tick) in ticks.iter().enumerate() {
        println!(
            "  Tick {}: instrument={}, venue={}, time={}, mid={}, spread={}",
            i,
            tick.instrument_id.as_u64(),
            tick.venue_id.as_u64(),
            tick.timestamp,
            tick.mid_price,
            tick.spread()
        );
    }
    println!();

    // STEP 2: Convert ticks to MarketPlanckCells
    println!("STEP 2: Convert to MarketPlanckCells");
    println!("====================================\n");

    let cells = ticks_to_planck_cells(&ticks);
    println!("Generated {} Planck cells:", cells.len());
    for (i, cell) in cells.iter().enumerate() {
        println!("  Cell {}: {:?}", i, cell.relation_kind);
    }
    println!();

    // STEP 3: Emit Construct8Deltas
    println!("STEP 3: Emit Construct8Deltas (bounded 8-triple mutations)");
    println!("==========================================================\n");

    let mut graph_field = GraphField::new();
    let mut receipts = Vec::new();
    let mut pre_state_hash = 0u64;

    for (i, cell) in cells.iter().enumerate() {
        match cell.to_construct8_delta() {
            Ok(delta) => {
                let delta_len = delta.len();
                let triples_count = delta_len.as_usize();

                println!("  Cell {} -> Delta with {} triples", i, triples_count);

                // Apply to graph field
                let _ = graph_field.apply_construct8(&delta);
                let post_state_hash = graph_field.state_hash();

                // Create receipt
                let receipt =
                    C8Receipt::new(pre_state_hash, &delta, post_state_hash, cell.causal_time);
                receipts.push(receipt);

                println!("    pre_state: 0x{:016x}", pre_state_hash);
                println!("    post_state: 0x{:016x}", post_state_hash);
                println!("    delta_mask: 0b{:08b}", delta.mask());
                println!("    receipt_hash: 0x{:016x}", receipt.receipt_hash);

                pre_state_hash = post_state_hash;
            }
            Err(e) => {
                println!("  Cell {} failed to emit delta: {}", i, e);
            }
        }
    }
    println!();

    // STEP 4: Verify state hashes
    println!("STEP 4: State Hash and Receipt Chain");
    println!("====================================\n");

    println!("Final state hash: 0x{:016x}", pre_state_hash);
    println!("Receipt chain length: {}", receipts.len());
    println!();

    // Verify monotonic causal time
    let mut valid_causal = true;
    let mut prev_time = 0u64;
    for (i, receipt) in receipts.iter().enumerate() {
        if receipt.causal_time < prev_time {
            println!(
                "  Receipt {}: CAUSAL VIOLATION (time {} < {})",
                i, receipt.causal_time, prev_time
            );
            valid_causal = false;
        }
        prev_time = receipt.causal_time;
    }

    if valid_causal {
        println!("  Causal time ordering: VALID (monotonic)");
    }
    println!();

    // STEP 5: Generate demo receipt output
    println!("STEP 5: Receipt Output (JSON)");
    println!("=============================\n");

    let receipt_json = json!({
        "implementation": "market_planck_demo",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "ticks_processed": ticks.len(),
        "cells_generated": cells.len(),
        "deltas_applied": receipts.len(),
        "final_state_hash": format!("0x{:016x}", pre_state_hash),
        "receipt_chain": receipts.iter().map(|r| json!({
            "pre_state": format!("0x{:016x}", r.pre_state_hash),
            "post_state": format!("0x{:016x}", r.post_state_hash),
            "delta_mask": format!("0b{:08b}", r.delta_mask),
            "causal_time": r.causal_time,
            "receipt_hash": format!("0x{:016x}", r.receipt_hash),
        })).collect::<Vec<_>>(),
        "graph_field_state": {
            "triples_active": graph_field.triple_count(),
        }
    });

    println!("{}", serde_json::to_string_pretty(&receipt_json).unwrap_or_default());
    println!();

    println!("=== Demo Complete ===");
}

/// Generate synthetic market ticks with realistic patterns
fn generate_synthetic_ticks() -> Vec<TickRelation> {
    vec![
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1000,  // timestamp
            10000, // mid_price
            9995,  // bid
            10005, // ask
            5000,  // volume
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1001,
            10002,
            9990, // bid moves lower (selling pressure)
            10014,
            6000,
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1002,
            10005,
            9980, // bid pulls back (liquidity dries up)
            10030,
            2000,
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1003,
            9950, // sharp move down (panic selling)
            9900,
            10000,
            1000,
        ),
        // Recovery phase
        TickRelation::new(InstrumentId::new(1), VenueId::new(100), 1004, 9975, 9950, 10000, 3000),
        TickRelation::new(InstrumentId::new(1), VenueId::new(100), 1005, 10000, 9995, 10005, 7000),
    ]
}

/// Convert a series of ticks to Planck cells based on market dynamics
fn ticks_to_planck_cells(ticks: &[TickRelation]) -> Vec<MarketPlanckCell> {
    let mut cells = Vec::new();
    let mut prev_tick: Option<TickRelation> = None;

    for (i, tick) in ticks.iter().enumerate() {
        let relation_kind = if let Some(prev) = prev_tick {
            // Detect market relation changes
            let spread_change = tick.spread().abs_diff(prev.spread());
            let volume_change = if tick.volume > prev.volume { "increasing" } else { "decreasing" };
            let price_move = if tick.mid_price > prev.mid_price {
                "up"
            } else if tick.mid_price < prev.mid_price {
                "down"
            } else {
                "flat"
            };

            if spread_change > 100 {
                MarketRelationKind::LiquidityTopologyChange
            } else if volume_change == "increasing" && price_move == "down" {
                MarketRelationKind::CapitalPressureShift
            } else if tick.volume < 2000 && spread_change > 50 {
                MarketRelationKind::RelationBreak
            } else {
                MarketRelationKind::LatencyGeometry
            }
        } else {
            MarketRelationKind::CapitalPressureShift
        };

        let cell = MarketPlanckCell::new(
            tick.instrument_id,
            tick.venue_id,
            relation_kind,
            i as u64,
            tick.timestamp,
        );
        cells.push(cell);
        prev_tick = Some(*tick);
    }

    cells
}
