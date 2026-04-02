# C4 Model: Container Diagram

## Containers within Federated Semantic Network

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    Federated Semantic Network                            │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ CLI Node (containerized application)                              │ │
│  │                                                                    │ │
│  │  ┌──────────────────────┐  ┌──────────────────────┐              │ │
│  │  │ Command Executor     │  │ RDF Ontology Server  │              │ │
│  │  │                      │  │                      │              │ │
│  │  │ - Rust binary        │  │ - Oxigraph (SPARQL)  │              │ │
│  │  │ - clap-noun-verb     │  │ - Serves ontology.ttl│              │ │
│  │  │ - Business logic     │  │ - HTTP/HTTPS         │              │ │
│  │  └──────────────────────┘  └──────────────────────┘              │ │
│  │            │                          │                           │ │
│  │            │                          │                           │ │
│  │  ┌─────────▼──────────────────────────▼──────────┐               │ │
│  │  │ gRPC Service Layer                            │               │ │
│  │  │                                               │               │ │
│  │  │ - tonic (Rust gRPC)                           │               │ │
│  │  │ - Auto-generated from RDF ontology            │               │ │
│  │  │ - TLS 1.3 with mutual authentication          │               │ │
│  │  │ - HTTP/2 transport                            │               │ │
│  │  └───────────────────────────────────────────────┘               │ │
│  │            │                                                      │ │
│  │            │                                                      │ │
│  │  ┌─────────▼──────────────────────────────────────┐              │ │
│  │  │ Security Layer                                 │              │ │
│  │  │                                                │              │ │
│  │  │ - Ed25519 signature verification               │              │ │
│  │  │ - JWT capability token validation              │              │ │
│  │  │ - SHACL type validation                        │              │ │
│  │  │ - Certificate binding (TLS)                    │              │ │
│  │  └────────────────────────────────────────────────┘              │ │
│  │            │                                                      │ │
│  │            │                                                      │ │
│  │  ┌─────────▼──────────────────────────────────────┐              │ │
│  │  │ Storage Layer                                  │              │ │
│  │  │                                                │              │ │
│  │  │ - Triple store (RocksDB-backed)                │              │ │
│  │  │ - Capability token cache                       │              │ │
│  │  │ - Trust graph database                         │              │ │
│  │  │ - Audit log (append-only)                      │              │ │
│  │  └────────────────────────────────────────────────┘              │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Discovery Service (shared infrastructure)                         │ │
│  │                                                                    │ │
│  │  ┌──────────────────────┐  ┌──────────────────────┐              │ │
│  │  │ DHT Node (Kademlia)  │  │ mDNS/DNS-SD Service  │              │ │
│  │  │                      │  │                      │              │ │
│  │  │ - libp2p             │  │ - Local discovery    │              │ │
│  │  │ - Peer routing       │  │ - Zero-config network│              │ │
│  │  │ - Capability indexing│  │ - Service announce   │              │ │
│  │  └──────────────────────┘  └──────────────────────┘              │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Consensus Service (BFT)                                            │ │
│  │                                                                    │ │
│  │  ┌──────────────────────┐  ┌──────────────────────┐              │ │
│  │  │ HotStuff Consensus   │  │ State Machine        │              │ │
│  │  │                      │  │                      │              │ │
│  │  │ - Leader election    │  │ - Capability registry│              │ │
│  │  │ - Vote collection    │  │ - Trust certificates │              │ │
│  │  │ - 2f+1 quorum        │  │ - Revocation lists   │              │ │
│  │  └──────────────────────┘  └──────────────────────┘              │ │
│  │            │                          │                           │ │
│  │            └──────────────┬───────────┘                           │ │
│  │                           │                                       │ │
│  │                 ┌─────────▼──────────┐                            │ │
│  │                 │ Merkle DAG         │                            │ │
│  │                 │                    │                            │ │
│  │                 │ - Immutable history│                            │ │
│  │                 │ - Content-addressed│                            │ │
│  │                 └────────────────────┘                            │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Monitoring & Observability                                         │ │
│  │                                                                    │ │
│  │  ┌──────────────────┐  ┌──────────────┐  ┌──────────────────┐    │ │
│  │  │ Metrics (Prom)   │  │ Tracing (OTel)│  │ Logs (structured)│    │ │
│  │  │                  │  │               │  │                  │    │ │
│  │  │ - Latency        │  │ - Distributed │  │ - Audit events   │    │ │
│  │  │ - Throughput     │  │   traces      │  │ - Security logs  │    │ │
│  │  │ - Error rates    │  │ - Span context│  │ - JSON format    │    │ │
│  │  └──────────────────┘  └──────────────┘  └──────────────────┘    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘

         │                    │                    │
         │ Publishes          │ Queries            │ Writes
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ External SPARQL │  │ External DNS    │  │ External DHT    │
│ Aggregators     │  │ Servers         │  │ Bootstrap Nodes │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

