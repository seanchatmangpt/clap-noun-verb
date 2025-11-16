#![no_main]

use libfuzzer_sys::fuzz_target;
use clap_noun_verb::autonomic::{
    CapabilityGraph, CapabilityId, EdgeType, InputSchema, OutputSchema, PrimitiveType, TypeSchema,
};

fuzz_target!(|data: &[u8]| {
    // Limit input size to prevent excessive memory usage
    if data.len() > 512 || data.len() < 4 {
        return;
    }

    // Use data to drive graph operations
    let mut graph = CapabilityGraph::new();

    // Extract operation count (max 20 to prevent timeouts)
    let op_count = (data[0] as usize % 20).min(data.len() / 4);

    let mut nodes = Vec::new();

    // Create nodes based on fuzzed data
    for i in 0..op_count {
        let capability_name = format!("cap_{}", i);
        let node_id = graph.add_node(
            CapabilityId::from_path(&capability_name),
            &format!("Node {}", i),
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );
        nodes.push(node_id);
    }

    // Create edges based on fuzzed data
    for i in 0..op_count.saturating_sub(1) {
        if nodes.len() >= 2 {
            let idx = i * 4;
            if idx + 3 < data.len() {
                let from_idx = (data[idx] as usize) % nodes.len();
                let to_idx = (data[idx + 1] as usize) % nodes.len();

                if from_idx != to_idx {
                    let edge_type = match data[idx + 2] % 3 {
                        0 => EdgeType::Produces,
                        1 => EdgeType::Requires,
                        _ => EdgeType::Composes,
                    };

                    let _ = graph.add_edge(nodes[from_idx], nodes[to_idx], edge_type);
                }
            }
        }
    }

    // Test graph operations don't panic
    if nodes.len() >= 2 {
        let _ = graph.is_reachable(nodes[0], nodes[nodes.len() - 1]);
        let _ = graph.shortest_path(nodes[0], nodes[nodes.len() - 1]);
        let _ = graph.stats();
    }

    // Verify graph invariants
    let stats = graph.stats();
    assert!(
        stats.node_count >= nodes.len(),
        "Graph should contain at least the nodes we added"
    );
});
