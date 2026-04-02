# ADR-001: Federated Architecture Over Centralized Registry

## Status
Accepted

## Context

We need to enable distributed CLI composition across independent systems. Two primary architectural patterns were considered:

1. **Centralized Registry**: All CLIs register with a central service discovery broker
2. **Federated Network**: Peer-to-peer discovery using semantic web technologies

## Decision

We will implement a **Federated Semantic Network** using RDF ontologies and SPARQL federation.

## Rationale

### Quality Attributes Analysis

| Attribute          | Centralized | Federated | Winner    |
|--------------------|-------------|-----------|-----------|
| Availability       | SPOF risk   | Resilient | Federated |
| Scalability        | Limited     | DHT-based | Federated |
| Latency (discovery)| Low         | Moderate  | Centralized |
| Security           | Trust broker| Zero-trust| Federated |
| Autonomy           | Low         | High      | Federated |
| Complexity         | Low         | High      | Centralized |

### Key Drivers

1. **No Single Point of Failure**: Centralized registries create availability bottlenecks
2. **Organizational Autonomy**: Teams must control their own CLI capabilities without central approval
3. **Resilience**: System must function with partial network availability
4. **Zero-Trust Security**: Cannot assume centralized broker is trustworthy
5. **Semantic Richness**: RDF enables expressive capability descriptions beyond simple key-value registries

### Trade-offs Accepted

**Costs**:
- Higher implementation complexity (SPARQL, RDF, DHT)
- Slower initial discovery (distributed query vs. centralized lookup)
- Eventual consistency (vs. immediate consistency in centralized model)
- More complex debugging (distributed system challenges)

**Benefits**:
- No operational dependency on central service
- Scales to 100K+ nodes via DHT sharding
- Survives Byzantine failures (malicious nodes)
- Rich semantic queries (find "all CLIs that process images in PNG format")
- Zero-trust security model

## Consequences

### Positive
- System operates independently without external dependencies
- CLIs can discover capabilities semantically, not just by name
- Resilient to network partitions and node failures
- Zero-trust security by design

### Negative
- Requires SPARQL and RDF expertise for implementation
- More complex testing (distributed system verification)
- Discovery latency higher than centralized (p50: 50ms vs. 5ms)
- Debugging requires distributed tracing infrastructure

### Mitigation Strategies

1. **Discovery Latency**: Implement local caching with TTL
2. **Complexity**: Use existing libraries (Oxigraph for SPARQL, libp2p for DHT)
3. **Debugging**: Integrate distributed tracing (OpenTelemetry)
4. **Testing**: Property-based testing for Byzantine scenarios

## Validation

Success metrics (measured after 6 months):
- Discovery p99 latency < 100ms
- System availability > 99.9% (no central SPOF)
- Byzantine fault tolerance: survives 33% malicious nodes
- Adoption: 100+ CLIs federated across 10+ organizations

## References

- [CAP Theorem](https://en.wikipedia.org/wiki/CAP_theorem)
- [SPARQL Federation](https://www.w3.org/TR/sparql11-federated-query/)
- [Byzantine Fault Tolerance](https://pmg.csail.mit.edu/papers/osdi99.pdf)