## Container Descriptions

### 1. CLI Node

**Technology**: Rust, clap-noun-verb, tonic (gRPC), Oxigraph (SPARQL)

**Responsibilities**:
- Execute CLI commands
- Serve RDF ontology via SPARQL endpoint
- Handle gRPC invocations from other CLIs
- Verify security credentials
- Store local state (triple store, cache, logs)

**Interfaces**:
- **gRPC API** (port 50051): Command invocation
- **HTTP SPARQL Endpoint** (port 8080): Ontology queries
- **Metrics** (port 9090): Prometheus scraping

### 2. Discovery Service

**Technology**: libp2p (DHT), Avahi/Bonjour (mDNS)

**Responsibilities**:
- Discover peers in local network (mDNS)
- Discover peers in wide area (DHT)
- Publish capability indices
- Route discovery queries

**Interfaces**:
- **DHT Protocol**: Kademlia (UDP)
- **mDNS**: Service announcement (multicast)

### 3. Consensus Service

**Technology**: Custom HotStuff implementation (Rust)

**Responsibilities**:
- Coordinate capability registry updates
- Achieve Byzantine fault tolerance
- Maintain Merkle DAG of state history
- Provide strongly consistent view

**Interfaces**:
- **Consensus Protocol**: HotStuff (TCP)
- **State Query API**: Read committed state

### 4. Monitoring & Observability

**Technology**: Prometheus (metrics), OpenTelemetry (tracing), JSON logs

**Responsibilities**:
- Track performance metrics
- Distributed tracing across CLI boundaries
- Structured logging for audits
- Alerting on anomalies

**Interfaces**:
- **Prometheus Metrics**: `/metrics` HTTP endpoint
- **Trace Export**: OTLP (OpenTelemetry Protocol)
- **Log Aggregation**: Stdout (JSON lines)

## Data Flow Between Containers

### Discovery Flow
```
CLI Node → DHT Node → External DHT Bootstrap Nodes
       ↓
    mDNS Service → Local network broadcast
```

### Invocation Flow
```
CLI Node (Client)
    → Security Layer (sign request, attach token)
    → gRPC Service Layer (serialize, send)
    → Network
    → CLI Node (Server)
        → gRPC Service Layer (deserialize)
        → Security Layer (verify signature, token)
        → Command Executor (execute)
        → Security Layer (sign response)
        → gRPC Service Layer (send response)
```

### Consensus Flow
```
CLI Node → Consensus Service (propose update)
    → HotStuff Protocol (collect 2f+1 votes)
    → State Machine (apply committed update)
    → Merkle DAG (append to history)
    → CLI Node (notify of committed state)
```

## Deployment Architecture

### Single-Host Deployment
```
┌─────────────────────────────┐
│ Host Machine                │
│                             │
│  ┌───────────────────────┐  │
│  │ CLI Node Container    │  │
│  │ - Port 50051 (gRPC)   │  │
│  │ - Port 8080 (SPARQL)  │  │
│  │ - Port 9090 (metrics) │  │
│  └───────────────────────┘  │
│                             │
│  ┌───────────────────────┐  │
│  │ Discovery Container   │  │
│  │ - DHT + mDNS          │  │
│  └───────────────────────┘  │
│                             │
│  ┌───────────────────────┐  │
│  │ Consensus Container   │  │
│  │ - HotStuff node       │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

### Multi-Host Deployment
```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Host A       │  │ Host B       │  │ Host C       │
│              │  │              │  │              │
│ CLI Node A   │  │ CLI Node B   │  │ CLI Node C   │
│ Consensus A  │  │ Consensus B  │  │ Consensus C  │
│ Discovery A  │  │ Discovery B  │  │ Discovery C  │
└───────┬──────┘  └───────┬──────┘  └───────┬──────┘
        │                 │                 │
        └─────────────────┴─────────────────┘
                    Network
```

## Container Interaction Protocols

| Source Container     | Target Container      | Protocol   | Purpose                    |
|----------------------|-----------------------|------------|----------------------------|
| CLI Node             | CLI Node (remote)     | gRPC/HTTP2 | Command invocation         |
| CLI Node             | RDF Ontology Server   | HTTP/SPARQL| Capability discovery       |
| CLI Node             | Consensus Service     | HotStuff   | Propose state updates      |
| Discovery Service    | External DHT          | Kademlia   | Peer discovery             |
| Consensus Service    | Consensus Service     | HotStuff   | Vote collection            |
| Monitoring           | CLI Node              | HTTP       | Scrape metrics             |
| Monitoring           | All Containers        | OTLP       | Collect distributed traces |

## References

- [C4 Model Documentation](https://c4model.com/)
- [Container Diagrams](https://c4model.com/#ContainerDiagram)
- [Structurizr (C4 tooling)](https://structurizr.com/)
