// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use c8_graph::{C8Error, Construct8Delta, Construct8Len, Construct8Triple, GraphField};

#[test]
fn test_delta_construction_workflow() {
    // Arrange
    let mut delta = Construct8Delta::new();
    let triples = vec![
        Construct8Triple::new(1, 10, 100),
        Construct8Triple::new(2, 20, 200),
        Construct8Triple::new(3, 30, 300),
    ];

    // Act
    for triple in &triples {
        delta.push_checked(*triple).expect("push should succeed");
    }

    // Assert
    assert_eq!(delta.len(), Construct8Len::Three);
    assert_eq!(delta.triple_count(), 3);
    assert!(!delta.is_empty());
}

#[test]
fn test_graph_apply_and_query() {
    // Arrange
    let mut delta = Construct8Delta::new();
    delta.push_checked(Construct8Triple::new(1, 10, 100)).unwrap();
    delta.push_checked(Construct8Triple::new(1, 11, 101)).unwrap();
    delta.push_checked(Construct8Triple::new(2, 20, 200)).unwrap();

    let mut graph = GraphField::new();

    // Act
    let stats = graph.apply_construct8(&delta).expect("apply should succeed");

    // Assert
    assert_eq!(stats.applied, 3);
    assert_eq!(stats.total, 3);
    assert!(stats.is_complete());
    assert_eq!(graph.triple_count(), 3);

    // Query
    let subjects = graph.subjects();
    assert_eq!(subjects.len(), 2);

    let preds_for_1 = graph.predicates(1);
    assert_eq!(preds_for_1.len(), 2);

    let objs = graph.objects(1, 10);
    assert_eq!(objs, vec![100]);
}

