# Construct8 Delta Engine

A fast, bounded graph mutation engine for RDF triples with fixed-capacity (8-triple) deltas.

## Overview

**Construct8** is a specialized data structure for efficiently batching RDF triple mutations with guaranteed bounded memory and O(1) insertion complexity.

### Key Features

- **Fixed Capacity**: 8-triple container for predictable stack allocation
- **Bounded Mutations**: Prevents unbounded graph growth
- **O(1) Operations**: Push, hash, and apply operations in constant time
- **Mask-Based Iteration**: Efficient bit-mask tracking of populated slots
- **State Hashing**: Compare graph state via content-addressable hashes
- **Idempotent Application**: Safe to apply same delta multiple times
- **Zero-Copy Iteration**: Non-consuming iterator over triples

## Architecture

### Construct8Triple

A single RDF triple with subject, predicate, and object as u64 identifiers:

```rust
pub struct Construct8Triple {
    pub subject: u64,
    pub predicate: u64,
    pub object: u64,
}
```

### Construct8Delta

A fixed-size array of up to 8 triples with mask-based tracking:

```rust
pub struct Construct8Delta {
    triples: [Option<Construct8Triple>; 8],
    len: Construct8Len,
    mask: u8,
}
```

**Invariants:**
- `len` ranges from Zero to Eight (0-8)
- `mask` has bits set for each populated slot (1-indexed)
- Pushing 9th triple returns `Err(ExceedsConstruct8Max)`
- Idempotent: applying same delta twice results in same graph state

### GraphField

Manages RDF relations with subject → predicate → object hierarchy:

```rust
pub struct GraphField {
    relations: BTreeMap<u64, BTreeMap<u64, HashSet<u64>>>,
}
```

**Features:**
- Query subjects, predicates, objects by path
- Apply Construct8Delta with stats tracking
- Compute content-addressable state hash
- Serialize/deserialize to JSON

## Usage

### Building a Delta

```rust
use c8_graph::{Construct8Delta, Construct8Triple};

let mut delta = Construct8Delta::new();
delta.push_checked(Construct8Triple::new(1, 2, 3))?;
delta.push_checked(Construct8Triple::new(4, 5, 6))?;

assert_eq!(delta.len().as_usize(), 2);
assert_eq!(delta.mask(), 0x03);
```

### Applying to a Graph

```rust
use c8_graph::{Construct8Delta, Construct8Triple, GraphField};

let mut delta = Construct8Delta::new();
delta.push_checked(Construct8Triple::new(1, 10, 100))?;

let mut graph = GraphField::new();
let stats = graph.apply_construct8(&delta)?;

assert_eq!(stats.applied, 1);
assert_eq!(stats.total, 1);
assert!(stats.is_complete());
```

### Querying the Graph

```rust
// Get all subjects
let subjects = graph.subjects();

// Get all predicates for a subject
let predicates = graph.predicates(1);

// Get all objects for subject-predicate pair
let objects = graph.objects(1, 10);
```

### State Hashing

```rust
let hash_before = graph.state_hash();
graph.apply_construct8(&delta)?;
let hash_after = graph.state_hash();

// Hashes differ after mutation
assert_ne!(hash_before, hash_after);
```

## Performance

Benchmark results on Apple Silicon (M1):

| Operation | Time | Iterations |
|-----------|------|-----------|
| Apply 1 triple | ~120 ns | 42M |
| Apply 2 triples | ~200 ns | 25M |
| Apply 4 triples | ~400 ns | 13M |
| Apply 8 triples | ~690 ns | 7M |

All operations maintain sub-microsecond latency.

## Error Handling

```rust
use c8_graph::{C8Error, C8Result};

match delta.push_checked(triple) {
    Ok(()) => { /* success */ }
    Err(C8Error::ExceedsConstruct8Max) => {
        // Delta is full (8 triples maximum)
    }
    Err(C8Error::GraphOperationFailed(msg)) => {
        // Graph operation error
    }
    Err(C8Error::InvalidStateHash) => {
        // State hash mismatch
    }
}
```

## Testing

All code follows AAA (Arrange-Act-Assert) test pattern:

- **Unit Tests** (15): Core functionality and bounds checking
- **Integration Tests** (14): End-to-end workflows and query operations
- **Benchmarks** (4): Performance characteristics across 1-8 triples

Run tests:
```bash
cargo test --lib              # Unit tests
cargo test --test integration # Integration tests
cargo bench                   # Performance benchmarks
```

## Design Decisions

1. **Fixed Array vs Vec**: Stack allocation for guaranteed performance
2. **u64 Identifiers**: Allows 64-bit IRIs or hash-based identifiers
3. **BTreeMap Hierarchy**: Ordered iteration, efficient range queries
4. **HashSet Objects**: Deduplicates triples naturally
5. **Mask Bit Pattern**: O(1) population count and iteration
6. **Idempotent Application**: Safe for distributed/retry scenarios

## License

MIT OR Apache-2.0
