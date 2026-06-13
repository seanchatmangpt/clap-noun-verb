// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construct8 Time Engine - vector clocks and monotonic time primitives
//!
//! This crate provides causal-time alignment for market operations:
//! - `VectorClock8`: 8-lane vector clock for distinguishing causality vs. concurrency
//! - `MonotonicStamp`: Global monotonic clock preventing time regressions
//!
//! Used to ensure that market events respect causal ordering and detect
//! concurrent/independent operations across venues and instruments.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use c8_core::C8Error;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

/// Result type for time operations
pub type TimeResult<T> = Result<T, C8Error>;

/// Vector clock with 8 lanes for causal-time tracking
///
/// A vector clock assigns a logical timestamp to each process/lane,
/// allowing us to distinguish between causally-ordered and concurrent events:
/// - **Before**: All lanes of `self` <= corresponding lanes of `other`, with at least one <
/// - **After**: All lanes of `self` >= corresponding lanes of `other`, with at least one >
/// - **Concurrent**: At least one lane where `self` < `other` and at least one where `self` > `other`
/// - **Equal**: All lanes identical
///
/// # Construct8 Mapping (8 lanes)
/// Each lane represents a distinct market dimension:
/// - Lane 0: Instrument dimension (equity orders)
/// - Lane 1: Venue dimension (exchange orders)
/// - Lane 2: Agent dimension (trader actions)
/// - Lane 3: Timeframe dimension (intraday events)
/// - Lanes 4-7: Extension/specialization dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock8 {
    lanes: [u64; 8],
}

impl VectorClock8 {
    /// Create a zero vector clock (no causality observed)
    pub fn zero() -> Self {
        Self { lanes: [0; 8] }
    }

    /// Create a vector clock from explicit lane values
    pub fn from_lanes(lanes: [u64; 8]) -> Self {
        Self { lanes }
    }

    /// Advance a specific lane by one, ensuring causal progress
    ///
    /// # Errors
    /// Returns `InvalidId` if lane >= 8
    pub fn tick_lane(&mut self, lane: usize) -> TimeResult<()> {
        if lane < 8 {
            self.lanes[lane] = self.lanes[lane].saturating_add(1);
            Ok(())
        } else {
            Err(C8Error::VectorClockLaneOutOfBounds)
        }
    }

    /// Advance a specific lane to an explicit value (for causality recovery)
    ///
    /// # Errors
    /// Returns `InvalidId` if lane >= 8
    pub fn set_lane(&mut self, lane: usize, value: u64) -> TimeResult<()> {
        if lane < 8 {
            self.lanes[lane] = value;
            Ok(())
        } else {
            Err(C8Error::VectorClockLaneOutOfBounds)
        }
    }

    /// Get the current value of a lane
    ///
    /// # Errors
    /// Returns `InvalidId` if lane >= 8
    pub fn get_lane(&self, lane: usize) -> TimeResult<u64> {
        if lane < 8 {
            Ok(self.lanes[lane])
        } else {
            Err(C8Error::VectorClockLaneOutOfBounds)
        }
    }

    /// Merge this clock with another by taking element-wise maximum
    ///
    /// This operation represents observing a causally-related event
    /// from another process/lane.
    pub fn merge(&mut self, other: &VectorClock8) {
        for i in 0..8 {
            self.lanes[i] = self.lanes[i].max(other.lanes[i]);
        }
    }

    /// Compare this clock with another to determine causal relationship
    pub fn compare(&self, other: &VectorClock8) -> VectorClockCompare {
        let mut before_count = 0;
        let mut after_count = 0;

        for i in 0..8 {
            match self.lanes[i].cmp(&other.lanes[i]) {
                Ordering::Less => before_count += 1,
                Ordering::Greater => after_count += 1,
                Ordering::Equal => {}
            }
        }

        match (before_count, after_count) {
            (0, 0) => VectorClockCompare::Equal,
            (0, _) => VectorClockCompare::After,
            (_, 0) => VectorClockCompare::Before,
            _ => VectorClockCompare::Concurrent,
        }
    }