#[test]
fn test_bounds_enforcement() {
    // Arrange
    let mut delta = Construct8Delta::new();
    let mut push_count = 0;

    // Act & Assert
    for i in 0..10 {
        let triple = Construct8Triple::new(i, i + 1, i + 2);
        match delta.push_checked(triple) {
            Ok(()) => push_count += 1,
            Err(C8Error::ExceedsConstruct8Max) => {
                // Expected after 8 triples
                assert_eq!(push_count, 8);
                assert!(i >= 8);
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    assert_eq!(push_count, 8);
    assert_eq!(delta.len(), Construct8Len::Eight);
    assert_eq!(delta.mask(), 0xFF);
}

#[test]
fn test_state_consistency_across_operations() {
    // Arrange
    let mut delta1 = Construct8Delta::new();
    let mut delta2 = Construct8Delta::new();

    let triple1 = Construct8Triple::new(5, 10, 15);
    let triple2 = Construct8Triple::new(20, 25, 30);

    // Act
    delta1.push_checked(triple1).unwrap();
    delta1.push_checked(triple2).unwrap();

    delta2.push_checked(triple1).unwrap();
    delta2.push_checked(triple2).unwrap();

    // Assert
    assert_eq!(delta1, delta2);
    assert_eq!(delta1.delta_hash(), delta2.delta_hash());
}

#[test]
fn test_graph_field_query_operations() {
    // Arrange
    let mut graph = GraphField::new();
    let triples = vec![
        Construct8Triple::new(1, 2, 3),
        Construct8Triple::new(1, 2, 4), // Same subject-predicate, different object
        Construct8Triple::new(1, 5, 6), // Same subject, different predicate
        Construct8Triple::new(10, 2, 3), // Different subject
    ];

    for triple in &triples {
        graph.add_triple(*triple);
    }

    // Act & Assert
    assert_eq!(graph.triple_count(), 4);

    let subjects = graph.subjects();
    assert_eq!(subjects.len(), 2);
    assert!(subjects.contains(&1));
    assert!(subjects.contains(&10));

    let predicates_of_1 = graph.predicates(1);
    assert_eq!(predicates_of_1.len(), 2);

    let objects_of_1_2 = graph.objects(1, 2);
    assert_eq!(objects_of_1_2.len(), 2);
    assert!(objects_of_1_2.contains(&3));
    assert!(objects_of_1_2.contains(&4));
}

#[test]
fn test_serialization_roundtrip() {
    // Arrange
    let mut delta = Construct8Delta::new();
    delta.push_checked(Construct8Triple::new(1, 2, 3)).unwrap();
    delta.push_checked(Construct8Triple::new(4, 5, 6)).unwrap();

    // Act
    let json = serde_json::to_string(&delta).expect("serialize should succeed");
    let deserialized: Construct8Delta =
        serde_json::from_str(&json).expect("deserialize should succeed");

    // Assert
    assert_eq!(delta, deserialized);
}

#[test]
fn test_graph_serialization_roundtrip() {
    // Arrange
    let mut graph = GraphField::new();
    graph.add_triple(Construct8Triple::new(1, 2, 3));
    graph.add_triple(Construct8Triple::new(4, 5, 6));

    let hash_before = graph.state_hash();

    // Act
    let json = serde_json::to_string(&graph).expect("serialize should succeed");
    let deserialized: GraphField = serde_json::from_str(&json).expect("deserialize should succeed");

    let hash_after = deserialized.state_hash();

    // Assert
    assert_eq!(hash_before, hash_after);
    assert_eq!(graph, deserialized);
}

#[test]
fn test_multiple_apply_operations() {
    // Arrange
    let mut delta = Construct8Delta::new();
    for i in 0..4 {
        delta.push_checked(Construct8Triple::new(i, i + 10, i + 100)).unwrap();
    }

    let mut graph = GraphField::new();

    // Act
    let result1 = graph.apply_construct8(&delta).expect("first apply");
    let result2 = graph.apply_construct8(&delta).expect("second apply");

    // Assert
    assert_eq!(result1.applied, 4);
    assert_eq!(result2.applied, 4);

    // Graph still has 4 unique triples (HashSet deduplicates)
    assert_eq!(graph.triple_count(), 4);
}

#[test]
fn test_delta_clear_operation() {
    // Arrange
    let mut delta = Construct8Delta::new();
    for i in 0..5 {
        delta.push_checked(Construct8Triple::new(i, i + 1, i + 2)).unwrap();
    }

    assert_eq!(delta.len(), Construct8Len::Five);
    assert_eq!(delta.triple_count(), 5);

    // Act
    delta.clear();

    // Assert
    assert_eq!(delta.len(), Construct8Len::Zero);
    assert_eq!(delta.triple_count(), 0);
    assert!(delta.is_empty());
    assert_eq!(delta.mask(), 0);
}

#[test]
fn test_mask_bit_pattern() {
    // Arrange
    let mut delta = Construct8Delta::new();

    // Expected mask values for each iteration: 1, 3, 7, 15, 31, 63, 127, 255
    let expected_masks = [0x01u8, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x7F, 0xFF];

    // Act & Assert
    for (i, &expected_mask) in expected_masks.iter().enumerate() {
        delta.push_checked(Construct8Triple::new(i as u64, i as u64 + 1, i as u64 + 2)).unwrap();
        assert_eq!(delta.mask(), expected_mask, "Mask at iteration {}", i);
    }
}

#[test]
fn test_graph_complex_queries() {
    // Arrange - Create a small RDF graph
    let mut graph = GraphField::new();

    // Subject 1: types, properties
    graph.add_triple(Construct8Triple::new(1, 100, 200)); // subject 1, predicate 100, object 200
    graph.add_triple(Construct8Triple::new(1, 100, 201)); // Multiple objects for same pred
    graph.add_triple(Construct8Triple::new(1, 101, 300));

    // Subject 2
    graph.add_triple(Construct8Triple::new(2, 100, 200));

    // Act & Assert
    assert_eq!(graph.subjects().len(), 2);
    assert_eq!(graph.objects(1, 100).len(), 2);
    assert_eq!(graph.objects(2, 100).len(), 1);
    assert_eq!(graph.objects(999, 999).len(), 0); // Non-existent

    // Triple count with HashSet (no duplicates)
    assert_eq!(graph.triple_count(), 4);
}

#[test]
fn test_triple_hash_properties() {
    // Arrange
    let t1 = Construct8Triple::new(1, 2, 3);
    let t2 = Construct8Triple::new(1, 2, 3);
    let t3 = Construct8Triple::new(1, 2, 4);

    // Act & Assert
    assert_eq!(t1.hash(), t2.hash());
    assert_ne!(t1.hash(), t3.hash());
}

#[test]
fn test_construct8_len_ordering() {
    // Arrange & Act & Assert
    assert!(Construct8Len::Zero < Construct8Len::One);
    assert!(Construct8Len::Four < Construct8Len::Eight);
    assert!(Construct8Len::Eight <= Construct8Len::Eight);
}

#[test]
fn test_empty_graph_operations() {
    // Arrange
    let graph = GraphField::new();

    // Act & Assert
    assert_eq!(graph.subjects().len(), 0);
    assert_eq!(graph.predicates(1).len(), 0);
    assert_eq!(graph.objects(1, 2).len(), 0);
    assert_eq!(graph.triple_count(), 0);

    let empty_triple = Construct8Triple::new(99, 88, 77);
    assert!(!graph.contains_triple(&empty_triple));
}
