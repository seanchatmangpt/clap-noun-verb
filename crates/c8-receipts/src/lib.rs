// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construct8 Receipts - lightweight proof infrastructure for state transitions
//!
//! This crate provides receipt types for tracking state transitions in the Construct8
//! pipeline. Receipts encode pre-state, delta, and post-state hashes along with causal
//! timestamps, enabling replay verification and conformance checking.
//!
//! ## Core Types
//!
//! - `C8Receipt` — atomic state transition proof (pre_state_hash, delta_mask, post_state_hash, causal_time)
//! - `ReceiptChain` — sequenced receipts for ordered state evolution
//! - `ImplementationReceipt` — build environment documentation

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use c8_core::C8Result;
use c8_graph::Construct8Delta;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A single state transition receipt
///
/// Encodes the pre-state hash, delta mask (which triples were applied),
/// post-state hash, and causal timestamp for deterministic replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct C8Receipt {
    /// Hash of the state before the delta was applied
    pub pre_state_hash: u64,
    /// Bitmask indicating which slots in the Construct8Delta were occupied
    pub delta_mask: u8,
    /// Hash of the state after the delta was applied
    pub post_state_hash: u64,
    /// Causal timestamp (monotonic vector clock lane or wall-clock)
    pub causal_time: u64,
    /// Hash of the receipt for tamper detection
    pub receipt_hash: u64,
}

impl C8Receipt {
    /// Create a new receipt from delta application
    ///
    /// # Arguments
    /// * `pre` - Pre-state hash
    /// * `delta` - The Construct8Delta that was applied
    /// * `post` - Post-state hash after applying delta
    /// * `time` - Causal timestamp
    pub fn new(pre: u64, delta: &Construct8Delta, post: u64, time: u64) -> Self {
        let receipt_hash = Self::hash_receipt(pre, delta.mask(), post, time);
        C8Receipt {
            pre_state_hash: pre,
            delta_mask: delta.mask(),
            post_state_hash: post,
            causal_time: time,
            receipt_hash,
        }
    }

    /// Hash a receipt to detect tampering
    ///
    /// Uses a simple XOR and shift hash of the receipt components.
    /// Sufficient for detecting accidental corruption; cryptographic
    /// strength not required for process conformance checking.
    fn hash_receipt(pre: u64, mask: u8, post: u64, time: u64) -> u64 {
        let mut result = 0u64;
        result = result.wrapping_mul(31).wrapping_add(pre);
        result = result.wrapping_mul(31).wrapping_add(mask as u64);
        result = result.wrapping_mul(31).wrapping_add(post);
        result = result.wrapping_mul(31).wrapping_add(time);
        result
    }

    /// Verify this receipt's integrity
    ///
    /// Recomputes the receipt hash and compares to the stored value.
    /// Returns `true` if the receipt has not been tampered with.
    pub fn verify_integrity(&self) -> bool {
        let computed = Self::hash_receipt(
            self.pre_state_hash,
            self.delta_mask,
            self.post_state_hash,
            self.causal_time,
        );
        self.receipt_hash == computed
    }

    /// Verify causal ordering: this receipt's pre-state must match previous receipt's post-state
    pub fn verify_causality(&self, prev: &C8Receipt) -> bool {
        self.pre_state_hash == prev.post_state_hash && self.causal_time > prev.causal_time
    }
}

impl fmt::Display for C8Receipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "C8Receipt {{ pre: {:#x}, mask: {:#04b}, post: {:#x}, time: {}, hash: {:#x} }}",
            self.pre_state_hash,
            self.delta_mask,
            self.post_state_hash,
            self.causal_time,
            self.receipt_hash
        )
    }
}

/// A chain of receipts encoding the complete state evolution
///
/// Receipts are ordered by causal timestamp and form a chain where each
/// receipt's post-state hash must match the next receipt's pre-state hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptChain {
    receipts: Vec<C8Receipt>,
}

impl ReceiptChain {
    /// Create a new empty receipt chain
    pub fn new() -> Self {
        Self { receipts: Vec::new() }
    }

    /// Append a receipt to the chain
    ///
    /// # Arguments
    /// * `receipt` - The receipt to append
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the receipt violates causality
    pub fn append(&mut self, receipt: C8Receipt) -> C8Result<()> {
        // Verify receipt integrity first
        if !receipt.verify_integrity() {
            return Err(c8_core::C8Error::OperationFailed(
                "Receipt integrity check failed (tampered)".to_string(),
            ));
        }

        // Check causality with previous receipt if chain is non-empty
        if let Some(prev) = self.receipts.last() {
            if !receipt.verify_causality(prev) {
                return Err(c8_core::C8Error::CausalityViolation);
            }
        }

        self.receipts.push(receipt);
        Ok(())
    }