    /// Check if this clock causally precedes another
    pub fn happens_before(&self, other: &VectorClock8) -> bool {
        matches!(self.compare(other), VectorClockCompare::Before)
    }

    /// Check if this clock causally follows another
    pub fn happens_after(&self, other: &VectorClock8) -> bool {
        matches!(self.compare(other), VectorClockCompare::After)
    }

    /// Check if this clock is concurrent with another (no causal relationship)
    pub fn is_concurrent(&self, other: &VectorClock8) -> bool {
        matches!(self.compare(other), VectorClockCompare::Concurrent)
    }

    /// Get all lanes as an array
    pub fn lanes(&self) -> [u64; 8] {
        self.lanes
    }

    /// Get the maximum value across all lanes (lamport timestamp approximation)
    pub fn max_lane(&self) -> u64 {
        *self.lanes.iter().max().unwrap_or(&0)
    }

    /// Get the sum of all lanes (for causality depth estimation)
    pub fn sum_lanes(&self) -> u64 {
        self.lanes.iter().fold(0u64, |acc, &x| acc.wrapping_add(x))
    }
}

impl Default for VectorClock8 {
    fn default() -> Self {
        Self::zero()
    }
}

impl PartialOrd for VectorClock8 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.compare(other) {
            VectorClockCompare::Before => Some(Ordering::Less),
            VectorClockCompare::After => Some(Ordering::Greater),
            VectorClockCompare::Equal => Some(Ordering::Equal),
            VectorClockCompare::Concurrent => None,
        }
    }
}

/// Result of comparing two vector clocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorClockCompare {
    /// `self` causally precedes `other`
    Before,
    /// `self` causally follows `other`
    After,
    /// `self` and `other` are concurrent (no causal relationship)
    Concurrent,
    /// `self` and `other` are identical
    Equal,
}

impl VectorClockCompare {
    /// Check if the comparison indicates causality (before or after)
    pub fn is_ordered(&self) -> bool {
        matches!(self, VectorClockCompare::Before | VectorClockCompare::After)
    }

    /// Check if the comparison indicates no causality
    pub fn is_concurrent_or_equal(&self) -> bool {
        matches!(self, VectorClockCompare::Concurrent | VectorClockCompare::Equal)
    }
}

/// Monotonic timestamp preventing time regression
///
/// A global counter that never decreases, ensuring:
/// - Strict ordering of events across all threads
/// - Detection of causality violations (when an event's timestamp is lower than expected)
/// - Market fairness (earlier submissions are detected regardless of clock skew)
///
/// # Implementation
/// Uses an atomic u64 with `Relaxed` ordering for maximum performance
/// (ordering is ensured by the calling code that uses these timestamps).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct MonotonicStamp(u64);

static MONOTONIC_COUNTER: AtomicU64 = AtomicU64::new(0);

impl MonotonicStamp {
    /// Generate a new monotonic timestamp
    ///
    /// Guaranteed to return a value strictly greater than all previously
    /// returned values. No two calls will ever return the same value.
    pub fn now() -> Self {
        MonotonicStamp(MONOTONIC_COUNTER.fetch_add(1, AtomicOrdering::Relaxed))
    }

    /// Get the current value of the global counter (peek-only)
    pub fn current() -> Self {
        MonotonicStamp(MONOTONIC_COUNTER.load(AtomicOrdering::Relaxed))
    }

    /// Create a stamp from an explicit value (use with caution)
    ///
    /// This is only safe if the value is guaranteed to be greater than
    /// all previously allocated stamps. Use for testing or recovery scenarios only.
    pub fn from_value(value: u64) -> Self {
        MonotonicStamp(value)
    }

