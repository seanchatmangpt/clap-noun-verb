// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construct8 Delta Engine - bounded graph mutation for RDF triples
//!
//! This crate implements a fixed-capacity (8-triple) delta container for efficient
//! graph mutations with bounded memory and guaranteed stack allocation.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

/// Maximum number of triples in a Construct8Delta
pub const CONSTRUCT8_MAX: usize = 8;

/// Error types for Construct8 operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum C8Error {
    #[error("Delta exceeds Construct8 maximum of 8 triples")]
    ExceedsConstruct8Max,

    #[error("Invalid state hash comparison")]
    InvalidStateHash,

    #[error("Graph operation failed: {0}")]
    GraphOperationFailed(String),
}

/// Result type for Construct8 operations
pub type C8Result<T> = Result<T, C8Error>;

/// Length representation for Construct8 triples (0-8)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Construct8Len {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

impl Construct8Len {
    /// Convert to usize
    pub fn as_usize(self) -> usize {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
        }
    }

    /// Try to increment length
    pub fn increment(self) -> C8Result<Self> {
        match self {
            Self::Zero => Ok(Self::One),
            Self::One => Ok(Self::Two),
            Self::Two => Ok(Self::Three),
            Self::Three => Ok(Self::Four),
            Self::Four => Ok(Self::Five),
            Self::Five => Ok(Self::Six),
            Self::Six => Ok(Self::Seven),
            Self::Seven => Ok(Self::Eight),
            Self::Eight => Err(C8Error::ExceedsConstruct8Max),
        }
    }
}

impl From<Construct8Len> for usize {
    fn from(len: Construct8Len) -> Self {
        len.as_usize()
    }
}

/// An RDF triple with subject, predicate, and object as u64 identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Construct8Triple {
    pub subject: u64,
    pub predicate: u64,
    pub object: u64,
}

impl Construct8Triple {
    /// Create a new triple
    pub fn new(subject: u64, predicate: u64, object: u64) -> Self {
        Self { subject, predicate, object }
    }

    /// Hash the triple for state tracking
    pub fn hash(&self) -> u64 {
        let mut result = 0u64;
        result = result.wrapping_mul(31).wrapping_add(self.subject);
        result = result.wrapping_mul(31).wrapping_add(self.predicate);
        result = result.wrapping_mul(31).wrapping_add(self.object);
        result
    }
}

/// Statistics for graph apply operations
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphApplyStats {
    pub applied: usize,
    pub skipped: usize,
    pub total: usize,
}

impl GraphApplyStats {
    /// Create a new stats struct
    pub fn new() -> Self {
        Self { applied: 0, skipped: 0, total: 0 }
    }

    /// Check if apply was successful (all triples applied)
    pub fn is_complete(&self) -> bool {
        self.applied == self.total
    }
}

/// Bounded delta container for 8 RDF triples
///
/// Fixed-capacity (8 triples) container with O(1) insertion, O(1) space,
/// and stack-allocated storage for guaranteed performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Construct8Delta {
    triples: [Option<Construct8Triple>; CONSTRUCT8_MAX],
    len: Construct8Len,
    mask: u8,
}

impl Construct8Delta {
    /// Create a new empty delta
    pub fn new() -> Self {
        Self { triples: [None; CONSTRUCT8_MAX], len: Construct8Len::Zero, mask: 0u8 }
    }

    /// Get the current length
    pub fn len(&self) -> Construct8Len {
        self.len
    }

    /// Check if delta is empty
    pub fn is_empty(&self) -> bool {
        self.len == Construct8Len::Zero
    }

    /// Get the current mask (bit set for each occupied slot)
    pub fn mask(&self) -> u8 {
        self.mask
    }

    /// Push a triple to the delta with bounds checking
    pub fn push_checked(&mut self, triple: Construct8Triple) -> C8Result<()> {
        match self.len {
            Construct8Len::Eight => Err(C8Error::ExceedsConstruct8Max),
            _ => {
                let idx = self.len.as_usize();
                self.triples[idx] = Some(triple);
                self.mask |= 1u8 << idx;
                self.len = self.len.increment()?;
                Ok(())
            }
        }
    }

    /// Push multiple triples up to capacity
    pub fn push_multiple(&mut self, triples: &[Construct8Triple]) -> C8Result<()> {
        for triple in triples {
            self.push_checked(*triple)?;
        }
        Ok(())
    }

    /// Access the fixed slots array
    pub fn as_fixed_slots(&self) -> &[Option<Construct8Triple>; CONSTRUCT8_MAX] {
        &self.triples
    }

    /// Get an iterator over populated triples
    pub fn iter(&self) -> impl Iterator<Item = &Construct8Triple> {
        self.triples.iter().filter_map(|opt| opt.as_ref())
    }

    /// Get the number of triples in the delta
    pub fn triple_count(&self) -> usize {
        self.len.as_usize()
    }

    /// Compute a hash of all triples in the delta
    pub fn delta_hash(&self) -> u64 {
        let mut result = 0u64;
        for triple in self.iter() {
            result = result.wrapping_mul(31).wrapping_add(triple.hash());
        }
        result
    }