    /// Verify the entire receipt chain for integrity and causal ordering
    ///
    /// Checks:
    /// 1. Each receipt's integrity (hash is correct)
    /// 2. Causal ordering (each receipt's pre-state matches previous post-state)
    /// 3. Monotonic timestamps
    ///
    /// # Returns
    /// `true` if chain is valid, `false` if any receipt is tampered or causal order is violated
    pub fn verify(&self) -> bool {
        if self.receipts.is_empty() {
            return true; // Empty chain is valid
        }

        // Verify first receipt
        if !self.receipts[0].verify_integrity() {
            return false;
        }

        // Verify chain causal integrity
        for i in 1..self.receipts.len() {
            let prev = &self.receipts[i - 1];
            let curr = &self.receipts[i];

            // Check integrity
            if !curr.verify_integrity() {
                return false;
            }

            // Check causality
            if !curr.verify_causality(prev) {
                return false;
            }
        }

        true
    }

    /// Get the number of receipts in the chain
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    /// Get a reference to a receipt by index
    pub fn get(&self, index: usize) -> Option<&C8Receipt> {
        self.receipts.get(index)
    }

    /// Iterate over all receipts
    pub fn iter(&self) -> impl Iterator<Item = &C8Receipt> {
        self.receipts.iter()
    }

    /// Get the final state hash (from last receipt's post-state)
    pub fn final_state_hash(&self) -> Option<u64> {
        self.receipts.last().map(|r| r.post_state_hash)
    }

    /// Get the initial state hash (from first receipt's pre-state)
    pub fn initial_state_hash(&self) -> Option<u64> {
        self.receipts.first().map(|r| r.pre_state_hash)
    }

    /// Clear all receipts from the chain
    pub fn clear(&mut self) {
        self.receipts.clear();
    }
}

impl Default for ReceiptChain {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ReceiptChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ReceiptChain [{}]",
            self.receipts.iter().map(|r| format!("{}", r)).collect::<Vec<_>>().join(", ")
        )
    }
}

/// Documentation of the implementation environment
///
/// Captures crate versions and build-time metadata to establish
/// the reproducibility context for a receipt chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationReceipt {
    /// Semantic version of c8-core crate
    pub c8_core_version: String,
    /// Semantic version of c8-graph crate
    pub c8_graph_version: String,
    /// Semantic version of c8-receipts crate
    pub c8_receipts_version: String,
    /// Rust compiler version (e.g., "1.74.0")
    pub rust_version: String,
    /// Build target triple (e.g., "x86_64-unknown-linux-gnu")
    pub build_target: String,
    /// Optional additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl ImplementationReceipt {
    /// Create an ImplementationReceipt from version strings
    pub fn new(
        c8_core_version: String,
        c8_graph_version: String,
        c8_receipts_version: String,
        rust_version: String,
        build_target: String,
    ) -> Self {
        Self {
            c8_core_version,
            c8_graph_version,
            c8_receipts_version,
            rust_version,
            build_target,
            metadata: None,
        }
    }

    /// Set additional metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl fmt::Display for ImplementationReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ImplementationReceipt {{ c8-core: {}, c8-graph: {}, c8-receipts: {}, rust: {}, target: {} }}",
            self.c8_core_version, self.c8_graph_version, self.c8_receipts_version, self.rust_version, self.build_target
        )
    }
}

