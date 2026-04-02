# ADR-005: Byzantine Fault Tolerance for Discovery Consensus

## Status
Accepted

## Context

In a federated network, nodes may be:
- **Faulty**: Crash, network partition, disk failure
- **Malicious**: Publish false capability data, forge signatures, spam network

Traditional fault tolerance (Paxos, Raft) handles crash failures but not **Byzantine failures** (arbitrary/malicious behavior).

## Decision

We will implement **Byzantine Fault Tolerant (BFT) consensus** for critical operations:
- Capability registry updates
- Revocation list publication
- Trust anchor distribution

## Rationale

### Threat Model

| Threat              | Attack Vector                  | Impact           | Mitigation          |
|---------------------|--------------------------------|------------------|---------------------|
| False Advertisement | Malicious CLI claims fake caps | Invocation fails | BFT consensus       |
| Signature Forgery   | Attacker signs with fake key   | Unauthorized exec| Ed25519 verification|
| Revocation Suppression| Hide revoked capabilities   | Security breach  | BFT CRL updates     |
| Sybil Attack        | Create 1000s of fake identities| Network spam     | Proof-of-work join  |
| Eclipse Attack      | Isolate node from honest peers | Stale data       | Diverse peer set    |

### BFT Requirements

From [Byzantine Generals Problem](https://lamport.azurewebsites.net/pubs/byz.pdf):
- **Safety**: No two honest nodes decide differently
- **Liveness**: Honest nodes eventually decide
- **Tolerance**: Handles ≤ f Byzantine nodes (f < n/3)

### Algorithm Choice: HotStuff (LibraBFT variant)

**Why HotStuff over PBFT/Tendermint**:

| Property           | PBFT    | Tendermint | HotStuff | Winner   |
|--------------------|---------|------------|----------|----------|
| Message complexity | O(n³)   | O(n²)      | O(n)     | HotStuff |
| View change cost   | High    | Moderate   | Low      | HotStuff |
| Responsiveness     | ✗       | ~          | ✓        | HotStuff |
| Safety threshold   | f < n/3 | f < n/3    | f < n/3  | Tie      |
| Implementation     | Complex | Moderate   | Moderate | HotStuff |

**Key Innovation**: Linear message complexity (O(n) vs. O(n³) for PBFT)

### When BFT Consensus is Used

**Critical Operations** (require consensus):
1. **Capability Registry Mutations**:
   - Adding new CLI to federated network
   - Updating capability ontology
   - Publishing new capability versions

2. **Revocation List Updates**:
   - Revoking compromised capability tokens
   - Updating certificate revocation lists

3. **Trust Anchor Changes**:
   - Adding new root trust authorities
   - Removing compromised trust anchors

**Non-Critical Operations** (no consensus needed):
- SPARQL discovery queries (read-only)
- gRPC invocations (bilateral; verified by signatures)
- Local caching (node-specific state)

### Architecture

```
┌──────────────────────────────────────────────────┐
│ BFT Consensus Layer (HotStuff)                   │
│                                                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ Node A  │  │ Node B  │  │ Node C  │         │
│  │ (Leader)│  │         │  │         │         │
│  └────┬────┘  └────┬────┘  └────┬────┘         │
│       │            │            │               │
│       │ 1. Propose │            │               │
│       ├───────────►│            │               │
│       │            ├───────────►│               │
│       │            │            │               │
│       │ 2. Vote    │            │               │
│       │◄───────────┤            │               │
│       │◄──────────────────────┤                 │
│       │            │            │               │
│       │ 3. Commit (2f+1 votes) │               │
│       ├───────────►│            │               │
│       │            ├───────────►│               │
│       │            │            │               │
│  ┌────▼────────────▼────────────▼────┐         │
│  │ Committed State (Merkle DAG)      │         │
│  │ - Capability Registry             │         │
│  │ - Revocation Lists                │         │
│  │ - Trust Anchors                   │         │
│  └───────────────────────────────────┘         │
└──────────────────────────────────────────────────┘
```

### Safety Proof (Sketch)

**Theorem**: If ≤ f nodes are Byzantine (f < n/3), all honest nodes agree on committed state.

**Proof Intuition**:
1. Commit requires 2f + 1 votes (quorum)
2. Two quorums overlap by ≥ f + 1 nodes
3. At least 1 honest node in overlap (since f < n/3)
4. Honest nodes never vote for conflicting proposals
5. ∴ No two proposals can both achieve quorum
6. ∴ Safety guaranteed □

**Liveness**: Guaranteed if network synchrony eventually holds (GST model).

### Trade-offs

**Costs**:
- High implementation complexity (consensus protocol)
- Latency overhead: ~100ms for consensus (3 round-trips)
- Requires ≥ 4 nodes (tolerate 1 Byzantine: 3f + 1 = 4)
- Increased network bandwidth (consensus messages)

**Benefits**:
- Security against malicious nodes (tolerates ≤ 33% Byzantine)
- Strong consistency (all honest nodes agree)
- No central authority (decentralized consensus)
- Auditable (Merkle DAG preserves history)

## Consequences

### Positive
- Network resilient to malicious CLIs (up to 33% adversarial)
- Provably correct consensus (formal safety/liveness proofs)
- No trusted third party required
- Capability registry guaranteed consistent across network

### Negative
- Consensus latency adds ~100ms to critical operations
- Requires minimum 4 nodes for f=1 tolerance
- Complex implementation and testing
- Potential for liveness issues under network partition

### Mitigation Strategies

1. **Latency**: Cache capability data; consensus only for writes
2. **Minimum Nodes**: Fall back to single-node mode for small deployments (no BFT)
3. **Complexity**: Use existing BFT library (e.g., `libhotstuff`)
4. **Liveness**: Implement view-change protocol; exponential backoff on timeouts

## When NOT to Use BFT

For **single-organization deployments** where all nodes are trusted:
- Use **Raft** instead (simpler, faster, crash-tolerant)
- BFT overhead not justified

BFT is **only for multi-organization federated networks** where some nodes may be adversarial.

## Architecture Integration

```
┌──────────────────────────────────────────────┐
│ Federated Network Layers                     │
│                                              │
│  ┌────────────────────────────────────┐     │
│  │ Application Layer                  │     │
│  │ - SPARQL queries (read)            │     │
│  │ - gRPC invocations (bilateral)     │     │
│  └────────────────────────────────────┘     │
│                    │                         │
│  ┌────────────────▼──────────────────┐      │
│  │ BFT Consensus Layer               │      │
│  │ - Capability registry updates     │      │
│  │ - CRL publication                 │      │
│  │ - Trust anchor management         │      │
│  └────────────────┬──────────────────┘      │
│                    │                         │
│  ┌────────────────▼──────────────────┐      │
│  │ Storage Layer                     │      │
│  │ - Merkle DAG (immutable history)  │      │
│  │ - Local triple store              │      │
│  └───────────────────────────────────┘      │
└──────────────────────────────────────────────┘
```

**Data Flow**:
1. Read operations: Direct query to local triple store (no consensus)
2. Write operations: Propose via BFT → Consensus → Commit to Merkle DAG → Update local store

## Alternatives Considered

### 1. No Consensus (Trust First Publisher)
- **Pros**: Zero latency overhead
- **Cons**: Vulnerable to race conditions, no Byzantine tolerance
- **Rejected**: Security requirement mandates consensus

### 2. Proof-of-Work (Bitcoin-style)
- **Pros**: Sybil resistance, no identity required
- **Cons**: Extremely high latency (minutes), energy waste
- **Rejected**: Latency requirement (p99 < 100ms) not achievable

### 3. Practical Byzantine Fault Tolerance (PBFT)
- **Pros**: Proven algorithm, formal verification exists
- **Cons**: O(n³) message complexity, slow view changes
- **Rejected**: Scalability issues above 10 nodes

### 4. Tendermint
- **Pros**: Battle-tested (Cosmos blockchain), good performance
- **Cons**: O(n²) messages, partial synchrony required
- **Rejected**: HotStuff has better message complexity

## Validation

Success metrics:
- Consensus latency p99 < 100ms (for 10-node network)
- Safety: Zero incorrect commits (even with 33% Byzantine nodes)
- Liveness: 99.9% availability (consensus completes despite f failures)
- Throughput: ≥ 100 txn/sec capability registry updates

## References

- [HotStuff: BFT Consensus in the Lens of Blockchain](https://arxiv.org/abs/1803.05069)
- [The Byzantine Generals Problem](https://lamport.azurewebsites.net/pubs/byz.pdf)
- [Practical Byzantine Fault Tolerance](https://pmg.csail.mit.edu/papers/osdi99.pdf)
- [LibraBFT Specification](https://developers.diem.com/papers/diem-consensus-state-machine-replication-in-the-diem-blockchain/2019-06-28.pdf)