    /// Get the inner u64 value
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Assert that this stamp is not before another stamp
    ///
    /// This check detects causality violations where an event's timestamp
    /// is earlier than expected.
    ///
    /// # Errors
    /// Returns `MonotonicTimeRegression` if `self < other`
    pub fn assert_not_before(&self, other: &MonotonicStamp) -> TimeResult<()> {
        if self.0 >= other.0 {
            Ok(())
        } else {
            Err(C8Error::MonotonicTimeRegression)
        }
    }

    /// Assert that this stamp is strictly after another stamp
    ///
    /// # Errors
    /// Returns `MonotonicTimeRegression` if `self <= other`
    pub fn assert_strictly_after(&self, other: &MonotonicStamp) -> TimeResult<()> {
        if self.0 > other.0 {
            Ok(())
        } else {
            Err(C8Error::MonotonicTimeRegression)
        }
    }

    /// Compute the logical time gap between two stamps
    pub fn delta_from(&self, other: &MonotonicStamp) -> u64 {
        self.0.saturating_sub(other.0)
    }
}

impl Default for MonotonicStamp {
    fn default() -> Self {
        Self::now()
    }
}

impl From<u64> for MonotonicStamp {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<MonotonicStamp> for u64 {
    fn from(stamp: MonotonicStamp) -> Self {
        stamp.0
    }
}

#[cfg(test)]
mod integration_market;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_clocks_equal() {
        let clock1 = VectorClock8::zero();
        let clock2 = VectorClock8::zero();
        assert_eq!(clock1.compare(&clock2), VectorClockCompare::Equal);
    }

    #[test]
    fn test_tick_lane_creates_causal_after() {
        let mut clock1 = VectorClock8::zero();
        let clock2 = VectorClock8::zero();

        clock1.tick_lane(0).expect("tick lane 0");
        assert_eq!(clock1.compare(&clock2), VectorClockCompare::After);
    }

    #[test]
    fn test_independent_lanes_concurrent() {
        let mut clock1 = VectorClock8::zero();
        let mut clock2 = VectorClock8::zero();

        clock1.tick_lane(0).expect("tick lane 0 for clock1");
        clock2.tick_lane(1).expect("tick lane 1 for clock2");

        assert_eq!(clock1.compare(&clock2), VectorClockCompare::Concurrent);
    }

    #[test]
    fn test_merge_dominates() {
        let mut clock1 = VectorClock8::zero();
        let mut clock2 = VectorClock8::zero();

        clock1.tick_lane(0).expect("tick lane 0");
        clock2.tick_lane(1).expect("tick lane 1");

        clock1.merge(&clock2);

        // After merge, clock1 should have both lane 0 and lane 1 ticked
        assert_eq!(clock1.get_lane(0).expect("get lane 0"), 1);
        assert_eq!(clock1.get_lane(1).expect("get lane 1"), 1);
    }

    #[test]
    fn test_monotonic_never_regresses() {
        let stamp1 = MonotonicStamp::now();
        let stamp2 = MonotonicStamp::now();
        let stamp3 = MonotonicStamp::now();

        assert!(stamp1 < stamp2);
        assert!(stamp2 < stamp3);
        assert!(stamp1 < stamp3);
    }

    #[test]
    fn test_causal_alignment_distinguishes_ordered_from_concurrent() {
        // Scenario: Instrument order followed by venue order on same instrument
        let mut instr_clock = VectorClock8::zero();
        let mut venue_clock = VectorClock8::zero();

        // Instrument creates order (lane 0)
        instr_clock.tick_lane(0).expect("instrument tick");

        // Venue receives and processes order (merge + own tick on lane 1)
        venue_clock.merge(&instr_clock);
        venue_clock.tick_lane(1).expect("venue tick");

        // Venue clock should causally follow instrument clock
        assert_eq!(venue_clock.compare(&instr_clock), VectorClockCompare::After);

        // Two independent orders on different venues should be concurrent
        let mut venue1_clock = VectorClock8::zero();
        let mut venue2_clock = VectorClock8::zero();

        venue1_clock.tick_lane(1).expect("venue 1 tick");
        venue2_clock.tick_lane(2).expect("venue 2 tick");

        assert_eq!(venue1_clock.compare(&venue2_clock), VectorClockCompare::Concurrent);
    }