    /// Clear the delta
    pub fn clear(&mut self) {
        self.triples = [None; CONSTRUCT8_MAX];
        self.len = Construct8Len::Zero;
        self.mask = 0u8;
    }
}

impl Default for Construct8Delta {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Construct8Delta {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.mask == other.mask && self.triples == other.triples
    }
}

impl Eq for Construct8Delta {}

/// Graph field with RDF relations
///
/// Manages a set of RDF relations (subject->predicate->object triples)
/// with support for applying Construct8 deltas and computing state hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphField {
    relations: BTreeMap<u64, BTreeMap<u64, HashSet<u64>>>,
}

impl GraphField {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self { relations: BTreeMap::new() }
    }

    /// Apply a Construct8Delta to the graph
    pub fn apply_construct8(&mut self, delta: &Construct8Delta) -> C8Result<GraphApplyStats> {
        let mut stats = GraphApplyStats::new();
        stats.total = delta.len.as_usize();

        for triple in delta.iter() {
            self.add_triple(*triple);
            stats.applied += 1;
        }

        Ok(stats)
    }

    /// Add a single triple to the graph
    pub fn add_triple(&mut self, triple: Construct8Triple) {
        self.relations
            .entry(triple.subject)
            .or_default()
            .entry(triple.predicate)
            .or_default()
            .insert(triple.object);
    }

    /// Check if a triple exists
    pub fn contains_triple(&self, triple: &Construct8Triple) -> bool {
        self.relations
            .get(&triple.subject)
            .and_then(|preds| preds.get(&triple.predicate))
            .map(|objs| objs.contains(&triple.object))
            .unwrap_or(false)
    }

    /// Get the number of triples in the graph
    pub fn triple_count(&self) -> usize {
        self.relations.values().flat_map(|preds| preds.values()).map(|objs| objs.len()).sum()
    }

    /// Compute a hash of the current graph state
    pub fn state_hash(&self) -> u64 {
        let mut result = 0u64;
        for (subject, predicates) in &self.relations {
            result = result.wrapping_mul(31).wrapping_add(*subject);
            for (predicate, objects) in predicates {
                result = result.wrapping_mul(31).wrapping_add(*predicate);
                for object in objects {
                    result = result.wrapping_mul(31).wrapping_add(*object);
                }
            }
        }
        result
    }

    /// Apply the same delta multiple times (idempotency check)
    pub fn apply_multiple(&mut self, delta: &Construct8Delta, times: usize) -> C8Result<()> {
        for _ in 0..times {
            self.apply_construct8(delta)?;
        }
        Ok(())
    }

    /// Get all subjects in the graph
    pub fn subjects(&self) -> Vec<u64> {
        self.relations.keys().copied().collect()
    }

    /// Get all predicates for a subject
    pub fn predicates(&self, subject: u64) -> Vec<u64> {
        self.relations
            .get(&subject)
            .map(|preds| preds.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Get all objects for a subject-predicate pair
    pub fn objects(&self, subject: u64, predicate: u64) -> Vec<u64> {
        self.relations
            .get(&subject)
            .and_then(|preds| preds.get(&predicate))
            .map(|objs| objs.iter().copied().collect())
            .unwrap_or_default()
    }
}

impl Default for GraphField {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for GraphField {
    fn eq(&self, other: &Self) -> bool {
        self.state_hash() == other.state_hash()
    }
}

impl Eq for GraphField {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_delta_has_len_0() {
        let delta = Construct8Delta::new();
        assert_eq!(delta.len(), Construct8Len::Zero);
        assert_eq!(delta.mask(), 0u8);
        assert!(delta.is_empty());
    }

    #[test]
    fn test_one_triple_sets_one_mask_bit() {
        let mut delta = Construct8Delta::new();
        let triple = Construct8Triple::new(1, 2, 3);

        delta.push_checked(triple).expect("should push triple");

        assert_eq!(delta.len(), Construct8Len::One);
        assert_eq!(delta.mask(), 0b0000_0001);
        assert!(!delta.is_empty());
    }

    #[test]
    fn test_eight_triples_succeed() {
        let mut delta = Construct8Delta::new();

        for i in 0..8 {
            let triple = Construct8Triple::new(i, i + 1, i + 2);
            delta.push_checked(triple).expect("should push triple");
        }

        assert_eq!(delta.len(), Construct8Len::Eight);
        assert_eq!(delta.mask(), 0b1111_1111);
    }

    #[test]
    fn test_ninth_triple_refuses_with_error() {
        let mut delta = Construct8Delta::new();

        for i in 0..8 {
            let triple = Construct8Triple::new(i, i + 1, i + 2);
            delta.push_checked(triple).expect("should push triple");
        }

        let ninth = Construct8Triple::new(99, 100, 101);
        let result = delta.push_checked(ninth);

        assert_eq!(result, Err(C8Error::ExceedsConstruct8Max));
    }

    #[test]
    fn test_apply_same_delta_twice_is_idempotent() {
        let mut delta = Construct8Delta::new();
        let triple = Construct8Triple::new(10, 20, 30);
        delta.push_checked(triple).expect("should push triple");

        let mut graph = GraphField::new();

        // Apply delta first time
        graph.apply_construct8(&delta).expect("first apply should succeed");
        let hash_after_first = graph.state_hash();
        let count_after_first = graph.triple_count();

        // Apply delta second time
        graph.apply_construct8(&delta).expect("second apply should succeed");
        let hash_after_second = graph.state_hash();
        let count_after_second = graph.triple_count();

        // Graph uses HashSet for objects, so applying same triple twice results in same count
        // This demonstrates idempotency - graph state is unchanged on second apply
        assert_eq!(count_after_first, count_after_second);
        assert_eq!(hash_after_first, hash_after_second);
    }

    #[test]
    fn test_state_hash_changes_after_apply() {
        let mut delta = Construct8Delta::new();
        let triple = Construct8Triple::new(100, 200, 300);
        delta.push_checked(triple).expect("should push triple");

        let mut graph = GraphField::new();
        let hash_before = graph.state_hash();

        graph.apply_construct8(&delta).expect("apply should succeed");
        let hash_after = graph.state_hash();

        assert_ne!(hash_before, hash_after);
        assert_eq!(graph.triple_count(), 1);
    }

    #[test]
    fn test_construct8_triple_hash_consistent() {
        let triple1 = Construct8Triple::new(1, 2, 3);
        let triple2 = Construct8Triple::new(1, 2, 3);

        assert_eq!(triple1.hash(), triple2.hash());
    }

    #[test]
    fn test_delta_hash_consistent() {
        let mut delta1 = Construct8Delta::new();
        let mut delta2 = Construct8Delta::new();

        let triple = Construct8Triple::new(5, 10, 15);
        delta1.push_checked(triple).expect("push should succeed");
        delta2.push_checked(triple).expect("push should succeed");

        assert_eq!(delta1.delta_hash(), delta2.delta_hash());
    }

    #[test]
    fn test_graph_contains_triple() {
        let mut graph = GraphField::new();
        let triple = Construct8Triple::new(1, 2, 3);

        graph.add_triple(triple);
        assert!(graph.contains_triple(&triple));

        let other = Construct8Triple::new(1, 2, 4);
        assert!(!graph.contains_triple(&other));
    }

    #[test]
    fn test_graph_subjects_predicates_objects() {
        let mut graph = GraphField::new();
        let t1 = Construct8Triple::new(1, 2, 3);
        let t2 = Construct8Triple::new(1, 5, 6);
        let t3 = Construct8Triple::new(10, 2, 7);

        graph.add_triple(t1);
        graph.add_triple(t2);
        graph.add_triple(t3);

        let subjects = graph.subjects();
        assert_eq!(subjects.len(), 2);
        assert!(subjects.contains(&1));
        assert!(subjects.contains(&10));

        let preds_for_1 = graph.predicates(1);
        assert_eq!(preds_for_1.len(), 2);

        let objs = graph.objects(1, 2);
        assert_eq!(objs, vec![3]);
    }

    #[test]
    fn test_construct8_len_increment() {
        let mut len = Construct8Len::Zero;
        for expected in 1..=8 {
            len = len.increment().expect("increment should succeed");
            assert_eq!(len.as_usize(), expected);
        }

        let result = len.increment();
        assert_eq!(result, Err(C8Error::ExceedsConstruct8Max));
    }

    #[test]
    fn test_delta_clear() {
        let mut delta = Construct8Delta::new();
        let triple = Construct8Triple::new(1, 2, 3);
        delta.push_checked(triple).expect("push should succeed");

        assert_eq!(delta.len(), Construct8Len::One);
        assert!(!delta.is_empty());

        delta.clear();

        assert_eq!(delta.len(), Construct8Len::Zero);
        assert!(delta.is_empty());
        assert_eq!(delta.mask(), 0u8);
    }

    #[test]
    fn test_push_multiple_triples() {
        let mut delta = Construct8Delta::new();
        let triples = vec![
            Construct8Triple::new(1, 2, 3),
            Construct8Triple::new(4, 5, 6),
            Construct8Triple::new(7, 8, 9),
        ];

        delta.push_multiple(&triples).expect("push_multiple should succeed");

        assert_eq!(delta.len(), Construct8Len::Three);
        assert_eq!(delta.triple_count(), 3);
    }

    #[test]
    fn test_delta_equality() {
        let mut delta1 = Construct8Delta::new();
        let mut delta2 = Construct8Delta::new();

        let triple = Construct8Triple::new(1, 2, 3);
        delta1.push_checked(triple).expect("push should succeed");
        delta2.push_checked(triple).expect("push should succeed");

        assert_eq!(delta1, delta2);
    }

    #[test]
    fn test_graph_equality_by_hash() {
        let mut graph1 = GraphField::new();
        let mut graph2 = GraphField::new();

        let triple = Construct8Triple::new(1, 2, 3);
        graph1.add_triple(triple);
        graph2.add_triple(triple);

        assert_eq!(graph1, graph2);
    }
}
