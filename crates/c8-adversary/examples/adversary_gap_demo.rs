// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Adversary Gap Demo: demonstrating representation-blind logic player vs. graph-aware player
//!
//! This example shows:
//! 1. Two players observing the same market stream
//! 2. LogicPlayer: reasons over feature vectors only (bid-ask spreads, volumes, imbalances)
//! 3. GraphPlayer: reasons over graph topology + causal relations
//! 4. Result: GraphPlayer discovers game tree nodes that LogicPlayer cannot see
//! 5. Explanation: This is a representation advantage, not omniscience

use c8_adversary::*;
use c8_core::{InstrumentId, TickRelation, VenueId};
use c8_market::{MarketPlanckCell, MarketRelationKind};

fn main() {
    println!("=== Construct8 Adversarial Game Theory: Representation Gap Engine ===\n");

    // Create synthetic market stream
    println!("STEP 1: Create synthetic market stream");
    println!("----------------------------------------");

    let ticks = vec![
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1000, // timestamp
            1000, // mid
            999,  // bid
            1001, // ask
            5000, // volume
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1001,
            1000,
            995, // spread widens
            1005,
            3000,
        ),
        TickRelation::new(
            InstrumentId::new(1),
            VenueId::new(100),
            1002,
            1000,
            900, // liquidity break begins
            1100,
            1000,
        ),
    ];

    println!("Created {} ticks with widening spreads and liquidity degradation\n", ticks.len());

    // Create market Planck cells representing relation changes
    println!("STEP 2: Create Planck cells representing relation changes");
    println!("----------------------------------------------------------");

    let cells = vec![
        MarketPlanckCell::new(
            InstrumentId::new(1),
            VenueId::new(100),
            MarketRelationKind::CapitalPressureShift,
            0,
            1000,
        ),
        MarketPlanckCell::new(
            InstrumentId::new(1),
            VenueId::new(100),
            MarketRelationKind::LiquidityTopologyChange,
            1,
            1001,
        ),
        MarketPlanckCell::new(
            InstrumentId::new(1),
            VenueId::new(100),
            MarketRelationKind::RelationBreak,
            2,
            1002,
        ),
    ];

    println!("Created {} Planck cells representing relation changes", cells.len());
    println!("  - CapitalPressureShift (initial imbalance)");
    println!("  - LiquidityTopologyChange (depth reducing)");
    println!("  - RelationBreak (critical: venue connectivity lost)\n");

    // Run LogicPlayer on the ticks
    println!("STEP 3: Run LogicPlayer (feature-vector-only observation)");
    println!("----------------------------------------------------------");

    let mut logic_player = LogicPlayer::new("Logic-1");
    println!("Rules in use:");
    for rule in &logic_player.rules {
        println!("  - {}", rule);
    }
    println!();

    let mut logic_actions = Vec::new();
    for (i, tick) in ticks.iter().enumerate() {
        if let Some(action) = logic_player.observe(tick) {
            println!(
                "Tick {}: spread={}, mid={} => Action: {:?}",
                i,
                tick.spread(),
                tick.mid_price,
                action
            );
            logic_actions.push(action);
        }
    }
    println!();

    // Build logic game tree
    let logic_tree = logic_player.build_game_tree(2).expect("Failed to build logic tree");
    println!("LogicPlayer game tree: {} nodes (branching: 2x per depth)", logic_tree.len());
    println!();

    // Run GraphPlayer on the cells
    println!("STEP 4: Run GraphPlayer (graph-topology-aware observation)");
    println!("----------------------------------------------------------");

    let mut graph_player = GraphPlayer::new("Graph-1");

    let graph_actions: Vec<_> = cells
        .iter()
        .enumerate()
        .filter_map(|(i, cell)| {
            let delta = cell.to_construct8_delta().ok()?;
            let result = graph_player.observe(cell, &delta);
            match result {
                Ok(Some(action)) => {
                    println!("Cell {}: {:?} => Action: {:?}", i, cell.relation_kind, action);
                    Some(action)
                }
                _ => None,
            }
        })
        .collect();

    println!(
        "GraphPlayer observed {} Planck cells with causal histories",
        graph_player.planck_cells.len()
    );
    println!();

    // Build graph game tree
    let graph_tree = graph_player.build_game_tree(2).expect("Failed to build graph tree");
    println!("GraphPlayer game tree: {} nodes (branching: 5x per depth)", graph_tree.len());
    println!();

    // Find representation gap
    println!("STEP 5: Find representation gap");
    println!("--------------------------------");

    let gap = find_missing_state_basis(&logic_tree, &graph_tree)
        .expect("Failed to compute representation gap");

    println!("Missing nodes in LogicPlayer's tree: {}", gap.missing_logic_nodes.len());
    println!("Dimensionality delta: {} additional relation dimensions", gap.dimensionality_delta());
    println!();

    // Explain the prophecy illusion
    println!("STEP 6: Prophecy Illusion Explanation");
    println!("--------------------------------------");
    println!();

    let explanation = explain_prophecy_illusion(&gap);
    println!("{}", explanation);
    println!();

    // Summary
    println!("STEP 7: Summary");
    println!("---------------");
    println!();
    println!("Same market stream, two different observations:");
    println!("  - LogicPlayer: {} actions from {} feature ticks", logic_actions.len(), ticks.len());
    println!(
        "  - GraphPlayer: {} actions from {} relation cells",
        graph_actions.len(),
        cells.len()
    );
    println!();
    println!("Game tree sizes:");
    println!("  - LogicPlayer: {} nodes (limited by feature dims)", logic_tree.len());
    println!("  - GraphPlayer: {} nodes (enriched by topology)", graph_tree.len());
    println!();
    println!("Key insight: GraphPlayer's apparent 'omniscience' is actually observation of");
    println!("a higher-dimensional state space (graph relations + causality) that LogicPlayer");
    println!("cannot access. This is a REPRESENTATION ADVANTAGE, not a temporal advantage.");
    println!();
    println!("When LogicPlayer's features changed (spread widened, volume dropped), GraphPlayer");
    println!("had already seen the topology break in the causal graph. LogicPlayer appears to");
    println!("have missed something 'obvious' because it was operating in a lower-dimensional");
    println!("observation space.");
}