    #[test]
    fn test_monotonic_stamp_assertions() {
        let stamp1 = MonotonicStamp::from_value(10);
        let stamp2 = MonotonicStamp::from_value(20);

        assert!(stamp1.assert_not_before(&stamp1).is_ok());
        assert!(stamp1.assert_not_before(&stamp2).is_err());
        assert!(stamp2.assert_not_before(&stamp1).is_ok());
    }

    #[test]
    fn test_monotonic_stamp_strictly_after() {
        let stamp1 = MonotonicStamp::from_value(10);
        let stamp2 = MonotonicStamp::from_value(20);

        assert!(stamp1.assert_strictly_after(&stamp2).is_err());
        assert!(stamp2.assert_strictly_after(&stamp1).is_ok());
        assert!(stamp2.assert_strictly_after(&stamp2).is_err());
    }

    #[test]
    fn test_vector_clock_happens_before() {
        let mut clock1 = VectorClock8::zero();
        let clock2 = VectorClock8::zero();

        clock1.tick_lane(0).expect("tick");
        assert!(clock1.happens_after(&clock2));
        assert!(clock2.happens_before(&clock1));
    }

    #[test]
    fn test_vector_clock_max_lane() {
        let clock = VectorClock8::from_lanes([1, 5, 3, 2, 0, 0, 0, 0]);
        assert_eq!(clock.max_lane(), 5);
    }

    #[test]
    fn test_vector_clock_sum_lanes() {
        let clock = VectorClock8::from_lanes([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(clock.sum_lanes(), 36);
    }

    #[test]
    fn test_vector_clock_lane_out_of_bounds() {
        let mut clock = VectorClock8::zero();

        assert!(clock.tick_lane(8).is_err());
        assert!(clock.tick_lane(100).is_err());
        assert!(clock.get_lane(8).is_err());
        assert!(clock.set_lane(8, 10).is_err());
    }

    #[test]
    fn test_vector_clock_partial_ord() {
        let mut clock1 = VectorClock8::zero();
        let mut clock2 = VectorClock8::zero();

        clock1.tick_lane(0).expect("tick");
        assert!(clock2 < clock1);
        assert_eq!(clock1.partial_cmp(&clock2), Some(Ordering::Greater));

        clock2.tick_lane(0).expect("tick");
        assert_eq!(clock1.partial_cmp(&clock2), Some(Ordering::Equal));
    }

    #[test]
    fn test_vector_clock_concurrent_has_no_ord() {
        let mut clock1 = VectorClock8::zero();
        let mut clock2 = VectorClock8::zero();

        clock1.tick_lane(0).expect("tick");
        clock2.tick_lane(1).expect("tick");

        // Concurrent clocks have no partial order
        assert_eq!(clock1.partial_cmp(&clock2), None);
    }

    #[test]
    fn test_monotonic_stamp_delta() {
        let stamp1 = MonotonicStamp::from_value(10);
        let stamp2 = MonotonicStamp::from_value(25);

        assert_eq!(stamp2.delta_from(&stamp1), 15);
        assert_eq!(stamp1.delta_from(&stamp2), 0); // saturating_sub
    }

    #[test]
    fn test_vector_clock_compare_enum_properties() {
        assert!(VectorClockCompare::Before.is_ordered());
        assert!(VectorClockCompare::After.is_ordered());
        assert!(!VectorClockCompare::Concurrent.is_ordered());
        assert!(!VectorClockCompare::Equal.is_ordered());

        assert!(!VectorClockCompare::Before.is_concurrent_or_equal());
        assert!(VectorClockCompare::Concurrent.is_concurrent_or_equal());
        assert!(VectorClockCompare::Equal.is_concurrent_or_equal());
    }
}