/// Document the build environment for reproducibility
///
/// Returns a formatted string containing version information for the Construct8 stack.
pub fn write_implementation_receipt(crate_versions: &str) -> String {
    format!(
        "Construct8 Receipt Implementation\n\
         \n\
         Crate Versions:\n\
         {}\n\
         \n\
         This receipt proves the exact build environment under which state transitions were recorded.",
        crate_versions
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use c8_graph::{Construct8Delta, Construct8Triple, GraphField};

    #[test]
    fn test_receipt_hash_changes_with_state() {
        let delta = Construct8Delta::new();

        let receipt1 = C8Receipt::new(100, &delta, 200, 1);
        let receipt2 = C8Receipt::new(100, &delta, 201, 1); // Different post-state

        assert_ne!(receipt1.receipt_hash, receipt2.receipt_hash);
    }

    #[test]
    fn test_receipt_hash_changes_with_time() {
        let delta = Construct8Delta::new();

        let receipt1 = C8Receipt::new(100, &delta, 200, 1);
        let receipt2 = C8Receipt::new(100, &delta, 200, 2); // Different time

        assert_ne!(receipt1.receipt_hash, receipt2.receipt_hash);
    }

    #[test]
    fn test_receipt_hash_changes_with_mask() {
        let mut delta1 = Construct8Delta::new();
        delta1.push_checked(Construct8Triple::new(1, 2, 3)).expect("should push");

        let mut delta2 = Construct8Delta::new();
        delta2.push_checked(Construct8Triple::new(1, 2, 3)).expect("should push");
        delta2.push_checked(Construct8Triple::new(4, 5, 6)).expect("should push");

        let receipt1 = C8Receipt::new(100, &delta1, 200, 1);
        let receipt2 = C8Receipt::new(100, &delta2, 200, 1);

        assert_ne!(receipt1.receipt_hash, receipt2.receipt_hash);
    }

    #[test]
    fn test_receipt_verify_integrity() {
        let delta = Construct8Delta::new();
        let receipt = C8Receipt::new(100, &delta, 200, 1);

        assert!(receipt.verify_integrity());
    }

    #[test]
    fn test_tampered_receipt_fails_verification() {
        let delta = Construct8Delta::new();
        let mut receipt = C8Receipt::new(100, &delta, 200, 1);

        // Tamper with the receipt
        receipt.receipt_hash = 0;

        assert!(!receipt.verify_integrity());
    }

    #[test]
    fn test_receipt_chain_verify_succeeds() {
        let delta = Construct8Delta::new();

        let receipt1 = C8Receipt::new(100, &delta, 200, 1);
        let receipt2 = C8Receipt::new(200, &delta, 300, 2); // pre-state matches receipt1's post-state
        let receipt3 = C8Receipt::new(300, &delta, 400, 3); // pre-state matches receipt2's post-state

        let mut chain = ReceiptChain::new();
        chain.append(receipt1).expect("should append r1");
        chain.append(receipt2).expect("should append r2");
        chain.append(receipt3).expect("should append r3");

        assert!(chain.verify());
    }

    #[test]
    fn test_receipt_chain_append_with_broken_causality_fails() {
        let delta = Construct8Delta::new();

        let receipt1 = C8Receipt::new(100, &delta, 200, 1);
        let receipt2_bad = C8Receipt::new(199, &delta, 300, 1); // pre-state does NOT match

        let mut chain = ReceiptChain::new();
        chain.append(receipt1).expect("should append r1");

        let result = chain.append(receipt2_bad);
        assert!(result.is_err());
    }

    #[test]
    fn test_receipt_chain_append_with_non_monotonic_time_fails() {
        let delta = Construct8Delta::new();

        let receipt1 = C8Receipt::new(100, &delta, 200, 5);
        let receipt2_bad = C8Receipt::new(200, &delta, 300, 3); // time went backward

        let mut chain = ReceiptChain::new();
        chain.append(receipt1).expect("should append r1");

        let result = chain.append(receipt2_bad);
        assert!(result.is_err());
    }

    #[test]
    fn test_receipt_chain_verify_detects_tamper() {
        let delta = Construct8Delta::new();

        let receipt1 = C8Receipt::new(100, &delta, 200, 1);
        let receipt2 = C8Receipt::new(200, &delta, 300, 2);

        let mut chain = ReceiptChain::new();
        chain.append(receipt1).expect("should append r1");
        chain.append(receipt2).expect("should append r2");

        // Manually tamper with receipt2
        chain.receipts[1].post_state_hash = 999;
        chain.receipts[1].receipt_hash = 0; // Make it invalid

        assert!(!chain.verify());
    }

    #[test]
    fn test_replay_construct8_reproduces_state_hash() {
        // Create a delta with one triple
        let mut delta = Construct8Delta::new();
        let triple = Construct8Triple::new(10, 20, 30);
        delta.push_checked(triple).expect("should push");

        // Start with empty graph (pre-state)
        let mut graph = GraphField::new();
        let pre_hash = graph.state_hash();

        // Apply delta
        graph.apply_construct8(&delta).expect("should apply");
        let post_hash = graph.state_hash();

        // Create receipt
        let receipt = C8Receipt::new(pre_hash, &delta, post_hash, 1);

        // Verify the receipt
        assert!(receipt.verify_integrity());
        assert_ne!(pre_hash, post_hash);

        // Replay: start fresh and apply same delta
        let mut replay_graph = GraphField::new();
        assert_eq!(
            replay_graph.state_hash(),
            pre_hash,
            "replay starting state must match original"
        );

        replay_graph.apply_construct8(&delta).expect("should apply on replay");
        let replayed_hash = replay_graph.state_hash();

        assert_eq!(replayed_hash, post_hash, "replayed state must match original post-state");
    }

    #[test]
    fn test_receipt_chain_final_state_hash() {
        let delta = Construct8Delta::new();

        let receipt1 = C8Receipt::new(100, &delta, 200, 1);
        let receipt2 = C8Receipt::new(200, &delta, 300, 2);

        let mut chain = ReceiptChain::new();
        chain.append(receipt1).expect("should append");
        chain.append(receipt2).expect("should append");

        assert_eq!(chain.final_state_hash(), Some(300));
        assert_eq!(chain.initial_state_hash(), Some(100));
    }

    #[test]
    fn test_receipt_chain_empty() {
        let chain = ReceiptChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        assert!(chain.verify()); // Empty chain is valid
        assert_eq!(chain.final_state_hash(), None);
    }

    #[test]
    fn test_implementation_receipt() {
        let impl_receipt = ImplementationReceipt::new(
            "0.1.0".to_string(),
            "0.1.0".to_string(),
            "0.1.0".to_string(),
            "1.74.0".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
        );

        assert_eq!(impl_receipt.c8_core_version, "0.1.0");
        assert_eq!(impl_receipt.rust_version, "1.74.0");
    }

    #[test]
    fn test_write_implementation_receipt() {
        let versions = "c8-core: 0.1.0\nc8-graph: 0.1.0\nc8-receipts: 0.1.0";
        let doc = write_implementation_receipt(versions);

        assert!(doc.contains("Construct8 Receipt Implementation"));
        assert!(doc.contains(versions));
    }
}
