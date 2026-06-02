# c8-time: Construct8 Time Engine

Vector clocks and monotonic time primitives for market causal-time alignment.

## Overview

`c8-time` provides causal-time ordering primitives for financial market operations:

- **`VectorClock8`**: An 8-lane logical clock for distinguishing causally-ordered events from concurrent (independent) operations
- **`MonotonicStamp`**: A global monotonic timestamp that prevents time regressions and detects causality violations
- **`VectorClockCompare`**: Relationship classification (Before, After, Concurrent, Equal)

## Use Cases

### Multi-Venue Order Flow
Track causality across trading venues to detect market-fair ordering:
```rust
let mut trader_clock = VectorClock8::zero();
trader_clock.tick_lane(0)?; // Instrument dimension

let mut venue_a = VectorClock8::zero();
venue_a.merge(&trader_clock);
venue_a.tick_lane(1)?; // Venue A dimension

// Venue A causally follows trader's submission
assert_eq!(venue_a.compare(&trader_clock), VectorClockCompare::After);
```

### Causality Violation Detection
Monotonic timestamps detect out-of-order settlements and clock skew:
```rust
let t1 = MonotonicStamp::from_value(100);
let t2 = MonotonicStamp::from_value(90);

// t2 should never appear to precede t1
assert!(t1.assert_not_before(&t2).is_ok());
assert!(t2.assert_not_before(&t1).is_err()); // Regression!
```

### Settlement Coordination
Ensure settlement messages respect causal ordering:
```rust
let mut venue_a = VectorClock8::zero();
venue_a.tick_lane(0)?; // Fill order

let mut broadcast = VectorClock8::zero();
broadcast.merge(&venue_a);
broadcast.tick_lane(0)?;

let mut venue_b = VectorClock8::zero();
venue_b.merge(&broadcast);
venue_b.tick_lane(1)?;

// Complete causal chain verified
assert!(venue_a.happens_before(&broadcast));
assert!(broadcast.happens_before(&venue_b));
```

## Lane Mapping (Construct8 Dimensions)

The 8 lanes of `VectorClock8` map to distinct market dimensions:

| Lane | Dimension | Purpose |
|------|-----------|---------|
| 0 | Instrument | Equity order sequencing |
| 1 | Venue A | First exchange operations |
| 2 | Venue B | Second exchange operations |
| 3 | Agent | Trader action causality |
| 4 | Timeframe | Intraday event ordering |
| 5-7 | Extension | Specialization dimensions |

## API Reference

### VectorClock8

#### Construction
- `zero()` - Create an all-zero clock
- `from_lanes([u64; 8])` - Create from explicit lane values

#### Mutation
- `tick_lane(usize)` - Advance a lane by 1
- `set_lane(usize, u64)` - Set a lane to explicit value
- `merge(&VectorClock8)` - Merge with another clock (element-wise max)

#### Query
- `get_lane(usize)` - Read a lane value
- `compare(&VectorClock8)` - Get causal relationship
- `happens_before(&VectorClock8)` - Boolean check
- `happens_after(&VectorClock8)` - Boolean check
- `is_concurrent(&VectorClock8)` - Boolean check
- `max_lane()` - Maximum value across all lanes
- `sum_lanes()` - Sum of all lanes (causality depth estimate)
- `lanes()` - Get all lanes as array

### MonotonicStamp

#### Construction
- `now()` - Allocate new unique timestamp (increments global counter)
- `current()` - Peek at global counter without incrementing
- `from_value(u64)` - Create from explicit value (testing only)

#### Query
- `as_u64()` - Get inner value
- `assert_not_before(&MonotonicStamp)` - Verify ordering
- `assert_strictly_after(&MonotonicStamp)` - Verify strict ordering
- `delta_from(&MonotonicStamp)` - Time gap between stamps

### VectorClockCompare

Enum representing causality relationship:
- `Before` - `self` causally precedes `other`
- `After` - `self` causally follows `other`
- `Concurrent` - No causal relationship (can occur in any order)
- `Equal` - Identical clocks

Methods:
- `is_ordered()` - True if Before or After
- `is_concurrent_or_equal()` - True if Concurrent or Equal

## Dependency Chain

```
c8-time
├── c8-core (error types: C8Error, C8Result)
└── serde (serialization)
```

## Testing

All 23 tests verify:
- Vector clock zero equality
- Lane ticking creates proper causality
- Independent lanes show concurrency
- Merge correctly dominates values
- Monotonic never regresses
- Full causal alignment scenarios

Run tests:
```bash
cargo test -p c8-time
```

## Design Notes

### Relaxed Atomics
`MonotonicStamp` uses `Relaxed` ordering for maximum performance. Ordering is enforced by the application code using these timestamps (e.g., message queues, settlement protocols).

### Saturating Arithmetic
Lane values use `saturating_add` to prevent overflow; at 1 billion ticks/second, a u64 lane takes ~584 years to saturate.

### No Automatic Tick
Vector clocks do not auto-increment on every operation. Callers must explicitly `tick_lane()` to record causality. This prevents accidentally conflating different causal dimensions.

## References

- Lamport, L. (1978). Time, Clocks, and the Ordering of Events in a Distributed System
- Mattern, F. (1989). Virtual Time and Global States of Distributed Systems
- Fidge, C. J. (1991). Logical Time in Distributed Computing Systems
