//! Comprehensive tests for capability graph and schema composition
//!
//! Critical 80/20 test coverage:
//! - Node and edge creation
//! - Reachability queries
//! - Shortest path finding
//! - Minimal composition
//! - Dominance and equivalence
//! - Graph traversal algorithms

use clap_noun_verb::autonomic::*;
use std::collections::HashMap;

#[test]
fn test_graph_creation() {
    // GIVEN: A new graph
    let mut graph = CapabilityGraph::new();

    // THEN: It's initially empty
    let stats = graph.stats();
    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
}

#[test]
fn test_add_nodes() {
    // GIVEN: A graph
    let mut graph = CapabilityGraph::new();

    // WHEN: We add nodes
    let n1 = graph.add_node(
        CapabilityId::from_path("user.create"),
        "Create User",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n2 = graph.add_node(
        CapabilityId::from_path("user.read"),
        "Read User",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // THEN: Nodes are created with sequential IDs
    assert!(n1.raw() < n2.raw());

    // AND: Stats reflect additions
    let stats = graph.stats();
    assert_eq!(stats.node_count, 2);
}

#[test]
fn test_add_edges() {
    // GIVEN: A graph with nodes
    let mut graph = CapabilityGraph::new();

    let n1 = graph.add_node(
        CapabilityId::from_path("data.fetch"),
        "Fetch Data",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n2 = graph.add_node(
        CapabilityId::from_path("data.transform"),
        "Transform Data",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // WHEN: We add an edge
    let edge = graph.add_edge(n1, n2, EdgeType::Produces).expect("Edge creation should succeed");

    // THEN: Edge is created
    assert!(graph.get_edge(edge).is_some());

    // AND: Stats reflect addition
    let stats = graph.stats();
    assert_eq!(stats.edge_count, 1);
}

#[test]
fn test_add_edge_to_nonexistent_node() {
    // GIVEN: A graph with one node
    let mut graph = CapabilityGraph::new();

    let n1 = graph.add_node(
        CapabilityId::from_path("node1"),
        "Node 1",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // WHEN: We try to add edge to nonexistent node
    let nonexistent = NodeId::new(9999);
    let result = graph.add_edge(n1, nonexistent, EdgeType::Produces);

    // THEN: Edge creation fails
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GraphError::NodeNotFound(_)));
}

#[test]
fn test_get_node_by_capability() {
    // GIVEN: A graph with a node
    let mut graph = CapabilityGraph::new();

    let cap_id = CapabilityId::from_path("unique.operation");

    graph.add_node(
        cap_id.clone(),
        "Unique Operation",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // WHEN: We query by capability ID
    let node = graph.get_node_by_capability(&cap_id);

    // THEN: Node is found
    assert!(node.is_some());
    assert_eq!(node.unwrap().capability_id, cap_id);
}

#[test]
fn test_outgoing_and_incoming_edges() {
    // GIVEN: A graph with nodes and edges
    let mut graph = CapabilityGraph::new();

    let n1 = graph.add_node(
        CapabilityId::from_path("a"),
        "A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n2 = graph.add_node(
        CapabilityId::from_path("b"),
        "B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n3 = graph.add_node(
        CapabilityId::from_path("c"),
        "C",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    graph.add_edge(n1, n2, EdgeType::Produces).unwrap();
    graph.add_edge(n2, n3, EdgeType::Produces).unwrap();

    // WHEN: We query outgoing edges
    let outgoing_n1 = graph.outgoing_edges(n1);
    let outgoing_n2 = graph.outgoing_edges(n2);

    // THEN: Outgoing edges are correct
    assert_eq!(outgoing_n1.len(), 1);
    assert_eq!(outgoing_n2.len(), 1);

    // AND: Incoming edges are correct
    let incoming_n2 = graph.incoming_edges(n2);
    let incoming_n3 = graph.incoming_edges(n3);

    assert_eq!(incoming_n2.len(), 1);
    assert_eq!(incoming_n3.len(), 1);
}

#[test]
fn test_reachability_simple() {
    // GIVEN: A linear graph A -> B -> C
    let mut graph = CapabilityGraph::new();

    let a = graph.add_node(
        CapabilityId::from_path("a"),
        "A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let b = graph.add_node(
        CapabilityId::from_path("b"),
        "B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let c = graph.add_node(
        CapabilityId::from_path("c"),
        "C",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    graph.add_edge(a, b, EdgeType::Produces).unwrap();
    graph.add_edge(b, c, EdgeType::Produces).unwrap();

    // THEN: A can reach C
    assert!(graph.is_reachable(a, c));

    // AND: C cannot reach A (no reverse edges)
    assert!(!graph.is_reachable(c, a));

    // AND: B can reach C
    assert!(graph.is_reachable(b, c));
}

#[test]
fn test_reachability_self() {
    // GIVEN: Any node
    let mut graph = CapabilityGraph::new();

    let n = graph.add_node(
        CapabilityId::from_path("node"),
        "Node",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // THEN: Node is reachable from itself
    assert!(graph.is_reachable(n, n));
}

#[test]
fn test_shortest_path_linear() {
    // GIVEN: A linear graph A -> B -> C -> D
    let mut graph = CapabilityGraph::new();

    let a = graph.add_node(
        CapabilityId::from_path("a"),
        "A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let b = graph.add_node(
        CapabilityId::from_path("b"),
        "B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let c = graph.add_node(
        CapabilityId::from_path("c"),
        "C",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let d = graph.add_node(
        CapabilityId::from_path("d"),
        "D",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    graph.add_edge(a, b, EdgeType::Produces).unwrap();
    graph.add_edge(b, c, EdgeType::Produces).unwrap();
    graph.add_edge(c, d, EdgeType::Produces).unwrap();

    // WHEN: We find shortest path from A to D
    let path = graph.shortest_path(a, d).expect("Path should exist");

    // THEN: Path is A -> B -> C -> D
    assert_eq!(path.len(), 4);
    assert_eq!(path[0], a);
    assert_eq!(path[1], b);
    assert_eq!(path[2], c);
    assert_eq!(path[3], d);
}

#[test]
fn test_shortest_path_with_shortcut() {
    // GIVEN: A graph with multiple paths
    //   A -> B -> C
    //   A -----> C (shortcut)
    let mut graph = CapabilityGraph::new();

    let a = graph.add_node(
        CapabilityId::from_path("a"),
        "A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let b = graph.add_node(
        CapabilityId::from_path("b"),
        "B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let c = graph.add_node(
        CapabilityId::from_path("c"),
        "C",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    graph.add_edge(a, b, EdgeType::Produces).unwrap();
    graph.add_edge(b, c, EdgeType::Produces).unwrap();
    graph.add_edge(a, c, EdgeType::Produces).unwrap(); // Shortcut

    // WHEN: We find shortest path from A to C
    let path = graph.shortest_path(a, c).expect("Path should exist");

    // THEN: Shortcut is taken (A -> C directly)
    assert_eq!(path.len(), 2);
    assert_eq!(path[0], a);
    assert_eq!(path[1], c);
}

#[test]
fn test_shortest_path_no_path() {
    // GIVEN: Two disconnected nodes
    let mut graph = CapabilityGraph::new();

    let a = graph.add_node(
        CapabilityId::from_path("a"),
        "A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let b = graph.add_node(
        CapabilityId::from_path("b"),
        "B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // WHEN: We try to find path
    let path = graph.shortest_path(a, b);

    // THEN: No path exists
    assert!(path.is_none());
}

#[test]
fn test_find_all_paths() {
    // GIVEN: A diamond graph
    //      A
    //     / \
    //    B   C
    //     \ /
    //      D
    let mut graph = CapabilityGraph::new();

    let a = graph.add_node(
        CapabilityId::from_path("a"),
        "A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let b = graph.add_node(
        CapabilityId::from_path("b"),
        "B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let c = graph.add_node(
        CapabilityId::from_path("c"),
        "C",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let d = graph.add_node(
        CapabilityId::from_path("d"),
        "D",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    graph.add_edge(a, b, EdgeType::Produces).unwrap();
    graph.add_edge(a, c, EdgeType::Produces).unwrap();
    graph.add_edge(b, d, EdgeType::Produces).unwrap();
    graph.add_edge(c, d, EdgeType::Produces).unwrap();

    // WHEN: We find all paths from A to D
    let paths = graph.find_all_paths(a, d, 10);

    // THEN: Two paths exist: A->B->D and A->C->D
    assert_eq!(paths.len(), 2);

    // Verify each path
    for path in &paths {
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], a);
        assert_eq!(path[2], d);
        assert!(path[1] == b || path[1] == c);
    }
}

#[test]
fn test_dominance_via_effects() {
    // GIVEN: Two nodes with different effects
    let mut graph = CapabilityGraph::new();

    let powerful = graph.add_node(
        CapabilityId::from_path("powerful"),
        "Powerful",
        InputSchema::default(),
        OutputSchema::default(),
        vec![
            EffectMetadata {
                effect_type: EffectType::ReadOnly,
                sensitivity: Sensitivity::Low,
                idempotent: true,
                required_role: None,
                data_sensitivity_tags: vec![],
                isolation_requirement: IsolationRequirement::Shared,
            },
            EffectMetadata {
                effect_type: EffectType::MutateState,
                sensitivity: Sensitivity::Medium,
                idempotent: false,
                required_role: Some("admin".to_string()),
                data_sensitivity_tags: vec![],
                isolation_requirement: IsolationRequirement::Isolated,
            },
        ],
    );

    let basic = graph.add_node(
        CapabilityId::from_path("basic"),
        "Basic",
        InputSchema::default(),
        OutputSchema::default(),
        vec![EffectMetadata {
            effect_type: EffectType::ReadOnly,
            sensitivity: Sensitivity::Low,
            idempotent: true,
            required_role: None,
            data_sensitivity_tags: vec![],
            isolation_requirement: IsolationRequirement::Shared,
        }],
    );

    // THEN: Powerful dominates basic (has all its effects and more)
    assert!(graph.dominates(powerful, basic));

    // AND: Basic doesn't dominate powerful
    assert!(!graph.dominates(basic, powerful));
}

#[test]
fn test_equivalence_via_explicit_edge() {
    // GIVEN: Two nodes with equivalence edge
    let mut graph = CapabilityGraph::new();

    let n1 = graph.add_node(
        CapabilityId::from_path("impl1"),
        "Implementation 1",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n2 = graph.add_node(
        CapabilityId::from_path("impl2"),
        "Implementation 2",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    graph.add_edge(n1, n2, EdgeType::Equivalent).unwrap();

    // THEN: They are equivalent
    assert!(graph.are_equivalent(n1, n2));
}

#[test]
fn test_find_equivalence_classes() {
    // GIVEN: A graph with multiple equivalent nodes
    let mut graph = CapabilityGraph::new();

    let a = graph.add_node(
        CapabilityId::from_path("a"),
        "A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let b = graph.add_node(
        CapabilityId::from_path("b"),
        "B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let c = graph.add_node(
        CapabilityId::from_path("c"),
        "C",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let d = graph.add_node(
        CapabilityId::from_path("d"),
        "D",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // A and B are equivalent
    graph.add_edge(a, b, EdgeType::Equivalent).unwrap();

    // C and D are equivalent
    graph.add_edge(c, d, EdgeType::Equivalent).unwrap();

    // WHEN: We find equivalence classes
    let classes = graph.find_equivalence_classes();

    // THEN: Two classes exist
    assert!(classes.len() >= 2);

    // Find the classes containing A and C
    let class_a = classes.iter().find(|c| c.contains(&a)).unwrap();
    let class_c = classes.iter().find(|c| c.contains(&c)).unwrap();

    // Verify class membership
    assert!(class_a.contains(&b));
    assert!(class_c.contains(&d));
}

#[test]
fn test_graph_stats() {
    // GIVEN: A populated graph
    let mut graph = CapabilityGraph::new();

    for i in 0..5 {
        graph.add_node(
            CapabilityId::from_path(&format!("node{}", i)),
            format!("Node {}", i),
            InputSchema::default(),
            OutputSchema::default(),
            vec![],
        );
    }

    let nodes: Vec<NodeId> = (0..5).map(NodeId::new).collect();

    // Create a chain
    for i in 0..4 {
        graph.add_edge(nodes[i], nodes[i + 1], EdgeType::Produces).ok();
    }

    // WHEN: We get stats
    let stats = graph.stats();

    // THEN: Stats are correct
    assert_eq!(stats.node_count, 5);
    assert_eq!(stats.edge_count, 4);
    assert!(stats.avg_degree > 0.0);
    assert!(stats.max_depth > 0);
}

#[test]
fn test_graph_dot_export() {
    // GIVEN: A simple graph
    let mut graph = CapabilityGraph::new();

    let a = graph.add_node(
        CapabilityId::from_path("a"),
        "Node A",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let b = graph.add_node(
        CapabilityId::from_path("b"),
        "Node B",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    graph.add_edge(a, b, EdgeType::Produces).unwrap();

    // WHEN: We export as DOT
    let dot = graph.export_dot();

    // THEN: Output contains graph structure
    assert!(dot.contains("digraph CapabilityGraph"));
    assert!(dot.contains("Node A"));
    assert!(dot.contains("Node B"));
    assert!(dot.contains("->"));
}

#[test]
fn test_edge_types() {
    // GIVEN: A graph with different edge types
    let mut graph = CapabilityGraph::new();

    let n1 = graph.add_node(
        CapabilityId::from_path("n1"),
        "N1",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n2 = graph.add_node(
        CapabilityId::from_path("n2"),
        "N2",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n3 = graph.add_node(
        CapabilityId::from_path("n3"),
        "N3",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    let n4 = graph.add_node(
        CapabilityId::from_path("n4"),
        "N4",
        InputSchema::default(),
        OutputSchema::default(),
        vec![],
    );

    // WHEN: We add edges of different types
    let e1 = graph.add_edge(n1, n2, EdgeType::Produces).unwrap();
    let e2 = graph.add_edge(n2, n3, EdgeType::Requires).unwrap();
    let e3 = graph.add_edge(n3, n4, EdgeType::Dominates).unwrap();

    // THEN: Edge types are preserved
    assert_eq!(graph.get_edge(e1).unwrap().edge_type, EdgeType::Produces);
    assert_eq!(graph.get_edge(e2).unwrap().edge_type, EdgeType::Requires);
    assert_eq!(graph.get_edge(e3).unwrap().edge_type, EdgeType::Dominates);
}
