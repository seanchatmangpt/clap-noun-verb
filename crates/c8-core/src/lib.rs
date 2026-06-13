// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construct8 Core Types - market and financial domain base types
//!
//! This crate provides the fundamental types for market identification and domain modeling.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod hotpath;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for Construct8 core operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum C8Error {
    #[error("Invalid identifier")]
    InvalidId,

    #[error("Vector clock lane out of bounds")]
    VectorClockLaneOutOfBounds,

    #[error("Monotonic time regression detected")]
    MonotonicTimeRegression,

    #[error("Causal violation: operation ordering violates causality")]
    CausalityViolation,

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Result type for Construct8 operations
pub type C8Result<T> = Result<T, C8Error>;

/// Instrument identifier (e.g., stock, commodity, cryptocurrency)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstrumentId(pub u64);

impl InstrumentId {
    /// Create a new InstrumentId
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner u64 value
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for InstrumentId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<InstrumentId> for u64 {
    fn from(id: InstrumentId) -> Self {
        id.0
    }
}

/// Venue identifier (e.g., exchange, OTC market, dark pool)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VenueId(pub u64);

impl VenueId {
    /// Create a new VenueId
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner u64 value
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for VenueId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<VenueId> for u64 {
    fn from(id: VenueId) -> Self {
        id.0
    }
}

/// Tick relation observable - atomic market event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickRelation {
    pub instrument_id: InstrumentId,
    pub venue_id: VenueId,
    pub timestamp: u64,
    pub mid_price: u64,
    pub bid: u64,
    pub ask: u64,
    pub volume: u64,
}

impl TickRelation {
    /// Create a new TickRelation
    pub fn new(
        instrument_id: InstrumentId,
        venue_id: VenueId,
        timestamp: u64,
        mid_price: u64,
        bid: u64,
        ask: u64,
        volume: u64,
    ) -> Self {
        Self { instrument_id, venue_id, timestamp, mid_price, bid, ask, volume }
    }

    /// Compute spread between ask and bid
    pub fn spread(&self) -> u64 {
        self.ask.saturating_sub(self.bid)
    }

    /// Check if spread is normal (typical market spread)
    pub fn is_normal_spread(&self) -> bool {
        let spread = self.spread();
        spread > 0 && spread < 1000 // Example: less than 0.1% of mid price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_id_creation() {
        let id = InstrumentId::new(12345);
        assert_eq!(id.as_u64(), 12345);
    }

    #[test]
    fn test_venue_id_creation() {
        let id = VenueId::new(67890);
        assert_eq!(id.as_u64(), 67890);
    }

    #[test]
    fn test_tick_relation_spread() {
        let tick =
            TickRelation::new(InstrumentId::new(1), VenueId::new(1), 1000, 100, 99, 101, 1000);
        assert_eq!(tick.spread(), 2);
    }

    #[test]
    fn test_tick_relation_normal_spread() {
        let tick =
            TickRelation::new(InstrumentId::new(1), VenueId::new(1), 1000, 100, 99, 101, 1000);
        assert!(tick.is_normal_spread());
    }
}
